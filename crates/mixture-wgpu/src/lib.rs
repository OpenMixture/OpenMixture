//! Explicit headless GPU ownership, checker compute, and readback diagnostics.
//!
//! Acquiring a context does not verify compute or readback. There is no global
//! context, implicit initialization, or alternate pixel executor.
//! `GpuContext::render_checker` runs the fixed built-in; `probe_checker` verifies it.
//!
//! ```no_run
//! use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions};
//!
//! async fn acquire() -> Result<GpuContext, mixture_wgpu::GpuContextError> {
//!     GpuContext::request(GpuContextOptions {
//!         backend: BackendPreference::Auto,
//!         ..Default::default()
//!     }).await
//! }
//! ```
//!
//! ```no_run
//! use mixture_core::SafetyLimits;
//! use mixture_wgpu::{CheckerOutput, CheckerRequest, GpuContext, GpuContextOptions};
//! async fn checker() -> Result<CheckerOutput, Box<dyn std::error::Error>> {
//!     let request = CheckerRequest::default();
//!     let limits = SafetyLimits::default();
//!     request.validate(&limits)?;
//!     let mut context = GpuContext::request(GpuContextOptions::default()).await?;
//!     Ok(context.render_checker(request, &limits).await?)
//! }
//! ```

pub mod checker;
pub mod context;
pub mod diagnostics;
mod operation;
mod readback;

pub use checker::{CheckerOutput, CheckerRequest, ExecutionReport, ExecutionTimings};
pub use context::{
    BackendPreference, GpuContext, GpuContextError, GpuContextOptions, PowerPreference,
};
pub use diagnostics::{AdapterDiagnostics, ContextReport, DeviceDiagnostics, DoctorVerdict};
pub use operation::GpuOperationError;

#[cfg(test)]
mod test_support {
    pub fn options() -> crate::GpuContextOptions {
        let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("auto") => crate::BackendPreference::Auto,
            Ok("vulkan") => crate::BackendPreference::Vulkan,
            Ok("metal") => crate::BackendPreference::Metal,
            Ok("dx12") => crate::BackendPreference::Dx12,
            other => panic!("invalid GPU test backend: {other:?}"),
        };
        let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => false,
            Ok("1") => true,
            other => panic!("invalid GPU test software policy: {other:?}"),
        };
        crate::GpuContextOptions {
            backend,
            software_adapter,
            ..Default::default()
        }
    }
}
