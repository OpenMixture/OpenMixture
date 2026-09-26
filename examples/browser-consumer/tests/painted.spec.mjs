import { test, expect } from '@playwright/test';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';

function endpointSource(preset, controls) {
  const values = {
    intact: [controls.paintColor, controls.paintRoughness, 0, 0.2 + controls.paintThickness],
    exposed: [controls.substrateColor, controls.substrateRoughness, 1, 0.2],
    rusted: [controls.rustColor, controls.rustRoughness, 0, 0.2 + controls.rustRelief],
  }[preset];
  if (!values) return null;
  const [color, roughness, metallic, height] = values;
  const node = (id, type, parameters) => ({ id, type, version: 1, parameters });
  const edge = (nodeId, portId, to, input) => ({ from: { nodeId, portId }, to: { nodeId: to, portId: input } });
  const nodes = [node('color', 'constant-color', { value: color }), node('normal', 'height-to-normal', { strength: 1 }), node('out', 'material-output', {})];
  const edges = [edge('color', 'color', 'out', 'baseColor'), edge('height', 'value', 'normal', 'in'), edge('normal', 'normal', 'out', 'normal')];
  for (const [name, value] of Object.entries({ roughness, metallic, height })) {
    nodes.push(node(name, 'constant-scalar', { value })); edges.push(edge(name, 'value', 'out', name));
  }
  return JSON.stringify({ version: 1, nodes, edges });
}

test('painted metal renders all frozen presets with repeat and package parity', async ({ page, browser }, testInfo) => {
  test.setTimeout(1_800_000);
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.8.0-alpha.0');
  const source = await readFile(new URL('../public/painted-metal/material.mix', import.meta.url), 'utf8');
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const rows = [];
  const manifest = JSON.parse(await readFile(new URL('../public/painted-metal/requests.json', import.meta.url)));
  expect(createHash('sha256').update(source).digest('hex')).toBe(manifest.sourceSha256);
  const packageBytes = [...await readFile(new URL('../public/painted-metal/material.mixpack', import.meta.url))];
  expect(manifest.rows).toHaveLength(28);
  const cases = manifest.rows.map(row => ({ id: row.id, ...row.request, endpointSource: endpointSource(row.preset, row.controls) }));
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
        if (a.textureCount !== 14n || a.textureCount !== e.textureCount || a.peakBytes !== e.peakBytes ||
            a.liveBytes !== 0n || a.releasedBytes !== a.cumulativeBytes ||
            a.reusedBytes !== e.logicalTextureBytes - e.textureBytes || a.reusedBytes <= 0n) throw Error('physical allocation accounting');
        if (item.size[0] === 2048 && a.peakBytes !== 503316944n) throw Error('frozen 2K budget');
        if (item.endpointSource) {
          const reference = await gpu.render(item.endpointSource, { size: [1, 1], channels });
          endpoints = result.channels.map(c => {
            const expected = reference.channels.find(r => r.channel === c.channel).pixels;
            if (expected.length !== 4 || c.pixels.some((v, i) => v !== expected[i % 4])) throw Error(`endpoint ${c.channel}`);
            return { channel: c.channel, expectedRgba8: [...expected], allPixelsExact: true };
          });
        }
        // Test-page CPU-owned samples only; no GPU state or runtime cache is retained.
        if (item.id === 'default-256x256') {
          window.paintedDefaultLow = Object.fromEntries(result.channels.filter(c => ['height', 'baseColor'].includes(c.channel)).map(c => [c.channel, c.pixels.slice()]));
        }
        if (item.id === 'default-1024x1024') {
          downsample = ['height', 'baseColor'].map(channel => {
            const low = window.paintedDefaultLow[channel];
            const high = result.channels.find(c => c.channel === channel).pixels;
            if (low.length !== 256 * 256 * 4 || high.length !== 1024 * 1024 * 4) throw Error('downsample dimensions');
            const total = [0, 0, 0];
            for (let y = 0; y < 256; y++) for (let x = 0; x < 256; x++) for (let c = 0; c < 3; c++) {
              let sum = 0;
              for (let dy = 0; dy < 4; dy++) for (let dx = 0; dx < 4; dx++) sum += high[((y * 4 + dy) * 1024 + x * 4 + dx) * 4 + c];
              total[c] += Math.abs(low[(y * 256 + x) * 4 + c] - sum / 16);
            }
            const componentMeanError = total.map(v => v / (256 * 256));
            if (componentMeanError.some(v => !Number.isFinite(v) || v > 4)) throw Error(`default downsample ${channel}: ${componentMeanError}`);
            return { channel, componentMeanError, limit: 4, passed: true, filter: 'exact 4x4 box mean of delivered RGB bytes' };
          });
          delete window.paintedDefaultLow;
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
        report: result.report, endpoints, downsample, repeatExact: true, packageExact: true, slicedExact: true, ownedAfterDestroy: true, images };
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
  }
  expect(rows.flatMap(row => row.endpoints ?? [])).toHaveLength(60);
  expect(rows.flatMap(row => row.downsample ?? [])).toHaveLength(2);
  await writeFile(testInfo.outputPath('painted-browser.json'), JSON.stringify({
    ok: true, materialAccepted: false, manifest,
    packageSha256: createHash('sha256').update(Buffer.from(packageBytes)).digest('hex'),
    browser: browser.version(), build: expectedBuild, rows,
  }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2));
});
