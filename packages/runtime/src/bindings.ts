import type { BrowserFailure, BuildInfo, Diagnostic, GpuRuntime, ImageBinding, Inspection, NodeContract, RenderResult, ResourceLimits, SafetyLimits, ValidationResult } from './types.js';

// Manually reviewed Rust/JS projection contract, verified against real WASM.
// Generated wasm-bindgen declarations expose JsValue as any; they do not prove it.
export interface BindingRequest {
  size: [number, number]; channels: string[]; limits: SafetyLimits; overridesJson: string;
  resources: ImageBinding[]; resourceLimits: ResourceLimits;
}
export interface Failure {
  diagnostics?: Diagnostic[]; browserFailure?: BrowserFailure; evidence?: unknown;
}
export interface PreparedBinding { free(): void; }
export interface GpuBinding {
  context_report(): GpuRuntime['context'];
  render(prepared: PreparedBinding): Promise<RenderResult>;
  destroy(): void;
  free(): void;
}
export interface Bindings {
  (options: { module_or_path: Uint8Array }): Promise<unknown>;
  build_info(): BuildInfo;
  default_limits(): SafetyLimits;
  default_resource_limits(): ResourceLimits;
  prepare_source(source: Uint8Array, request: BindingRequest): PreparedBinding;
  node_catalog(): NodeContract[];
  validate_source(source: Uint8Array, request: BindingRequest): ValidationResult;
  inspect_source(source: Uint8Array, request: BindingRequest): Inspection;
  create_gpu(power: 'low-power' | 'high-performance'): Promise<GpuBinding>;
}
