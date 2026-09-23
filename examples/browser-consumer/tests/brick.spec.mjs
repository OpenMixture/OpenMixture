import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

test('brick material candidate renders the complete frozen public matrix', async ({ page, browser }, testInfo) => {
  test.setTimeout(600_000);
  const read = name => readFile(new URL(`../public/brick-paving/${name}`, import.meta.url), 'utf8');
  const source = await read('material.mix');
  const controls = JSON.parse(await read('controls.json'));
  const matrix = JSON.parse(await read('qualification-plan.json'));
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(['0.6.0-alpha.0', '0.8.0-alpha.0']).toContain(expectedBuild.runtimeVersion);
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const rows = [];
  for (const variant of matrix.cases) for (const size of matrix.sizes) {
    const overrides = {};
    for (const [control, value] of Object.entries(variant.overrides)) {
      for (const id of controls[control]) overrides[id] = value;
    }
    const row = await page.evaluate(async ({ source, size, overrides, channels }) => {
      const runtime = await window.sdk.loadRuntime();
      const request = { size, overrides, channels };
      const inspection = runtime.inspect(source, request);
      const gpu = await runtime.createGpu({ powerPreference: 'high-performance' });
      let result;
      try {
        result = await gpu.render(source, request);
        const repeat = await gpu.render(source, request);
        for (const channel of result.channels) {
          const other = repeat.channels.find(c => c.channel === channel.channel);
          if (!other || channel.pixels.length !== other.pixels.length || channel.pixels.some((v, i) => v !== other.pixels[i])) {
            throw Error(`Repeated ${channel.channel} pixels differ`);
          }
        }
      } finally { await gpu.destroy(); }
      const images = [];
      for (const c of result.channels) {
        const canvas = document.createElement('canvas');
        canvas.width = size[0]; canvas.height = size[1];
        canvas.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(c.pixels), ...size), 0, 0);
        images.push({ channel: c.channel, png: canvas.toDataURL('image/png').split(',')[1],
          sha256: [...new Uint8Array(await crypto.subtle.digest('SHA-256', c.pixels))].map(v => v.toString(16).padStart(2, '0')).join('') });
      }
      return { build: runtime.getBuildInfo(), planHash: inspection.plan.hash,
        report: result.report, repeatExact: true, images };
    }, { source, size, overrides, channels: matrix.channels });
    expect(row.build).toEqual(expectedBuild);
    expect(row.images.map(v => v.channel).sort()).toEqual([...matrix.channels].sort());
    expect(Number(row.report.passCount)).toBeLessThanOrEqual(matrix.maxPasses);
    expect(Number(row.report.allocations.peakBytes)).toBeLessThanOrEqual(matrix.maxDescriptorPeakBytes);
    expect(Number(row.report.allocations.liveBytes)).toBe(0);
    const files = [];
    for (const image of row.images) {
      const name = `${variant.id}-${size[0]}x${size[1]}-${image.channel}.png`;
      await writeFile(testInfo.outputPath(name), Buffer.from(image.png, 'base64'));
      files.push({ name, channel: image.channel, sha256: image.sha256 });
    }
    delete row.images;
    rows.push({ case: variant.id, size, ...row, files });
  }
  expect(rows).toHaveLength(20);
  await writeFile(testInfo.outputPath('brick-browser.json'), JSON.stringify({
    ok: true, browser: browser.version(), build: expectedBuild, rows,
  }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2));
});
