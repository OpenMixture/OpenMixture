// Public packaged CPU API: no generated binding imports, GPU or asset URL lookup.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';

const directory=resolve(process.argv[2]??'target/browser-runtime/package');
const {loadRuntime,MixtureRuntimeError}=await import(pathToFileURL(join(directory,'src/index.js')));
const runtime=await loadRuntime({wasm:new Uint8Array(await readFile(join(directory,'wasm/mixture_wasm_bg.wasm')))});
const root=resolve('fixtures/packages/mixpack-v1');
const cases=JSON.parse(await readFile(join(root,'cases.json'),'utf8'));
for(const row of cases){
  const bytes=new Uint8Array(await readFile(join(root,row.file)));
  if(row.expectedCode)assert.throws(()=>runtime.inspectPackage(bytes),error=>error instanceof MixtureRuntimeError&&error.code===row.expectedCode,row.file);
  else {
    const result=runtime.inspectPackage(bytes);
    assert.equal(result.packageSha256,row.sha256);assert.equal(result.packageBytes,BigInt(bytes.length));
    assert.equal(result.buffers.jsPackageBytes,BigInt(bytes.length));assert.equal(result.buffers.rustPackageBytes,BigInt(bytes.length));
    assert.deepEqual(runtime.inspectPackage(bytes,{packageLimits:{packageBufferBytes:result.buffers.chargedBytes}}),result);
    assert.throws(()=>runtime.inspectPackage(bytes,{packageLimits:{packageBufferBytes:result.buffers.chargedBytes-1n}}),e=>e.code==='MIX_PACKAGE_LIMIT_EXCEEDED');
  }
}
const measurements=[];
// Optional CLI-authored normal/maximum archives exercise actual large WASM copies.
for(const file of process.argv.slice(3)){
  const bytes=new Uint8Array(await readFile(resolve(file)));const report=runtime.inspectPackage(bytes);
  assert.equal(report.buffers.jsPackageBytes,BigInt(bytes.length));assert.equal(report.buffers.rustPackageBytes,BigInt(bytes.length));
  assert.throws(()=>runtime.inspectPackage(bytes,{packageLimits:{packageBufferBytes:report.buffers.chargedBytes-1n}}),e=>e.code==='MIX_PACKAGE_LIMIT_EXCEEDED');
  measurements.push({file,report});
}
const receipt={build:runtime.getBuildInfo(),corpusCases:cases.length,gpuAcquired:false,measurements};
await mkdir(resolve('tmp'),{recursive:true});
await writeFile(resolve('tmp/m6b04-wasm-assets.json'),JSON.stringify(receipt,(_k,v)=>typeof v==='bigint'?v.toString():v,2)+'\n');
console.log(`Public WASM package inspection passed: ${cases.length} corpus cases, ${measurements.length} large copy ledgers.`);
