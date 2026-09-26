import assert from 'node:assert/strict';
import { test } from 'node:test';
import { paintedMetalRequest, paintedMetalMatrix } from './painted-metal-requests.mjs';

test('frozen seven presets, four sizes and five channels remain complete', () => {
  const matrix = paintedMetalMatrix();
  assert.equal(matrix.length, 28);
  assert.equal(new Set(matrix.map(row => row.id)).size, 28);
  assert.deepEqual([...new Set(matrix.map(row => row.preset))],
    ['default', 'intact', 'exposed', 'rusted', 'edge-rust', 'macro-seed', 'detail-seed']);
  for (const preset of ['default', 'intact', 'exposed', 'rusted', 'edge-rust', 'macro-seed', 'detail-seed']) {
    assert.deepEqual(matrix.filter(row => row.preset === preset).map(row => row.request.size),
      [[256, 256], [1024, 1024], [2048, 2048], [257, 129]]);
  }
  for (const row of matrix) assert.deepEqual(row.request.channels, ['baseColor', 'metallic', 'roughness', 'height', 'normal']);
});

test('endpoints bypass noise extrema and retain explicit seeds', () => {
  for (const amount of [0, 1]) {
    const { overrides: o } = paintedMetalRequest({ exposureAmount: amount, detailAmount: 0, macroSeed: 0, detailSeed: 0xffffffff }).request;
    assert.deepEqual([o.exposureInputMin, o.exposureInputMax, o.exposureOutputMin, o.exposureOutputMax], [0, 1, amount, amount]);
    assert.equal(o.detailMin, 1); assert.equal(o.macroSeed, 0); assert.equal(o.detailSeed, 0xffffffff);
  }
  const o = paintedMetalRequest({ exposureAmount: 0.75 }).request.overrides;
  assert.deepEqual([o.exposureInputMin, o.exposureInputMax, o.exposureOutputMin, o.exposureOutputMax], [0.2, 0.4, 0, 1]);
});

test('radius mapping handles zero, per-axis rounding and maximum without clamping', () => {
  for (const [width, size, expected] of [
    [0, [2048, 2048], [0, 0]], [1, [1, 17], [1, 1]],
    [4, [257, 129], [1, 1]], [8, [257, 129], [2, 1]],
    [4, [384, 128], [2, 1]], [8, [2048, 2048], [16, 16]],
  ]) {
    const o = paintedMetalRequest({ edgeWidth: width }, size).request.overrides;
    assert.deepEqual([o.radiusX, o.radiusY], expected);
  }
});

test('reject unknown, non-finite, fractional, out-of-range and coupled invalid controls', () => {
  for (const changes of [null, [], { extra: 0 }, { exposureAmount: NaN }, { rustAmount: Infinity },
    { detailAmount: -0.1 }, { rustFill: 1.01 }, { exposureScale: 0 }, { exposureScale: 65 },
    { edgeWidth: 0.5 }, { edgeWidth: 9 }, { macroSeed: -1 }, { detailSeed: 0x100000000 },
    { macroSeed: 1.5 }, { normalStrength: 8.1 }, { paintThickness: 0.51 },
    { paintThickness: 0.01, rustRelief: 0.02 }, { paintColor: [0, 0, 0] },
    { paintColor: [0, 0, 0, 0.5] }, { rustColor: [0, -1, 0, 1] },
    { substrateColor: [0, 0, Infinity, 1] }, { paintRoughness: '0.5' },
    { rustColor: Array(4) }]) assert.throws(() => paintedMetalRequest(changes));
  for (const size of [null, [], Array(2), [0, 1], [1, 2049], [1.5, 1], [1, NaN], [1, 1, 1]]) {
    assert.throws(() => paintedMetalRequest({}, size));
  }
});

test('caller arrays and successive requests are isolated', () => {
  const color = [0.8, 0.1, 0.4, 1], size = [257, 129];
  const a = paintedMetalRequest({ paintColor: color }, size);
  color[0] = 0; size[0] = 1; a.controls.paintColor[1] = 0;
  assert.deepEqual(a.request.overrides.paintColor, [0.8, 0.1, 0.4, 1]);
  assert.deepEqual(a.request.size, [257, 129]);
  a.request.channels.length = 0;
  assert.equal(paintedMetalRequest().request.channels.length, 5);
});
