// Cross-backend check of the ENG-04 fixture through the independent public Rust consumer.
import assert from 'node:assert/strict';
import {readFile, readdir, mkdir, copyFile, writeFile} from 'node:fs/promises';
import {join, resolve} from 'node:path';
import {execFileSync, spawnSync} from 'node:child_process';
const [input, destination] = process.argv.slice(2).map(p=>resolve(p));
assert.ok(input && destination, 'usage: check-scalar.mjs <candidate-qualification> <fresh-output>');
const receipt=JSON.parse(await readFile(join(input,'qualification.json')));
assert.equal(receipt.ok,true);assert.equal(receipt.mode,'candidate');
assert.equal(receipt.build.runtimeVersion,'0.4.0-alpha.0');
assert.equal(receipt.consumerRevision,execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim());
await mkdir(destination);const expected=new Set([0,0.25,0.5,1].flatMap(w=>['height','normal'].map(c=>`scalar-${w}-${c}.png`)));
const seen=new Set();
async function collect(directory){for(const entry of await readdir(directory,{withFileTypes:true})){
 const path=join(directory,entry.name);if(entry.isDirectory())await collect(path);
 else if(expected.has(entry.name)){assert.equal(seen.has(entry.name),false);seen.add(entry.name);await copyFile(path,join(destination,entry.name));}
}}
await collect(join(input,'test-results'));assert.equal(seen.size,8);
await writeFile(join(destination,'comparison.json'),JSON.stringify({ok:false,status:'incomplete'}));
const result=spawnSync('cargo',['test','--manifest-path','examples/native-consumer/Cargo.toml','--locked','--all-features','--target-dir','target/native-consumer','--test','scalar_blend','--','--ignored','--nocapture'],{encoding:'utf8',env:{...process.env,MIXTURE_SCALAR_BROWSER_DIR:destination,MIXTURE_SCALAR_EVIDENCE:join(destination,'comparison.json')}});
await writeFile(join(destination,'stdout.log'),result.stdout??'');await writeFile(join(destination,'stderr.log'),result.stderr??'');
if(result.error)throw result.error;assert.equal(result.status,0,'scalar comparison failed; inspect retained output');
const report=JSON.parse(await readFile(join(destination,'comparison.json')));assert.equal(report.ok,true);assert.equal(report.cases.length,4);
for(const row of report.cases){assert.equal(row.browserComparison.length,2);for(const channel of row.browserComparison)assert.ok(channel.maxComponentDelta<=1);}
await copyFile(join(input,'qualification.json'),join(destination,'browser-qualification.json'));
console.log(`Scalar native/browser comparison passed: ${destination}`);
