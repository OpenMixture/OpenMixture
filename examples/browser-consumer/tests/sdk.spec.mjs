import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

// Expected identity comes from the installed package, independently of the browser bundle.
const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
const source = await readFile(new URL('../public/input.mix', import.meta.url), 'utf8');

async function openHost(page) {
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
}

test('inert import and CPU APIs use the exact public package without GPU acquisition', async ({ page }) => {
  const requests = [];
  page.on('request', request => requests.push(request.url()));
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'gpu', { configurable: true, value: {
      requestAdapter() { throw new Error('CPU operations must not acquire a GPU'); },
    } });
  });
  await openHost(page);
  expect(requests.filter(url => url.endsWith('.wasm'))).toHaveLength(0);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    const validation = runtime.validate(source);
    const changed = runtime.inspect(source, { size: [65, 3], channels: ['roughness'], overrides: { roughness: 0.75 } });
    return { build: runtime.getBuildInfo(), valid: validation.ok, catalog: runtime.getNodeCatalog().map(c => c.typeId),
      size: changed.plan.size, channels: changed.plan.outputs.map(o => o.channel),
      roughness: changed.exposedParameters.find(p => p.id === 'roughness').effectiveValue,
      nullablePort: runtime.getNodeCatalog().find(c => c.typeId === 'material-output').inputs.find(p => p.id === 'baseColor').default,
      dimensionType: typeof changed.plan.size[0], parameterType: typeof changed.exposedParameters.find(p => p.id === 'roughness').effectiveValue,
      bytesType: typeof changed.plan.estimates.peakBytes };
  }, source);
  expect(result.build).toEqual(expectedBuild);
  expect(result.valid).toBe(true);
  expect(result.catalog).toContain('checker');
  expect(result).toMatchObject({ size: [65, 3], channels: ['roughness'], roughness: 0.75, bytesType: 'bigint', nullablePort: null, dimensionType: 'number', parameterType: 'number' });
  const wasm = requests.filter(url => url.endsWith('.wasm'));
  expect(wasm).toHaveLength(1);
  expect(new URL(wasm[0]).pathname).toMatch(/^\/consumer\/assets\/.*\.wasm$/);
  expect(requests.every(url => new URL(url).origin === 'http://127.0.0.1:4175')).toBe(true);
});

test('invalid source and overrides retain structured diagnostics before GPU creation', async ({ page }) => {
  await openHost(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    const invalid = runtime.validate('{');
    const override = runtime.validate(source, { overrides: { roughness: 2 } });
    let acquired = false;
    try {
      await window.sdk.renderMaterial({ ...runtime, createGpu: async () => {
        acquired = true; throw new Error('must not acquire');
      } }, '{', {});
      throw new Error('invalid source accepted');
    } catch (error) {
      return { invalid, override, acquired, code: error.code, typed: error instanceof window.sdk.MixtureRuntimeError };
    }
  }, source);
  expect(result.acquired).toBe(false);
  expect(result.typed).toBe(true);
  for (const validation of [result.invalid, result.override]) {
    expect(validation.ok).toBe(false);
    expect(validation.diagnostics[0]).toMatchObject({ code: expect.stringMatching(/^MIX_/), stage: expect.any(String), message: expect.any(String) });
  }
  expect(result.code).toBe(result.invalid.diagnostics[0].code);
});

test('65×3 public render has exact checker/scalar bytes and survives helper destruction', async ({ page, browser }, testInfo) => {
  await openHost(page);
  const rendered = await page.evaluate(async () => {
    const runtime = await window.sdk.loadRuntime();
    const bytes = await window.sdk.readMaterial(new URL('/consumer/input.mix', location.href));
    const { build, inspection, result } = await window.sdk.renderMaterial(runtime, bytes, {
      size: [65, 3], channels: ['baseColor', 'roughness'], overrides: { frequency: 4, roughness: 0.25 },
    });
    return { build, planHash: inspection.plan.hash, resultHash: result.plan.hash,
      adapter: result.report.adapter, allocations: result.report.allocations,
      projection: { passCount: typeof result.report.passCount, hits: typeof result.report.pipelineCache.hits,
        entries: typeof result.report.pipelineCache.entries, readbackBytes: typeof result.report.readbackBytes,
        pixels: result.channels.every(c => c.pixels instanceof Uint8Array) },
      channels: result.channels.map(c => ({ ...c, pixels: [...c.pixels] })) };
  });
  expect(rendered.build).toEqual(expectedBuild);
  expect(rendered.projection).toEqual({ passCount: 'bigint', hits: 'number', entries: 'bigint', readbackBytes: 'bigint', pixels: true });
  expect(rendered.resultHash).toBe(rendered.planHash);
  expect(rendered.channels.map(c => c.channel)).toEqual(['baseColor', 'roughness']);
  expect(rendered.channels.map(c => c.encoding)).toEqual(['rgba8-srgb', 'rgba8-linear']);
  for (const channel of rendered.channels) {
    expect(channel.size).toEqual([65, 3]);
    expect(channel.pixels).toHaveLength(65 * 3 * 4);
    // Analytic expectations for the fixed checker and constant, not a material renderer.
    for (let y = 0; y < 3; y++) for (let x = 0; x < 65; x++) {
      const value = channel.channel === 'roughness' ? 64 : Math.floor(x * 4 / 65) % 2 * 255;
      expect(channel.pixels.slice((y * 65 + x) * 4, (y * 65 + x) * 4 + 4)).toEqual([value, value, value, 255]);
    }
  }
  await testInfo.attach('render-evidence', { body: JSON.stringify({ ...rendered,
    browser: browser.version(), channel: testInfo.project.use.channel }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2),
    contentType: 'application/json' });
});

test('owned outputs remain unchanged through later renders and repeated destruction', async ({ page }) => {
  await openHost(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    const gpu = await runtime.createGpu();
    try {
      const first = await gpu.render(source, { size: [5, 3], channels: ['roughness'] });
      const owned = first.channels[0].pixels;
      const before = [...owned];
      const second = await gpu.render(source, { size: [5, 3], channels: ['roughness'], overrides: { roughness: 0.75 } });
      const secondValue = second.channels[0].pixels[0];
      second.channels[0].pixels.fill(0);
      await gpu.destroy();
      await gpu.destroy();
      return { before, after: [...owned], secondValue };
    } finally { await gpu.destroy(); }
  }, source);
  expect(result.after).toEqual(result.before);
  expect(result.before[0]).toBe(64);
  expect(result.secondValue).toBe(191);
});

test('busy and closing runtimes reject while accepted work settles', async ({ page }) => {
  await openHost(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    const gpu = await runtime.createGpu();
    const code = promise => promise.then(() => 'unexpected success', error => error.code);
    try {
      const accepted = gpu.render(source, { size: [128, 128] });
      const busy = code(gpu.render(source));
      const destroyed = gpu.destroy();
      const sameDestroy = destroyed === gpu.destroy();
      const closing = code(gpu.render(source));
      const pixels = (await accepted).channels[0].pixels;
      await destroyed;
      return { busy: await busy, closing: await closing, sameDestroy, length: pixels.length };
    } finally { await gpu.destroy(); }
  }, source);
  expect(result).toEqual({ busy: 'MIX_BROWSER_RUNTIME_BUSY', closing: 'MIX_BROWSER_RUNTIME_DESTROYED', sameDestroy: true, length: 128 * 128 * 4 });
});

test('GPU unavailability rejects explicitly while validation remains usable', async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(navigator, 'gpu', { configurable: true, value: undefined }));
  await openHost(page);
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    try { await runtime.createGpu(); return { unexpected: true }; }
    catch (error) { return { code: error.code, operation: error.operation, typed: error instanceof window.sdk.MixtureRuntimeError,
      valid: runtime.validate(source).ok }; }
  }, source);
  expect(result).toEqual({ code: 'MIX_BROWSER_WEBGPU_UNAVAILABLE', operation: 'createGpu', typed: true, valid: true });
});

test('example UI renders from static non-root assets and releases its runtime', async ({ page }, testInfo) => {
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('./');
  await page.getByRole('button', { name: 'Render material' }).click();
  await expect(page.getByRole('status')).toContainText('GPU runtime destroyed');
  await expect(page.locator('canvas')).toHaveCount(2);
  await expect(page.locator('#report')).toContainText(expectedBuild.buildId);
  expect(errors).toEqual([]);
  await page.screenshot({ path: testInfo.outputPath('example.png'), fullPage: true });
  await page.getByLabel('Base color', { exact: true }).uncheck();
  await page.getByRole('button', { name: 'Render material' }).click();
  await expect(page.getByRole('status')).toContainText('GPU runtime destroyed');
  await expect(page.locator('canvas')).toHaveCount(1);
  await expect(page.locator('figcaption')).toContainText('roughness');
});

test('normal build excludes the qualification host', async () => {
  // The producer runs the normal build as well as the qualification build.
  const { existsSync } = await import('node:fs');
  expect(existsSync(new URL('../dist/index.html', import.meta.url))).toBe(true);
  expect(existsSync(new URL('../dist/tests/contracts.html', import.meta.url))).toBe(false);
});

test('scalar composition executes in the candidate and fails explicitly in the published runtime', async ({ page, browser }, testInfo) => {
  test.setTimeout(120000);
  await openHost(page);
  const fixture = expectedBuild.runtimeVersion === '0.6.0-alpha.0' ? 'scalar-blend-v2.mix' : 'scalar-blend.mix';
  const source = await readFile(new URL(`../public/${fixture}`, import.meta.url), 'utf8');
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime();
    const build = runtime.getBuildInfo();
    if (build.runtimeVersion === '0.1.0-alpha.0') return { build, validation: runtime.validate(source) };
    const rows = [], outputs = [];
    for (const weight of [0, 0.25, 0.5, 1]) {
      const request = { size: [1024, 1024], channels: ['height', 'normal'], overrides: { detailWeight: weight } };
      const { inspection, result } = await window.sdk.renderMaterial(runtime, source, request);
      if (weight === 0 || weight === 1) {
        const direct = JSON.parse(source);
        for (const edge of direct.edges) if (edge.from.nodeId === 'combine') edge.from.nodeId = weight === 0 ? 'a' : 'b';
        const reference = (await window.sdk.renderMaterial(runtime, JSON.stringify(direct), request)).result;
        for (let c = 0; c < 2; c++) if (result.channels[c].pixels.some((v,i) => v !== reference.channels[c].pixels[i])) throw Error('endpoint mismatch');
      }
      const pixels = result.channels.find(c=>c.channel==='height').pixels;
      let min = 255, max = 0, interior = 0, seam = 0;
      for (let y=0;y<1024;y++) { for(let x=0;x<1024;x++) {
        const offset=(y*1024+x)*4, v=pixels[offset];min=Math.min(min,v);max=Math.max(max,v);
        if(x>0)interior+=Math.abs(v-pixels[offset-4]); if(y>0)interior+=Math.abs(v-pixels[offset-4096]);
      } seam+=Math.abs(pixels[y*4096]-pixels[y*4096+4092])+Math.abs(pixels[y*4]-pixels[1023*4096+y*4]); }
      const seamRatio=(seam/2048)/(interior/(2*1024*1023));
      const changed = outputs.length ? pixels.reduce((n,v,i)=>n+(v!==outputs.at(-1)[i]),0)/pixels.length : null;
      outputs.push(pixels);
      const images=result.channels.map(c=>{const canvas=document.createElement('canvas');canvas.width=1024;canvas.height=1024;canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(c.pixels),1024,1024),0,0);return {channel:c.channel,png:canvas.toDataURL('image/png').split(',')[1]};});
      rows.push({weight,images,planHash:inspection.plan.hash,range:[min,max],seamRatio,changed,adapter:result.report.adapter,
        hashes:await Promise.all(result.channels.map(async c=>({channel:c.channel,sha256:[...new Uint8Array(await crypto.subtle.digest('SHA-256',c.pixels))].map(v=>v.toString(16).padStart(2,'0')).join('')})))});
    }
    return {build, rows, owned: outputs.every(p=>p.length===1024*1024*4)};
  }, source);
  expect(result.build).toEqual(expectedBuild);
  if(expectedBuild.runtimeVersion==='0.1.0-alpha.0') {
    expect(result.validation.ok).toBe(false);
    expect(result.validation.diagnostics.map(d=>d.code)).toContain('MIX_NODE_UNKNOWN_TYPE');
  } else {
    expect(['0.2.0-alpha.0','0.3.0-alpha.0','0.6.0-alpha.0']).toContain(expectedBuild.runtimeVersion);
    expect(result.owned).toBe(true); expect(result.rows).toHaveLength(4);
    for(const row of result.rows) { expect(row.range[1]-row.range[0]).toBeGreaterThan(20);expect(row.seamRatio).toBeLessThan(2);if(row.changed!==null)expect(row.changed).toBeGreaterThan(0.1); }
    expect(new Set(result.rows.map(r=>r.planHash)).size).toBe(4);
  }
  for(const row of result.rows ?? []) {
    for(const image of row.images) await writeFile(testInfo.outputPath('scalar-'+row.weight+'-'+image.channel+'.png'),Buffer.from(image.png,'base64'));
    delete row.images;
  }
  await testInfo.attach('scalar-evidence', {body:JSON.stringify({...result,browser:browser.version()},(_key,value)=>typeof value==='bigint'?value.toString():value,2),contentType:'application/json'});
});


test('brick height and normals render through the public candidate; old registry rejects the type', async ({page},testInfo)=>{
  test.setTimeout(120000);
  await openHost(page);
  const source=await readFile(new URL('../public/brick-pattern.mix',import.meta.url),'utf8');
  const result=await page.evaluate(async source=>{
    const runtime=await window.sdk.loadRuntime(),build=runtime.getBuildInfo();
    if(build.runtimeVersion!=='0.6.0-alpha.0')return {build,validation:runtime.validate(source)};
    const rows=[];
    for(const [id,size,overrides] of [['offset',[1024,1024],{}],['aligned',[1024,1024],{layout:'aligned'}],['wide',[1024,1024],{gap:0.2}],['rect',[65,3],{columns:1,rows:1,layout:'aligned'}]]){
      const {inspection,result}=await window.sdk.renderMaterial(runtime,source,{size,channels:['height','normal'],overrides});
      const images=result.channels.map(c=>{const canvas=document.createElement('canvas');canvas.width=size[0];canvas.height=size[1];canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(c.pixels),...size),0,0);return {channel:c.channel,png:canvas.toDataURL('image/png').split(',')[1]};});
      const height=result.channels.find(c=>c.channel==='height').pixels;
      let lo=255,hi=0;for(let i=0;i<height.length;i+=4){lo=Math.min(lo,height[i]);hi=Math.max(hi,height[i]);}
      rows.push({id,size,planHash:inspection.plan.hash,range:[lo,hi],images,adapter:result.report.adapter});
    }
    return {build,rows};
  },source);
  expect(result.build).toEqual(expectedBuild);
  if(result.rows){
    expect(result.rows).toHaveLength(4);
    for(const row of result.rows){expect(row.range).toEqual([0,255]);for(const image of row.images)await writeFile(testInfo.outputPath('brick-'+row.id+'-'+image.channel+'.png'),Buffer.from(image.png,'base64'));delete row.images;}
  }else{expect(result.validation.ok).toBe(false);expect(result.validation.diagnostics.map(d=>d.code)).toContain('MIX_NODE_UNKNOWN_TYPE');}
  await writeFile(testInfo.outputPath('brick-evidence.json'),JSON.stringify(result,(_key,value)=>typeof value==='bigint'?value.toString():value,2));
});
