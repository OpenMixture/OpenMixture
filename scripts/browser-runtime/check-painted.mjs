// Bind the complete MAT-02 public matrix to one clean browser candidate.
import assert from 'node:assert/strict';
import { paintedMetalMatrix } from '../painted-metal-requests.mjs';
import { readFile, readdir, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { join, resolve, dirname } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';

assert.equal(process.argv.length, 4, 'usage: check-painted.mjs <candidate-qualification> <fresh-output>');
const [input, destination] = process.argv.slice(2).map(p => resolve(p));
const receipt = JSON.parse(await readFile(join(input, 'qualification.json')));
assert.equal(receipt.ok, true);
assert.equal(receipt.mode, 'candidate');
assert.equal(receipt.consumerDirty, false);
assert.equal(receipt.build.runtimeVersion, '0.8.0-alpha.0');
assert.equal(receipt.build.apiSchemaVersion, 3);
assert.equal(receipt.consumerRevision, execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim());
assert.equal(receipt.reuseSourceSha256, createHash('sha256').update(await readFile('docs/evidence/perf-mat-before/material.mix')).digest('hex'));
const matches = [];
async function find(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await find(path);
    else if (entry.name === 'painted-browser.json') matches.push(path);
  }
}
await find(join(input, 'test-results'));
assert.equal(matches.length, 1);
const browser = JSON.parse(await readFile(matches[0]));
assert.equal(browser.ok, true);
assert.deepEqual(browser.build, receipt.build);
assert.equal(browser.manifest.sourceRevision, receipt.consumerRevision);
assert.equal(browser.manifest.workingTreeStatus, '');
assert.equal(createHash('sha256').update(JSON.stringify(browser.manifest, null, 2) + '\n').digest('hex'), receipt.paintedRequestsSha256);
assert.equal(browser.packageSha256, receipt.paintedPackageSha256);
const expectedRows = paintedMetalMatrix();
const contract = JSON.parse(await readFile('fixtures/materials/painted-metal/qualification-plan.json'));
assert.deepEqual(browser.manifest.causality, contract.causality);
assert.deepEqual(browser.rows.map(r => r.id), expectedRows.map(r => r.id));
assert.deepEqual(browser.manifest.rows, expectedRows);
assert.equal(createHash('sha256').update(await readFile('docs/evidence/perf-mat-before/material.mix')).digest('hex'), browser.manifest.sourceSha256);
for (let i = 0; i < expectedRows.length; i++) {
  assert.deepEqual(browser.rows[i].size, expectedRows[i].request.size);
  assert.deepEqual(browser.rows[i].overrides, expectedRows[i].request.overrides);
  for (const flag of ['repeatExact', 'packageExact', 'slicedExact', 'ownedAfterDestroy']) assert.equal(browser.rows[i][flag], true);
  if (['intact', 'exposed', 'rusted'].includes(expectedRows[i].preset)) {
    const endpoints = browser.rows[i].endpoints;
    assert.deepEqual(endpoints.map(row => row.channel).sort(), ['baseColor', 'height', 'metallic', 'normal', 'roughness']);
    for (const row of endpoints) {
      assert.equal(row.allPixelsExact, true);
      assert.equal(row.expectedRgba8.length, 4);
    }
  }
}
const browserDownsample = browser.rows.find(row => row.id === 'default-1024x1024').downsample;
const browserControls = browser.rows.find(row => row.id === 'default-257x129').controls;
assert.equal(browserControls.length, 11);
assert.deepEqual(browserControls.map(row => row.control), [
  'paintColor', 'substrateColor', 'rustColor', 'paintRoughness', 'substrateRoughness', 'rustRoughness',
  'normalStrength', 'normalStrength', 'normalStrength', 'macroSeed', 'detailSeed',
]);
for (const row of browserControls) {
  assert.equal(row.repeatExact, true);
  assert.equal(row.isolationPassed, true);
}
assert.deepEqual(browserDownsample.map(row => row.channel), ['height', 'baseColor']);
for (const row of browserDownsample) {
  assert.equal(row.passed, true);
  assert.equal(row.limit, 4);
  assert.equal(row.componentMeanError.length, 3);
  assert.ok(row.componentMeanError.every(value => Number.isFinite(value) && value >= 0 && value <= 4));
}
await mkdir(destination);
await writeFile(join(destination, 'requests.json'), JSON.stringify(browser.manifest, null, 2) + '\n');
await copyFile('docs/evidence/perf-mat-before/material.mix', join(destination, 'material.mix'));
await writeFile(join(destination, 'comparison.json'), '{"ok":false,"completed":false}');
await copyFile(matches[0], join(destination, 'painted-browser.json'));
const channels = ['baseColor', 'height', 'metallic', 'normal', 'roughness'];
for (const row of browser.rows) {
  assert.deepEqual(row.files.map(f => f.channel).sort(), channels);
  for (const file of row.files) {
    assert.equal(file.name, `${row.id}-${file.channel}.png`);
    await copyFile(join(dirname(matches[0]), file.name), join(destination, file.name));
  }
}
const result = spawnSync('cargo', ['test', '--release', '--locked', '--all-features', '--manifest-path', 'examples/native-consumer/Cargo.toml', '--target-dir', 'target/native-consumer', '--test', 'painted_material', '--', '--ignored', '--nocapture'], {
  encoding: 'utf8', env: { ...process.env, MIXTURE_PAINTED_ROOT: resolve('.'), MIXTURE_PAINTED_REQUESTS: destination, MIXTURE_PAINTED_BROWSER: destination, MIXTURE_PAINTED_EVIDENCE: join(destination, 'native') },
});
await writeFile(join(destination, 'stdout.log'), result.stdout ?? '');
await writeFile(join(destination, 'stderr.log'), result.stderr ?? '');
if (result.error) throw result.error;
assert.equal(result.status, 0, 'public painted-metal check failed; inspect retained native receipt/logs');
const native = JSON.parse(await readFile(join(destination, 'native/native.json')));
assert.equal(native.ok, true); assert.equal(native.completed, true);
assert.equal(native.debugAssertions, false); assert.equal(native.browserCompared, true);
assert.equal(native.rows.length, 28); assert.equal(native.timing.length, 2);
assert.equal(native.endpointChannels, 60, 'all endpoint channels must match independent constants');
assert.deepEqual(native.controls, browserControls, 'both public hosts must prove the same control isolation and effects');
assert.deepEqual(native.downsample.map(row => row.channel), ['height', 'baseColor']);
for (const row of native.downsample) {
  assert.equal(row.passed, true);
  assert.equal(row.limit, 4);
  assert.equal(row.componentMeanError.length, 3);
  assert.ok(row.componentMeanError.every(value => Number.isFinite(value) && value >= 0 && value <= 4));
}
for (const row of native.timing) assert.equal(row.budgetPassed, true, 'matched frozen timing budget required');
for (const row of native.rows) {
  assert.equal(row.comparisons.length, 5);
  for (const c of row.comparisons) assert.ok(c.maxComponentDelta <= 1);
}
await copyFile(join(input, 'qualification.json'), join(destination, 'browser-qualification.json'));
await writeFile(join(destination, 'comparison.json'), JSON.stringify({ ok: true, completed: true, comparisons: 140,
  revision: receipt.consumerRevision, build: receipt.build,
  native: 'native/native.json', browser: 'painted-browser.json', materialAccepted: false }, null, 2));
console.log(`Public painted-metal matrix passed: ${destination}`);
