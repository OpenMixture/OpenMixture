// Compile against emitted declarations, not the implementation sources.
import { loadRuntime, MixtureRuntimeError } from '../dist/index.js';
import type { BuildInfo, Diagnostic, GpuRuntime, PortContract, RenderRequest, RenderResult, RuntimeModule } from '../dist/index.js';

const loading: Promise<RuntimeModule> = loadRuntime({ wasm: new Uint8Array() });
const module = await loading;
const request: RenderRequest = { size: [65, 3], channels: ['baseColor'], limits: { decodedBytes: 2048n }, overrides: { seed: 42, tint: [1, 0, 0, 1] } };
const validation = module.validate('{}', request);
if (validation.ok) {
  const bytes: bigint = validation.plan.estimates.peakBytes;
  const dimension: number = validation.plan.size[0];
} else {
  const diagnostics: Diagnostic[] = validation.diagnostics;
  // @ts-expect-error Rejected validation has no plan.
  validation.plan;
}
const build: BuildInfo = { ...module.getBuildInfo(), engineRevision: null, engineDirty: null };
const port: PortContract = { id: 'source', kind: 'color', default: null };
const gpu: GpuRuntime = await module.createGpu({ powerPreference: 'low-power' });
const result: RenderResult = await gpu.render('{}', request);
const pixels: Uint8Array = result.channels[0].pixels;
const count: bigint = result.report.passCount;
const hits: number = result.report.pipelineCache.hits;
const destroyed: Promise<void> = gpu.destroy();
const failure = new MixtureRuntimeError('render', { diagnostics: [], evidence: null });
const code: string | undefined = failure.code;
// @ts-expect-error A failure without diagnostics or browserFailure has no code.
failure.code.toLowerCase();
if (failure.code !== undefined) {
  const normalizedCode: string = failure.code.toLowerCase();
}
// @ts-expect-error Dimensions are numbers, not bigint.
module.inspect('{}', { size: [1n, 2n] });
// @ts-expect-error Budgets are bigint, not number.
module.inspect('{}', { limits: { decodedBytes: 2048 } });
// @ts-expect-error Channel IDs remain a closed public union.
gpu.render('{}', { channels: ['unknown'] });
// @ts-expect-error Nullable Rust metadata must be narrowed.
const revision: string = build.engineRevision;
// @ts-expect-error Report counters retain bigint.
const passes: number = result.report.passCount;
// @ts-expect-error Runtime context remains readonly.
gpu.context = {};
// @ts-expect-error Error diagnostics remain readonly.
failure.diagnostics = [];
