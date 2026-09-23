import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

test('saturating subtraction preserves equal masks and the half-precision edge difference', async ({ page, browser }, testInfo) => {
  test.setTimeout(120_000);
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.7.0-alpha.0');
  const source = await readFile(new URL('../public/scalar-subtract.mix', import.meta.url), 'utf8');
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const result = await page.evaluate(async source => {
    const runtime = await window.sdk.loadRuntime(), gpu = await runtime.createGpu();
    const rows = [];
    const amplified = JSON.parse(source);
    amplified.nodes.push({ id: 'amplify', type: 'levels', version: 1, parameters: { inputMin: 0, inputMax: 1 / 4096 } });
    for (const edge of amplified.edges) if (edge.from.nodeId === 'mix') edge.from.nodeId = 'amplify';
    amplified.edges.push({ from: { nodeId: 'mix', portId: 'value' }, to: { nodeId: 'amplify', portId: 'in' } });
    // Literal expected gray codes; amplification distinguishes a retained 1/4096 edge from zero.
    const cases = [[0, 0, 0], [1, 0, 255], [0, 1, 0], [1, 1, 0], [0.5, 0.5, 0], [0.5, 0.5 - 1 / 4096, 0], [0.25, 0.75, 0], [0.75, 0.25, 128]];
    try {
      for (const size of [[1, 1], [1, 17], [17, 1], [19, 11]]) for (const [id, [a, b, gray]] of cases.entries()) for (const amplify of [false, true]) {
        const input = amplify ? JSON.stringify(amplified) : source;
        const request = { size, channels: ['height'], overrides: { a, b } };
        const output = await gpu.render(input, request), again = await gpu.render(input, request);
        const pixels = output.channels[0].pixels;
        const want = amplify ? (id === 1 || id === 5 || id === 7 ? 255 : 0) : gray;
        for (let i = 0; i < pixels.length; i++) if (pixels[i] !== (i % 4 === 3 ? 255 : want) || pixels[i] !== again.channels[0].pixels[i]) throw Error(`subtraction mismatch ${size}/${id}/${amplify}, byte ${i}`);
        if (output.report.allocations.liveBytes !== 0n) throw Error('retained GPU allocations');
        rows.push({ size, id, a, b, amplified: amplify, planHash: output.plan.hash, pixels: [...pixels], repeatExact: true });
      }
      return { build: runtime.getBuildInfo(), adapter: gpu.context, rows };
    } finally { await gpu.destroy(); }
  }, source);
  expect(result.build).toEqual(expectedBuild);
  expect(result.rows).toHaveLength(64);
  const content = JSON.stringify({ ...result, browser: browser.version() }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2);
  await writeFile(testInfo.outputPath('subtract-evidence.json'), content);
  await testInfo.attach('subtract-evidence', { body: content, contentType: 'application/json' });
});
