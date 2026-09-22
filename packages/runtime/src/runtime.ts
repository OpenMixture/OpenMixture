import type { BrowserFailure, BuildInfo, Diagnostic, GpuRuntime, Inspection, ParameterValue, RenderRequest, RenderResult, RuntimeModule, SafetyLimits, ResourceLimits, ImageBinding, Source, ValidationResult } from './types.js';
import type { BindingRequest, Bindings, Failure } from './bindings.js';
import type { PackageLimits } from './types.js';
import type { PackageBindingRequest, PreparedBinding, GpuBinding } from './bindings.js';

const U32_MAX = 4294967295;
const U64_MAX = 18446744073709551615n;


/** Structured public failures preserve engine diagnostics independently of browser failures. */
export class MixtureRuntimeError extends Error {
  declare readonly operation: string;
  declare readonly diagnostics: Diagnostic[];
  declare readonly browserFailure?: BrowserFailure;
  declare readonly evidence?: unknown;

  constructor(operation: string, failure: Failure) {
    super(failure.browserFailure?.message ?? failure.diagnostics?.[0]?.message ?? `Mixture ${operation} failed`);
    this.name = 'MixtureRuntimeError';
    this.operation = operation;
    this.diagnostics = failure.diagnostics ?? [];
    if (failure.browserFailure) this.browserFailure = failure.browserFailure;
    if (failure.evidence !== undefined) this.evidence = failure.evidence;
  }
  get code(): string | undefined { return this.diagnostics[0]?.code ?? this.browserFailure?.code; }
}

export function browserError(operation: string, code: string, message: string, evidence?: unknown) {
  return new MixtureRuntimeError(operation, { diagnostics: [], browserFailure: {
    code: `MIX_BROWSER_${code}`, operation, message, ...(evidence === undefined ? {} : { evidence }),
  } });
}

function invalid(operation: string, message: string) { return browserError(operation, 'INVALID_ARGUMENT', message); }

function boundary<T>(operation: string, action: () => T): T {
  try { return action(); } catch (error) { throw normalizeError(operation, error); }
}

export function normalizeError(operation: string, error: unknown): MixtureRuntimeError {
  if (error instanceof MixtureRuntimeError) return error;
  // Rust throws projected objects, not Error instances. This assertion describes
  // that tested wire contract; TypeScript cannot verify Rust serialization.
  if (error && (typeof error === 'object' || typeof error === 'function')) {
    const failure = error as Failure;
    if (Array.isArray(failure.diagnostics) || failure.browserFailure) return new MixtureRuntimeError(operation, failure);
  }
  return browserError(operation, 'BINDING_FAILED', 'The binding could not complete the operation', { cause: String(error) });
}

// Read only own data descriptors. Accessors and non-plain objects are rejected,
// including accessors whose getter would otherwise appear harmless.
function record(value: unknown, operation: string, label: string, allowed?: readonly string[]): Record<string, unknown> {
  if (value === null || typeof value !== 'object' || ![Object.prototype, null].includes(Object.getPrototypeOf(value))) {
    throw invalid(operation, `${label} must be a plain object`);
  }
  const output: Record<string, unknown> = Object.create(null);
  for (const key of Reflect.ownKeys(value)) {
    if (typeof key !== 'string' || (allowed && !allowed.includes(key))) throw invalid(operation, `Unsupported ${label} key`);
    const descriptor = Object.getOwnPropertyDescriptor(value, key);
    if (!descriptor || !('value' in descriptor) || !descriptor.enumerable) throw invalid(operation, `${label} must contain enumerable data properties`);
    output[key] = descriptor.value;
  }
  return output;
}

function array(value: unknown, operation: string, label: string, maxLength: number): unknown[] {
  if (!Array.isArray(value) || Object.getPrototypeOf(value) !== Array.prototype) throw invalid(operation, `${label} must be an array`);
  const length = Object.getOwnPropertyDescriptor(value, 'length')!.value;
  if (length > maxLength) throw invalid(operation, `${label} is too long`);
  const keys = Reflect.ownKeys(value);
  if (keys.length !== length + 1) throw invalid(operation, `${label} cannot contain holes or extra properties`);
  const output = [];
  for (let index = 0; index < length; index += 1) {
    const descriptor = Object.getOwnPropertyDescriptor(value, String(index));
    if (!descriptor || !('value' in descriptor)) throw invalid(operation, `${label} cannot contain accessors or holes`);
    output.push(descriptor.value);
  }
  return output;
}

export function captureOptions(options: unknown, operation: string, allowed: readonly string[]) { return record(options, operation, 'options', allowed); }

export function captureRequest(options: unknown = {}, operation: string, defaultLimits: Readonly<SafetyLimits>, defaultResources: Readonly<ResourceLimits>): BindingRequest {
  const captured = record(options, operation, 'request', ['size', 'channels', 'overrides', 'limits', 'resources', 'resourceLimits']);
  const limits = { ...defaultLimits };
  if ('limits' in captured) {
    for (const [key, value] of Object.entries(record(captured.limits, operation, 'limits', Object.keys(defaultLimits)))) {
      if (typeof value !== 'bigint' || value < 0n || value > U64_MAX) throw invalid(operation, `limits.${key} must be a nonnegative u64 bigint`);
      limits[key as keyof SafetyLimits] = value;
    }
  }
  const size = 'size' in captured ? array(captured.size, operation, 'size', 2) : [64, 64];
  if (size.length !== 2 || size.some(value => typeof value !== 'number' || !Number.isSafeInteger(value) || value < 1 || value > U32_MAX)) {
    throw invalid(operation, 'size must contain two positive u32 integers');
  }
  const channels = 'channels' in captured ? array(captured.channels, operation, 'channels', 8) : ['baseColor'];
  if (!channels.length || channels.some(value => typeof value !== 'string') || new Set(channels).size !== channels.length) {
    throw invalid(operation, 'channels must contain unique case-sensitive channel identifiers');
  }
  const overrides: Record<string, ParameterValue> = Object.create(null);
  if ('overrides' in captured) {
    const entries = record(captured.overrides, operation, 'overrides');
    if (BigInt(Object.keys(entries).length) > limits.exposedParameters) throw invalid(operation, 'Override count exceeds the exposed parameter limit');
    for (const [key, value] of Object.entries(entries)) {
      if (typeof value === 'number') {
        if (!Number.isFinite(value) || (Number.isInteger(value) && !Number.isSafeInteger(value))) throw invalid(operation, 'Override numbers must be finite and exact');
        overrides[key] = value;
      } else if (typeof value === 'string') overrides[key] = value;
      else if (Array.isArray(value)) {
        const components = array(value, operation, 'color override', 4);
        if (components.length !== 4 || components.some(component => typeof component !== 'number' || !Number.isFinite(component))) {
          throw invalid(operation, 'Color overrides need exactly four finite numbers');
        }
        overrides[key] = components as [number, number, number, number];
      } else throw invalid(operation, 'Override values must be a number, enum string, or four-number color');
    }
  }
  const resourceLimits = { ...defaultResources };
  if ('resourceLimits' in captured) {
    for (const [key, value] of Object.entries(record(captured.resourceLimits, operation, 'resourceLimits', Object.keys(defaultResources)))) {
      if (typeof value !== 'bigint' || value < 0n || value > U64_MAX) throw invalid(operation, 'Resource limits must be nonnegative u64 bigints');
      resourceLimits[key as keyof ResourceLimits] = value;
    }
  }
  const resources: ImageBinding[] = [];
  if ('resources' in captured) {
    if (Array.isArray(captured.resources)) {
      resourceBudget(operation, 'resourceCount', resourceLimits.resourceCount, BigInt(Object.getOwnPropertyDescriptor(captured.resources, 'length')!.value));
    }
    const entries = array(captured.resources, operation, 'resources', Number.MAX_SAFE_INTEGER);
    for (const entry of entries) {
      const image = record(entry, operation, 'resource', ['id', 'width', 'height', 'format', 'bytesPerRow', 'data']);
      if (typeof image.id !== 'string' || typeof image.format !== 'string') throw invalid(operation, 'Resource id and format must be strings');
      for (const key of ['width', 'height', 'bytesPerRow']) {
        const value = image[key];
        if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0 || (key !== 'bytesPerRow' && value > U32_MAX)) throw invalid(operation, 'Resource dimensions and stride must be unsigned exact integers');
      }
      const data = imageBytes(image.data, operation);
      resources.push({ id: image.id, format: image.format as 'rgba8-linear', width: image.width as number,
        height: image.height as number, bytesPerRow: image.bytesPerRow as number, data });
    }
  }
  // Borrow only until synchronous Rust preparation returns, before any yield.
  return { resources, resourceLimits, size: size as [number, number], channels: channels as string[], limits, overridesJson: JSON.stringify(overrides) };
}

export function captureSource(source: unknown, limit: bigint, operation: string): Uint8Array<ArrayBuffer> {
  if (typeof source === 'string') {
    // Count UTF-8 before allocating; reject unpaired UTF-16 rather than replace it.
    let bytes = 0;
    for (let index = 0; index < source.length; index += 1) {
      const code = source.charCodeAt(index);
      if (code >= 0xd800 && code <= 0xdbff) {
        const low = source.charCodeAt(index + 1);
        if (!(low >= 0xdc00 && low <= 0xdfff)) throw invalid(operation, 'Source contains an unpaired UTF-16 surrogate');
        index += 1; bytes += 4;
      } else if (code >= 0xdc00 && code <= 0xdfff) throw invalid(operation, 'Source contains an unpaired UTF-16 surrogate');
      else bytes += code < 0x80 ? 1 : code < 0x800 ? 2 : 3;
      if (BigInt(bytes) > limit) throw sourceLimit(operation, limit, BigInt(bytes));
    }
    return new TextEncoder().encode(source);
  }
  return copyBytes(source, operation, limit);
}

const typedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
const byteLengthOf = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'byteLength')!.get!;
const bufferOf = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'buffer')!.get!;

export function copyBytes(source: unknown, operation: string, limit?: bigint): Uint8Array<ArrayBuffer> {
  if (!(source instanceof Uint8Array) || Object.getPrototypeOf(source) !== Uint8Array.prototype) throw invalid(operation, 'Bytes must be an ordinary Uint8Array');
  // Intrinsic access ignores caller-owned byteLength/buffer/slice overrides.
  const buffer = bufferOf.call(source);
  const length = byteLengthOf.call(source);
  if (typeof SharedArrayBuffer !== 'undefined' && buffer instanceof SharedArrayBuffer) throw invalid(operation, 'Shared byte buffers are unsupported');
  if (limit !== undefined && BigInt(length) > limit) throw sourceLimit(operation, limit, BigInt(length));
  const copy = new Uint8Array(length);
  Uint8Array.prototype.set.call(copy, source);
  return copy;
}

function resourceBudget(operation: string, name: keyof ResourceLimits, configured: bigint, observed: bigint) {
  if (observed <= configured) return;
  const codes = { resourceCount: 'COUNT', resourcePixels: 'PIXELS', resourceBytes: 'BYTES' };
  throw new MixtureRuntimeError(operation, { diagnostics: [{ code: 'MIX_LIMIT_RESOURCE_' + codes[name] + '_EXCEEDED',
    stage: 'compile', severity: 'error', message: 'Resource budget exceeded.', evidence: { limit: name, configured, observed },
    suggestion: 'Reduce resource input or explicitly select a larger policy.' }] });
}

function imageBytes(value: unknown, operation: string): Uint8Array<ArrayBuffer> {
  if (!(value instanceof Uint8Array) || Object.getPrototypeOf(value) !== Uint8Array.prototype) throw invalid(operation, 'Resource data must be an ordinary Uint8Array');
  const buffer = bufferOf.call(value);
  const resizable = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, 'resizable')?.get;
  if (!(buffer instanceof ArrayBuffer) || (resizable && resizable.call(buffer))) throw invalid(operation, 'Resource buffers must be non-shared and non-resizable');
  try { new Uint8Array(buffer, 0, 0); } catch { throw invalid(operation, 'Detached resource buffer'); }
  const offset = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'byteOffset')!.get!.call(value);
  return new Uint8Array(buffer, offset, byteLengthOf.call(value));
}

function sourceLimit(operation: string, configured: bigint, observed: bigint) {
  // This failure precedes copying, but keeps the core limit identity/evidence.
  return new MixtureRuntimeError(operation, { diagnostics: [{ code: 'MIX_LIMIT_DECODED_BYTES_EXCEEDED',
    stage: 'parse', severity: 'error', message: 'Decoded source exceeds the byte limit',
    evidence: { limit: 'decodedBytes', configured, observed },
    suggestion: 'Reduce the source or explicitly select a different safety policy.' }] });
}

function packageBudget(operation: string, name: keyof PackageLimits, configured: bigint, observed: bigint) {
  if (observed <= configured) return;
  throw new MixtureRuntimeError(operation, { diagnostics: [{code:'MIX_PACKAGE_LIMIT_EXCEEDED',stage:'package',severity:'error',
    message:'Package budget exceeded.',evidence:{limit:name,configured,observed},suggestion:'Reduce the asset or choose a policy within the v1 ceilings.'}] });
}

function capturePackage(source: unknown, options: unknown, operation: string, safety: Readonly<SafetyLimits>, resources: Readonly<ResourceLimits>, ceilings: Readonly<PackageLimits>, render: boolean): [Uint8Array<ArrayBuffer>, PackageBindingRequest] {
  const captured = record(options === undefined ? {} : options, operation, 'package options', render ? ['size','channels','overrides','limits','resourceLimits','packageLimits'] : ['limits','resourceLimits','packageLimits']);
  const packageLimits = {...ceilings};
  if ('packageLimits' in captured) {
    for (const [key,value] of Object.entries(record(captured.packageLimits,operation,'packageLimits',Object.keys(ceilings)))) {
      if (typeof value !== 'bigint' || value < 0n || value > U64_MAX) throw invalid(operation,'Package limits must be nonnegative u64 bigints');
      packageBudget(operation,key as keyof PackageLimits,ceilings[key as keyof PackageLimits],value);
      packageLimits[key as keyof PackageLimits] = value;
    }
  }
  delete captured.packageLimits;
  const request = captureRequest(captured,operation,safety,resources);
  // Do not enumerate millions of indexed byte properties. Only intrinsic view
  // state matters; reject caller-owned overrides without invoking accessors.
  if (!(source instanceof Uint8Array) || Object.getPrototypeOf(source) !== Uint8Array.prototype) throw invalid(operation,'Package bytes must be an ordinary Uint8Array');
  for (const key of ['buffer','byteOffset','byteLength','length','slice','subarray','set']) {
    if (Object.getOwnPropertyDescriptor(source,key)) throw invalid(operation,'Package views cannot override intrinsic byte properties');
  }
  let view: Uint8Array<ArrayBuffer>;
  try { view = imageBytes(source,operation); } catch { throw invalid(operation,'Package bytes must be a non-detached, non-shared, non-resizable ordinary view'); }
  const length = BigInt(view.byteLength);
  packageBudget(operation,'packageBytes',packageLimits.packageBytes,length);
  packageBudget(operation,'packageBufferBytes',packageLimits.packageBufferBytes,2n*length);
  let bytes: Uint8Array<ArrayBuffer>;
  try { bytes = new Uint8Array(view.byteLength); Uint8Array.prototype.set.call(bytes,view); }
  catch { throw new MixtureRuntimeError(operation,{diagnostics:[{code:'MIX_PACKAGE_ALLOCATION_FAILED',stage:'package',severity:'error',message:'Cannot allocate the accepted package snapshot.'}]}); }
  return [bytes,{request,packageLimits}];
}

export function createRuntimeModule(bindings: Bindings, expectedBuild: BuildInfo): RuntimeModule {
  const actual = bindings.build_info();
  for (const key of ['buildId', 'apiSchemaVersion', 'runtimeVersion', 'engineVersion', 'engineRevision', 'engineDirty'] as const) {
    if (actual[key] !== expectedBuild[key]) throw browserError('loadRuntime', 'BUILD_MISMATCH', `JS/WASM ${key} mismatch`, { expected: expectedBuild[key], actual: actual[key] });
  }
  const defaultLimits = Object.freeze(bindings.default_limits());
  const defaultResources = Object.freeze(bindings.default_resource_limits());
  const defaultPackages = Object.freeze(bindings.default_package_limits());
  function cpu(operation: 'inspect', source: Source, options?: RenderRequest): Inspection;
  function cpu(operation: 'validate', source: Source, options?: RenderRequest): ValidationResult;
  function cpu(operation: 'inspect' | 'validate', source: Source, options?: RenderRequest): ValidationResult {
    return boundary(operation, () => {
      const request = captureRequest(options, operation, defaultLimits, defaultResources);
      const bytes = captureSource(source, request.limits.decodedBytes, operation);
      return bindings[operation === 'validate' ? 'validate_source' : 'inspect_source'](bytes, request);
    });
  }
  return Object.freeze<RuntimeModule>({
    getBuildInfo: () => bindings.build_info(),
    getNodeCatalog: () => boundary('getNodeCatalog', () => bindings.node_catalog()),
    validate: (source, request) => {
      try { return cpu('validate', source, request); }
      catch (error) { if (error instanceof MixtureRuntimeError && error.diagnostics.length) return { ok: false, diagnostics: error.diagnostics }; throw error; }
    },
    inspect: (source, request) => cpu('inspect', source, request),
    inspectPackage: (bytes, options) => boundary('inspectPackage', () => bindings.inspect_package(...capturePackage(bytes,options,'inspectPackage',defaultLimits,defaultResources,defaultPackages,false))),
    async createGpu(options = {}) {
      const captured = captureOptions(options, 'createGpu', ['powerPreference']);
      const power = captured.powerPreference ?? 'high-performance';
      if (power !== 'low-power' && power !== 'high-performance') throw invalid('createGpu', 'Unsupported power preference');
      if (globalThis.isSecureContext !== true || !(globalThis.navigator as (Navigator & { gpu?: unknown }) | undefined)?.gpu) {
        throw browserError('createGpu', 'WEBGPU_UNAVAILABLE', 'WebGPU requires a supported browser and secure serving context');
      }
      let low: GpuBinding;
      try { low = await bindings.create_gpu(power); }
      catch (error) { throw normalizeError('createGpu', error); }
      let closing = false;
      let active: Promise<RenderResult> | null = null;
      let destroyed: Promise<void> | null = null;
      const context = low.context_report();
      function accept(operation: string, capture: () => PreparedBinding): Promise<RenderResult> {
        if (closing) return Promise.reject(browserError(operation,'RUNTIME_DESTROYED','Runtime destruction has started'));
        if (active) return Promise.reject(browserError(operation,'RUNTIME_BUSY','This runtime already has an accepted render'));
        let prepared: PreparedBinding;
        try { prepared = capture(); } catch (error) { return Promise.reject(normalizeError(operation,error)); }
        active = Promise.resolve().then(() => low.render(prepared))
          .catch(error => { throw normalizeError(operation,error); }).finally(() => { active = null; });
        return active;
      }
      return Object.freeze<GpuRuntime>({ context,
        render(source, options = {}) {
          return accept('render', () => {
            const request = captureRequest(options, 'render', defaultLimits, defaultResources);
            const bytes = captureSource(source, request.limits.decodedBytes, 'render');
            return bindings.prepare_source(bytes, request);
          });
        },
        renderPackage(bytes, options = {}) {
          return accept('renderPackage', () => bindings.prepare_package(...capturePackage(bytes,options,'renderPackage',defaultLimits,defaultResources,defaultPackages,true)));
        },
        destroy() {
          if (destroyed) return destroyed;
          closing = true;
          const accepted = active;
          destroyed = Promise.resolve(accepted).catch(() => {}).then(() => {
            try { low.destroy(); } finally { low.free(); }
          });
          return destroyed;
        },
      });
    },
  });
}
