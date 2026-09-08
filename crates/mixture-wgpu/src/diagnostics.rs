//! Serializable context acquisition evidence, independent of CLI presentation.

use mixture_core::error::{Diagnostic, DiagnosticReport};
use serde::Serialize;

use crate::GpuContextOptions;

/// Health is based on an actual compute/readback probe, or acquisition alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorVerdict {
    /// The fixed checker probe executed, mapped, and passed pixel checks.
    Healthy,
    /// Context acquired; compute and readback have not been tested.
    Unverified,
    /// Adapter acquisition or device creation failed.
    Unhealthy,
}

/// Selection policy and exact device requirements used for acquisition.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestedPolicy {
    /// The caller-provided selection policy.
    #[serde(flatten)]
    pub options: GpuContextOptions,
    /// Sorted permitted backends compiled for the current platform.
    pub effective_backends: Vec<String>,
    /// Sorted requested optional features; empty in PR-003.
    pub required_features: Vec<String>,
    /// Baseline wgpu device requirements, never silently reduced.
    /// This field exposes wgpu's version-coupled `Limits` type.
    pub required_limits: wgpu::Limits,
}

impl RequestedPolicy {
    pub(crate) fn new(options: GpuContextOptions, backends: wgpu::Backends) -> Self {
        let mut effective_backends: Vec<_> = backends
            .iter_names()
            .map(|(name, _)| name.to_owned())
            .collect();
        effective_backends.sort();
        Self {
            options,
            effective_backends,
            required_features: Vec::new(),
            required_limits: wgpu::Limits::default(),
        }
    }
}

/// Actual adapter identity and capabilities, not the user's selection preference.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterDiagnostics {
    /// Adapter name supplied by the backend.
    pub name: String,
    /// wgpu device type, such as IntegratedGpu, DiscreteGpu, or Cpu.
    pub device_type: String,
    /// Actual native backend: Vulkan, Metal, or Dx12.
    pub backend: String,
    /// Backend-specific vendor identifier.
    pub vendor: u32,
    /// Backend-specific device identifier.
    pub device: u32,
    /// Driver name, possibly empty when the backend does not provide it.
    pub driver: String,
    /// Additional backend-supplied driver information.
    pub driver_info: String,
    /// Actual adapter limits with limit bucketing disabled.
    /// This field exposes wgpu's version-coupled `Limits` type.
    pub supported_limits: wgpu::Limits,
    /// Sorted names of all optional features supported by this adapter.
    pub supported_features: Vec<String>,
}

impl AdapterDiagnostics {
    pub(crate) fn from_adapter(adapter: &wgpu::Adapter) -> Self {
        let info = adapter.get_info();
        Self {
            name: info.name,
            device_type: format!("{:?}", info.device_type),
            backend: format!("{:?}", info.backend),
            vendor: info.vendor,
            device: info.device,
            driver: info.driver,
            driver_info: info.driver_info,
            supported_limits: adapter.limits(),
            supported_features: feature_names(adapter.features()),
        }
    }
}

/// Capabilities actually enabled on the acquired device.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDiagnostics {
    /// Limits exposed by the acquired device, using wgpu's version-coupled `Limits` type.
    pub limits: wgpu::Limits,
    /// Sorted enabled optional features; empty in PR-003.
    pub features: Vec<String>,
}

impl DeviceDiagnostics {
    pub(crate) fn from_device(device: &wgpu::Device) -> Self {
        Self {
            limits: device.limits(),
            features: feature_names(device.features()),
        }
    }
}

/// Snapshot of acquisition or a checker probe. Only the verified probe can
/// construct a healthy report; callers cannot mutate its verdict or evidence.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextReport {
    schema_version: u32,
    verdict: DoctorVerdict,
    requested: RequestedPolicy,
    adapter: Option<AdapterDiagnostics>,
    device: Option<DeviceDiagnostics>,
    compute_probe: &'static str,
    readback_probe: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution: Option<crate::ExecutionReport>,
    #[serde(flatten)]
    diagnostics: DiagnosticReport,
}

impl ContextReport {
    pub(crate) fn acquired(
        requested: RequestedPolicy,
        adapter: AdapterDiagnostics,
        device: DeviceDiagnostics,
    ) -> Self {
        Self {
            schema_version: 1,
            verdict: DoctorVerdict::Unverified,
            requested,
            adapter: Some(adapter),
            device: Some(device),
            compute_probe: "notRun",
            readback_probe: "notRun",
            execution: None,
            diagnostics: DiagnosticReport::new([]),
        }
    }
    pub(crate) fn failed(
        requested: RequestedPolicy,
        adapter: Option<AdapterDiagnostics>,
        diagnostic: Diagnostic,
    ) -> Self {
        Self {
            schema_version: 1,
            verdict: DoctorVerdict::Unhealthy,
            requested,
            adapter,
            device: None,
            compute_probe: "notRun",
            readback_probe: "notRun",
            execution: None,
            diagnostics: DiagnosticReport::new([diagnostic]),
        }
    }
    /// Compute probe status: notRun, passed, or failed.
    pub fn compute_probe(&self) -> &str {
        self.compute_probe
    }
    /// Readback probe status: notRun, passed, or failed.
    pub fn readback_probe(&self) -> &str {
        self.readback_probe
    }
    /// Evidence from the completed checker execution, when available.
    pub fn execution(&self) -> Option<&crate::ExecutionReport> {
        self.execution.as_ref()
    }
    pub(crate) fn probe_passed(&mut self, execution: crate::ExecutionReport) {
        self.verdict = DoctorVerdict::Healthy;
        self.compute_probe = "passed";
        self.readback_probe = "passed";
        self.execution = Some(execution);
    }
    pub(crate) fn probe_failed(&mut self, diagnostic: Diagnostic, compute_passed: bool) {
        self.verdict = DoctorVerdict::Unhealthy;
        if diagnostic.stage == mixture_core::Stage::Readback {
            self.compute_probe = if compute_passed { "passed" } else { "failed" };
            self.readback_probe = "failed";
        } else {
            self.compute_probe = "failed";
            self.readback_probe = "notRun";
        }
        self.diagnostics = DiagnosticReport::new([diagnostic]);
    }
    /// Acquisition-only or actual probe verdict, depending on the invoked API.
    pub fn verdict(&self) -> DoctorVerdict {
        self.verdict
    }
    /// Requested selection policy and device requirements.
    pub fn requested(&self) -> &RequestedPolicy {
        &self.requested
    }
    /// Selected adapter, also retained if device creation fails.
    pub fn adapter(&self) -> Option<&AdapterDiagnostics> {
        self.adapter.as_ref()
    }
    /// Device capabilities, present only after successful acquisition.
    pub fn device(&self) -> Option<&DeviceDiagnostics> {
        self.device.as_ref()
    }
    /// Deterministically ordered diagnostics and derived acquisition success.
    pub fn diagnostics(&self) -> &DiagnosticReport {
        &self.diagnostics
    }
}

fn feature_names(features: wgpu::Features) -> Vec<String> {
    let mut names: Vec<_> = features
        .iter_names()
        .map(|(name, _)| name.to_owned())
        .collect();
    names.sort();
    names
}
