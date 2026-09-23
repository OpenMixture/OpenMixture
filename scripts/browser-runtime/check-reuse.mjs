// Bind the actual public 2K render, timing and comparison to a clean candidate.
import assert from 'node:assert/strict';
import { readFile, readdir, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { join, resolve, dirname } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';

assert.equal(process.argv.length, 4, 'usage: check-reuse.mjs <candidate-qualification> <fresh-output>');
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
    else if (entry.name === 'reuse-browser.json') matches.push(path);
  }
}
await find(join(input, 'test-results'));
assert.equal(matches.length, 1);
const browser = JSON.parse(await readFile(matches[0]));
assert.equal(browser.ok, true);
assert.deepEqual(browser.build, receipt.build);
assert.deepEqual(browser.rows.map(r => r.id), ['odd', 'intact', 'default-1k', 'default-2k']);
await mkdir(destination);
await writeFile(join(destination, 'comparison.json'), '{"ok":false,"completed":false}');
await copyFile(matches[0], join(destination, 'reuse-browser.json'));
const channels = ['baseColor', 'height', 'metallic', 'normal', 'roughness'];
for (const row of browser.rows) {
  assert.deepEqual(row.files.map(f => f.channel).sort(), channels);
  for (const file of row.files) {
    assert.equal(file.name, `${row.id}-${file.channel}.png`);
    await copyFile(join(dirname(matches[0]), file.name), join(destination, file.name));
  }
}
const result = spawnSync('cargo', ['test', '--release', '--locked', '--all-features', '--manifest-path', 'examples/native-consumer/Cargo.toml', '--target-dir', 'target/native-consumer', '--test', 'texture_reuse', '--', '--ignored', '--nocapture'], {
  encoding: 'utf8', env: { ...process.env, MIXTURE_REUSE_ROOT: resolve('.'), MIXTURE_REUSE_BROWSER: destination, MIXTURE_REUSE_EVIDENCE: join(destination, 'native') },
});
await writeFile(join(destination, 'stdout.log'), result.stdout ?? '');
await writeFile(join(destination, 'stderr.log'), result.stderr ?? '');
if (result.error) throw result.error;
assert.equal(result.status, 0, 'public reuse check failed; inspect retained native receipt/logs');
const native = JSON.parse(await readFile(join(destination, 'native/native.json')));
assert.equal(native.ok, true); assert.equal(native.completed, true);
assert.equal(native.debugAssertions, false); assert.equal(native.browserCompared, true);
assert.equal(native.rows.length, 4); assert.equal(native.timing.length, 2);
for (const row of native.timing) assert.equal(row.budgetPassed, true, 'matched frozen timing budget required');
for (const row of native.rows) {
  assert.equal(row.comparisons.length, 5);
  for (const c of row.comparisons) assert.ok(c.maxComponentDelta <= 1);
}
await copyFile(join(input, 'qualification.json'), join(destination, 'browser-qualification.json'));
await writeFile(join(destination, 'comparison.json'), JSON.stringify({ ok: true, completed: true, comparisons: 20,
  revision: receipt.consumerRevision, build: receipt.build, beforeExact: native.beforeExact,
  native: 'native/native.json', browser: 'reuse-browser.json', materialAccepted: false }, null, 2));
console.log(`Public texture reuse qualification passed: ${destination}`);
