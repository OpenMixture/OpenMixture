import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { join, dirname, basename, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { assertBrowserReport, candidateManifests, safePackagePath, verifyInstalled } from './consumer.mjs';
import { hash } from './candidate.mjs';

test('candidate substitution preserves the frozen dependency graph and does not accept floating versions', () => {
  const manifest = { dependencies: { '@openmixture/runtime': '0.3.0-alpha.0' } };
  const lock = { lockfileVersion: 3, packages: {
    '': structuredClone(manifest),
    'node_modules/@openmixture/runtime': { version: '0.3.0-alpha.0', resolved: 'https://registry.npmjs.org/runtime.tgz', integrity: 'old' },
    'node_modules/vite': { version: '8.3.0', integrity: 'unchanged' },
  } };
  const result = candidateManifests(manifest, lock, '0.5.0-alpha.0', Buffer.from('candidate'));
  assert.equal(result.manifest.dependencies['@openmixture/runtime'], 'file:vendor/runtime.tgz');
  assert.equal(result.lock.packages['node_modules/@openmixture/runtime'].resolved, 'file:vendor/runtime.tgz');
  assert.match(result.lock.packages['node_modules/@openmixture/runtime'].integrity, /^sha512-/);
  assert.deepEqual(result.lock.packages['node_modules/vite'], lock.packages['node_modules/vite']);
  assert.equal(manifest.dependencies['@openmixture/runtime'], '0.3.0-alpha.0');
  assert.throws(() => candidateManifests({ dependencies: { '@openmixture/runtime': '^0.1.0' } }, lock, '0.5.0-alpha.0', Buffer.from('candidate')));
});

test('archive paths cannot escape the installed package', () => {
  assert.equal(safePackagePath('wasm/mixture_wasm_bg.wasm'), 'wasm/mixture_wasm_bg.wasm');
  for (const path of ['../outside', '/absolute', 'src/../../outside', 'src\\outside', 'C:/outside', 'src//file', './file']) {
    assert.throws(() => safePackagePath(path));
  }
});

test('partial, skipped, flaky and failed browser evidence cannot pass qualification', () => {
  const report = { stats: { expected: 13, unexpected: 0, skipped: 0, flaky: 0 }, errors: [] };
  assertBrowserReport(report);
  assertBrowserReport({...report,stats:{...report.stats,expected:16}},'candidate');
  assert.throws(()=>assertBrowserReport(report,'candidate'));
  for (const stats of [{ expected: 7 }, { unexpected: 1 }, { skipped: 1 }, { flaky: 1 }]) {
    assert.throws(() => assertBrowserReport({ ...report, stats: { ...report.stats, ...stats } }));
  }
  assert.throws(() => assertBrowserReport({ ...report, errors: [{ message: 'worker crashed' }] }));
});

test('installed verification rejects changed WASM and mismatched build metadata', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'mixture-consumer-test-'));
  const installed = join(directory, 'node_modules/@openmixture/runtime');
  const build = { runtimeVersion: '0.1.0-alpha.0', apiSchemaVersion: 1, engineVersion: '0.1.0',
    engineRevision: 'a'.repeat(40), engineDirty: false, buildId: `sha256:${'b'.repeat(64)}` };
  try {
    await mkdir(join(installed, 'wasm'), { recursive: true });
    await writeFile(join(installed, 'build-info.json'), JSON.stringify(build));
    await writeFile(join(installed, 'wasm/runtime.wasm'), 'candidate');
    const files = { 'wasm/runtime.wasm': hash('candidate') };
    await verifyInstalled(directory, build, files);
    await writeFile(join(installed, 'wasm/runtime.wasm'), 'historical');
    await assert.rejects(verifyInstalled(directory, build, files), /installed bytes differ/);
    await writeFile(join(installed, 'build-info.json'), JSON.stringify({ ...build, engineRevision: 'c'.repeat(40) }));
    await assert.rejects(verifyInstalled(directory, build, files), /engineRevision mismatch/);
  } finally {
    assert.equal(dirname(resolve(directory)), resolve(tmpdir()));
    assert.ok(basename(directory).startsWith('mixture-consumer-test-'));
    await rm(directory, { recursive: true });
  }
});

test('ENG-04 native and browser hosts consume the same source fixture', async () => {
 const fixture = await readFile(new URL('../../fixtures/nodes/scalar-blend/two-noise.mix', import.meta.url));
 for(const path of ['../../examples/native-consumer/tests/scalar-blend.mix','../../examples/browser-consumer/public/scalar-blend.mix']) {
  assert.deepEqual(await readFile(new URL(path,import.meta.url)),fixture);
 }
});

test('published M6A resource cases are required in both modes and share the frozen source', async () => {
  const report = { stats: { expected: 13, unexpected: 0, skipped: 0, flaky: 0 }, errors: [] };
  assertBrowserReport({...report,stats:{...report.stats,expected:16}}, 'candidate');
  assert.throws(() => assertBrowserReport({ ...report, stats: { ...report.stats, expected: 9 } }, 'candidate'));
  assertBrowserReport(report, 'registry');
  assert.throws(() => assertBrowserReport({ ...report, stats: { ...report.stats, expected: 9 } }, 'registry'));
  const fixture = await readFile(new URL('../../fixtures/nodes/image-input/height.mix', import.meta.url));
  for (const path of ['../../examples/native-consumer/tests/image-input.mix','../../examples/browser-consumer/public/image-input.mix']) {
    assert.deepEqual(await readFile(new URL(path, import.meta.url)), fixture);
  }
});


test('asset closeout rejects missing, duplicate, mismatched and failed matrix evidence',async()=>{
  const {verifyAssetMatrix}=await import('./asset-matrix.mjs');
  const rows=['1024x1024','65x3'].flatMap(size=>[0,0.25,0.5,1].map(weight=>({size:size.split('x').map(Number),weight,planHash:'same',packageSha256:size,exactLoosePixels:true,repeatedLoads:2,allocations:{liveBytes:'0'},execution:{allocations:{liveBytes:0}},browserComparison:['height','normal'].map(channel=>({channel,maxComponentDelta:1,limit:1}))})));
  const browser={ownedAfterDestroy:true,rows:structuredClone(rows)},native={ok:true,ownedAfterDestroy:true,cases:rows};
  const receipt={assetFixtures:{'1024x1024.mixpack':'1024x1024','65x3.mixpack':'65x3'}};
  verifyAssetMatrix(native,browser,receipt);
  for(const mutate of [n=>n.cases.pop(),n=>n.cases[1]=n.cases[0],n=>n.cases[0].planHash='wrong',n=>n.cases[0].packageSha256='wrong',n=>n.cases[0].execution.allocations.liveBytes=1,n=>n.cases[0].browserComparison[0].maxComponentDelta=2,n=>n.ownedAfterDestroy=false]){
    const bad=structuredClone(native);mutate(bad);assert.throws(()=>verifyAssetMatrix(bad,browser,receipt));
  }
});
