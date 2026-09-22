// Produce an explicitly experimental capture shader; never edit runtime sources.
import { readFile, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
const [mode, output] = process.argv.slice(2);
assert.ok(['fma', 'prehalf'].includes(mode) && output, 'usage: numerical-experiment.mjs <fma|prehalf> <new-file.wgsl>');
const source = await readFile('crates/mixture-wgpu/shaders/precision.wgsl','utf8') + '\n'
  + await readFile('crates/mixture-wgpu/shaders/nodes/fractal-noise.wgsl','utf8');
const before = mode === 'fma'
  ? 'return mix(mix(a, b, fade.x), mix(c, d, fade.x), fade.y);'
  : 'mixture_half4(vec4<f32>(clamp(sum / weights, 0.0, 1.0), 0.0, 0.0, 1.0))';
const after = mode === 'fma'
  ? 'let ab = fma(b - a, fade.x, a);\n    let cd = fma(d - c, fade.x, c);\n    return fma(cd - ab, fade.y, ab);'
  // Each byte/256 is exactly representable in f16; this captures f32 bits without
  // rounding them to a scalar half. Normal output in this mode is meaningless.
  : 'vec4<f32>((vec4<u32>(bitcast<u32>(clamp(sum / weights, 0.0, 1.0))) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 256.0';
assert.equal(source.split(before).length, 2, 'production shader changed; review instrumentation');
await writeFile(output, source.replace(before,after), {flag:'wx'});
