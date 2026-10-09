import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir, writeFile, copyFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { wovenFabricMatrix, wovenFabricStress } from './woven-fabric-requests.mjs';
export const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const root = new URL('../',import.meta.url);
const read = p => readFile(new URL(p,root));
export function replaceOnce(source, anchor, replacement) {
  assert.equal(source.split(anchor).length,2,`unique preview anchor required: ${anchor}`);
  return source.replace(anchor,replacement);
}
// Reuse the untouched MAT-01 dielectric renderer, with review-only layout/settings.
export function wovenHtml(original) {
  let s=original.replaceAll('MAT-01','MAT-03');
  for (const [a,b] of [
    ['.views,.channels{display:flex;gap:16px}', '.views,.channels{display:flex;gap:16px;flex-wrap:wrap}'],
    ['sphere: f32, repeats: f32, padding: vec2<f32>', 'sphere: f32, repeats: f32, zoom: f32, forcedRoughness: f32'],
    ['*2.5;', '*2.5/settings.zoom;'],
    ['let r=clamp(textureSampleLevel(roughness,samp,uv,0.).r,0.04,1.);','let r=clamp(select(textureSampleLevel(roughness,samp,uv,0.).r,settings.forcedRoughness,settings.forcedRoughness>=0.),0.04,1.);'],
    ['for(const [sphere,repeats] of [[0,1],[0,3],[1,1],[1,3]]){', 'for(const [sphere,repeats,zoom,forced] of [...config.views.map(v=>[v.shape===\'sphere\'?1:0,v.tiling,v.zoom,-1]),...(variant===config.cases[0]?[[1,1,1,0],[1,1,1,1],[1,1,1,0]]:[])]){'],
    ['canvas.width=600;canvas.height=600;', 'canvas.width=600;canvas.height=600;if(forced>=0)canvas.className="roughness-probe";'],
    ["section.querySelector('.views').append(figure);", "caption.textContent+=` · ${zoom}× close-up`;if(forced>=0){figure.className='sanity';document.querySelector('main').append(figure);}else section.querySelector('.views').append(figure);"],
    ['[sphere,repeats,0,0]', '[sphere,repeats,zoom,forced]'],
    ["['baseColor','normal','roughness','height']", "['baseColor','normal','roughness','metallic','height']"],
    ['const owned=[];', `const owned=[];
    // Verify the displayed Native PNGs against producer-bound browser raw hashes.
    for(const item of config.inputs){
      const decode=async url=>{const image=await createImageBitmap(await(await fetch(url)).blob(),{colorSpaceConversion:'none'});if(image.width!==1024||image.height!==1024)throw Error('dimensions');const c=new OffscreenCanvas(1024,1024);const x=c.getContext('2d');x.drawImage(image,0,0);image.close();return x.getImageData(0,0,1024,1024).data;};
      const native=await decode('/maps/'+item.name),browser=await decode('/browser/'+item.name);
      const sha=[...new Uint8Array(await crypto.subtle.digest('SHA-256',browser))].map(x=>x.toString(16).padStart(2,'0')).join('');
      if(sha!==item.browserPixelSha256)throw Error('browser pixel identity '+item.name);
      let max=0;for(let i=0;i<native.length;i++){max=Math.max(max,Math.abs(native[i]-browser[i]));if(item.channel==='metallic'&&native[i]!== (i%4===3?255:0))throw Error('non-dielectric metallic');}
      if(max>1)throw Error('input parity '+item.name);item.maxComponentDelta=max;
    }`],
    ['window.previewResult={ok:true,adapter:', 'window.previewResult={ok:true,inputChecks:config.inputs,adapter:']
  ]) s=replaceOnce(s,a,b);
  return s;
}
export function validateReceipts(comparison,native,browser,qualification,manifest) {
  for(const r of [comparison,native,browser,qualification])assert.equal(r.ok,true);
  for(const r of [comparison,native,browser])assert.equal(r.completed,true);
  assert.equal(comparison.build.engineDirty,false);assert.equal(comparison.build.engineRevision,comparison.revision);
  assert.equal(qualification.mode,'candidate');assert.equal(qualification.consumerDirty,false);
  assert.equal(native.browserCompared,true);assert.equal(manifest.workingTreeStatus,'');
  assert.equal(manifest.sourceRevision,comparison.revision);assert.equal(qualification.consumerRevision,comparison.revision);
  assert.deepEqual(native.requestManifest,manifest);assert.deepEqual(browser.manifest,manifest);
  assert.deepEqual(browser.build,comparison.build);assert.deepEqual(qualification.build,comparison.build);
  assert.equal(browser.packageSha256,qualification.wovenPackageSha256);
  assert.equal(hash(JSON.stringify(manifest,null,2)+'\n'),qualification.wovenRequestsSha256);
  assert.deepEqual(manifest.rows,wovenFabricMatrix());assert.deepEqual(manifest.stress,wovenFabricStress());
  const ids=[...manifest.rows,...manifest.stress].map(x=>x.id);
  assert.deepEqual(native.rows.map(x=>x.id),ids);assert.deepEqual(browser.rows.map(x=>x.id),ids);
  for(const [i,n] of native.rows.entries()) {
    const b=browser.rows[i];assert.deepEqual(n.size,b.size);assert.equal(n.planHash,b.planHash);assert.deepEqual(n.overrides,b.overrides);
    for(const row of [n,b])for(const flag of ['repeatExact','packageExact','slicedExact','ownedAfterDestroy'])assert.equal(row[flag],true);
    assert.equal(n.comparisons.length,5);for(const c of n.comparisons)assert.ok(Number.isInteger(c.maxComponentDelta)&&c.maxComponentDelta<=1&&c.maxComponentDelta>=0);
  }
  assert.equal(native.timing.length,4);// Software-adapter budgets are advisory; the adapter must still match.
  for(const t of native.timing)assert.ok(t.budgetPolicy==='software'?t.matchedAdapter===true:t.budgetPassed===true);
}
export async function main(args) {
  assert.equal(args.length,2,'usage: woven-material-preview.mjs <woven-comparison> <fresh-output>');
  const [input,output]=args.map(p=>resolve(p));
  const receiptHashes={};const json=async p=>{const b=await readFile(join(input,p));receiptHashes[p]=hash(b);return JSON.parse(b);};
  const comparison=await json('comparison.json'),native=await json('native/native.json'),browserInput=await json('woven-browser.json'),qualification=await json('browser-qualification.json'),manifest=await json('requests.json');
  validateReceipts(comparison,native,browserInput,qualification,manifest);
  for(const [field,p] of [['sourceSha256','fixtures/materials/woven-fabric/material.mix'],['qualificationPlanSha256','fixtures/materials/woven-fabric/qualification-plan.json'],['builderSha256','scripts/woven-fabric-requests.mjs']])assert.equal(hash(await read(p)),manifest[field],field);
  assert.equal(hash(await readFile(join(input,'material.mix'))),manifest.sourceSha256);
  assert.equal(hash(await readFile(join(input,'qualification-plan.json'))),manifest.qualificationPlanSha256);
  const planBytes=await read('fixtures/materials/woven-fabric/pbr-review-plan.json');const plan=JSON.parse(planBytes);
  assert.equal(plan.status,'frozen');assert.equal(plan.materialAccepted,false);assert.equal(plan.humanAccepted,false);
  const revision=execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim();
  assert.equal(execFileSync('git',['status','--porcelain'],{encoding:'utf8'}).trim(),'','preview must use clean source');
  assert.equal(revision,comparison.revision,'generate from the committed producer/tooling revision');
  const maps=new Map(),hashes={},inputs=[];
  for(const variant of plan.cases)for(const channel of plan.channels){
    const name=`${variant}-1024x1024-${channel}.png`;
    for(const [prefix,folder] of [['maps','native'],['browser','']]){
      const bytes=await readFile(join(input,folder,name));assert.equal(bytes.subarray(0,8).toString('hex'),'89504e470d0a1a0a');assert.equal(bytes.readUInt32BE(16),1024);assert.equal(bytes.readUInt32BE(20),1024);
      hashes[join(folder,name).replaceAll('\\','/')]=hash(bytes);maps.set(`/${prefix}/${name}`,bytes);
    }
    const row=browserInput.rows.find(r=>r.id===`${variant}-1024x1024`);assert.ok(row);
    const file=row.files.find(f=>f.channel===channel);assert.equal(file.name,name);
    inputs.push({name,channel,browserPixelSha256:file.sha256});
  }
  const original=await read('scripts/brick-material-preview.html'),html=Buffer.from(wovenHtml(original.toString()));
  await mkdir(output);await writeFile(join(output,'preview.json'),JSON.stringify({ok:false,completed:false,humanAccepted:false,materialAccepted:false}));
  const server=createServer((req,res)=>{const bytes=req.url==='/'?html:req.url==='/config'?Buffer.from(JSON.stringify({cases:plan.cases,views:plan.views,inputs})):maps.get(req.url);res.writeHead(bytes?200:404,{'content-type':req.url==='/'?'text/html':req.url==='/config'?'application/json':'image/png'});res.end(bytes??'missing');});
  await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser;
  try {
    const require=createRequire(new URL('../examples/browser-consumer/package.json',import.meta.url));const {chromium}=require('playwright');
    const browserChannel=process.env.MIXTURE_BROWSER_CHANNEL??'chrome',browserArgs=['--enable-unsafe-webgpu','--ignore-gpu-blocklist'];
    browser=await chromium.launch({channel:browserChannel,headless:true,args:browserArgs});const page=await browser.newPage({viewport:{width:1320,height:1000},deviceScaleFactor:1});
    await page.goto(`http://127.0.0.1:${server.address().port}/`);await page.waitForFunction(()=>window.previewResult,{},{timeout:120000});
    const result=await page.evaluate(()=>window.previewResult);assert.equal(result.ok,true,result.error);assert.deepEqual(result.cases,plan.cases);
    const probes=page.locator('canvas.roughness-probe');assert.equal(await probes.count(),3);const bytes=[];for(let i=0;i<3;i++)bytes.push(await probes.nth(i).screenshot());
    assert.deepEqual(bytes[0],bytes[2]);assert.notDeepEqual(bytes[0],bytes[1]);
    const screenshots={};for(const id of plan.cases)screenshots[`${id}-pbr.png`]=hash(await page.locator(`#${id}`).screenshot({path:join(output,`${id}-pbr.png`)}));
    for(let i=0;i<3;i++){const name=`roughness-sanity-${i}.png`;await writeFile(join(output,name),bytes[i]);screenshots[name]=hash(bytes[i]);}
    for(const f of Object.keys(receiptHashes))await copyFile(join(input,f),join(output,f.replaceAll('/','-')));
    await writeFile(join(output,'renderer.html'),html);await writeFile(join(output,'pbr-review-plan.json'),planBytes);
    const index=`<!doctype html><meta charset="utf-8"><title>Woven fabric review</title><style>body{font:16px system-ui;background:#15191f;color:#edf0f3;margin:28px}img{max-width:100%;height:auto}section{margin:36px 0}</style><h1>Woven fabric — human review pending</h1><p>Fixed dielectric GGX views. Height shown, not displaced. Human and material acceptance remain false. Stage D is separate.</p>${plan.cases.map(id=>`<section><h2>${id}</h2><img src="${id}-pbr.png" alt="${id} plane, sphere, tiling, close-ups and channels"></section>`).join('')}`;
    await writeFile(join(output,'index.html'),index);
    await writeFile(join(output,'preview.json'),JSON.stringify({ok:true,completed:true,humanAccepted:false,materialAccepted:false,producerRevision:comparison.revision,producerBuild:comparison.build,producerManifest:manifest,previewRevision:revision,previewDirty:false,browser:browser.version(),browserChannel,browserArgs,...result,settings:plan,inputReceiptSha256:receiptHashes,inputPngSha256:hashes,screenshotSha256:screenshots,rendererSha256:{brickHtml:hash(original),generatedHtml:hash(html),runner:hash(await readFile(new URL(import.meta.url)))},reviewPlanSha256:hash(planBytes),indexSha256:hash(index),roughnessCheck:{roughnessZero:hash(bytes[0]),roughnessOne:hash(bytes[1]),repeat:hash(bytes[2]),changed:true,repeatExact:true},shading:'unmodified MAT-01 dielectric GGX light model; woven views only, no graph execution or displacement'},null,2)+'\n');
    console.log(`Woven PBR sheets: ${output}`);
  }finally{if(browser)await browser.close();await new Promise(r=>server.close(r));}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)await main(process.argv.slice(2));
