import type { RuntimeModule } from './types.js';
export type * from './types.js';
import { buildInfo } from './build-info.js';
import { browserError, captureOptions, copyBytes, createRuntimeModule, MixtureRuntimeError } from './runtime.js';
export { MixtureRuntimeError };

/** Import is inert. Only this call imports generated glue and initializes WASM. */
export async function loadRuntime(options: { wasm?: URL | string | Uint8Array } = {}): Promise<RuntimeModule> {
  const captured = captureOptions(options, 'loadRuntime', ['wasm']);
  const input = captured.wasm;
  if (input !== undefined && !(typeof input === 'string' || input instanceof URL || input instanceof Uint8Array)) {
    throw browserError('loadRuntime', 'INVALID_ARGUMENT', 'wasm must be a URL, string, or Uint8Array');
  }
  // Capture caller-owned bytes and resolve locations before asynchronous work.
  let phase = 'resolve';
  let url: URL | null = null;
  try {
    let bytes = input instanceof Uint8Array ? copyBytes(input, 'loadRuntime') : null;
    url = bytes ? null : input === undefined
      ? new URL('../wasm/mixture_wasm_bg.wasm', import.meta.url)
      : new URL(input as string | URL, globalThis.location?.href ?? import.meta.url);
    phase = 'binding-import';
    const { default: createBindings } = await import('../wasm/bindings.mjs');
    if (!bytes) {
      phase = 'fetch';
      const response = await fetch(url!);
      if (!response.ok) throw new Error(`HTTP ${response.status} ${response.statusText}`);
      bytes = new Uint8Array(await response.arrayBuffer());
    }
    phase = 'compile-instantiate';
    const bindings = createBindings();
    await bindings({ module_or_path: bytes });
    return createRuntimeModule(bindings, buildInfo);
  } catch (error) {
    if (error instanceof MixtureRuntimeError) throw error;
    throw browserError('loadRuntime', 'WASM_LOAD_FAILED', 'Could not load the packaged Mixture WebAssembly runtime',
      { phase, ...(url ? { url: url.href } : {}), cause: String(error) });
  }
}
