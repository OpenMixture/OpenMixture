import assert from 'node:assert/strict';
import test from 'node:test';
import { captureRequest as captureWithLimits, captureSource, createRuntimeModule, MixtureRuntimeError } from '../dist/runtime.js';

const policy = { decodedBytes: 2097152n, nodes: 128n, edges: 512n, exposedParameters: 64n, outputDimension: 2048n, requestedOutputs: 8n, transientBytes: 536870912n };
const resourcePolicy = { resourceCount: 8n, resourcePixels: 16777216n, resourceBytes: 67108864n };
const captureRequest = (options, operation) => captureWithLimits(options, operation, policy, resourcePolicy);
const build = { runtimeVersion: 'test', apiSchemaVersion: 1, engineVersion: 'test', engineRevision: null, engineDirty: false, buildId: 'test' };
const defaults = { default_package_limits: () => ({packageBytes:70254592n,manifestBytes:65536n,packageBufferBytes:211812352n}), default_resource_limits: () => ({ ...resourcePolicy }),
  prepare_source: (bytes, request) => structuredClone({ bytes, request }), default_limits: () => ({ ...policy }), build_info: () => ({ ...build }), node_catalog: () => [],
  validate_source: (bytes, request) => ({ ok: true, bytes, request }), inspect_source: (bytes, request) => ({ bytes, request }) };
const invalid = error => error instanceof MixtureRuntimeError && error.code === 'MIX_BROWSER_INVALID_ARGUMENT';

test('package input capture checks views and both transfer buffers before binding entry', () => {
  let calls=0, observed;
  const module=createRuntimeModule({...defaults,inspect_package(bytes,options){calls++;observed={bytes,options};return {}; }},build);
  const buffer=new Uint8Array([9,1,2,3,9]);const view=buffer.subarray(1,4);
  module.inspectPackage(view,{packageLimits:{packageBytes:3n,packageBufferBytes:6n}});
  buffer.fill(0);assert.deepEqual([...observed.bytes],[1,2,3]);assert.equal(observed.bytes.byteOffset,0);assert.equal(calls,1);
  for(const options of [{packageLimits:{packageBytes:2n}},{packageLimits:{packageBufferBytes:5n}},{packageLimits:{manifestBytes:65537n}}]) {
    assert.throws(()=>module.inspectPackage(new Uint8Array(3),options),e=>e.code==='MIX_PACKAGE_LIMIT_EXCEEDED');
  }
  let getters=0;
  const hostile=new Uint8Array(3);Object.defineProperty(hostile,'byteOffset',{get(){getters++;return 0;}});
  for(const bytes of [hostile,new Uint8Array(new SharedArrayBuffer(3)),new Uint8Array(new ArrayBuffer(3,{maxByteLength:6})),new Uint16Array(3),'url',new Proxy(new Uint8Array(3),{})]) assert.throws(()=>module.inspectPackage(bytes),invalid);
  const detached=new Uint8Array(3);structuredClone(detached.buffer,{transfer:[detached.buffer]});assert.throws(()=>module.inspectPackage(detached),invalid);
  for(const options of [{get packageLimits(){getters++;return {};}},{packageLimits:{get packageBytes(){getters++;return 3n;}}},{packageLimits:{packageBytes:3}},{resources:[]},{size:[2,2]}]) assert.throws(()=>module.inspectPackage(new Uint8Array(3),options),invalid);
  assert.equal(getters,0);assert.equal(calls,1);
});

test('package preparation is synchronous, shares busy/destroy, and recovers after rejection', async () => {
  const secure=Object.getOwnPropertyDescriptor(globalThis,'isSecureContext'),navigator=Object.getOwnPropertyDescriptor(globalThis,'navigator');
  Object.defineProperty(globalThis,'isSecureContext',{value:true,configurable:true});Object.defineProperty(globalThis,'navigator',{value:{gpu:{}},configurable:true});
  try {
    let captures=0,received,settle,released=0;
    const module=createRuntimeModule({...defaults,prepare_package(bytes,options){captures++;if(bytes[0]===0)throw {diagnostics:[{code:'MIX_PACKAGE_INVALID',stage:'package',severity:'error',message:'bad'}]};return {bytes:bytes.slice(),options:structuredClone(options)};},create_gpu:async()=>({context_report:()=>({}),render:input=>{received=input;return new Promise(resolve=>{settle=resolve;});},destroy(){released++;},free(){}})},build);
    const gpu=await module.createGpu();await assert.rejects(gpu.renderPackage(new Uint8Array([0])),e=>e.code==='MIX_PACKAGE_INVALID');
    const bytes=new Uint8Array([1,2]),options={size:[2,2],overrides:{weight:0.25}};
    const pending=gpu.renderPackage(bytes,options);bytes.fill(9);options.size[0]=1;options.overrides.weight=1;
    let getters=0;const hostile={get size(){getters++;return [1,1];}};
    await assert.rejects(gpu.renderPackage(new Uint8Array(),hostile),e=>e.code==='MIX_BROWSER_RUNTIME_BUSY');
    await assert.rejects(gpu.render('{}',hostile),e=>e.code==='MIX_BROWSER_RUNTIME_BUSY');
    assert.equal(captures,2);assert.equal(getters,0);assert.deepEqual([...received.bytes],[1,2]);assert.deepEqual(received.options.request.size,[2,2]);
    assert.equal(JSON.parse(received.options.request.overridesJson).weight,0.25);
    const destroyed=gpu.destroy();assert.equal(gpu.destroy(),destroyed);
    await assert.rejects(gpu.renderPackage(new Uint8Array(),hostile),e=>e.code==='MIX_BROWSER_RUNTIME_DESTROYED');assert.equal(captures,2);assert.equal(getters,0);
    settle({channels:[]});await pending;await destroyed;assert.equal(released,1);
  }finally{for(const [key,value] of [['isSecureContext',secure],['navigator',navigator]])if(value)Object.defineProperty(globalThis,key,value);else delete globalThis[key];}
});

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
