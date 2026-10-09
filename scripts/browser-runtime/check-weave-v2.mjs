// Exact public Native/browser weave comparison; neither adapter implements pixels.
import assert from 'node:assert/strict';
import { readFile, readdir, mkdir, writeFile, copyFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
const args = process.argv.slice(2);
assert.equal(args.length, 2, 'usage: check-weave-v2.mjs <candidate-qualification> <fresh-output>');
const [input, output] = args.map(p => resolve(p));
const receipt = JSON.parse(await readFile(join(input, 'qualification.json')));
assert.equal(receipt.ok, true);
assert.equal(receipt.mode, 'candidate');
assert.equal(receipt.build.runtimeVersion, '0.10.0-alpha.0');
assert.equal(receipt.consumerRevision, execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim());
await mkdir(output);
await writeFile(join(output, 'comparison.json'), JSON.stringify({ ok: false, status: 'incomplete' }));
const matches = [];
async function collect(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await collect(path);
    else if (entry.name === 'weave-v2-evidence.json') matches.push(path);
  }
}
await collect(join(input, 'test-results'));
assert.equal(matches.length, 1);
const browserBytes = await readFile(matches[0]);
const browser = JSON.parse(browserBytes);
assert.deepEqual(browser.build, receipt.build);
assert.equal(browser.rows.length, 18);
const browserPath = join(output, 'browser.json');
await writeFile(browserPath, browserBytes);
for (const row of browser.rows) {
  assert.match(row.file, /^weave-v2-[0-9]+\.rgba$/);
  const pixels = await readFile(join(dirname(matches[0]), row.file));
  assert.equal(pixels.length, row.size[0] * row.size[1] * 4);
  await writeFile(join(output, row.file), pixels);
}
const nativePath = join(output, 'native.json');
const result = spawnSync('cargo', ['test', '--manifest-path', 'examples/native-consumer/Cargo.toml', '--locked', '--all-features', '--target-dir', 'target/native-consumer', '--test', 'weave_v2', '--', '--ignored', '--nocapture'], { encoding: 'utf8', env: { ...process.env, MIXTURE_WEAVE_V2_BROWSER: browserPath, MIXTURE_WEAVE_V2_EVIDENCE: nativePath } });
await writeFile(join(output, 'stdout.log'), result.stdout ?? '');
await writeFile(join(output, 'stderr.log'), result.stderr ?? '');
if (result.error) throw result.error;
assert.equal(result.status, 0, 'weave Native/browser comparison failed; inspect retained output');
const native = JSON.parse(await readFile(nativePath));
assert.equal(native.ok, true);
assert.equal(native.browserCompared, true);
assert.ok(native.maxComponentDelta <= 1);
assert.equal(native.rows.length, 18);
await copyFile(join(input, 'qualification.json'), join(output, 'browser-qualification.json'));
await writeFile(join(output, 'comparison.json'), JSON.stringify({ ok: true, cases: 18, maxComponentDelta: native.maxComponentDelta, sourceRevision: receipt.consumerRevision, browserBuild: receipt.build, nativeAdapter: native.adapter }, null, 2) + '\n');
console.log(`Weave Native/browser comparison passed: ${output}`);
