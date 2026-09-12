const U32_MAX = 4294967295;
const U64_MAX = 18446744073709551615n;


/** Structured public failures preserve engine diagnostics independently of browser failures. */
export class MixtureRuntimeError extends Error {
  constructor(operation, failure) {
    super(failure.browserFailure?.message ?? failure.diagnostics?.[0]?.message ?? `Mixture ${operation} failed`);
    this.name = 'MixtureRuntimeError';
    this.operation = operation;
    this.diagnostics = failure.diagnostics ?? [];
    if (failure.browserFailure) this.browserFailure = failure.browserFailure;
    if (failure.evidence !== undefined) this.evidence = failure.evidence;
  }
  get code() { return this.diagnostics[0]?.code ?? this.browserFailure?.code; }
}

export function browserError(operation, code, message, evidence) {
  return new MixtureRuntimeError(operation, { diagnostics: [], browserFailure: {
    code: `MIX_BROWSER_${code}`, operation, message, ...(evidence === undefined ? {} : { evidence }),
  } });
}

function invalid(operation, message) { return browserError(operation, 'INVALID_ARGUMENT', message); }

function boundary(operation, action) {
  try { return action(); } catch (error) { throw normalizeError(operation, error); }
}

export function normalizeError(operation, error) {
  if (error instanceof MixtureRuntimeError) return error;
  if (error && (Array.isArray(error.diagnostics) || error.browserFailure)) return new MixtureRuntimeError(operation, error);
  return browserError(operation, 'BINDING_FAILED', 'The binding could not complete the operation', { cause: String(error) });
}

// Read only own data descriptors. Accessors and non-plain objects are rejected,
// including accessors whose getter would otherwise appear harmless.
function record(value, operation, label, allowed) {
  if (value === null || typeof value !== 'object' || ![Object.prototype, null].includes(Object.getPrototypeOf(value))) {
    throw invalid(operation, `${label} must be a plain object`);
  }
  const output = Object.create(null);
  for (const key of Reflect.ownKeys(value)) {
    if (typeof key !== 'string' || (allowed && !allowed.includes(key))) throw invalid(operation, `Unsupported ${label} key`);
    const descriptor = Object.getOwnPropertyDescriptor(value, key);
    if (!descriptor || !('value' in descriptor) || !descriptor.enumerable) throw invalid(operation, `${label} must contain enumerable data properties`);
    output[key] = descriptor.value;
  }
  return output;
}

function array(value, operation, label, maxLength) {
  if (!Array.isArray(value) || Object.getPrototypeOf(value) !== Array.prototype) throw invalid(operation, `${label} must be an array`);
  const length = Object.getOwnPropertyDescriptor(value, 'length').value;
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

export function captureOptions(options, operation, allowed) { return record(options, operation, 'options', allowed); }

export function captureRequest(options = {}, operation, defaultLimits) {
  const captured = record(options, operation, 'request', ['size', 'channels', 'overrides', 'limits']);
  const limits = { ...defaultLimits };
  if ('limits' in captured) {
    for (const [key, value] of Object.entries(record(captured.limits, operation, 'limits', Object.keys(defaultLimits)))) {
      if (typeof value !== 'bigint' || value < 0n || value > U64_MAX) throw invalid(operation, `limits.${key} must be a nonnegative u64 bigint`);
      limits[key] = value;
    }
  }
  const size = 'size' in captured ? array(captured.size, operation, 'size', 2) : [64, 64];
  if (size.length !== 2 || size.some(value => !Number.isSafeInteger(value) || value < 1 || value > U32_MAX)) {
    throw invalid(operation, 'size must contain two positive u32 integers');
  }
  const channels = 'channels' in captured ? array(captured.channels, operation, 'channels', 8) : ['baseColor'];
  if (!channels.length || channels.some(value => typeof value !== 'string') || new Set(channels).size !== channels.length) {
    throw invalid(operation, 'channels must contain unique case-sensitive channel identifiers');
  }
  const overrides = Object.create(null);
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
        overrides[key] = components;
      } else throw invalid(operation, 'Override values must be a number, enum string, or four-number color');
    }
  }
  return { size, channels, limits, overridesJson: JSON.stringify(overrides) };
}

export function captureSource(source, limit, operation) {
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
const byteLengthOf = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'byteLength').get;
const bufferOf = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'buffer').get;

export function copyBytes(source, operation, limit) {
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

function sourceLimit(operation, configured, observed) {
  // This failure precedes copying, but keeps the core limit identity/evidence.
  return new MixtureRuntimeError(operation, { diagnostics: [{ code: 'MIX_LIMIT_DECODED_BYTES_EXCEEDED',
    stage: 'parse', severity: 'error', message: 'Decoded source exceeds the byte limit',
    evidence: { limit: 'decodedBytes', configured, observed },
    suggestion: 'Reduce the source or explicitly select a different safety policy.' }] });
}

export function createRuntimeModule(bindings, expectedBuild) {
  const actual = bindings.build_info();
  for (const key of ['buildId', 'apiSchemaVersion', 'runtimeVersion', 'engineVersion', 'engineRevision', 'engineDirty']) {
    if (actual[key] !== expectedBuild[key]) throw browserError('loadRuntime', 'BUILD_MISMATCH', `JS/WASM ${key} mismatch`, { expected: expectedBuild[key], actual: actual[key] });
  }
  const defaultLimits = Object.freeze(bindings.default_limits());
  const cpu = (operation, source, options) => boundary(operation, () => {
    const request = captureRequest(options, operation, defaultLimits);
    const bytes = captureSource(source, request.limits.decodedBytes, operation);
    return bindings[operation === 'validate' ? 'validate_source' : 'inspect_source'](bytes, request);
  });
  return Object.freeze({
    getBuildInfo: () => bindings.build_info(),
    getNodeCatalog: () => boundary('getNodeCatalog', () => bindings.node_catalog()),
    validate: (source, request) => {
      try { return cpu('validate', source, request); }
      catch (error) { if (error.diagnostics?.length) return { ok: false, diagnostics: error.diagnostics }; throw error; }
    },
    inspect: (source, request) => cpu('inspect', source, request),
    async createGpu(options = {}) {
      const captured = captureOptions(options, 'createGpu', ['powerPreference']);
      const power = captured.powerPreference ?? 'high-performance';
      if (power !== 'low-power' && power !== 'high-performance') throw invalid('createGpu', 'Unsupported power preference');
      if (globalThis.isSecureContext !== true || !globalThis.navigator?.gpu) {
        throw browserError('createGpu', 'WEBGPU_UNAVAILABLE', 'WebGPU requires a supported browser and secure serving context');
      }
      let low;
      try { low = await bindings.create_gpu(power); }
      catch (error) { throw normalizeError('createGpu', error); }
      let closing = false;
      let active = null;
      let destroyed = null;
      const context = low.context_report();
      return Object.freeze({ context,
        render(source, options = {}) {
          if (closing) return Promise.reject(browserError('render', 'RUNTIME_DESTROYED', 'Runtime destruction has started'));
          if (active) return Promise.reject(browserError('render', 'RUNTIME_BUSY', 'This runtime already has an accepted render'));
          let bytes, request;
          try {
            request = captureRequest(options, 'render', defaultLimits);
            bytes = captureSource(source, request.limits.decodedBytes, 'render');
          } catch (error) { return Promise.reject(normalizeError('render', error)); }
          // Acceptance and input capture precede the first asynchronous yield.
          active = Promise.resolve().then(() => low.render(bytes, request))
            .catch(error => { throw normalizeError('render', error); })
            .finally(() => { active = null; });
          return active;
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
