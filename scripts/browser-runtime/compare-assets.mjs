// Cross-backend check of the M6B-05 fixture through the independent public Rust consumer.
import assert from 'node:assert/strict';
import {verifyAssetMatrix} from './asset-matrix.mjs';
import {readFile, readdir, mkdir, copyFile, writeFile} from 'node:fs/promises';
import {join, resolve} from 'node:path';
import {execFileSync, spawnSync} from 'node:child_process';
const [input, destination] = process.argv.slice(2).map(p=>resolve(p));
assert.ok(input && destination, 'usage: compare-assets.mjs <candidate-qualification> <fresh-output>');
const receipt=JSON.parse(await readFile(join(input,'qualification.json')));
assert.equal(receipt.ok,true);assert.equal(receipt.mode,'candidate');
assert.equal(receipt.build.runtimeVersion,'0.7.0-alpha.0');
assert.equal(receipt.consumerRevision,execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim());
await mkdir(destination);const expected=new Set(['1024x1024','65x3'].flatMap(size=>[0,0.25,0.5,1].flatMap(w=>['height','normal'].map(c=>`asset-${size}-${w}-${c}.png`))));
expected.add('asset-evidence.json');
const seen=new Set();
async function collect(directory){for(const entry of await readdir(directory,{withFileTypes:true})){
 const path=join(directory,entry.name);if(entry.isDirectory())await collect(path);
 else if(expected.has(entry.name)){assert.equal(seen.has(entry.name),false);seen.add(entry.name);const bytes=await readFile(path);await writeFile(join(destination,entry.name),bytes);assert.deepEqual(await readFile(join(destination,entry.name)),bytes,'copied asset evidence differs');}
}}
await collect(join(input,'test-results'));assert.equal(seen.size,17);
await writeFile(join(destination,'comparison.json'),JSON.stringify({ok:false,status:'incomplete'}));
const result=spawnSync('cargo',['test','--manifest-path','examples/native-consumer/Cargo.toml','--locked','--all-features','--target-dir','target/native-consumer','--test','asset_qualification','--','--ignored','--nocapture'],{encoding:'utf8',env:{...process.env,MIXTURE_ASSET_BROWSER_DIR:destination,MIXTURE_ASSET_EVIDENCE:join(destination,'native.json')}});
await writeFile(join(destination,'stdout.log'),result.stdout??'');await writeFile(join(destination,'stderr.log'),result.stderr??'');
if(result.error)throw result.error;assert.equal(result.status,0,'asset comparison failed; inspect retained output');
const browser=JSON.parse(await readFile(join(destination,'asset-evidence.json')));
assert.equal(browser.build.buildId,receipt.build.buildId);
const report=JSON.parse(await readFile(join(destination,'native.json')));assert.equal(report.ok,true);assert.equal(report.cases.length,8);
verifyAssetMatrix(report,browser,receipt);
await copyFile(join(input,'qualification.json'),join(destination,'browser-qualification.json'));
await writeFile(join(destination,'comparison.json'),JSON.stringify({...report,browserBuild:receipt.build,sourceRevision:receipt.consumerRevision},null,2)+'\n');
console.log(`Asset native/browser comparison passed: ${destination}`);
