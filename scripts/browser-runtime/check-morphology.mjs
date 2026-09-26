// Exact public Native/browser morphology comparison; neither adapter implements pixels.
import assert from 'node:assert/strict';
import { readFile, readdir, mkdir, writeFile, copyFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
const args = process.argv.slice(2);
assert.equal(args.length, 2, 'usage: check-morphology.mjs <candidate-qualification> <fresh-output>');
const [input, output] = args.map(p => resolve(p));
const receipt = JSON.parse(await readFile(join(input, 'qualification.json')));
assert.equal(receipt.ok, true);
assert.equal(receipt.mode, 'candidate');
assert.equal(receipt.build.runtimeVersion, '0.8.0-alpha.0');
assert.equal(receipt.consumerRevision, execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim());
await mkdir(output);
await writeFile(join(output, 'comparison.json'), JSON.stringify({ ok: false, status: 'incomplete' }));
const matches = [];
async function collect(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await collect(path);
    else if (entry.name === 'morphology-evidence.json') matches.push(path);
  }
}
await collect(join(input, 'test-results'));
assert.equal(matches.length, 1);
const browserBytes = await readFile(matches[0]);
const browser = JSON.parse(browserBytes);
assert.deepEqual(browser.build, receipt.build);
assert.equal(browser.rows.length, 192);
const browserPath = join(output, 'browser.json');
await writeFile(browserPath, browserBytes);
const nativePath = join(output, 'native.json');
const result = spawnSync('cargo', ['test', '--manifest-path', 'examples/native-consumer/Cargo.toml', '--locked', '--all-features', '--target-dir', 'target/native-consumer', '--test', 'morphology', '--', '--ignored', '--nocapture'], { encoding: 'utf8', env: { ...process.env, MIXTURE_MORPHOLOGY_BROWSER: browserPath, MIXTURE_MORPHOLOGY_EVIDENCE: nativePath } });
await writeFile(join(output, 'stdout.log'), result.stdout ?? '');
await writeFile(join(output, 'stderr.log'), result.stderr ?? '');
if (result.error) throw result.error;
assert.equal(result.status, 0, 'morphology Native/browser comparison failed; inspect retained output');
const native = JSON.parse(await readFile(nativePath));
assert.equal(native.ok, true);
assert.equal(native.browserCompared, true);
assert.equal(native.maxComponentDelta, 0);
assert.equal(native.rows.length, 192);
await copyFile(join(input, 'qualification.json'), join(output, 'browser-qualification.json'));
await writeFile(join(output, 'comparison.json'), JSON.stringify({ ok: true, cases: 192, maxComponentDelta: 0, sourceRevision: receipt.consumerRevision, browserBuild: receipt.build, nativeAdapter: native.adapter }, null, 2) + '\n');
console.log(`Morphology Native/browser comparison passed: ${output}`);
