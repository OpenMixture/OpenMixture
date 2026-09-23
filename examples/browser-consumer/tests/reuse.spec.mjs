import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

test('physical texture slots preserve public outputs and the five-channel 2K workload', async ({ page, browser }, testInfo) => {
  test.setTimeout(900_000);
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.8.0-alpha.0');
  const source = await readFile(new URL('../public/reuse-material.mix', import.meta.url), 'utf8');
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const rows = [];
  const cases = [
    { id: 'odd', size: [65, 3], overrides: { radiusX: 1, radiusY: 1, macroSeed: 4294967295 } },
    { id: 'intact', size: [257, 129], overrides: { radiusX: 1, radiusY: 1, exposureInputMin: 0, exposureInputMax: 1, exposureOutputMin: 0, exposureOutputMax: 0 } },
    { id: 'default-1k', size: [1024, 1024], overrides: {} },
    { id: 'default-2k', size: [2048, 2048], overrides: { radiusX: 8, radiusY: 8 } },
  ];
  for (const item of cases) {
    const row = await page.evaluate(async ({ source, item }) => {
      const runtime = await window.sdk.loadRuntime();
      const channels = ['baseColor', 'normal', 'roughness', 'metallic', 'height'];
      const request = { size: item.size, channels, overrides: item.overrides };
      const inspected = runtime.inspect(source, request);
      if (inspected.plan.version !== 3 || typeof inspected.plan.estimates.textureCount !== 'bigint' ||
          typeof inspected.plan.estimates.logicalTextureBytes !== 'bigint') throw Error('v3 typed plan contract');
      const gpu = await runtime.createGpu();
      let result;
      try {
        result = await gpu.render(source, request);
        const repeat = await gpu.render(source, request);
        if (result.plan.hash !== inspected.plan.hash || result.plan.hash !== repeat.plan.hash) throw Error('plan identity');
        for (const c of result.channels) {
          const other = repeat.channels.find(v => v.channel === c.channel);
          if (!other || c.pixels.some((v, i) => v !== other.pixels[i])) throw Error('repeat pixels');
        }
        const partial = await gpu.render(source, { ...request, channels: ['height'] });
        if (partial.channels[0].pixels.some((v, i) => v !== result.channels.find(c => c.channel === 'height').pixels[i])) throw Error('sliced output pixels');
        const a = result.report.allocations, e = inspected.plan.estimates;
        if (a.textureCount !== 14n || a.textureCount !== e.textureCount || a.peakBytes !== e.peakBytes ||
            a.liveBytes !== 0n || a.releasedBytes !== a.cumulativeBytes ||
            a.reusedBytes !== e.logicalTextureBytes - e.textureBytes || a.reusedBytes <= 0n) throw Error('physical allocation accounting');
        if (item.id === 'default-2k' && a.peakBytes !== 503316944n) throw Error('frozen 2K budget');
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
        report: result.report, repeatExact: true, slicedExact: true, ownedAfterDestroy: true, images };
    }, { source, item });
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
  await writeFile(testInfo.outputPath('reuse-browser.json'), JSON.stringify({
    ok: true, browser: browser.version(), build: expectedBuild, rows,
  }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2));
});
