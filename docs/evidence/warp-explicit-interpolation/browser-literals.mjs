import {readFileSync,writeFileSync} from 'node:fs';
import {createServer} from 'node:http';
import {pathToFileURL} from 'node:url';
const {chromium}=await import(pathToFileURL(process.env.MIXTURE_PROBE_PRODUCT+'/node_modules/@playwright/test/index.mjs'));
const [endpoint,casesPath,shaderPath,out]=process.argv.slice(2);
const cases=JSON.parse(readFileSync(casesPath)).cases,code=readFileSync(shaderPath,'utf8');
const server=createServer((q,s)=>s.end('<!doctype html><title>Warp literal regression</title>'));await new Promise(r=>server.listen(4191,'127.0.0.1',r));
const browser=await chromium.connectOverCDP(endpoint),page=await browser.contexts()[0].newPage();
try {
 await page.goto('http://127.0.0.1:4191');
 const report=await page.evaluate(async({cases,code})=>{
 const a=await navigator.gpu.requestAdapter({powerPreference:'high-performance'}),d=await a.requestDevice();
 const result={userAgent:navigator.userAgent,adapter:{...a.info.toJSON?.(),vendor:a.info.vendor,architecture:a.info.architecture},cases:[]};
 // Only encode/decode storage; all pixel arithmetic runs in production WGSL.
 const fv=new Float32Array(1),uv=new Uint32Array(fv.buffer);
 function half(v){fv[0]=v;const b=uv[0],sign=(b>>>16)&32768,exp=((b>>>23)&255)-127+15,m=b&8388607;if(exp<=0){if(exp < -10)return sign;const shift=14-exp,mm=m|8388608;return sign|((mm+(1<<(shift-1))-1+((mm>>>shift)&1))>>>shift);}return sign|(((b&2147483647)+4095+((b>>>13)&1)-939524096)>>>13);}
 function number(h){const sign=(h&32768)?-1:1,e=(h>>>10)&31,m=h&1023;return sign*(e===0?m*2**-24:e===31?(m?NaN:Infinity):(1+m/1024)*2**(e-15));}
 try {
 d.pushErrorScope('validation');const module=d.createShaderModule({code});const p=await d.createComputePipelineAsync({layout:'auto',compute:{module,entryPoint:'warp'}});
 for(const c of cases){const [w,h]=c.size,row=Math.ceil(w*8/256)*256;const textures=[];
 function tex(values){const t=d.createTexture({size:[w,h],format:'rgba16float',usage:GPUTextureUsage.TEXTURE_BINDING|GPUTextureUsage.STORAGE_BINDING|GPUTextureUsage.COPY_DST|GPUTextureUsage.COPY_SRC});textures.push(t);if(values){const buf=new Uint16Array(w*h*4);values.forEach((v,i)=>{buf[i*4]=half(v);buf[i*4+3]=half(1)});d.queue.writeTexture({texture:t},buf,{bytesPerRow:w*8,rowsPerImage:h},[w,h]);}return t;}
 const input=tex(c.input),field=tex(c.displacement),output=tex();const u=d.createBuffer({size:16,usage:GPUBufferUsage.UNIFORM|GPUBufferUsage.COPY_DST});d.queue.writeBuffer(u,0,new Float32Array([...c.parameters.strength,0,0]));
 const bg=d.createBindGroup({layout:p.getBindGroupLayout(0),entries:[{binding:0,resource:{buffer:u}},{binding:1,resource:output.createView()},{binding:2,resource:input.createView()},{binding:3,resource:field.createView()}]});
 const read=d.createBuffer({size:row*h,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});const e=d.createCommandEncoder(),pass=e.beginComputePass();pass.setPipeline(p);pass.setBindGroup(0,bg);pass.dispatchWorkgroups(Math.ceil(w/8),Math.ceil(h/8));pass.end();e.copyTextureToBuffer({texture:output},{buffer:read,bytesPerRow:row,rowsPerImage:h},[w,h]);d.queue.submit([e.finish()]);await read.mapAsync(GPUMapMode.READ);const v=new DataView(read.getMappedRange()),actual=[];for(let y=0;y<h;y++)for(let x=0;x<w;x++)actual.push(Array.from({length:4},(_,k)=>number(v.getUint16(y*row+x*8+k*2,true))));
 const failures=[];actual.forEach((rgba,i)=>{if(rgba.some((v,k)=>v!==[c.expectedScalar[i],0,0,1][k]))failures.push({pixel:i,expected:c.expectedScalar[i],actual:rgba});});result.cases.push({case:c.case,size:c.size,pass:failures.length===0,failures,actualRgba:actual});read.unmap();read.destroy();u.destroy();textures.forEach(t=>t.destroy());
 }
 const err=await d.popErrorScope();if(err)throw Error(err.message);result.ok=result.cases.every(c=>c.pass);return result;
 }finally{d.destroy();}
 },{cases,code});writeFileSync(out,JSON.stringify(report,null,2));console.log(JSON.stringify({ok:report.ok,cases:report.cases.length,failures:report.cases.filter(c=>!c.pass).map(c=>c.case)}));if(!report.ok)process.exitCode=1;
}finally{await page.close();await browser.close();server.closeAllConnections();server.close();}
