import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';

const bytes = [...await readFile(new URL('../public/asset.mixpack',import.meta.url))];
const source = await readFile(new URL('../public/asset.mix',import.meta.url),'utf8');
async function host(page) { await page.goto('tests/contracts.html');await page.waitForFunction(()=>Boolean(window.sdk)); }

test('package inspection stays CPU-only and measures both bounded transfer copies',async({page})=>{
  await host(page);
  const result=await page.evaluate(async bytes=>{
    let gpuCalls=0;
    const descriptor=Object.getOwnPropertyDescriptor(Navigator.prototype,'gpu');
    Object.defineProperty(Navigator.prototype,'gpu',{configurable:true,get(){gpuCalls++;throw Error('CPU must not touch GPU');}});
    try {
      const runtime=await window.sdk.loadRuntime();
      const padded=new Uint8Array(bytes.length+7);padded.set(bytes,3);const input=padded.subarray(3,3+bytes.length);
      const first=runtime.inspectPackage(input);const again=runtime.inspectPackage(input,{packageLimits:{packageBufferBytes:first.buffers.chargedBytes}});
      const codes=[];
      for(const options of [{packageLimits:{packageBufferBytes:first.buffers.chargedBytes-1n}},{packageLimits:{packageBytes:BigInt(bytes.length-1)}},{packageLimits:{manifestBytes:0n}}]){
        try{runtime.inspectPackage(input,options);throw Error('Accepted under-budget package');}catch(e){codes.push(e.code);}
      }
      padded.fill(0);let rejected;
      try{runtime.inspectPackage(input);}catch(e){rejected=e.code;}
      return {gpuCalls,first,again,codes,rejected,type:typeof first.resources[0].byteLength};
    }finally{if(descriptor)Object.defineProperty(Navigator.prototype,'gpu',descriptor);else delete Navigator.prototype.gpu;}
  },bytes);
  expect(result.gpuCalls).toBe(0);expect(result.first).toEqual(result.again);expect(result.type).toBe('bigint');
  expect(result.first.buffers.jsPackageBytes).toBe(BigInt(bytes.length));expect(result.first.buffers.rustPackageBytes).toBe(BigInt(bytes.length));
  expect(result.codes).toEqual(Array(3).fill('MIX_PACKAGE_LIMIT_EXCEEDED'));expect(result.rejected).toBe('MIX_PACKAGE_INVALID');
});

test('public package rendering captures before yielding and shares rejection recovery and destroy',async({page},testInfo)=>{
  await host(page);
  const result=await page.evaluate(async({bytes,source})=>{
    const runtime=await window.sdk.loadRuntime();const asset=runtime.inspectPackage(new Uint8Array(bytes));
    const image={id:'Input',width:65,height:3,format:'rgba8-linear',bytesPerRow:260,data:new Uint8Array(780)};
    for(let i=0;i<image.data.length;i+=4)image.data.set([128,37,91,255],i);
    const request={size:[65,3],channels:['height']};const loose=runtime.inspect(source,{...request,resources:[image]});
    const gpu=await runtime.createGpu();const failures=[];
    for(const options of [{...request,overrides:{source:'Input'}},{...request,packageLimits:{packageBufferBytes:asset.buffers.chargedBytes+780n-1n}}]) {
      try{await gpu.renderPackage(new Uint8Array(bytes),options);throw Error('Expected package rejection');}catch(e){failures.push(e.code);}
    }
    const input=new Uint8Array(bytes),options={...request,size:[65,3],packageLimits:{packageBufferBytes:asset.buffers.chargedBytes+780n}};
    const pending=gpu.renderPackage(input,options);input.fill(0);options.size[0]=1;
    let busy;try{await gpu.renderPackage(new Uint8Array());}catch(e){busy=e.code;}
    const destroy=gpu.destroy(),sameDestroy=destroy===gpu.destroy();let closing;
    try{await gpu.renderPackage(new Uint8Array());}catch(e){closing=e.code;}
    const output=await pending;await destroy;
    return {failures,busy,closing,sameDestroy,hash:output.plan.hash,looseHash:loose.plan.hash,
      pixels:[...output.channels[0].pixels],adapter:output.report.adapter,allocations:output.report.allocations,
      package:asset.packageSha256,buffers:asset.buffers};
  },{bytes,source});
  expect(result.failures).toEqual(['MIX_PACKAGE_RESOURCE_OVERRIDE','MIX_PACKAGE_LIMIT_EXCEEDED']);
  expect(result.busy).toBe('MIX_BROWSER_RUNTIME_BUSY');expect(result.closing).toBe('MIX_BROWSER_RUNTIME_DESTROYED');expect(result.sameDestroy).toBe(true);
  expect(result.hash).toBe(result.looseHash);expect(result.pixels).toHaveLength(780);
  for(let i=0;i<780;i+=4)expect(result.pixels.slice(i,i+4)).toEqual([128,128,128,255]);
  delete result.pixels;
  await testInfo.attach('asset-adapter-evidence',{body:JSON.stringify(result,(_k,v)=>typeof v==='bigint'?v.toString():v,2),contentType:'application/json'});
});
