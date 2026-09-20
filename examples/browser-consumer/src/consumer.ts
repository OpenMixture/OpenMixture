import type { RenderRequest, RuntimeModule, Source } from '@openmixture/runtime';

/** The host owns I/O. Only the public SDK parses and validates material bytes. */
export async function readMaterial(url: URL): Promise<Uint8Array> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Material fetch failed: HTTP ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
}

/** GPU ownership is per invocation; returned pixel arrays outlive destruction. */
export async function renderMaterial(runtime: RuntimeModule, source: Source, request: RenderRequest) {
  const inspection = runtime.inspect(source, request);
  const gpu = await runtime.createGpu({ powerPreference: 'high-performance' });
  try {
    const result = await gpu.render(source, request);
    return { build: runtime.getBuildInfo(), inspection, result };
  } finally {
    await gpu.destroy();
  }
}

/** This host's display format; it is not a Mixture document/report decoder. */
export const formatReport = (value: unknown) => JSON.stringify(value,
  (_key, item) => typeof item === 'bigint' ? item.toString() : item, 2);
