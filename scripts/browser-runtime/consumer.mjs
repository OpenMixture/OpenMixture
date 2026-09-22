// Qualify the public SDK in a fresh engine-owned consumer outside the checkout.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import { cp, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { assertBuild, buildKeys, hash, validateCandidate } from './candidate.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const example = join(root, 'examples/browser-consumer');
const runtime = 'node_modules/@openmixture/runtime';
const vendor = 'vendor/runtime.tgz';
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const save = (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n');
const archiveFile = (archive, file) => execFileSync('tar', ['-xOf', archive, `package/${file}`], { maxBuffer: 32 * 1024 * 1024 });
const integrity = bytes => `sha512-${createHash('sha512').update(bytes).digest('base64')}`;

export function candidateManifests(manifest, lock, version, bytes) {
  const published = '0.3.0-alpha.0';
  assert.equal(version, '0.5.0-alpha.0', 'review the candidate compatibility contract before upgrading');
  assert.equal(manifest.dependencies['@openmixture/runtime'], published, 'example must pin the exact published version');
  assert.equal(lock.lockfileVersion, 3);
  assert.equal(lock.packages[''].dependencies['@openmixture/runtime'], published);
  assert.equal(lock.packages[runtime].version, published);
  const nextManifest = structuredClone(manifest), nextLock = structuredClone(lock);
  nextManifest.dependencies['@openmixture/runtime'] = `file:${vendor}`;
  nextLock.packages[''].dependencies['@openmixture/runtime'] = `file:${vendor}`;
  nextLock.packages[runtime].version = version;
  nextLock.packages[runtime].resolved = `file:${vendor}`;
  nextLock.packages[runtime].integrity = integrity(bytes);
  return { manifest: nextManifest, lock: nextLock };
}

export function safePackagePath(path) {
  assert.match(path, /^[a-zA-Z0-9_./-]+$/);
  assert.ok(!path.startsWith('/') && path.split('/').every(part => part && part !== '.' && part !== '..'), 'unsafe package path');
  return path;
}

export async function verifyInstalled(directory, expectedBuild, files) {
  assertBuild(await json(join(directory, runtime, 'build-info.json')), expectedBuild);
  for (const [file, digest] of Object.entries(files)) {
    assert.equal(hash(await readFile(join(directory, runtime, safePackagePath(file)))), digest, `installed bytes differ: ${file}`);
  }
}

export function assertBrowserReport(report, mode = 'registry') {
  assert.ok(['candidate', 'registry'].includes(mode));
  assert.equal(report.stats.expected, mode === 'candidate' ? 16 : 13, 'all public consumer tests must execute');
  for (const key of ['unexpected', 'skipped', 'flaky']) assert.equal(report.stats[key], 0, `browser ${key}`);
  assert.deepEqual(report.errors ?? [], [], 'browser runner errors');
}

async function sourceFiles(directory, prefix = '') {
  const result = {};
  for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
    if (['node_modules', 'dist', 'test-dist', 'test-results', 'playwright-report', 'vendor'].includes(entry.name)) continue;
    const path = join(directory, entry.name), name = prefix + entry.name;
    if (entry.isDirectory()) Object.assign(result, await sourceFiles(path, `${name}/`));
    else { assert.ok(entry.isFile(), 'consumer sources cannot be symlinks'); result[name] = hash(await readFile(path)); }
  }
  return result;
}

export async function qualify(mode, packageDirectory, output) {
  assert.ok(['candidate', 'registry'].includes(mode));
  await mkdir(dirname(output), { recursive: true });
  await mkdir(output); // Fresh attempt: never reuse a successful receipt.
  const receiptPath = join(output, 'qualification.json');
  await save(receiptPath, { ok: false, status: 'incomplete', mode });
  const staging = await mkdtemp(join(tmpdir(), 'mixture-browser-consumer-'));
  let success = false;
  const record = { schemaVersion: 1, mode, ok: false, startedAt: new Date().toISOString(),
    verifierSha256: hash(await readFile(fileURLToPath(import.meta.url))),
    consumerRevision: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
    consumerDirty: execFileSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8' }).trim().length > 0,
    staging, platform: process.platform, arch: process.arch, node: process.version,
    run: process.env.GITHUB_RUN_ID ?? null, attempt: process.env.GITHUB_RUN_ATTEMPT ?? null };
  const env = { ...process.env, npm_config_cache: join(staging, '.npm-cache'), MIXTURE_RESOURCE_TESTS: '1', MIXTURE_ASSET_TESTS: mode === 'candidate' ? '1' : '0' };
  // The automated browser selects its adapter using its recorded launch args.
  delete env.VK_ICD_FILENAMES;
  delete env.VK_DRIVER_FILES;
  let sequence = 0;
  function npm(args) {
    const program = process.platform === 'win32' ? process.execPath : 'npm';
    const argv = process.platform === 'win32'
      ? [join(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js'), ...args] : args;
    const result = spawnSync(program, argv, { cwd: staging, env, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
    const name = `${++sequence}-${args[0]}`;
    return Promise.all([
      save(join(output, `${name}.command.json`), { program, args: argv, cwd: staging, status: result.status }),
      writeFile(join(output, `${name}.stdout.log`), result.stdout ?? ''),
      writeFile(join(output, `${name}.stderr.log`), result.stderr ?? ''),
    ]).then(() => {
      if (result.error) throw result.error;
      assert.equal(result.status, 0, `npm ${args.join(' ')} failed; see ${output}/${name}.stderr.log`);
      return result.stdout;
    });
  }
  try {
    record.sourceFiles = await sourceFiles(example);
    for (const file of Object.keys(record.sourceFiles)) {
      await mkdir(dirname(join(staging, file)), { recursive: true });
      await cp(join(example, file), join(staging, file));
    }
    const manifest = await json(join(staging, 'package.json'));
    const lock = await json(join(staging, 'package-lock.json'));
    let version = manifest.dependencies['@openmixture/runtime'];
    assert.match(version, /^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/, 'exact version required');
    let archive, expected;
    if (mode === 'candidate') {
      expected = await json(join(packageDirectory, 'receipt.json'));
      archive = join(packageDirectory, basename(expected.tarball.replaceAll('\\', '/')));
      validateCandidate(expected, await readFile(archive), record.consumerRevision);
      version = expected.runtimeVersion;
      execFileSync('cargo', ['run','--locked','--manifest-path',join(root,'examples/native-consumer/Cargo.toml'),'--target-dir',join(root,'target/native-consumer'),'--example','asset-fixtures','--',join(staging,'public/m6b05')], {cwd:root,stdio:'pipe'});
      record.assetFixtures = {};
      for (const name of ['1024x1024.mixpack','65x3.mixpack']) record.assetFixtures[name] = hash(await readFile(join(staging,'public/m6b05',name)));
      await mkdir(join(staging, 'vendor'));
      await cp(archive, join(staging, vendor));
      const next = candidateManifests(manifest, lock, version, await readFile(archive));
      await save(join(staging, 'package.json'), next.manifest);
      await save(join(staging, 'package-lock.json'), next.lock);
    } else {
      const packed = JSON.parse(await npm(['pack', `@openmixture/runtime@${version}`, '--json', '--ignore-scripts',
        '--registry=https://registry.npmjs.org/', '--pack-destination', output]));
      assert.equal(packed.length, 1);
      archive = join(output, basename(packed[0].filename));
      const bytes = await readFile(archive);
      assert.equal(packed[0].integrity, integrity(bytes));
      assert.equal(lock.packages[runtime].integrity, integrity(bytes), 'registry bytes differ from committed lock');
      assert.equal(lock.packages[runtime].version, version);
      assert.match(lock.packages[runtime].resolved, /^https:\/\/registry\.npmjs\.org\/@openmixture\/runtime\//);
      expected = JSON.parse(archiveFile(archive, 'build-info.json'));
    }
    const bytes = await readFile(archive);
    const metadata = JSON.parse(archiveFile(archive, 'build-info.json'));
    assertBuild(metadata, expected);
    assert.equal(metadata.runtimeVersion, version);
    const packageManifest = JSON.parse(archiveFile(archive, 'package.json'));
    assert.equal(packageManifest.name, '@openmixture/runtime');
    assert.equal(packageManifest.version, version);
    for (const hook of ['preinstall', 'install', 'postinstall']) assert.equal(packageManifest.scripts?.[hook], undefined);
    const entries = execFileSync('tar', ['-tzf', archive], { encoding: 'utf8' }).trim().split(/\r?\n/).filter(path => !path.endsWith('/'));
    assert.ok(entries.length > 0 && entries.length <= 64);
    const fileHashes = {};
    for (const entry of entries) {
      assert.ok(entry.startsWith('package/'));
      const file = safePackagePath(entry.slice('package/'.length));
      assert.equal(fileHashes[file], undefined, 'duplicate archive entry');
      fileHashes[file] = hash(archiveFile(archive, file));
    }
    if (mode === 'candidate') assert.deepEqual(Object.keys(fileHashes).sort(), [...expected.files].sort());
    record.build = Object.fromEntries(buildKeys.map(key => [key, metadata[key]]));
    record.archiveSha256 = hash(bytes);
    record.archiveIntegrity = integrity(bytes);
    record.fileHashes = fileHashes;
    record.lockSha256 = hash(await readFile(join(staging, 'package-lock.json')));
    record.browserArgs = ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist', ...JSON.parse(env.MIXTURE_BROWSER_ARGS ?? '[]')];
    record.browserChannel = env.MIXTURE_BROWSER_CHANNEL ?? 'chromium';
    await cp(join(staging, 'package-lock.json'), join(output, 'package-lock.json'));
    await save(join(output, 'installation.json'), record);
    await npm(['ci', '--ignore-scripts', '--registry=https://registry.npmjs.org/']);
    await verifyInstalled(staging, metadata, fileHashes);
    record.toolchain = { npm: (await npm(['--version'])).trim(),
      playwright: (await json(join(staging, 'node_modules/@playwright/test/package.json'))).version,
      vite: (await json(join(staging, 'node_modules/vite/package.json'))).version,
      typescript: (await json(join(staging, 'node_modules/typescript/package.json'))).version };
    await npm(['run', 'check']);
    await npm(['run', 'build']);
    await npm(['exec', '--', 'playwright', 'install', ...(process.platform === 'linux' ? ['--with-deps'] : []), 'chromium']);
    await npm(['run', 'test:browser']);
    const browser = await json(join(staging, 'test-results/report.json'));
    assertBrowserReport(browser, mode);
    await verifyInstalled(staging, metadata, fileHashes);
    assert.equal(hash(await readFile(join(staging, 'package-lock.json'))), record.lockSha256);
    record.browserTests = browser.stats;
    record.ok = true;
    success = true;
  } catch (error) {
    record.error = String(error);
    throw error;
  } finally {
    record.finishedAt = new Date().toISOString();
    try {
      await cp(join(staging, 'test-results'), join(output, 'test-results'), { recursive: true });
    } catch (error) {
      if (error.code !== 'ENOENT' || success) {
        record.ok = false;
        record.error = `Could not retain browser evidence: ${error}`;
        success = false;
        throw error;
      }
    } finally { await save(receiptPath, record); }
    // Delete only this invocation's verified OS-temp staging directory on success.
    if (success && dirname(resolve(staging)) === resolve(tmpdir()) && basename(staging).startsWith('mixture-browser-consumer-')) {
      await rm(staging, { recursive: true });
    }
  }
  console.log(`${mode} independent browser consumption passed: ${output}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [mode, packageDirectory, output] = process.argv.slice(2);
  assert.ok(mode && packageDirectory && output, 'usage: consumer.mjs <candidate|registry> <package-directory|-> <fresh-output>');
  await qualify(mode, resolve(packageDirectory), resolve(output));
}
