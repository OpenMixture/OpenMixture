import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

test('weave candidate preserves all structural modes through public WebGPU', async ({ page, browser }, testInfo) => {
  test.setTimeout(240_000);
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.10.0-alpha.0');
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const identity = await page.evaluate(async () => {
    const runtime = await window.sdk.loadRuntime(), gpu = await runtime.createGpu();
    window.weave = { runtime, gpu };
    return { build: runtime.getBuildInfo(), adapter: gpu.context, catalog: runtime.getNodeCatalog().map(n => [n.typeId, n.version]) };
  });
  expect(identity.build).toEqual(expectedBuild);
  expect(identity.catalog).toEqual([
    ['blend',1],['brick-pattern',1],['checker',1],['constant-color',1],['constant-scalar',1],
    ['fractal-noise',2],['gradient-map',1],['height-to-normal',1],['image-input',1],['levels',1],
    ['material-output',1],['scalar-blend',1],['scalar-mask-blend',1],['scalar-morphology',1],
    ['scalar-subtract',1],['transform-2d',1],['warp',1],['weave-pattern',2],
  ]);
  const rows = [];
  try {
    for (const [name, size, geometry] of [
      ['default-256',[256,256],{}],['default-1024',[1024,1024],{}],['default-2048',[2048,2048],{}],
      ['unequal',[257,129],{warpCount:12,weftCount:8,warpWidth:0.55,weftWidth:0.9}],
      ['low',[256,256],{warpCount:4,weftCount:4,warpWidth:0.55,weftWidth:0.55,bevel:0.02,crown:0,underRatio:0.25}],
      ['high',[256,256],{warpCount:32,weftCount:32,warpWidth:0.9,weftWidth:0.9,bevel:0.12,crown:1,underRatio:0.75}],
    ]) for (const mode of ['height','coverage','warp-share']) {
      const result = await page.evaluate(async ({size, geometry, mode}) => {
        const edge = (a,ap,b,bp) => ({from:{nodeId:a,portId:ap},to:{nodeId:b,portId:bp}});
        const source = JSON.stringify({version:1,nodes:[{id:'weave',type:'weave-pattern',version:2},{id:'color',type:'constant-color',version:1},{id:'out',type:'material-output',version:1}],edges:[edge('weave','value','out','height'),edge('color','color','out','baseColor')],exposedParameters:['mode','warpCount','weftCount','warpWidth','weftWidth','bevel','crown','underRatio'].map(id=>({id,nodeId:'weave',parameterId:id}))});
        const request = {size,channels:['height'],overrides:{...geometry,mode}};
        const result = await window.weave.gpu.render(source,request), repeat = await window.weave.gpu.render(source,request);
        const pixels = result.channels[0].pixels;
        for (let i=0;i<pixels.length;i++) if(pixels[i]!==repeat.channels[0].pixels[i]) throw Error('weave repeat differs');
        if(result.report.allocations.liveBytes!==0n) throw Error('retained allocations');
        let binary=''; for(let i=0;i<pixels.length;i+=16384) binary+=String.fromCharCode(...pixels.subarray(i,i+16384));
        return {planHash:result.plan.hash, base64:btoa(binary)};
      }, {size,geometry,mode});
      const file = 'weave-v2-'+rows.length+'.rgba';
      const bytes = Buffer.from(result.base64,'base64');
      expect(bytes.length).toBe(size[0]*size[1]*4);
      await writeFile(testInfo.outputPath(file),bytes);
      rows.push({case:name,size,mode,planHash:result.planHash,file,repeatExact:true});
    }
  } finally { await page.evaluate(async () => { await window.weave.gpu.destroy(); delete window.weave; }); }
  expect(rows).toHaveLength(18);
  const content=JSON.stringify({...identity,rows,browser:browser.version()},(_key,value)=>typeof value==='bigint'?value.toString():value,2);
  await writeFile(testInfo.outputPath('weave-v2-evidence.json'),content);
  await testInfo.attach('weave-v2-evidence',{body:content,contentType:'application/json'});
});
