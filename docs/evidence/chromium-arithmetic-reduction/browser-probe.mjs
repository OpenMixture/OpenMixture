import {pathToFileURL} from 'node:url';
import {resolve,join} from 'node:path';
const {chromium}=await import(pathToFileURL(join(resolve(process.env.MIXTURE_PROBE_PRODUCT), 'node_modules/@playwright/test/index.mjs')));
import {readFileSync,writeFileSync} from 'node:fs';
import {createServer} from 'node:http';
const [endpoint,shader,out,countArg,inputPath]=process.argv.slice(2);
const server=createServer((req,res)=>{res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Arithmetic diagnostic</title>');});
await new Promise(r=>server.listen(4189,'127.0.0.1',r));
const browser=await chromium.connectOverCDP(endpoint);const page=await browser.contexts()[0].newPage();
try {
 await page.goto('http://127.0.0.1:4189');
 const result=await page.evaluate(async({code,count,inputBytes})=>{
  const adapter=await navigator.gpu.requestAdapter({powerPreference:'high-performance'});if(!adapter)throw Error('No adapter');
  const device=await adapter.requestDevice();device.pushErrorScope('validation');
  const input=device.createBuffer({size:count*4,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_DST});
  if(inputBytes)device.queue.writeBuffer(input,0,new Uint8Array(inputBytes));
  const output=device.createBuffer({size:count*4,usage:GPUBufferUsage.STORAGE|GPUBufferUsage.COPY_SRC});
  const staging=device.createBuffer({size:count*4,usage:GPUBufferUsage.COPY_DST|GPUBufferUsage.MAP_READ});
  const module=device.createShaderModule({code});
  const info=await module.getCompilationInfo();if(info.messages.some(x=>x.type==='error'))throw Error(JSON.stringify(info.messages));
  const pipeline=await device.createComputePipelineAsync({layout:'auto',compute:{module,entryPoint:'probe'}});
  const group=device.createBindGroup({layout:pipeline.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:input}},{binding:1,resource:{buffer:output}}]});
  const encoder=device.createCommandEncoder();const pass=encoder.beginComputePass();pass.setPipeline(pipeline);pass.setBindGroup(0,group);pass.dispatchWorkgroups(Math.ceil(count/64));pass.end();encoder.copyBufferToBuffer(output,0,staging,0,count*4);device.queue.submit([encoder.finish()]);
  await staging.mapAsync(GPUMapMode.READ);const data=Array.from(new Uint32Array(staging.getMappedRange()));staging.unmap();
  const error=await device.popErrorScope();if(error)throw Error(error.message);
  const context={userAgent:navigator.userAgent,adapter:{vendor:adapter.info.vendor,architecture:adapter.info.architecture,device:adapter.info.device,description:adapter.info.description},diagnosticOnly:true};
  input.destroy();output.destroy();staging.destroy();device.destroy();return {data,context};
 },{code:readFileSync(shader,'utf8'),count:Number(countArg),inputBytes:inputPath?Array.from(readFileSync(inputPath)):null});
 const buffer=Buffer.alloc(result.data.length*4);result.data.forEach((x,i)=>buffer.writeUInt32LE(x,i*4));writeFileSync(out,buffer);writeFileSync(out+'.json',JSON.stringify(result.context,null,2));
}finally{await page.close();await browser.close();server.closeAllConnections();server.close();}
