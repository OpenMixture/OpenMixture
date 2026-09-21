import type { BrowserFailure, BuildInfo, Diagnostic, GpuRuntime, Inspection, NodeContract, RenderResult, SafetyLimits, ValidationResult } from './types.js';

// Manually reviewed Rust/JS projection contract, verified against real WASM.
// Generated wasm-bindgen declarations expose JsValue as any; they do not prove it.
export interface BindingRequest {
  size: [number, number]; channels: string[]; limits: SafetyLimits; overridesJson: string;
}
export interface Failure {
  diagnostics?: Diagnostic[]; browserFailure?: BrowserFailure; evidence?: unknown;
}
export interface GpuBinding {
  context_report(): GpuRuntime['context'];
  render(source: Uint8Array, request: BindingRequest): Promise<RenderResult>;
  destroy(): void;
  free(): void;
}
export interface Bindings {
  (options: { module_or_path: Uint8Array }): Promise<unknown>;
  build_info(): BuildInfo;
  default_limits(): SafetyLimits;
  node_catalog(): NodeContract[];
  validate_source(source: Uint8Array, request: BindingRequest): ValidationResult;
  inspect_source(source: Uint8Array, request: BindingRequest): Inspection;
  create_gpu(power: 'low-power' | 'high-performance'): Promise<GpuBinding>;
}
