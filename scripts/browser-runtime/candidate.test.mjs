import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { consumerCompatibility, assertBuild, assertComparison, qualityProfile, qualityProfileSha256, validateCandidate, candidateLock, validateEvidence, installed, verify, hash } from './candidate.mjs';

const bytes = Buffer.from('new candidate archive');
const receipt = {
  runtimeVersion: '0.6.0-alpha.0', apiSchemaVersion: 2, engineVersion: '0.1.0',
  engineRevision: 'a'.repeat(40), engineDirty: false, buildId: `sha256:${'b'.repeat(64)}`, sha256: hash(bytes),
  files: ['package.json', 'build-info.json', 'src/build-info.js', 'src/index.d.ts', 'src/runtime.d.ts', 'src/types.d.ts', 'src/bindings.d.ts', 'src/index.js',
    'src/runtime.js', 'wasm/bindings.mjs', 'wasm/mixture_wasm_bg.wasm'],
};
const runtime = 'node_modules/@openmixture/runtime';
const vendor = 'file:vendor/openmixture-runtime-0.1.0-alpha.0.tgz';

test('candidate rejects historical archives, wrong revision, dirty sources and incomplete packages', () => {
  validateCandidate(receipt, bytes, receipt.engineRevision);
  assert.throws(() => validateCandidate(receipt, Buffer.from('old archive'), receipt.engineRevision), /digest/);
  assert.throws(() => validateCandidate(receipt, bytes, 'c'.repeat(40)), /revision/);
  assert.throws(() => validateCandidate({ ...receipt, engineDirty: true }, bytes, receipt.engineRevision), /clean/);
  for (const missing of receipt.files) {
    assert.throws(() => validateCandidate({ ...receipt, files: receipt.files.filter(file => file !== missing) }, bytes, receipt.engineRevision), /missing candidate/);
  }
  assert.throws(() => assertBuild({ ...receipt, buildId: 'old JS' }, receipt), /buildId/);
  assert.throws(() => assertBuild({ ...receipt, engineRevision: 'old WASM' }, receipt), /engineRevision/);
  assert.throws(() => assertBuild(receipt, { ...receipt, buildId: undefined }), /missing expected/);
});

test('archive substitution preserves the entire pinned registry dependency graph', () => {
  const lock = { lockfileVersion: 3, packages: {
    '': { dependencies: { '@openmixture/runtime': vendor } },
    [runtime]: { version: 'old', resolved: vendor, integrity: 'old', license: 'MIT OR Apache-2.0' },
    'node_modules/vite': { version: '8.3.0', integrity: 'frozen registry digest' },
  } };
  const changed = candidateLock(lock, receipt, bytes);
  assert.equal(changed.packages[runtime].version, receipt.runtimeVersion);
  assert.match(changed.packages[runtime].integrity, /^sha512-/);
  assert.equal(lock.packages[runtime].integrity, 'old');
  changed.packages[runtime] = lock.packages[runtime];
  assert.deepEqual(changed, lock);
  assert.throws(() => candidateLock({ ...lock, lockfileVersion: 2 }, receipt, bytes));
});

function evidence() {
  return [
    { receipt, lockSha256: 'lock' },
    { engineRevision: receipt.engineRevision, runtimeRevision: receipt.engineRevision, engineDirty: false },
    { build: receipt, archiveSha256: receipt.sha256, lockSha256: 'lock', cases: Array(11).fill({}), stress: { renders: 12 } },
    { schemaVersion: 3, ok: true, mode: 'acceptance', profile: qualityProfile, profileSha256: qualityProfileSha256,
      gates: { semantics: true, materialStructure: true, numericalAgreement: true },
      cases: Object.entries({'glazed-ceramic':['default','fine-tiles','matte'], leather:['default','detail-min','detail-max','coarse-grain'], wood:['default','coarse-grain','straight-grain','horizontal-grain']}).flatMap(([material, cases]) => cases.map(caseId => ({
        material, case:caseId, planMatches:true, relationships:[], channels:Object.fromEntries(['baseColor','normal','roughness','height'].map(channel => [channel, {
          comparison:{ok:true,profile:qualityProfile.id},structure:{ok:true},causality:caseId==='default'?null:{ok:true},
        }])),
      }))) },
    { stats: { expected: 28, unexpected: 0, skipped: 0, flaky: 0 } },
    { result: 'passed', testHarnessAbsent: true, archiveSha256: receipt.sha256 },
    { ok: true, archiveSha256: receipt.sha256 },
    { ok: true, build: receipt, mixedCode: 'MIX_BROWSER_BUILD_MISMATCH' },
  ];
}

test('qualification cannot substitute old browser identity, skip gates or accept partial evidence', () => {
  validateEvidence(...evidence());
  for (const mutate of [
    v => { v[1].engineDirty = true; },
    v => { v[2].build.buildId = 'old'; },
    v => { v[2].archiveSha256 = 'old'; },
    v => { v[2].lockSha256 = 'old'; },
    v => { v[2].cases.pop(); },
    v => { v[2].stress.renders = 0; },
    v => { v[3].ok = false; },
    v => { v[3].mode = 'measurement only'; },
    v => { v[3].schemaVersion = 1; },
    v => { v[3].schemaVersion = 2; },
    v => { v[3].profileSha256 = 'different rules'; },
    v => { v[3].profile.maxComponentDelta = 255; },
    v => { delete v[3].gates; },
    v => { v[3].gates.numericalAgreement = false; },
    v => { v[3].gates.materialStructure = false; },
    v => { v[3].cases[1] = v[3].cases[0]; },
    v => { delete v[3].cases[0].channels.height; },
    v => { v[3].cases[0].channels.height.comparison.ok = false; },
    v => { v[4].stats.expected = 0; },
    v => { v[4].stats.skipped = 1; },
    v => { v[4].stats.unexpected = 1; },
    v => { v[5].archiveSha256 = 'old'; },
    v => { v[6].ok = false; },
    v => { v[7].mixedCode = 'unexpected success'; },
  ]) {
    // Clone each entry separately: simulate independently written receipts.
    const changed = evidence().map(value => structuredClone(value));
    mutate(changed);
    assert.throws(() => validateEvidence(...changed));
  }
});

test('current qualification uses only the three quality gates', () => {
  const comparison = structuredClone(evidence()[3]);
  assert.deepEqual(Object.keys(comparison.gates).sort(), ['materialStructure', 'numericalAgreement', 'semantics']);
  assertComparison(comparison);
  for (const gate of Object.keys(comparison.gates)) {
    const changed = structuredClone(comparison);
    delete changed.gates[gate];
    assert.throws(() => assertComparison(changed));
  }
});

test('installed byte verification catches mixed components and a failed recheck invalidates success', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'mixture-candidate-test-'));
  try {
    const product = join(directory, 'product'), output = join(directory, 'evidence');
    await mkdir(join(product, 'vendor'), { recursive: true });
    await mkdir(join(product, runtime, 'wasm'), { recursive: true });
    await mkdir(output);
    const lock = Buffer.from('{}');
    await writeFile(join(product, 'package-lock.json'), lock);
    await writeFile(join(product, 'vendor/openmixture-runtime-0.1.0-alpha.0.tgz'), bytes);
    await writeFile(join(product, runtime, 'build-info.json'), JSON.stringify(receipt));
    await writeFile(join(product, runtime, 'wasm/mixture_wasm_bg.wasm'), 'current WASM');
    await writeFile(join(output, 'candidate.json'), JSON.stringify({ receipt, lockSha256: hash(lock),
      fileHashes: { 'wasm/mixture_wasm_bg.wasm': hash('current WASM') } }));
    await installed(product, output);
    await writeFile(join(product, runtime, 'wasm/mixture_wasm_bg.wasm'), 'historical WASM');
    await assert.rejects(installed(product, output), /installed package differs/);
    await writeFile(join(output, 'qualification.json'), JSON.stringify({ ok: true }));
    await assert.rejects(verify(product, output, 'missing-native', 'missing-browser'), /installed package differs/);
    assert.equal(JSON.parse(await readFile(join(output, 'qualification.json'))).ok, false);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test('ENG-04 disposable host compatibility preserves all other test assertions', () => {
 const before = "expect(result.catalogSize).toBe(11);\nexpect(explicit.runtimeVersion).toBe('0.1.0-alpha.0');\nexpect(result.estimateTypes).toEqual({\n    cumulativeBytes: 'bigint', peakBytes: 'bigint',\n});\nkeepLifecycleAndPixels();";
 const after = consumerCompatibility(before);
 assert.equal(after, "expect(result.catalogSize).toBe(14);\nexpect(explicit.runtimeVersion).toBe('0.6.0-alpha.0');\nexpect(result.estimateTypes).toEqual({\n    cumulativeBytes: 'bigint', peakBytes: 'bigint',\n    resourceCount: 'bigint', resourceUploadBytes: 'bigint',\n    resourceTextureBytes: 'bigint', resourceStagingBytes: 'bigint',\n});\nkeepLifecycleAndPixels();");
 assert.throws(()=>consumerCompatibility(before.replace("peakBytes: 'bigint'", "peakBytes: 'number'")));
 assert.throws(()=>consumerCompatibility(after)); assert.throws(()=>consumerCompatibility(before+before));
});
