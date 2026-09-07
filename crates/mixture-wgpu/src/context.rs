//! Caller-owned adapter, device, and queue acquisition. No rendering occurs here.

use std::{error::Error, fmt};

use mixture_core::error::{Diagnostic, DiagnosticCode, Stage};
use serde::Serialize;

use crate::diagnostics::{AdapterDiagnostics, ContextReport, DeviceDiagnostics, RequestedPolicy};

/// Native backends permitted for this request. `None` disables GPU acquisition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendPreference {
    /// Let wgpu select among the compiled native backends.
    #[default]
    Auto,
    /// Require the Vulkan backend.
    Vulkan,
    /// Require the Metal backend.
    Metal,
    /// Require the Direct3D 12 backend.
    Dx12,
    /// Disable acquisition, returning an adapter diagnostic without initializing wgpu.
    None,
}

impl BackendPreference {
    fn backends(self) -> wgpu::Backends {
        match self {
            Self::Auto => wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12,
            Self::Vulkan => wgpu::Backends::VULKAN,
            Self::Metal => wgpu::Backends::METAL,
            Self::Dx12 => wgpu::Backends::DX12,
            Self::None => wgpu::Backends::empty(),
        }
    }
}

/// An adapter preference, not a guarantee about the selected device type.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PowerPreference {
    /// Prefer a lower-power adapter.
    LowPower,
    /// Prefer a higher-performance adapter.
    #[default]
    HighPerformance,
}

/// Explicit selection policy. No `WGPU_*` environment overrides are applied.
///
/// PR-003 requests no optional device features and `wgpu::Limits::default()`.
/// These requirements are reported, checked, and never silently reduced.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuContextOptions {
    /// Backends that may participate in adapter selection.
    pub backend: BackendPreference,
    /// Preference passed directly to wgpu for adapter selection.
    pub power_preference: PowerPreference,
    /// Require a software adapter through wgpu's `force_fallback_adapter` policy.
    /// This is still the same wgpu execution path; it is not a CPU renderer.
    pub software_adapter: bool,
}

/// Owns all GPU handles. Construction is explicit and asynchronous.
#[derive(Debug)]
pub struct GpuContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    report: ContextReport,
}

impl GpuContext {
    /// Acquire a headless adapter and device exactly once under `options`.
    ///
    /// Success means acquisition only, and always reports `unverified`.
    /// No shaders, command buffers, textures, or readback probes are created.
    pub async fn request(options: GpuContextOptions) -> Result<Self, GpuContextError> {
        let backends = options.backend.backends() & wgpu::Instance::enabled_backend_features();
        let requested = RequestedPolicy::new(options, backends);
        if backends.is_empty() {
            return Err(GpuContextError::new(
                requested,
                None,
                Diagnostic::error(
                    DiagnosticCode::GpuAdapterUnavailable,
                    Stage::GpuAdapter,
                    "No compiled GPU backend is permitted by the requested policy.",
                )
                .with_suggestion("Select auto or a native backend available on this platform."),
            ));
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: match options.power_preference {
                    PowerPreference::LowPower => wgpu::PowerPreference::LowPower,
                    PowerPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
                },
                force_fallback_adapter: options.software_adapter,
                compatible_surface: None,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|source| {
                GpuContextError::new(
                    requested.clone(),
                    None,
                    acquisition_diagnostic(
                        DiagnosticCode::GpuAdapterUnavailable,
                        Stage::GpuAdapter,
                        "No adapter satisfied the requested GPU policy.",
                        source,
                    )
                    .with_suggestion(
                        "Check the installed driver and requested backend/software policy.",
                    ),
                )
            })?;
        let adapter_info = AdapterDiagnostics::from_adapter(&adapter);
        if let Some(diagnostic) = unsupported_limits(&requested.required_limits, &adapter.limits())
        {
            return Err(GpuContextError::new(
                requested,
                Some(adapter_info),
                diagnostic,
            ));
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Mixture GPU context"),
                required_features: wgpu::Features::empty(),
                required_limits: requested.required_limits.clone(),
                ..Default::default()
            })
            .await
            .map_err(|source| {
                GpuContextError::new(
                    requested.clone(),
                    Some(adapter_info.clone()),
                    acquisition_diagnostic(
                        DiagnosticCode::GpuDeviceRequestFailed,
                        Stage::GpuDevice,
                        "The selected adapter could not create the requested device.",
                        source,
                    )
                    .with_suggestion(
                        "Inspect the driver evidence and reported device requirements.",
                    ),
                )
            })?;
        let report = ContextReport::acquired(
            requested,
            adapter_info,
            DeviceDiagnostics::from_device(&device),
        );
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            report,
        })
    }

    /// Borrow the instance owned by this context.
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
    /// Borrow the selected adapter.
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
    /// Borrow the device created for this context.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    /// Borrow the queue associated with this device.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    /// Borrow the acquisition report, including requested and actual evidence.
    pub fn report(&self) -> &ContextReport {
        &self.report
    }
}

/// A stable Mixture diagnostic with requested policy and any selected adapter.
#[derive(Debug)]
pub struct GpuContextError {
    report: Box<ContextReport>,
    diagnostic: Box<Diagnostic>,
}

impl GpuContextError {
    fn new(
        requested: RequestedPolicy,
        adapter: Option<AdapterDiagnostics>,
        diagnostic: Diagnostic,
    ) -> Self {
        Self {
            report: Box::new(ContextReport::failed(
                requested,
                adapter,
                diagnostic.clone(),
            )),
            diagnostic: Box::new(diagnostic),
        }
    }
    /// Borrow the original stable Mixture diagnostic.
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
    /// Borrow the acquisition report, including requested and actual evidence.
    pub fn report(&self) -> &ContextReport {
        &self.report
    }
}

impl fmt::Display for GpuContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.diagnostic.fmt(f)
    }
}

impl Error for GpuContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.diagnostic.as_ref())
    }
}

fn acquisition_diagnostic(
    code: DiagnosticCode,
    stage: Stage,
    message: &str,
    source: impl Error + Send + Sync + 'static,
) -> Diagnostic {
    Diagnostic::error(code, stage, message)
        .with_evidence("driverMessage", source.to_string())
        .with_source(source)
}

fn unsupported_limits(required: &wgpu::Limits, supported: &wgpu::Limits) -> Option<Diagnostic> {
    let mut first = None;
    required.check_limits_with_fail_fn(supported, true, |name, requested, available| {
        first = Some(
            Diagnostic::error(
                DiagnosticCode::GpuDeviceRequestFailed,
                Stage::GpuDevice,
                "The selected adapter does not support the required device limits.",
            )
            .with_evidence("limit", name)
            .with_evidence("requested", requested)
            .with_evidence("supported", available)
            .with_suggestion(
                "Use an adapter that supports the reported baseline device requirements.",
            ),
        );
    });
    first
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_limit_failures_are_typed_before_request_device_can_panic() {
        let required = wgpu::Limits::default();
        assert!(unsupported_limits(&required, &required).is_none());
        let supported = wgpu::Limits {
            max_texture_dimension_2d: 1,
            ..required.clone()
        };
        let diagnostic =
            unsupported_limits(&required, &supported).expect("insufficient adapter limits");
        assert_eq!(diagnostic.code, DiagnosticCode::GpuDeviceRequestFailed);
        assert_eq!(diagnostic.stage, Stage::GpuDevice);
        assert_eq!(
            diagnostic.evidence["limit"],
            "max_texture_dimension_2d".into()
        );
        // Alignment limits have the opposite comparison direction to maximum sizes.
        let supported = wgpu::Limits {
            min_uniform_buffer_offset_alignment: required.min_uniform_buffer_offset_alignment * 2,
            ..required.clone()
        };
        assert!(unsupported_limits(&required, &supported).is_some());
    }

    #[test]
    fn context_native_device_errors_preserve_source_and_json_evidence() {
        let source = std::io::Error::other("driver device creation failure");
        let diagnostic = acquisition_diagnostic(
            DiagnosticCode::GpuDeviceRequestFailed,
            Stage::GpuDevice,
            "Device request failed.",
            source,
        );
        let error = GpuContextError::new(
            RequestedPolicy::new(GpuContextOptions::default(), wgpu::Backends::METAL),
            Some(AdapterDiagnostics {
                name: "Test adapter whose device request failed".into(),
                device_type: "IntegratedGpu".into(),
                backend: "Metal".into(),
                vendor: 0,
                device: 0,
                driver: "Test driver".into(),
                driver_info: String::new(),
                supported_limits: wgpu::Limits::default(),
                supported_features: Vec::new(),
            }),
            diagnostic,
        );
        let original = error.source().unwrap().source().unwrap();
        assert!(original.downcast_ref::<std::io::Error>().is_some());
        let json = serde_json::to_value(error.report()).unwrap();
        assert_eq!(json["verdict"], "unhealthy");
        assert_eq!(
            json["adapter"]["name"],
            "Test adapter whose device request failed"
        );
        assert!(json["device"].is_null());
        assert_eq!(
            json["diagnostics"][0]["code"],
            "MIX_GPU_DEVICE_REQUEST_FAILED"
        );
        assert_eq!(json["diagnostics"][0]["stage"], "gpuDevice");
        assert_eq!(
            json["diagnostics"][0]["evidence"]["driverMessage"],
            "driver device creation failure"
        );
        assert!(json["diagnostics"][0].get("source").is_none());
    }
}
