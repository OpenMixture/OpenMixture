// Engine-owned qualification of a new archive in an unchanged, pinned product.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFile, writeFile, mkdir, copyFile } from 'node:fs/promises';
import { basename, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

export const buildKeys = ['runtimeVersion', 'apiSchemaVersion', 'engineVersion', 'engineRevision', 'engineDirty', 'buildId'];
const vendor = 'vendor/openmixture-runtime-0.1.0-alpha.0.tgz';
const runtime = 'node_modules/@openmixture/runtime';
export const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const save = (path, data) => writeFile(path, JSON.stringify(data, null, 2) + '\n');
const git = (cwd, ...args) => execFileSync('git', args, { cwd, encoding: 'utf8' }).trim();
const archiveFile = (archive, file) => execFileSync('tar', ['-xOf', archive, `package/${file}`], { maxBuffer: 32 * 1024 * 1024 });
const qualityBytes = await readFile(new URL('../../docs/browser-quality-v2.json', import.meta.url));
export const qualityProfile = JSON.parse(qualityBytes);
export const qualityProfileSha256 = `sha256:${hash(qualityBytes)}`;

export function assertComparison(comparison) {
  assert.equal(comparison.schemaVersion, 3, 'current browser comparison schema required');
  assert.equal(comparison.mode, 'acceptance');
  assert.equal(comparison.ok, true);
  assert.deepEqual(comparison.profile, qualityProfile);
  assert.equal(comparison.profileSha256, qualityProfileSha256);
  for (const gate of ['semantics', 'materialStructure', 'numericalAgreement']) {
    assert.equal(comparison.gates?.[gate], true, `missing/failed ${gate}`);
  }
  assert.equal(comparison.cases.length, 11);
  const seen = new Set();
  for (const record of comparison.cases) {
    assert.equal(record.planMatches, true);
    const key = `${record.material}/${record.case}`;
    assert.equal(seen.has(key), false, 'duplicate comparison case');
    seen.add(key);
    assert.deepEqual(Object.keys(record.channels).sort(), ['baseColor', 'height', 'normal', 'roughness']);
    for (const channel of Object.values(record.channels)) {
      assert.equal(channel.comparison?.ok, true);
      assert.equal(channel.comparison.profile, qualityProfile.id);
      assert.equal(channel.structure?.ok, true);
      if (record.case !== 'default') assert.equal(channel.causality?.ok, true);
    }
    assert.ok(Array.isArray(record.relationships));
    for (const relationship of record.relationships) assert.equal(relationship.ok, true);
  }
  assert.deepEqual([...seen].sort(), [
    'glazed-ceramic/default', 'glazed-ceramic/fine-tiles', 'glazed-ceramic/matte',
    'leather/default', 'leather/detail-min', 'leather/detail-max', 'leather/coarse-grain',
    'wood/default', 'wood/coarse-grain', 'wood/straight-grain', 'wood/horizontal-grain',
  ].sort());
}

export function assertBuild(actual, expected) {
  for (const key of buildKeys) {
    assert.notEqual(expected[key], undefined, `missing expected ${key}`);
    assert.deepEqual(actual[key], expected[key], `candidate ${key} mismatch`);
  }
}

export function validateCandidate(receipt, bytes, revision) {
  assert.match(revision, /^[a-f0-9]{40}$/);
  assert.equal(receipt.engineRevision, revision, 'candidate must come from the tested revision');
  assert.equal(receipt.engineDirty, false, 'candidate requires clean sources');
  assert.match(receipt.buildId, /^sha256:[a-f0-9]{64}$/);
  assert.equal(receipt.sha256, hash(bytes), 'candidate archive digest mismatch');
  assert.equal(receipt.runtimeVersion, '0.2.0-alpha.0', 'update the pinned consumer contract for a new version');
  assert.equal(receipt.apiSchemaVersion, 1);
  for (const file of ['package.json', 'build-info.json', 'src/build-info.js', 'src/index.d.ts', 'src/runtime.d.ts', 'src/types.d.ts', 'src/bindings.d.ts',
    'src/index.js', 'src/runtime.js', 'wasm/bindings.mjs', 'wasm/mixture_wasm_bg.wasm']) {
    assert.ok(receipt.files.includes(file), `missing candidate file: ${file}`);
  }
}

// Versioned expectations in the pinned disposable CI host only. Its repository is not edited.
export function consumerCompatibility(source) {
  const changes = [["expect(result.catalogSize).toBe(11);", "expect(result.catalogSize).toBe(12);"],
    ["expect(explicit.runtimeVersion).toBe('0.1.0-alpha.0');", "expect(explicit.runtimeVersion).toBe('0.2.0-alpha.0');"]];
  for (const [before, after] of changes) {
    assert.equal(source.split(before).length, 2, 'pinned compatibility assertion changed; review host contract');
    source = source.replace(before, after);
  }
  return source;
}

export function candidateLock(lock, receipt, bytes) {
  const changed = structuredClone(lock);
  assert.equal(changed.lockfileVersion, 3);
  assert.equal(changed.packages[''].dependencies['@openmixture/runtime'], `file:${vendor}`);
  assert.equal(changed.packages[runtime].resolved, `file:${vendor}`);
  // Update only this local archive's identity. npm ci verifies its SHA-512 and
  // installs the frozen registry graph; no dependency resolution or upgrades.
  changed.packages[runtime].version = receipt.runtimeVersion;
  changed.packages[runtime].integrity = `sha512-${createHash('sha512').update(bytes).digest('base64')}`;
  return changed;
}

export async function stage(packageDirectory, product, output, revision, consumerRevision) {
  await mkdir(output); // Fresh evidence directory: never reuse prior success.
  await save(join(output, 'qualification.json'), { ok: false, status: 'incomplete' });
  const receipt = await json(join(packageDirectory, 'receipt.json'));
  assert.equal(basename(receipt.tarball), `openmixture-runtime-${receipt.runtimeVersion}.tgz`);
  const archive = join(packageDirectory, basename(receipt.tarball));
  const bytes = await readFile(archive);
  validateCandidate(receipt, bytes, revision);
  assert.equal(git(product, 'rev-parse', 'HEAD'), consumerRevision, 'unexpected consumer revision');
  assert.equal(git(product, 'status', '--porcelain'), '', 'consumer must start clean');
  const testPath = join(product, 'tests/browser.spec.ts');
  const originalTest = await readFile(testPath, 'utf8');
  const adaptedTest = consumerCompatibility(originalTest);
  await writeFile(testPath, adaptedTest);
  await save(join(output, 'consumer-compatibility.json'), { consumerRevision,
    originalSha256: hash(Buffer.from(originalTest)), adaptedSha256: hash(Buffer.from(adaptedTest)),
    reason: 'ENG-04: twelve nodes and unpublished runtime 0.2.0-alpha.0; only two exact assertions updated',
    original: originalTest, adapted: adaptedTest });
  const lockBytes = await readFile(join(product, 'package-lock.json'));
  const originalArchive = await readFile(join(product, vendor));
  assert.notEqual(hash(originalArchive), receipt.sha256, 'historical archive is not a new candidate');
  const metadata = JSON.parse(archiveFile(archive, 'build-info.json'));
  assertBuild(metadata, receipt);
  const manifest = JSON.parse(archiveFile(archive, 'package.json'));
  assert.equal(manifest.name, '@openmixture/runtime');
  assert.equal(manifest.version, receipt.runtimeVersion);
  const fileHashes = {};
  for (const file of receipt.files) {
    assert.match(file, /^[a-zA-Z0-9_./-]+$/);
    assert.ok(!file.split('/').includes('..') && !file.startsWith('/'), 'unsafe package path');
    fileHashes[file] = hash(archiveFile(archive, file));
  }
  const oldWasm = archiveFile(join(product, vendor), 'wasm/mixture_wasm_bg.wasm');
  await writeFile(join(output, 'historical.wasm'), oldWasm);
  await copyFile(archive, join(product, vendor));
  await save(join(product, 'vendor/runtime-build.json'), receipt);
  await save(join(product, 'package-lock.json'), candidateLock(JSON.parse(lockBytes), receipt, bytes));
  await save(join(output, 'candidate.json'), {
    schemaVersion: 1, receipt, fileHashes, consumerRevision, originalLockSha256: hash(lockBytes),
    lockSha256: hash(await readFile(join(product, 'package-lock.json'))),
    originalArchiveSha256: hash(originalArchive), historicalWasmSha256: hash(oldWasm),
    run: process.env.GITHUB_RUN_ID ?? null, attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    startedAt: new Date().toISOString(),
  });
}

export async function installed(product, output) {
  const candidate = await json(join(output, 'candidate.json'));
  assert.equal(hash(await readFile(join(product, vendor))), candidate.receipt.sha256);
  assert.equal(hash(await readFile(join(product, 'package-lock.json'))), candidate.lockSha256);
  assertBuild(await json(join(product, runtime, 'build-info.json')), candidate.receipt);
  for (const [file, digest] of Object.entries(candidate.fileHashes)) {
    assert.equal(hash(await readFile(join(product, runtime, file))), digest, `installed package differs: ${file}`);
  }
  await save(join(output, 'installed.json'), { ok: true, archiveSha256: candidate.receipt.sha256 });
}

export async function probe(product, output) {
  const candidate = await json(join(output, 'candidate.json'));
  await save(join(output, 'probe.json'), { ok: false, status: 'incomplete' });
  const oldWasm = await readFile(join(output, 'historical.wasm'));
  assert.equal(hash(oldWasm), candidate.historicalWasmSha256);
  const { chromium } = await import(pathToFileURL(join(product, 'node_modules/@playwright/test/index.mjs')));
  const { serveStatic } = await import(pathToFileURL(join(product, 'scripts/static-server.mjs')));
  const args = ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist', ...JSON.parse(process.env.MIXTURE_BROWSER_ARGS ?? '[]')];
  const server = await serveStatic();
  let browser;
  try {
    browser = await chromium.launch({ channel: 'chromium', args });
    const page = await browser.newPage();
    await page.goto('http://127.0.0.1:4173/player/tests/contract.html');
    const build = await page.evaluate(async () => (await window.mixtureContract.loadRuntime()).getBuildInfo());
    assertBuild(build, candidate.receipt);
    // Feed the real historical WASM to today's JS: test the shipped guard,
    // rather than a mock binding or only a declaration/manifest comparison.
    await page.route('**/*.wasm', route => route.fulfill({ contentType: 'application/wasm', body: oldWasm }));
    const mixedCode = await page.evaluate(async () => {
      try { await window.mixtureContract.loadRuntime(); return 'unexpected success'; }
      catch (error) { return error.code; }
    });
    assert.equal(mixedCode, 'MIX_BROWSER_BUILD_MISMATCH');
    await save(join(output, 'probe.json'), { ok: true, build, mixedCode, browser: browser.version(), args });
  } finally {
    await browser?.close();
    await new Promise(resolve => server.close(resolve));
  }
}

export function validateEvidence(candidate, native, material, comparison, browser, deployment, installation, probeResult) {
  assert.equal(native.engineRevision, candidate.receipt.engineRevision);
  assert.equal(native.runtimeRevision, candidate.receipt.engineRevision);
  assert.equal(native.engineDirty, false);
  assertBuild(material.build, candidate.receipt);
  assert.equal(material.archiveSha256, candidate.receipt.sha256);
  assert.equal(material.lockSha256, candidate.lockSha256);
  assert.equal(material.cases.length, 11);
  assert.equal(material.stress.renders, 12);
  assertComparison(comparison);
  assert.equal(browser.stats.expected, 28);
  for (const field of ['unexpected', 'skipped', 'flaky']) assert.equal(browser.stats[field], 0);
  assert.equal(deployment.result, 'passed');
  assert.equal(deployment.testHarnessAbsent, true);
  assert.equal(deployment.archiveSha256, candidate.receipt.sha256);
  assert.equal(installation.ok, true);
  assert.equal(installation.archiveSha256, candidate.receipt.sha256);
  assert.equal(probeResult.ok, true);
  assertBuild(probeResult.build, candidate.receipt);
  assert.equal(probeResult.mixedCode, 'MIX_BROWSER_BUILD_MISMATCH');
}

export async function verify(product, output, nativeDirectory, browserDirectory) {
  await save(join(output, 'qualification.json'), { ok: false, status: 'incomplete' });
  await installed(product, output);
  const candidate = await json(join(output, 'candidate.json'));
  const paths = [join(nativeDirectory, 'manifest.json'), join(browserDirectory, 'receipt.json'),
    join(browserDirectory, 'comparison.json'), join(product, 'test-results/browser.json'),
    join(product, 'test-results/deployment/receipt.json'), join(output, 'installed.json'), join(output, 'probe.json')];
  const bytes = await Promise.all(paths.map(path => readFile(path)));
  const evidence = bytes.map(value => JSON.parse(value));
  validateEvidence(candidate, ...evidence);
  assert.equal(evidence[1].manifestSha256, hash(bytes[0]));
  assert.equal(evidence[2].nativeManifestSha256, `sha256:${hash(bytes[0])}`);
  assert.equal(evidence[2].browserReceiptSha256, `sha256:${hash(bytes[1])}`);
  await save(join(output, 'qualification.json'), { schemaVersion: 1, ok: true, candidate,
    evidence: paths.map((path, i) => ({ path, sha256: hash(bytes[i]) })), completedAt: new Date().toISOString(),
    scope: 'Pinned Chromium candidate qualification; not default-browser support or npm publication' });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [mode, ...args] = process.argv.slice(2);
  const operation = { stage, installed, probe, verify }[mode];
  assert.ok(operation && args.length === operation.length, 'Usage: candidate.mjs stage <package-dir> <product> <new-evidence-dir> <engine-sha> <consumer-sha> | installed/probe <product> <evidence-dir> | verify <product> <evidence-dir> <native-dir> <browser-dir>');
  await operation(...args);
}
