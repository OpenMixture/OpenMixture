// Stage A source-bound public matrix parity; no material acceptance claim.
import assert from 'node:assert/strict';
import { wovenFabricMatrix, wovenFabricStress } from '../woven-fabric-requests.mjs';
import { readFile, readdir, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { join, resolve, dirname } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
assert.equal(process.argv.length, 4, 'usage: check-woven.mjs <candidate-qualification> <fresh-output>');
const [input, destination] = process.argv.slice(2).map(p => resolve(p));
const receipt = JSON.parse(await readFile(join(input, 'qualification.json')));
assert.equal(receipt.ok, true); assert.equal(receipt.mode, 'candidate'); assert.equal(receipt.consumerDirty, false);
assert.equal(receipt.build.runtimeVersion, '0.9.0-alpha.0'); assert.equal(receipt.build.apiSchemaVersion, 3);
assert.equal(receipt.consumerRevision, execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim());
const matches = [];
async function find(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) await find(path); else if (entry.name === 'woven-browser.json') matches.push(path);
  }
}
await find(join(input, 'test-results')); assert.equal(matches.length, 1);
const browser = JSON.parse(await readFile(matches[0]));
assert.equal(browser.ok, true); assert.equal(browser.completed, true); assert.deepEqual(browser.build, receipt.build);
assert.equal(browser.manifest.sourceRevision, receipt.consumerRevision); assert.equal(browser.manifest.workingTreeStatus, '');
assert.equal(hash(JSON.stringify(browser.manifest, null, 2) + '\n'), receipt.wovenRequestsSha256);
assert.equal(browser.packageSha256, receipt.wovenPackageSha256);
for (const [field, path] of [['sourceSha256','fixtures/materials/woven-fabric/material.mix'],['qualificationPlanSha256','fixtures/materials/woven-fabric/qualification-plan.json'],['builderSha256','scripts/woven-fabric-requests.mjs']]) assert.equal(hash(await readFile(path)), browser.manifest[field]);
assert.deepEqual(browser.manifest.rows, wovenFabricMatrix()); assert.deepEqual(browser.manifest.stress, wovenFabricStress());
const expected = [...wovenFabricMatrix(), ...wovenFabricStress()];
assert.deepEqual(browser.rows.map(r => r.id), expected.map(r => r.id));
for (const [i,row] of browser.rows.entries()) {
  assert.deepEqual(row.size, expected[i].request.size); assert.deepEqual(row.overrides, expected[i].request.overrides);
  for (const flag of ['repeatExact','packageExact','slicedExact','ownedAfterDestroy']) assert.equal(row[flag],true);
  assert.ok(row.endpoints.some(e => e.channel === 'metallic'));
  for (const endpoint of row.endpoints) assert.equal(endpoint.allPixelsExact,true);
  for (const measurement of row.downsample ?? []) if (measurement.gated) {assert.equal(measurement.passed,true);assert.ok(measurement.componentMeanError.every(v=>Number.isFinite(v)&&v<=4));}
}
assert.equal(browser.rows.flatMap(r=>r.downsample??[]).filter(m=>m.gated).length,8);
await mkdir(destination);
await writeFile(join(destination,'comparison.json'),'{"ok":false,"completed":false}');
await writeFile(join(destination,'requests.json'),JSON.stringify(browser.manifest,null,2)+'\n');
await copyFile('fixtures/materials/woven-fabric/material.mix',join(destination,'material.mix'));
await copyFile('fixtures/materials/woven-fabric/qualification-plan.json',join(destination,'qualification-plan.json'));
await copyFile(matches[0],join(destination,'woven-browser.json'));
for (const row of browser.rows) {
  assert.deepEqual(row.files.map(f=>f.channel).sort(),['baseColor','height','metallic','normal','roughness']);
  for(const file of row.files) {assert.equal(file.name,`${row.id}-${file.channel}.png`);await copyFile(join(dirname(matches[0]),file.name),join(destination,file.name));}
}
if (process.env.MIXTURE_GPU_SOFTWARE === '1') {
  const setup = await readFile('.github/scripts/setup-swiftshader.sh','utf8');
  const pinned = setup.match(/^swiftshader_revision=([a-f0-9]{40})$/m)?.[1];
  assert.ok(pinned); assert.equal(process.env.MIXTURE_SWIFTSHADER_COMMIT,pinned,'software timing requires the repository pin');
}
const result=spawnSync('cargo',['test','--release','--locked','--all-features','--manifest-path','examples/native-consumer/Cargo.toml','--target-dir','target/native-consumer','--test','woven_material','--','--ignored','--nocapture'],{encoding:'utf8',env:{...process.env,MIXTURE_WOVEN_ROOT:resolve('.'),MIXTURE_WOVEN_REQUESTS:destination,MIXTURE_WOVEN_BROWSER:destination,MIXTURE_WOVEN_EVIDENCE:join(destination,'native')}});
await writeFile(join(destination,'stdout.log'),result.stdout??'');await writeFile(join(destination,'stderr.log'),result.stderr??'');
if(result.error)throw result.error;
assert.equal(result.status,0,'frozen woven matrix failed; inspect retained native receipt/logs; do not relax gates');
const native=JSON.parse(await readFile(join(destination,'native/native.json')));
assert.equal(native.ok,true);assert.equal(native.completed,true);assert.equal(native.browserCompared,true);assert.equal(native.rows.length,51);assert.equal(native.timing.length,4);
for(const row of native.timing)assert.equal(row.budgetPassed,true);
for(const row of native.rows){assert.equal(row.comparisons.length,5);for(const c of row.comparisons)assert.ok(c.maxComponentDelta<=1);}
for(const row of native.downsample)if(row.preset!=='stress')for(const m of row.channels){assert.equal(m.passed,true);assert.ok(m.componentMeanError.every(v=>Number.isFinite(v)&&v<=4));}
await copyFile(join(input,'qualification.json'),join(destination,'browser-qualification.json'));
await writeFile(join(destination,'comparison.json'),JSON.stringify({ok:true,completed:true,comparisons:255,revision:receipt.consumerRevision,build:receipt.build,native:'native/native.json',browser:'woven-browser.json',materialAccepted:false},null,2));
console.log(`Public woven matrix passed: ${destination}`);
