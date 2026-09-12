export type Source = string | Uint8Array;
export type ChannelId = 'baseColor' | 'normal' | 'roughness' | 'metallic' | 'height' | 'ambientOcclusion' | 'opacity' | 'emissive';
export type PortKind = 'color' | 'scalar' | 'normal';
export type ParameterValue = number | string | [number, number, number, number];
export type ProjectedValue = null | boolean | number | bigint | string | ProjectedValue[] | { [key: string]: ProjectedValue };
export interface SafetyLimits {
  decodedBytes: bigint; nodes: bigint; edges: bigint; exposedParameters: bigint;
  outputDimension: bigint; requestedOutputs: bigint; transientBytes: bigint;
}
export interface RenderRequest {
  size?: [number, number]; channels?: ChannelId[];
  overrides?: Record<string, ParameterValue>; limits?: Partial<SafetyLimits>;
}
export interface BuildInfo {
  runtimeVersion: string; apiSchemaVersion: number; engineVersion: string;
  engineRevision: string | null; engineDirty: boolean | null; buildId: string;
}
export interface Diagnostic {
  code: string; stage: string; severity: 'error' | 'warning' | 'info'; message: string;
  nodeId?: string; portId?: string; parameterId?: string; documentPath?: string;
  evidence?: Record<string, string | bigint | boolean>; suggestion?: string;
}
export interface BrowserFailure { code: string; operation: string; message: string; evidence?: unknown; suggestion?: string; }
export class MixtureRuntimeError extends Error {
  readonly operation: string; readonly code: string | undefined; readonly diagnostics: Diagnostic[];
  readonly browserFailure?: BrowserFailure; readonly evidence?: unknown;
}
export type ParameterKind = { type: 'float' | 'integer'; min: number; max: number }
  | { type: 'color' } | { type: 'enum'; values: string[] };
export interface ParameterContract { id: string; kind: ParameterKind; default: ParameterValue | null; }
export interface PortDefault { kind: PortKind; value: number | number[]; }
export interface PortContract { id: string; kind: PortKind; default: PortDefault | null; }
export interface NodeContract {
  typeId: string; version: number; label: string; description: string;
  inputs: PortContract[]; outputs: PortContract[]; parameters: ParameterContract[];
}
export interface Endpoint { nodeId: string; portId: string; }
export type InputSource = { source: 'connected'; from: Endpoint } | { source: 'default'; value: PortDefault };
export interface MaterialChannel { id: ChannelId; kind: PortKind; input: InputSource; }
export interface ExposedParameter {
  id: string; nodeId: string; nodeType: string; nodeVersion: number; parameterId: string;
  contract: ParameterContract; sourceValue: ParameterValue; effectiveValue: ParameterValue;
}
export interface PlanEstimates {
  textureBytes: bigint; uniformBytes: bigint; paddedBytesPerRow: number;
  readbackBufferBytes: bigint; readbackBytes: bigint; cumulativeReadbackBytes: bigint;
  cumulativeBytes: bigint; peakBytes: bigint;
}
export interface RenderPlan {
  version: number; documentVersion: number; hash: string; size: [number, number];
  materialOutput: { id: string; typeId: string; version: number };
  passes: Array<{ id: number; origin: ProjectedValue; kernel: ProjectedValue; output: number;
    outputDesc: ProjectedValue; dispatch: [number, number, number] }>;
  outputs: Array<{ channel: ChannelId; kind: PortKind; input: InputSource; resource: number }>;
  estimates: PlanEstimates;
}
export interface Inspection {
  ok: true; diagnostics: Diagnostic[]; documentVersion: number; plan: RenderPlan;
  exposedParameters: ExposedParameter[]; materialChannels: MaterialChannel[];
}
export type ValidationResult = Inspection | { ok: false; diagnostics: Diagnostic[] };
export interface RenderedChannel {
  channel: ChannelId; kind: PortKind; source: InputSource; size: [number, number];
  encoding: 'rgba8-srgb' | 'rgba8-linear'; pixels: Uint8Array;
}
export interface RenderReport {
  planHash: string; adapter: Record<string, ProjectedValue>; size: [number, number];
  passCount: bigint; pipelineCache: { hits: number; misses: number; entries: bigint }; estimates: PlanEstimates;
  allocations: Record<string, bigint>; readbackBytes: bigint; mappedBytes: bigint; rgbaBytes: bigint;
  timings: Record<string, number>;
}
export interface RenderResult { documentVersion: number; plan: RenderPlan; channels: RenderedChannel[]; report: RenderReport; }
export interface GpuRuntime {
  readonly context: Record<string, ProjectedValue>;
  render(source: Source, request?: RenderRequest): Promise<RenderResult>;
  destroy(): Promise<void>;
}
export interface RuntimeModule {
  getBuildInfo(): BuildInfo;
  getNodeCatalog(): NodeContract[];
  validate(source: Source, request?: RenderRequest): ValidationResult;
  inspect(source: Source, request?: RenderRequest): Inspection;
  createGpu(options?: { powerPreference?: 'low-power' | 'high-performance' }): Promise<GpuRuntime>;
}
/** Default URL is package-relative. Supplied bytes perform no loader fetch. */
export function loadRuntime(options?: { wasm?: URL | string | Uint8Array }): Promise<RuntimeModule>;
