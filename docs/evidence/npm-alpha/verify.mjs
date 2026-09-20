import assert from 'node:assert/strict';
import {readFile, mkdir, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {join, dirname, resolve} from 'node:path';
const root=fileURLToPath(new URL('.',import.meta.url));
const old=resolve(root,'../alpha-04');
const json=async p=>JSON.parse((await readFile(p,'utf8')).replace(/^\uFEFF/,''));
const hash=b=>createHash('sha256').update(b).digest('hex');
for(const [file,expected]of Object.entries(await json(join(root,'files.json')))){
 assert.ok(!file.startsWith('/')&&!file.split('/').includes('..'));
 const b=await readFile(join(root,file));assert.equal(b.length,expected.bytes);assert.equal(hash(b),expected.sha256,file);
}
execFileSync(process.execPath,[join(old,'verify.mjs')],{stdio:'inherit'});
const registry=await json(join(root,'registry-at-publication.json'));
const lock=await json(join(root,'windows-lock.json'));
assert.equal(lock.packages[''].dependencies['@openmixture/runtime'],'0.1.0-alpha.0');
assert.equal(lock.packages['node_modules/@openmixture/runtime'].resolved,registry.dist.tarball);
assert.equal(lock.packages['node_modules/@openmixture/runtime'].integrity,registry.dist.integrity);
const producer=await json(join(old,'producer.json'));
const oldIndex=await json(join(old,'evidence-index.json'));
const index=await json(join(root,'replay-index.json'));
for(const entry of index){
 const [dataset,...parts]=entry.path.split('/');assert.ok(['windows','linux'].includes(dataset));assert.ok(!parts.includes('..'));
 if(entry.source==='alpha-04-bundle'){
  const original=oldIndex.datasets[dataset].find(e=>e.file===parts.join('/'));
  assert.equal(original.sha256,entry.sha256);assert.equal(original.bytes,entry.bytes);
 }else assert.equal(hash(await readFile(join(root,entry.source))),entry.sha256);
}
for(const env of ['windows','linux']){
 const report=await json(join(root,`replay/${env}/receipt.json`));
 assert.equal(report.productRevision,'87ded9351e1c426e03aa7fb2b4c641f32399b85b');
 assert.equal(report.clean,true);assert.equal(report.build.buildId,producer.buildId);
 assert.equal(report.archiveSha256,producer.sha256);
 const browser=await json(join(root,`${env}-browser.json`));assert.equal(browser.stats.expected,52);
 for(const key of ['skipped','unexpected','flaky'])assert.equal(browser.stats[key],0);
 const comparison=index.find(e=>e.path===`${env}/comparison.json`);
 const comparisonPath=comparison.source==='alpha-04-bundle'?join(old,`${env}-comparison.json`):join(root,comparison.source);
 const result=await json(comparisonPath);assert.equal(result.ok,true);assert.equal(result.cases.length,7);
}
assert.equal((await json(join(root,'ordinary/receipt.json'))).ok,true);
if(process.argv[2]){
 const target=resolve(process.argv[2]);
 execFileSync(process.execPath,[join(old,'verify.mjs'),target],{stdio:'inherit'});
 for(const entry of index){
  const out=join(target,entry.path);
  if(entry.source!=='alpha-04-bundle'){await mkdir(dirname(out),{recursive:true});await copyFile(join(root,entry.source),out);}
  assert.equal(hash(await readFile(out)),entry.sha256,entry.path);
 }
 console.log(`Restored registry-consumer replay at ${target}`);
}
console.log('Published archive identity and Windows/Linux registry-consumer evidence verified.');
