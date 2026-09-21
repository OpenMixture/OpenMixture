import assert from 'node:assert/strict';
import test from 'node:test';
import { captureRequest as captureWithLimits, captureSource, createRuntimeModule, MixtureRuntimeError } from '../dist/runtime.js';

const policy = { decodedBytes: 2097152n, nodes: 128n, edges: 512n, exposedParameters: 64n, outputDimension: 2048n, requestedOutputs: 8n, transientBytes: 536870912n };
const resourcePolicy = { resourceCount: 8n, resourcePixels: 16777216n, resourceBytes: 67108864n };
const captureRequest = (options, operation) => captureWithLimits(options, operation, policy, resourcePolicy);
const build = { runtimeVersion: 'test', apiSchemaVersion: 1, engineVersion: 'test', engineRevision: null, engineDirty: false, buildId: 'test' };
const defaults = { default_resource_limits: () => ({ ...resourcePolicy }),
  prepare_source: (bytes, request) => structuredClone({ bytes, request }), default_limits: () => ({ ...policy }), build_info: () => ({ ...build }), node_catalog: () => [],
  validate_source: (bytes, request) => ({ ok: true, bytes, request }), inspect_source: (bytes, request) => ({ bytes, request }) };
const invalid = error => error instanceof MixtureRuntimeError && error.code === 'MIX_BROWSER_INVALID_ARGUMENT';

test('errors without diagnostics or a browser failure have no code', () => {
  assert.equal(new MixtureRuntimeError('render', {}).code, undefined);
  assert.equal(new MixtureRuntimeError('render', { diagnostics: [] }).code, undefined);
});

test('request integers, limits, shapes and accessor-free capture', () => {
  for (const value of [0, -1, 1.5, Infinity, NaN, 4294967296, Number.MAX_SAFE_INTEGER + 1, true, '64']) {
    assert.throws(() => captureRequest({ size: [value, 3] }, 'render'), invalid);
  }
  assert.deepEqual(captureRequest({ size: [4294967295, 1] }, 'render').size, [4294967295, 1]);
  assert.equal(captureRequest({ overrides: { seed: 4294967295, zero: 0 } }, 'render').overridesJson, '{"seed":4294967295,"zero":0}');
  assert.equal(captureRequest({ limits: { transientBytes: 18446744073709551615n } }, 'render').limits.transientBytes, 18446744073709551615n);
  for (const value of [0, -1n, 18446744073709551616n, '2']) assert.throws(() => captureRequest({ limits: { nodes: value } }, 'render'), invalid);
  for (const overrides of [{ x: undefined }, { x: null }, { x: true }, { x: Infinity }, { x: () => {} }, { x: Symbol() }, { x: {} }, { x: [1, 2] }]) {
    assert.throws(() => captureRequest({ overrides }, 'render'), invalid);
  }
  const cycle = {}; cycle.x = cycle;
  assert.throws(() => captureRequest({ overrides: cycle }, 'render'), invalid);
  let calls = 0;
  assert.throws(() => captureRequest({ get size() { calls++; return [2, 2]; } }, 'render'), invalid);
  assert.throws(() => captureRequest({ overrides: { get x() { calls++; return 2; } } }, 'render'), invalid);
  const color = [1, 1, 1, 1]; Object.defineProperty(color, '0', { get() { calls++; return 1; } });
  assert.throws(() => captureRequest({ overrides: { tint: color } }, 'render'), invalid);
  assert.equal(calls, 0);
  for (const channels of [[], ['baseColor', 'baseColor'], [undefined], new Array(1)]) assert.throws(() => captureRequest({ channels }, 'render'), invalid);
  assert.throws(() => captureRequest({ extra: 1 }, 'render'), invalid);
});

test('source limits are UTF-8 bytes; source is captured without JSON parsing', () => {
  for (const source of ['\ud800', '\udc00', '\ud800x']) assert.throws(() => captureSource(source, 100n, 'validate'), invalid);
  assert.deepEqual([...captureSource('😀', 4n, 'validate')], [240, 159, 152, 128]);
  assert.throws(() => captureSource('😀', 3n, 'validate'), error => error.code === 'MIX_LIMIT_DECODED_BYTES_EXCEEDED');
  const bytes = new Uint8Array([255]);
  const copy = captureSource(bytes, 10n, 'validate'); bytes[0] = 0;
  assert.equal(copy[0], 255); // Malformed UTF-8 remains available to strict Rust parsing.
  const hostile = new Uint8Array([1, 2]);
  hostile.slice = () => hostile;
  Object.defineProperty(hostile, 'byteLength', { value: 0 });
  Object.defineProperty(hostile, 'buffer', { get() { throw Error('must not execute'); } });
  assert.throws(() => captureSource(hostile, 1n, 'render'), error => error.code === 'MIX_LIMIT_DECODED_BYTES_EXCEEDED');
  const independent = captureSource(hostile, 2n, 'render'); hostile[0] = 9; assert.equal(independent[0], 1);
  const raw = '{"version":1,"version":1.0}';
  assert.equal(new TextDecoder().decode(captureSource(raw, 100n, 'validate')), raw);
});

test('CPU access is explicit and build identity is checked before use', () => {
  let gpuCalls = 0;
  const module = createRuntimeModule({ ...defaults, create_gpu() { gpuCalls++; throw Error('unexpected'); } }, build);
  assert.equal(module.validate('{}').ok, true);
  module.inspect('{}'); module.getNodeCatalog(); module.getBuildInfo();
  assert.equal(gpuCalls, 0);
  assert.throws(() => createRuntimeModule(defaults, { ...build, buildId: 'different' }), error => error.code === 'MIX_BROWSER_BUILD_MISMATCH');
  const result = module.validate('😀', { limits: { decodedBytes: 3n } });
  assert.equal(result.ok, false);
  assert.equal(typeof result.diagnostics[0].evidence.observed, 'bigint');
});

test('accepted inputs are captured; busy and in-flight idempotent destruction settle', async () => {
  const previousSecure = Object.getOwnPropertyDescriptor(globalThis, 'isSecureContext');
  const previousNavigator = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  Object.defineProperty(globalThis, 'isSecureContext', { value: true, configurable: true });
  Object.defineProperty(globalThis, 'navigator', { value: { gpu: {} }, configurable: true });
  try {
    let settle, received, released = 0;
    const engineResult = { channels: [{ pixels: new Uint8Array([1, 2, 3, 255]) }] };
    const module = createRuntimeModule({ ...defaults, create_gpu: async () => ({ context_report: () => ({ verdict: 'unverified' }),
      render: input => { received = input; return new Promise(resolve => { settle = resolve; }); },
      destroy: () => { released++; }, free() {},
    }) }, build);
    const gpu = await module.createGpu();
    const source = new Uint8Array([1, 2]); const options = { size: [65, 3], overrides: { seed: 8, tint: [1, 1, 1, 1] } };
    const pending = gpu.render(source, options);
    source[0] = 99; options.size[0] = 32; options.overrides.seed = 0; options.overrides.tint[0] = 0;
    await assert.rejects(gpu.render('bad'), error => error.code === 'MIX_BROWSER_RUNTIME_BUSY');
    assert.equal(received.bytes[0], 1); assert.equal(received.request.size[0], 65);
    assert.equal(JSON.parse(received.request.overridesJson).seed, 8); assert.equal(JSON.parse(received.request.overridesJson).tint[0], 1);
    const destroy = gpu.destroy(); assert.equal(gpu.destroy(), destroy); assert.equal(released, 0);
    await assert.rejects(gpu.render('{}'), error => error.code === 'MIX_BROWSER_RUNTIME_DESTROYED');
    settle(engineResult); assert.equal(await pending, engineResult); await destroy;
    assert.equal(released, 1); assert.equal(engineResult.channels[0].pixels[0], 1);
  } finally {
    if (previousSecure) Object.defineProperty(globalThis, 'isSecureContext', previousSecure); else delete globalThis.isSecureContext;
    if (previousNavigator) Object.defineProperty(globalThis, 'navigator', previousNavigator); else delete globalThis.navigator;
  }
});

test('an engine failure survives wrapping and pending destruction still releases', async () => {
  const previousSecure = Object.getOwnPropertyDescriptor(globalThis, 'isSecureContext');
  const previousNavigator = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  Object.defineProperty(globalThis, 'isSecureContext', { value: true, configurable: true });
  Object.defineProperty(globalThis, 'navigator', { value: { gpu: {} }, configurable: true });
  try {
    let reject, released = false;
    const failure = { diagnostics: [{ code: 'MIX_GPU_DEVICE_LOST', stage: 'gpuExecution', message: 'Delivered loss' }], evidence: { deviceLoss: { reason: 'unknown' } } };
    const module = createRuntimeModule({ ...defaults, create_gpu: async () => ({ context_report: () => ({}),
      render: () => new Promise((_, fail) => { reject = fail; }), destroy: () => { released = true; }, free() {},
    }) }, build);
    const gpu = await module.createGpu(); const pending = gpu.render('{}');
    const check = assert.rejects(pending, error => error.diagnostics === failure.diagnostics && error.evidence === failure.evidence);
    const destroy = gpu.destroy(); await Promise.resolve(); reject(failure);
    await check; await destroy; assert.equal(released, true);
  } finally {
    if (previousSecure) Object.defineProperty(globalThis, 'isSecureContext', previousSecure); else delete globalThis.isSecureContext;
    if (previousNavigator) Object.defineProperty(globalThis, 'navigator', previousNavigator); else delete globalThis.navigator;
  }
});

test('resource transport rejects unsafe buffers and shapes before entering Rust', () => {
  const image = data => ({ id: 'height', width: 2, height: 1, format: 'rgba8-linear', bytesPerRow: 8, data });
  const storage = new Uint8Array([99, 1, 2, 3, 4, 5, 6, 7, 8, 99]);
  const view = storage.subarray(1, 9);
  assert.deepEqual([...captureRequest({ resources: [image(view)] }, 'render').resources[0].data], [1,2,3,4,5,6,7,8]);
  const detached = new Uint8Array(8); structuredClone(detached.buffer, { transfer: [detached.buffer] });
  const bad = [detached, new Uint8Array(new SharedArrayBuffer(8)), new Uint8Array(new ArrayBuffer(8, { maxByteLength: 16 })), new Uint16Array(4), [1,2]];
  for (const data of bad) assert.throws(() => captureRequest({ resources: [image(data)] }, 'render'), invalid);
  for (const value of [NaN, Infinity, 1.5, '2', -1, 2 ** 32]) assert.throws(() => captureRequest({ resources: [{ ...image(view), width: value }] }, 'render'), invalid);
  for (const value of [-1n, 1, 2n ** 64n]) assert.throws(() => captureRequest({ resourceLimits: { resourceBytes: value } }, 'render'), invalid);
  let calls = 0;
  const accessor = image(view); Object.defineProperty(accessor, 'data', { get() { calls++; return view; } });
  assert.throws(() => captureRequest({ resources: [accessor] }, 'render'), invalid);
  assert.equal(calls, 0);
  assert.throws(() => captureRequest({ resources: [image(view)], resourceLimits: { resourceCount: 0n } }, 'render'), e => e.code === 'MIX_LIMIT_RESOURCE_COUNT_EXCEEDED');
});
