import { test, expect } from '@playwright/test';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';

function endpointSource(preset, controls) {
  const value = preset === 'constant-high' ? 1 : 0;
  const [color, roughness, metallic, height] = [[value,value,value,1],value,0,0];
  const node = (id, type, parameters) => ({ id, type, version: 1, parameters });
  const edge = (nodeId, portId, to, input) => ({ from: { nodeId, portId }, to: { nodeId: to, portId: input } });
  const nodes = [node('color', 'constant-color', { value: color }), node('normal', 'height-to-normal', { strength: 1 }), node('out', 'material-output', {})];
  const edges = [edge('color', 'color', 'out', 'baseColor'), edge('height', 'value', 'normal', 'in'), edge('normal', 'normal', 'out', 'normal')];
  for (const [name, value] of Object.entries({ roughness, metallic, height })) {
    nodes.push(node(name, 'constant-scalar', { value })); edges.push(edge(name, 'value', 'out', name));
  }
  return JSON.stringify({ version: 1, nodes, edges });
}

test('woven fabric renders all frozen presets with repeat and package parity', async ({ page, browser }, testInfo) => {
  test.setTimeout(1_800_000);
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.9.0-alpha.0');
  const source = await readFile(new URL('../public/woven-fabric/material.mix', import.meta.url), 'utf8');
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const rows = [];
  const manifest = JSON.parse(await readFile(new URL('../public/woven-fabric/requests.json', import.meta.url)));
  expect(createHash('sha256').update(source).digest('hex')).toBe(manifest.sourceSha256);
  const packageBytes = [...await readFile(new URL('../public/woven-fabric/material.mixpack', import.meta.url))];
  expect(manifest.rows).toHaveLength(48);
  expect(manifest.stress).toHaveLength(3);
  const cases = [...manifest.rows, ...manifest.stress].map(row => ({ id: row.id, preset: row.preset ?? 'stress', qualityScope: row.qualityScope, ...row.request, endpointSource: endpointSource(row.preset, row.controls) }));
  const receipt = () => JSON.stringify({ok: rows.length === 51, completed: rows.length === 51, materialAccepted:false,manifest,packageSha256:createHash('sha256').update(Buffer.from(packageBytes)).digest('hex'),browser:browser.version(),build:expectedBuild,rows},(_key,value)=>typeof value==='bigint'?value.toString():value,2);
  await writeFile(testInfo.outputPath('woven-browser.json'),receipt());
  for (const item of cases) {
    const row = await page.evaluate(async ({ source, item, packageBytes }) => {
      const runtime = await window.sdk.loadRuntime();
      const channels = ['baseColor', 'normal', 'roughness', 'metallic', 'height'];
      const request = { size: item.size, channels, overrides: item.overrides };
      if (JSON.stringify(channels.slice().sort()) !== JSON.stringify(item.channels.slice().sort())) throw Error('request channel coverage');
      const inspected = runtime.inspect(source, request);
      if (inspected.plan.version !== 3 || typeof inspected.plan.estimates.textureCount !== 'bigint' ||
          typeof inspected.plan.estimates.logicalTextureBytes !== 'bigint') throw Error('v3 typed plan contract');
      const gpu = await runtime.createGpu();
      let result;
      let endpoints = null, downsample = null;
      const controls = [];
      try {
        result = await gpu.render(source, request);
        const repeat = await gpu.render(source, request);
        if (result.plan.hash !== inspected.plan.hash || result.plan.hash !== repeat.plan.hash) throw Error('plan identity');
        for (const c of result.channels) {
          const other = repeat.channels.find(v => v.channel === c.channel);
          if (!other || c.pixels.some((v, i) => v !== other.pixels[i])) throw Error('repeat pixels');
        }
        const packaged = await gpu.renderPackage(new Uint8Array(packageBytes), request);
        if (packaged.plan.hash !== result.plan.hash) throw Error('package plan identity');
        for (const c of result.channels) {
          const other = packaged.channels.find(v => v.channel === c.channel);
          if (!other || c.pixels.length !== other.pixels.length || c.pixels.some((v, i) => v !== other.pixels[i])) throw Error('package pixels');
        }
        const partial = await gpu.render(source, { ...request, channels: ['height'] });
        if (partial.channels[0].pixels.some((v, i) => v !== result.channels.find(c => c.channel === 'height').pixels[i])) throw Error('sliced output pixels');
        const a = result.report.allocations, e = inspected.plan.estimates;
        if (a.textureCount !== 8n || a.textureCount !== e.textureCount || a.peakBytes !== e.peakBytes ||
            a.liveBytes !== 0n || a.releasedBytes !== a.cumulativeBytes ||
            a.reusedBytes !== e.logicalTextureBytes - e.textureBytes || a.reusedBytes <= 0n) throw Error('physical allocation accounting');
        if (inspected.plan.passes.length > 64 || a.peakBytes > 536870912n) throw Error('frozen 2K budget');
        if (item.endpointSource) {
          const reference = await gpu.render(item.endpointSource, { size: [1, 1], channels });
          endpoints = result.channels.filter(c => c.channel === 'metallic' || (item.preset === 'flat' && ['height','normal'].includes(c.channel)) || (item.preset === 'neutral-normal' && c.channel === 'normal') || (item.preset.startsWith('constant-') && ['baseColor','roughness'].includes(c.channel))).map(c => {
            const expected = reference.channels.find(r => r.channel === c.channel).pixels;
            if (expected.length !== 4 || c.pixels.some((v, i) => v !== expected[i % 4])) throw Error(`endpoint ${c.channel}`);
            return { channel: c.channel, expectedRgba8: [...expected], allPixelsExact: true };
          });
        }
        // Only CPU-owned delivered bytes survive between rows. No retained GPU state.
        if (['plain','varied','stress'].includes(item.preset) && [256,1024,2048].includes(item.size[0])) {
          window.wovenLow ??= {};
          const width=item.size[0], lowWidth=width===1024?256:1024;
          if (width>256) {
            downsample=['baseColor','height'].map(channel=>{
              const low=window.wovenLow[item.preset+'-'+lowWidth][channel],high=result.channels.find(c=>c.channel===channel).pixels;
              const factor=width/lowWidth,total=[0,0,0];
              if(low.length!==lowWidth*lowWidth*4||high.length!==width*width*4)throw Error('downsample dimensions');
              for(let y=0;y<lowWidth;y++)for(let x=0;x<lowWidth;x++)for(let c=0;c<3;c++){
                let sum=0;for(let dy=0;dy<factor;dy++)for(let dx=0;dx<factor;dx++)sum+=high[((y*factor+dy)*width+x*factor+dx)*4+c];
                total[c]+=Math.abs(low[(y*lowWidth+x)*4+c]-sum/(factor*factor));
              }
              const componentMeanError=total.map(v=>v/(lowWidth*lowWidth));
              const passed=componentMeanError.every(v=>Number.isFinite(v)&&v<=4);
              return {channel,componentMeanError,limit:4,passed,from:[width,width],to:[lowWidth,lowWidth],box:factor,gated:item.preset!=='stress',qualityScope:item.qualityScope};
            });
          }
          if(width<2048)window.wovenLow[item.preset+'-'+width]=Object.fromEntries(result.channels.filter(c=>['baseColor','height'].includes(c.channel)).map(c=>[c.channel,c.pixels.slice()]));
        }
      } finally { await gpu.destroy(); await gpu.destroy(); }
      // Encode only after destruction to check owned output lifetime.
      const images = [];
      for (const c of result.channels) {
        const canvas = document.createElement('canvas');
        canvas.width = item.size[0]; canvas.height = item.size[1];
        canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(c.pixels), ...item.size), 0, 0);
        images.push({ channel: c.channel, png: canvas.toDataURL('image/png').split(',')[1],
          sha256: [...new Uint8Array(await crypto.subtle.digest('SHA-256', c.pixels))].map(v => v.toString(16).padStart(2, '0')).join('') });
      }
      return { build: runtime.getBuildInfo(), planHash: result.plan.hash, allocation: inspected.plan.allocation,
        report: result.report, endpoints, downsample, controls, repeatExact: true, packageExact: true, slicedExact: true, ownedAfterDestroy: true, images };
    }, { source, item, packageBytes });
    expect(row.build).toEqual(expectedBuild);
    expect(row.images.map(v => v.channel).sort()).toEqual(['baseColor', 'height', 'metallic', 'normal', 'roughness']);
    const files = [];
    for (const image of row.images) {
      const name = `${item.id}-${image.channel}.png`;
      await writeFile(testInfo.outputPath(name), Buffer.from(image.png, 'base64'));
      files.push({ name, channel: image.channel, sha256: image.sha256 });
    }
    delete row.images;
    rows.push({ ...item, ...row, files });
    await writeFile(testInfo.outputPath('woven-browser.json'),receipt());
    if(row.downsample?.some(m=>m.gated&&!m.passed))throw Error('frozen woven downsample gate failed; retained measurements, stop without relaxing');
  }
  expect(rows).toHaveLength(51);
  expect(rows.flatMap(r=>r.downsample??[]).filter(m=>m.gated)).toHaveLength(8);
  await writeFile(testInfo.outputPath('woven-browser.json'),receipt());
});
