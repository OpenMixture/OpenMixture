//! Explicit headless GPU ownership and acquisition diagnostics.
//!
//! Acquiring a context does not verify compute or readback. There is no global
//! context, implicit initialization, or alternate pixel executor.
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

pub mod context;
pub mod diagnostics;

pub use context::{
    BackendPreference, GpuContext, GpuContextError, GpuContextOptions, PowerPreference,
};
pub use diagnostics::{AdapterDiagnostics, ContextReport, DeviceDiagnostics, DoctorVerdict};
