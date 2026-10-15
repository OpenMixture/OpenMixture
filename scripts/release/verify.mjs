import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir, appendFile, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const repository = 'OpenMixture/OpenMixture';
export function validateRequest({ ref, sha, version, digest, tag }) {
  assert.equal(ref, 'refs/heads/main', 'Dispatch only from main');
  assert.match(sha, /^[a-f0-9]{40}$/);
  assert.match(version, /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(?:alpha|beta|rc)\.(0|[1-9]\d*))?$/);
  assert.match(digest, /^[a-f0-9]{64}$/);
  assert.ok(['alpha', 'next', 'latest'].includes(tag));
  assert.ok(tag !== 'latest' || !version.includes('-'), 'Prereleases cannot use latest');
}

export function validateRun(run, sha, workflow) {
  assert.equal(run.repository.full_name, repository);
  assert.equal(run.head_repository.full_name, repository);
  assert.equal(run.head_sha, sha);
  assert.equal(run.head_branch, 'main');
  assert.equal(run.event, 'push');
  assert.equal(run.path, `.github/workflows/${workflow}`);
  assert.equal(run.status, 'completed');
  assert.equal(run.conclusion, 'success');
}

export function validateJobs(jobs, names) {
  for (const name of names) {
    const job = jobs.find(item => item.name === name);
    assert.ok(job, `Missing executed job: ${name}`);
    assert.equal(job.status, 'completed');
    assert.equal(job.conclusion, 'success', `Job did not execute successfully: ${name}`);
  }
}

export function validateArchive(receipt, manifest, build, request, bytes) {
  assert.equal(receipt.engineRevision, request.sha);
  assert.equal(receipt.engineDirty, false);
  assert.equal(receipt.runtimeVersion, request.version);
  assert.equal(receipt.sha256, request.digest);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), request.digest);
  assert.equal(manifest.name, '@openmixture/runtime');
  assert.equal(manifest.version, request.version);
  assert.equal(manifest.repository.url, 'https://github.com/OpenMixture/OpenMixture.git');
  for (const key of ['engineRevision', 'engineDirty', 'runtimeVersion', 'engineVersion', 'apiSchemaVersion', 'buildId']) {
    assert.equal(build[key], receipt[key], `Archive identity mismatch: ${key}`);
  }
}

function command(program, args) {
  const result = spawnSync(program, args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${program} failed: ${result.stderr}`);
  return result.stdout;
}
function api(path) { return JSON.parse(command('gh', ['api', `repos/${repository}/${path}`])); }
async function main() {
  const request = { ref: process.env.GITHUB_REF, sha: process.env.GITHUB_SHA,
    version: process.env.RELEASE_VERSION, digest: process.env.RELEASE_SHA256, tag: process.env.RELEASE_TAG };
  validateRequest(request);
  assert.equal(process.env.GITHUB_REPOSITORY, repository);
  const selected = [];
  const workflows = {
    'ci.yml': ['Check (ubuntu-latest)', 'Check (macos-latest)', 'Check (windows-latest)'],
    'gpu-smoke.yml': ['Pinned SwiftShader Vulkan materials and packaged consumption'],
    'browser-runtime.yml': ['WASM and npm package'],
    'browser-materials.yml': ['Chromium WebGPU material matrix'],
  };
  for (const [workflow, names] of Object.entries(workflows)) {
    const runs = api(`actions/workflows/${workflow}/runs?head_sha=${request.sha}&event=push&branch=main&per_page=100`).workflow_runs;
    assert.ok(runs.length, `No main push qualification for ${workflow}`);
    const run = runs.sort((a, b) => b.id - a.id)[0];
    validateRun(run, request.sha, workflow);
    const jobs = api(`actions/runs/${run.id}/attempts/${run.run_attempt}/jobs?per_page=100`);
    assert.ok(jobs.total_count <= 100, 'Unexpected job pagination');
    validateJobs(jobs.jobs, names);
    // A successful aggregate alone is not evidence that material partitions ran.
    if (workflow === 'browser-materials.yml') {
      const partitions = jobs.jobs.filter(job => job.name !== 'Qualification scope' && job.name !== names[0]);
      assert.equal(partitions.length, 3, 'Expected all three material partitions');
      validateJobs(partitions, partitions.map(job => job.name));
      const artifacts = api(`actions/runs/${run.id}/artifacts?per_page=100`);
      assert.ok(artifacts.total_count <= 100, 'Unexpected artifact pagination');
      const artifact = artifacts.artifacts.find(item => item.name === 'chromium-material-matrix-core');
      assert.ok(artifact && !artifact.expired, 'Qualified archive expired or missing; rerun qualification');
      command('gh', ['run', 'download', String(run.id), '--repo', repository,
        '--name', artifact.name, '--dir', 'tmp/npm-release']);
    }
    selected.push({ workflow, id: run.id, attempt: run.run_attempt, url: run.html_url });
  }
  const directory = 'tmp/npm-release/target/browser-runtime';
  const receipt = JSON.parse(await readFile(`${directory}/receipt.json`, 'utf8'));
  const archives = (await readdir(directory)).filter(name => name.endsWith('.tgz'));
  assert.equal(archives.length, 1);
  const archive = `${directory}/${archives[0]}`;
  const bytes = await readFile(archive);
  const manifest = JSON.parse(command('tar', ['-xOf', archive, 'package/package.json']));
  const build = JSON.parse(command('tar', ['-xOf', archive, 'package/build-info.json']));
  validateArchive(receipt, manifest, build, request, bytes);
  assert.equal(receipt.tarball, `target/browser-runtime/${archives[0]}`);
  const evidence = { ...request, archive, runs: selected, build };
  await writeFile('tmp/npm-release/verification.json', JSON.stringify(evidence, null, 2) + '\n');
  await appendFile(process.env.GITHUB_OUTPUT, `archive=${archive}\n`);
  await appendFile(process.env.GITHUB_STEP_SUMMARY, `## Verified npm candidate\n\nVersion: ${request.version}\n\nCommit: ${request.sha}\n\nSHA-256: ${request.digest}\n\nDist-tag: ${request.tag}\n\n${selected.map(run => `- [${run.workflow}, attempt ${run.attempt}](${run.url})`).join('\n')}\n`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await main();
