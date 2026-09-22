// Controlled consumer visualization of already-rendered textures; no graph execution.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { readFile, mkdir, writeFile, copyFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
const require = createRequire(new URL('../examples/browser-consumer/package.json', import.meta.url));
const { chromium } = require('playwright');
const args=process.argv.slice(2);assert.equal(args.length,2,'usage: brick-material-preview.mjs <native-evidence> <fresh-output>');
const [input,output]=args.map(p=>resolve(p));
const native=JSON.parse(await readFile(join(input,'native-matrix.json')));
assert.equal(native.ok,true);assert.equal(native.completed,true);
const cases=[...new Set(native.cases.map(c=>c.case))];assert.equal(cases.length,5);
const hashes={};const maps=new Map();
for(const variant of cases)for(const channel of ['baseColor','normal','roughness','height']){
    const name=`${variant}-1024x1024-${channel}.png`;const bytes=await readFile(join(input,name));maps.set(`/maps/${name}`,bytes);hashes[name]=createHash('sha256').update(bytes).digest('hex');
}
await mkdir(output);await writeFile(join(output,'preview.json'),JSON.stringify({ok:false,completed:false}));
const html=await readFile(new URL('./brick-material-preview.html',import.meta.url));
const server=createServer((req,res)=>{
    const data=req.url==='/'?html:req.url==='/config'?Buffer.from(JSON.stringify({cases})):maps.get(req.url);
    res.writeHead(data?200:404,{'content-type':req.url==='/'?'text/html':req.url==='/config'?'application/json':'image/png'});res.end(data??'missing');
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
let browser;
try{
    browser=await chromium.launch({channel:process.env.MIXTURE_BROWSER_CHANNEL??'chrome',headless:true,args:['--enable-unsafe-webgpu','--ignore-gpu-blocklist']});
    const page=await browser.newPage({viewport:{width:1320,height:900},deviceScaleFactor:1});
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.waitForFunction(()=>window.previewResult,{},{timeout:120000});
    const result=await page.evaluate(()=>window.previewResult);assert.equal(result.ok,true,result.error);
    for(const variant of cases)await page.locator(`#${variant}`).screenshot({path:join(output,`${variant}-pbr.png`)});
    await page.screenshot({path:join(output,'overview.png'),fullPage:true});
    await copyFile(join(input,'native-matrix.json'),join(output,'native-matrix.json'));
    await writeFile(join(output,'preview.json'),JSON.stringify({ok:true,completed:true,humanAccepted:false,
        browser:browser.version(),...result,inputPngSha256:hashes,previewHtmlSha256:createHash('sha256').update(html).digest('hex'),
        shading:'dielectric GGX, fixed key/fill, no displacement; visualization only'},null,2));
    console.log(`Controlled PBR views: ${output}`);
}finally{if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));}
