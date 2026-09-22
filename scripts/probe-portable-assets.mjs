// M6B-01 entry evidence. CPU-only; not a package implementation or pixel renderer.
// Build the baseline CLI first. The destination must not already exist.
import assert from 'node:assert/strict';
import {execFileSync, spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import {resolve, join} from 'node:path';
import {platform, release, arch} from 'node:os';

const [destination, binary] = process.argv.slice(2);
assert.ok(destination, 'Usage: node scripts/probe-portable-assets.mjs <fresh-output> [baseline-cli]');
const root = resolve(import.meta.dirname, '..');
const out = resolve(destination);
const cli = resolve(binary ?? join(root, 'target/debug', platform() === 'win32' ? 'mixture.exe' : 'mixture'));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const git = (...args) => execFileSync('git', args, {cwd:root, encoding:'utf8'}).trim();
const source = await readFile(join(root, 'fixtures/nodes/image-input/height.mix'));
await mkdir(out); // Refuse stale evidence, never overwrite a previous result.
const sourcePath = join(out, 'material.mix');
await writeFile(sourcePath, source);
const run = args => {
 const result = spawnSync(cli, args, {encoding:'utf8', cwd:root});
 if (result.error) throw result.error;
 assert.equal(result.signal, null);
 return {args, exitCode:result.status, report:JSON.parse(result.stdout), stderr:result.stderr};
};
const validation = run(['validate', sourcePath, '--json']);
assert.equal(validation.exitCode, 0);
const inspect = output => run(['inspect', sourcePath, '--plan', '--size','1024','--output',output,'--json']);
const baseColor = inspect('baseColor');
assert.equal(baseColor.exitCode, 0);
const missing = inspect('height,normal');
assert.equal(missing.exitCode, 2);
assert.equal(missing.report.diagnostics[0].code, 'MIX_RESOURCE_MISSING');
// Synthetic linear RGBA8 resource input: no material evaluation is performed.
const pixels = Buffer.alloc(1024*1024*4);
for (let y=0;y<1024;y++) for(let x=0;x<1024;x++) {
 const offset=(y*1024+x)*4;
 pixels[offset]=(x+y)&255; pixels[offset+3]=255;
}
await writeFile(join(out,'heightSource.rgba'),pixels);
const adjacent = inspect('height,normal');
assert.equal(adjacent.exitCode, 2);
assert.equal(adjacent.report.diagnostics[0].code, 'MIX_RESOURCE_MISSING');
const resource = {id:'heightSource',width:1024,height:1024,format:'rgba8-linear',bytesPerRow:4096,dataBase64:pixels.toString('base64')};
// Representation-size experiment only. This shape is NOT a selected public format.
const experiment = Buffer.from(JSON.stringify({document:source.toString('utf8'),resources:[resource]}));
const receipt = {
 schemaVersion:1, kind:'M6B-01-entry-measurement-not-format-or-GPU-acceptance',
 completedAt:new Date().toISOString(), sourceRevision:git('rev-parse','HEAD'),
 dirty:Boolean(git('status','--porcelain')), node:process.version,
 host:{platform:platform(),release:release(),arch:arch()},
 probeSha256:digest(await readFile(import.meta.filename)), cliSha256:digest(await readFile(cli)),
 sourceSha256:digest(source), resourceSha256:digest(pixels),
 validation, baseColor, missing, adjacent,
 sizes:{sourceBytes:source.length,packedResourceBytes:pixels.length,
  sourceAndResourceBytes:source.length+pixels.length,base64PayloadBytes:resource.dataBase64.length,
  experimentalJsonBytes:experiment.length,experimentalJsonSha256:digest(experiment),
  base64ExpansionRatio:resource.dataBase64.length/pixels.length},
 conclusion:'Source validates and resource-free slicing works. Copying .mix alone or next to raw pixels cannot recover the binding. An explicit portable binding/identity contract is missing; this does not prove a binary container or compression is necessary.'
};
await writeFile(join(out,'measurement.json'),JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify({ok:true,sizes:receipt.sizes,output:join(out,'measurement.json')},null,2));
