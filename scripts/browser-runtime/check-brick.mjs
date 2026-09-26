// MAT-01 cross-runtime receipt, bound to the candidate consumer and frozen inputs.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { join, resolve, dirname } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';

const args = process.argv.slice(2);
assert.equal(args.length, 2, 'usage: check-brick.mjs <candidate-qualification> <fresh-output>');
const [input, destination] = args.map(p => resolve(p));
const receipt = JSON.parse(await readFile(join(input, 'qualification.json')));
assert.equal(receipt.ok, true);
assert.equal(receipt.mode, 'candidate');
assert.equal(receipt.build.runtimeVersion, '0.8.0-alpha.0');
assert.equal(receipt.consumerRevision, execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim());
assert.equal(receipt.consumerDirty, false);
const fixture = resolve('fixtures/materials/brick-paving');
for (const name of ['material.mix', 'controls.json', 'qualification-plan.json']) {
  const digest = createHash('sha256').update(await readFile(join(fixture, name))).digest('hex');
  assert.equal(receipt.brickFixtures[name], digest, `fixture identity ${name}`);
}
const matrix = JSON.parse(await readFile(join(fixture, 'qualification-plan.json')));
const expected = new Set(matrix.cases.flatMap(c => matrix.sizes.flatMap(([w,h]) => matrix.channels.map(channel => `${c.id}-${w}x${h}-${channel}.png`))));
expected.add('brick-browser.json');
await mkdir(destination);
await writeFile(join(destination, 'comparison.json'), JSON.stringify({ ok: false, completed: false }));
const reports = [];
async function findReport(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await findReport(path);
    else if (entry.name === 'brick-browser.json') reports.push(path);
  }
}
await findReport(join(input, 'test-results'));
assert.equal(reports.length, 1, 'exactly one brick browser receipt is required');
assert.equal(expected.size, 81);
// Other material tests legitimately reuse preset/channel names. Only the files
// beside the unique brick receipt belong to this matrix; siblings cannot fill gaps.
for (const name of expected) {
  await copyFile(join(dirname(reports[0]), name), join(destination, name));
}
const browser = JSON.parse(await readFile(join(destination, 'brick-browser.json')));
assert.equal(browser.ok, true);
assert.deepEqual(browser.build, receipt.build);
assert.equal(browser.rows.length, 20);
const nativeDirectory = join(destination, 'native');
const result = spawnSync('cargo', ['test', '--release', '--locked', '--all-features', '--manifest-path', 'examples/native-consumer/Cargo.toml', '--target-dir', 'target/native-consumer', '--test', 'brick_material', 'brick_material_public_gpu_matrix', '--', '--ignored', '--nocapture'], {
  encoding: 'utf8', env: { ...process.env, MIXTURE_BRICK_FIXTURE_DIR: fixture,
    MIXTURE_BRICK_BROWSER_DIR: destination, MIXTURE_BRICK_EVIDENCE_DIR: nativeDirectory },
});
await writeFile(join(destination, 'stdout.log'), result.stdout ?? '');
await writeFile(join(destination, 'stderr.log'), result.stderr ?? '');
if (result.error) throw result.error;
assert.equal(result.status, 0, 'native brick comparison failed; inspect retained receipt/logs');
const native = JSON.parse(await readFile(join(nativeDirectory, 'native-matrix.json')));
assert.equal(native.ok, true);
assert.equal(native.completed, true);
assert.equal(native.debugAssertions, false);
assert.equal(native.browserCompared, true);
assert.equal(native.cases.length, 20);
for (const row of native.timing) assert.equal(row.budgetPassed, true, 'recorded adapter must pass a frozen timing budget');
for (const row of native.cases) {
  const matching = browser.rows.filter(b => b.case === row.case && JSON.stringify(b.size) === JSON.stringify(row.size));
  assert.equal(matching.length, 1);
  assert.equal(matching[0].planHash, row.execution.planHash);
  assert.equal(row.browserComparison.length, 4);
  for (const channel of row.browserComparison) assert.ok(channel.maxComponentDelta <= 1);
}
await copyFile(join(input, 'qualification.json'), join(destination, 'browser-qualification.json'));
await writeFile(join(destination, 'comparison.json'), JSON.stringify({ ok: true, completed: true,
  consumerRevision: receipt.consumerRevision, build: receipt.build, comparisons: 80,
  native: 'native/native-matrix.json', browser: 'brick-browser.json', materialAccepted: false }, null, 2));
console.log(`Brick Native/browser comparison passed: ${destination}`);
