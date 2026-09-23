import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
const fixture = ['0.6.0-alpha.0', '0.7.0-alpha.0'].includes(expectedBuild.runtimeVersion) ? 'image-input-v2.mix' : 'image-input.mix';
const source = await readFile(new URL(`../public/${fixture}`, import.meta.url), 'utf8');
async function host(page) {
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
}

test('resource CPU APIs retain Core diagnostics, content identity and typed catalog', async ({ page }) => {
  await host(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    const data = new Uint8Array([0,11,22,0,64,33,44,1,128,55,66,2,255,77,88,3]);
    const image = { id: 'heightSource', width: 2, height: 2, format: 'rgba8-linear', bytesPerRow: 8, data };
    const request = { size: [2,2], channels: ['height'], resources: [image] };
    const a = runtime.inspect(source, request);
    const again = runtime.validate(source, request);
    data[3] = 255;
    const changed = runtime.inspect(source, request);
    const cases = [
      { resources: [] }, { resources: [image, image] }, { resources: [{ ...image, id: 'unknown' }] },
      { resources: [{ ...image, format: 'rgba8-srgb' }] }, { resources: [{ ...image, width: 1 }] },
      { resources: [{ ...image, bytesPerRow: 16 }] }, { resources: [{ ...image, data: data.subarray(1) }] },
      { resourceLimits: { resourceCount: 0n } }, { resourceLimits: { resourcePixels: 3n } },
      { resourceLimits: { resourceBytes: 15n } }, { overrides: { heightSource: '../invalid' } },
    ].map(change => runtime.validate(source, { ...request, ...change }));
    const sliced = runtime.inspect(source, { size: [2,2], channels: ['baseColor'] });
    return { hash: a.plan.hash, again: again.plan.hash, changed: changed.plan.hash,
      estimateType: typeof a.plan.estimates.resourceUploadBytes,
      strideType: typeof a.plan.imageResources[0].bytesPerRow,
      kind: runtime.getNodeCatalog().find(n => n.typeId === 'image-input').parameters[0].kind,
      cases: cases.map(v => ({ ok: v.ok, codes: v.diagnostics.map(d => d.code) })),
      sliced: sliced.plan.imageResources.length };
  }, source);
  expect(result.again).toBe(result.hash); expect(result.changed).not.toBe(result.hash);
  expect(result).toMatchObject({ estimateType: 'bigint', strideType: 'bigint', kind: { type: 'resourceRef' }, sliced: 0 });
  const codes = ['MIX_RESOURCE_MISSING','MIX_RESOURCE_DUPLICATE_ID','MIX_RESOURCE_UNKNOWN_ID',
    'MIX_RESOURCE_FORMAT_UNSUPPORTED','MIX_RESOURCE_SIZE_MISMATCH','MIX_RESOURCE_LENGTH_MISMATCH',
    'MIX_RESOURCE_LENGTH_MISMATCH','MIX_LIMIT_RESOURCE_COUNT_EXCEEDED','MIX_LIMIT_RESOURCE_PIXELS_EXCEEDED',
    'MIX_LIMIT_RESOURCE_BYTES_EXCEEDED'];
  result.cases.forEach((c,i) => { expect(c.ok).toBe(false); if (i < codes.length) expect(c.codes).toContain(codes[i]); });
});

test('offset views, all R codes, orientation and repeated upload cleanup use public rendering', async ({ page }) => {
  await host(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime(), gpu = await runtime.createGpu();
    const doc = JSON.parse(source);
    for (const e of doc.edges) if (e.from.nodeId === 'mix') e.from.nodeId = 'image';
    const direct = JSON.stringify(doc), rows = [];
    try {
      for (const [width,height] of [[1,1],[1,256],[256,1],[2,2],[65,3]]) {
        const backing = new Uint8Array(width*height*4+14).fill(99), data = backing.subarray(7,-7);
        for (let i=0;i<width*height;i++) data.set([width===2 ? [0,64,128,255][i] : i%256, 219, 73, 0],i*4);
        const options = { size: [width,height], channels: ['height','normal'], resources: [{ id:'heightSource',width,height,format:'rgba8-linear',bytesPerRow:width*4,data }] };
        const first = await gpu.render(direct, options);
        let exact = true;
        for (let i=0;i<data.length;i+=4) {
          const actual = first.channels.find(c=>c.channel==='height').pixels;
          exact &&= actual[i]===data[i] && actual[i+1]===data[i] && actual[i+2]===data[i] && actual[i+3]===255;
          data[i+1] ^= 255; data[i+2] ^= 255; data[i+3] ^= 255;
        }
        const changed = await gpu.render(direct, options);
        const identical = first.channels.every((c,n)=>c.pixels.every((b,i)=>b===changed.channels[n].pixels[i]));
        rows.push({ size:[width,height],exact,identical,hashChanged:first.plan.hash!==changed.plan.hash,
          allocations:changed.report.allocations, estimates:changed.plan.estimates });
      }
      return rows;
    } finally { await gpu.destroy(); }
  }, source);
  expect(result).toHaveLength(5);
  for (const row of result) {
    expect(row).toMatchObject({ exact:true, identical:true, hashChanged:true });
    expect(row.allocations.liveBytes).toBe(0n);
    expect(row.allocations.releasedBytes).toBe(row.allocations.cumulativeBytes);
    expect(row.allocations.resourceCount).toBe(1n);
    expect(row.allocations.resourceUploadBytes).toBe(BigInt(row.size[0]*row.size[1]*4));
    expect(row.allocations.peakBytes <= row.estimates.peakBytes).toBe(true);
  }
});

test('synchronous capture rejects unsafe storage and preserves active work across destroy', async ({ page }) => {
  await host(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime(), gpu = await runtime.createGpu();
    const data = new Uint8Array([0,11,22,0,64,33,44,1,128,55,66,2,255,77,88,3]);
    const image = { id:'heightSource',width:2,height:2,format:'rgba8-linear',bytesPerRow:8,data };
    const options = { size:[2,2],channels:['height'],overrides:{detailWeight:0},resources:[image] };
    const errors = [];
    const detached = new Uint8Array(16); structuredClone(detached.buffer,{transfer:[detached.buffer]});
    const invalidData = [detached,new Uint8Array(new ArrayBuffer(16,{maxByteLength:32})),new Uint16Array(8)];
    // SharedArrayBuffer is tested in Node even on hosts without cross-origin isolation.
    if (typeof SharedArrayBuffer !== 'undefined') invalidData.push(new Uint8Array(new SharedArrayBuffer(16)));
    for (const value of invalidData) {
      try { await gpu.render(source,{...options,resources:[{...image,data:value}]}); errors.push('accepted'); }
      catch(e) { errors.push(e.code); }
    }
    let reads=0; const accessor = {...image}; Object.defineProperty(accessor,'data',{get(){reads++;return data;}});
    try { await gpu.render(source,{...options,resources:[accessor]}); errors.push('accepted'); } catch(e) { errors.push(e.code); }
    const expectedHash = runtime.inspect(source, options).plan.hash;
    const active = gpu.render(source, options);
    data.fill(255); image.id='changed'; options.overrides.detailWeight=1;
    structuredClone(data.buffer,{transfer:[data.buffer]});
    let busy,closed;
    try { await gpu.render(source,{get resources(){reads++;return [];}}); } catch(e) { busy=e.code; }
    const destroy = gpu.destroy(), idempotent = destroy===gpu.destroy();
    try { await gpu.render(source,{get resources(){reads++;return [];}}); } catch(e) { closed=e.code; }
    const rendered = await active; await destroy;
    return {errors,reads,busy,closed,idempotent,expectedHash,hash:rendered.plan.hash,pixels:[...rendered.channels[0].pixels]};
  }, source);
  expect(result.errors.every(c=>c==='MIX_BROWSER_INVALID_ARGUMENT')).toBe(true);
  expect(result).toMatchObject({reads:0,busy:'MIX_BROWSER_RUNTIME_BUSY',closed:'MIX_BROWSER_RUNTIME_DESTROYED',idempotent:true});
  expect(result.hash).toBe(result.expectedHash);
  expect(result.pixels).toEqual([0,0,0,255,64,64,64,255,128,128,128,255,255,255,255,255]);
});

test('1K imported height and noise composition retains endpoint and cross-runtime evidence', async ({ page, browser }, testInfo) => {
  await host(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime(), gpu = await runtime.createGpu();
    const size=1024, data=new Uint8Array(size*size*4);
    // Frozen M6A-03 64-texel triangular tile, identical in the Native consumer.
    for(let y=0;y<size;y++) for(let x=0;x<size;x++) {
      const u=x%64,v=y%64;
      const r=(Math.min(u,63-u)*5+Math.min(v,63-v)*3)%256;data.set([r,19,201,0],(y*size+x)*4);
    }
    const resource={id:'heightSource',width:size,height:size,format:'rgba8-linear',bytesPerRow:size*4,data};
    const request={size:[size,size],channels:['height','normal'],resources:[resource]};
    const rows=[];
    try {
      const direct=async node=>{const doc=JSON.parse(source);for(const e of doc.edges)if(e.from.nodeId==='mix')e.from.nodeId=node;return gpu.render(JSON.stringify(doc),request);};
      const image=await direct('image'), noise=await direct('detail');
      let previous;
      for(const weight of [0,0.25,0.5,1]) {
        const out=await gpu.render(source,{...request,overrides:{detailWeight:weight}});
        if(out.plan.imageResources[0].contentDigest!=='aeb0e01748d9b4d25b7404498e4e6f11c5f4e247444cd28eac79504a5df4a2c2') throw Error('Frozen M6A-03 input changed');
        const reference=weight===0?image:weight===1?noise:null;
        const endpoints=reference?out.channels.every((c,n)=>c.pixels.every((v,i)=>v===reference.channels[n].pixels[i])):true;
        const height=out.channels.find(c=>c.channel==='height').pixels;
        let changed=0,min=255,max=0;
        for(let i=0;i<height.length;i+=4){min=Math.min(min,height[i]);max=Math.max(max,height[i]);if(previous&&height[i]!==previous[i])changed++;}
        const images=[];
        for(const c of out.channels){const canvas=document.createElement('canvas');canvas.width=size;canvas.height=size;
          canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(c.pixels),size,size),0,0);
          images.push({channel:c.channel,png:canvas.toDataURL('image/png').split(',')[1]});}
        rows.push({weight,endpoints,range:[min,max],changed:previous?changed/(size*size):null,planHash:out.plan.hash,allocations:out.report.allocations,images});previous=height;
      }
      return {rows,build:runtime.getBuildInfo(),adapter:gpu.context};
    } finally {await gpu.destroy();}
  }, source);
  for(const row of result.rows){expect(row.endpoints).toBe(true);expect(row.range[1]-row.range[0]).toBeGreaterThan(20);
    if(row.changed!==null)expect(row.changed).toBeGreaterThan(0.1);
    expect(row.allocations.liveBytes).toBe(0n);
    for(const image of row.images)await writeFile(testInfo.outputPath(`resource-${row.weight}-${image.channel}.png`),Buffer.from(image.png,'base64'));
    delete row.images;
  }
  const evidence=JSON.stringify({...result,browser:browser.version()},(_k,v)=>typeof v==='bigint'?v.toString():v,2);
  await writeFile(testInfo.outputPath('resource-evidence.json'),evidence);
  await testInfo.attach('resource-evidence',{body:evidence,contentType:'application/json'});
});
