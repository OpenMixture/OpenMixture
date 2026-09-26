import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

test('morphology candidate preserves periodic support sets through public WebGPU', async ({ page, browser }, testInfo) => {
  test.setTimeout(180_000);
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.8.0-alpha.0');
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const evidence = await page.evaluate(async () => {
    const runtime = await window.sdk.loadRuntime(), gpu = await runtime.createGpu();
    const edge = (a, ap, b, bp) => ({ from: { nodeId: a, portId: ap }, to: { nodeId: b, portId: bp } });
    const document = { version: 1, nodes: [
      { id: 'image', type: 'image-input', version: 1, parameters: { resourceId: 'source' } },
      { id: 'morph', type: 'scalar-morphology', version: 1 },
      { id: 'color', type: 'constant-color', version: 1 },
      { id: 'out', type: 'material-output', version: 1 },
    ], edges: [edge('image', 'value', 'morph', 'in'), edge('morph', 'value', 'out', 'height'), edge('color', 'color', 'out', 'baseColor')],
    exposedParameters: ['operation', 'axis', 'radius'].map(id => ({ id, nodeId: 'morph', parameterId: id })) };
    const source = JSON.stringify(document), rows = [];
    try {
      for (const [w, h] of [[1, 1], [1, 17], [17, 1], [19, 11]]) {
        const size = [w, h];
        // Literal geometric sets; the expected reduction is set translation/union/intersection.
        const fields = [new Set([0]), new Set([Math.floor(h / 2) * w + Math.floor(w / 2)]),
          new Set(Array.from({ length: h }, (_, y) => y * w + Math.floor(w / 2))),
          new Set([0, w - 1, (h - 1) * w, w * h - 1])];
        for (const [field, support] of fields.entries()) {
          const data = new Uint8Array(w * h * 4);
          for (let i = 0; i < w * h; i++) data.set([support.has(i) ? 255 : 0, 31, 199, 0], i * 4);
          const resources = [{ id: 'source', width: w, height: h, bytesPerRow: w * 4, format: 'rgba8-linear', data }];
          for (const operation of ['erode', 'dilate']) for (const axis of ['x', 'y']) for (const radius of [0, 1, 16]) {
            let expected = new Set(support);
            for (let k = -radius; k <= radius; k++) {
              const moved = new Set([...support].map(i => {
                const x = i % w, y = Math.floor(i / w);
                const nx = ((x + (axis === 'x' ? k : 0)) % w + w) % w;
                const ny = ((y + (axis === 'y' ? k : 0)) % h + h) % h;
                return ny * w + nx;
              }));
              expected = operation === 'erode' ? new Set([...expected].filter(i => moved.has(i))) : new Set([...expected, ...moved]);
            }
            const request = { size, channels: ['height'], resources, overrides: { operation, axis, radius } };
            const result = await gpu.render(source, request), repeat = await gpu.render(source, request);
            const pixels = result.channels[0].pixels;
            for (let i = 0; i < pixels.length; i++) {
              const want = i % 4 === 3 ? 255 : expected.has(Math.floor(i / 4)) ? 255 : 0;
              if (pixels[i] !== want || repeat.channels[0].pixels[i] !== pixels[i]) throw Error(`morphology mismatch ${size}/${field}/${operation}/${axis}/${radius} byte ${i}`);
            }
            if (result.report.allocations.liveBytes !== 0n) throw Error('retained GPU allocation');
            rows.push({ size, field, operation, axis, radius, planHash: result.plan.hash, pixels: [...pixels], repeatExact: true });
          }
        }
      }
      return { build: runtime.getBuildInfo(), adapter: gpu.context, rows };
    } finally { await gpu.destroy(); }
  });
  expect(evidence.build).toEqual(expectedBuild);
  expect(evidence.rows).toHaveLength(192);
  const content = JSON.stringify({ ...evidence, browser: browser.version() }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2);
  await writeFile(testInfo.outputPath('morphology-evidence.json'), content);
  await testInfo.attach('morphology-evidence', { body: content, contentType: 'application/json' });
});
