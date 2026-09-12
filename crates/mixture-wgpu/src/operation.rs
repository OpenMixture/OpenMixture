//! Stage-specific failures and scoped GPU operations.

use mixture_core::{Diagnostic, DiagnosticCode, Stage};
use serde::Serialize;
use std::{error::Error, fmt};

use crate::{AdapterDiagnostics, AllocationReport, DeviceLoss, DeviceLossReason, GpuContext};

/// Primary failure classification; stage and the original source remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum GpuFailureReason {
    /// An operation failure without a typed OOM or primary device-loss notification.
    Other,
    /// wgpu reported a typed GPU out-of-memory error, not a parsed driver message.
    OutOfMemory,
    /// A delivered loss notification prevented further work or successful completion.
    DeviceLost,
}

/// Failure in request validation, shader creation, execution, or readback.
#[derive(Debug)]
pub struct GpuOperationError {
    diagnostic: Box<Diagnostic>,
    compute_completed: bool,
    adapter: Option<Box<AdapterDiagnostics>>,
    device_loss: Option<Box<DeviceLoss>>,
    allocations: Option<Box<AllocationReport>>,
}

impl GpuOperationError {
    /// The stable diagnostic, including its original native error chain.
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
    /// Primary typed reason. A later device loss does not replace an earlier error;
    /// inspect [`Self::device_loss`] as well before attempting to reuse a context.
    pub fn reason(&self) -> GpuFailureReason {
        match self.diagnostic.code {
            DiagnosticCode::GpuDeviceLost => GpuFailureReason::DeviceLost,
            DiagnosticCode::GpuOutOfMemory => GpuFailureReason::OutOfMemory,
            _ => GpuFailureReason::Other,
        }
    }
    /// Selected adapter for an execution attempt, when a context was available.
    pub fn adapter(&self) -> Option<&AdapterDiagnostics> {
        self.adapter.as_deref()
    }
    /// The first loss observed by the context, including loss concurrent with a
    /// different primary failure. This record survives context/renderer drop.
    pub fn device_loss(&self) -> Option<&DeviceLoss> {
        self.device_loss.as_deref()
    }
    /// Successful per-call descriptors and their cleanup before returning failure.
    /// Absent for errors before entering the executor; excludes driver overhead
    /// and incomplete allocation batches whose temporary handles were dropped.
    pub fn allocations(&self) -> Option<&AllocationReport> {
        self.allocations.as_deref()
    }
    pub(crate) fn after_compute(mut self) -> Self {
        self.compute_completed = true;
        self
    }
    pub(crate) fn compute_completed(&self) -> bool {
        self.compute_completed
    }

    pub(crate) fn in_plan(mut self, plan: &mixture_core::RenderPlan) -> Self {
        if let Some(mixture_core::EvidenceValue::Unsigned(index)) =
            self.diagnostic.evidence.get("passIndex")
            && let Ok(index) = usize::try_from(*index)
            && let Some(pass) = plan.passes().get(index)
        {
            let node = match &pass.origin {
                mixture_core::plan::PassOrigin::Node { node }
                | mixture_core::plan::PassOrigin::InputDefault { node, .. } => node,
            };
            self.diagnostic.node_id = Some(node.id.clone());
        }
        self.evidence("planHash", plan.hash().as_str())
    }

    pub(crate) fn at(stage: Stage, message: &str) -> Self {
        let code = match stage {
            Stage::Validation => DiagnosticCode::ParameterInvalidValue,
            Stage::GpuShader => DiagnosticCode::GpuShaderValidationFailed,
            Stage::Readback => DiagnosticCode::ReadbackFailed,
            _ => DiagnosticCode::GpuExecutionFailed,
        };
        Diagnostic::error(code, stage, message).with_suggestion(match stage {
            Stage::Validation => "Use positive dimensions within the explicit safety limits.",
            Stage::Readback => "Inspect the readback layout and driver evidence; rerun doctor on the same adapter.",
            _ => "Inspect the GPU stage and driver evidence; reproduce on the pinned software adapter.",
        }).into()
    }
    pub(crate) fn source_error(
        stage: Stage,
        operation: &str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        let mut error = Self::at(stage, operation);
        error.diagnostic = Box::new(
            (*error.diagnostic)
                .with_evidence("driverMessage", source.to_string())
                .with_source(source),
        );
        error
    }

    fn from_wgpu(stage: Stage, operation: &str, source: wgpu::Error) -> Self {
        let out_of_memory = matches!(source, wgpu::Error::OutOfMemory { .. });
        let cause = source.source().map(ToString::to_string);
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        let mut error = Self::source_error(stage, operation, source);
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        // Browser wgpu errors may own non-Send JS objects. Keep typed OOM
        // classification and cause evidence without putting them into core's
        // Send + Sync native source chain or asserting unsafe Send/Sync.
        let mut error = Self::at(stage, operation).evidence("driverMessage", source.to_string());
        if let Some(cause) = cause {
            error = error.evidence("driverCause", cause);
        }
        if out_of_memory {
            error.diagnostic.code = DiagnosticCode::GpuOutOfMemory;
            error.diagnostic.suggestion = Some(
                "Reduce the requested GPU workload; inspect device-loss evidence before reusing this context.".into(),
            );
            error = error.evidence("failureReason", "outOfMemory");
        }
        error
    }

    pub(crate) fn lost(stage: Stage, loss: DeviceLoss) -> Self {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::GpuDeviceLost,
            stage,
            "The GPU context received a device-loss notification.",
        )
        .with_evidence("failureReason", "deviceLost")
        .with_source(loss.clone())
        .with_suggestion(
            "Stop using this context. The application may explicitly acquire a new context; Mixture does not retry or recover automatically.",
        );
        Self::from(diagnostic).with_device_loss(loss)
    }

    fn with_device_loss(mut self, loss: DeviceLoss) -> Self {
        self = self
            .evidence("deviceLost", true)
            .evidence(
                "deviceLostReason",
                match loss.reason {
                    DeviceLossReason::Unknown => "unknown",
                    DeviceLossReason::Destroyed => "destroyed",
                },
            )
            .evidence("deviceLostMessage", loss.message.clone());
        self.device_loss = Some(Box::new(loss));
        self
    }

    pub(crate) fn in_context(mut self, context: &GpuContext) -> Self {
        if let Some(adapter) = context.report().adapter() {
            self = self
                .evidence("adapterName", adapter.name.clone())
                .evidence("backend", adapter.backend.clone());
            self.adapter = Some(Box::new(adapter.clone()));
        }
        if let Some(loss) = context.device_loss() {
            self = self.with_device_loss(loss.clone());
        }
        self
    }

    pub(crate) fn with_allocations(mut self, report: AllocationReport) -> Self {
        for (key, value) in [
            ("allocationCumulativeBytes", report.cumulative_bytes),
            ("allocationPeakBytes", report.peak_bytes),
            ("allocationReleasedBytes", report.released_bytes),
            ("allocationLiveBytes", report.live_bytes),
            ("allocationTextureCount", report.texture_count),
            ("allocationUniformCount", report.uniform_count),
            ("allocationStagingCount", report.staging_count),
        ] {
            self = self.evidence(key, value);
        }
        self.allocations = Some(Box::new(report));
        self
    }

    pub(crate) fn with_cleanup_error(mut self, cleanup: Self) -> Self {
        self = self
            .evidence("cleanupCode", cleanup.diagnostic.code.as_str())
            .evidence("cleanupMessage", cleanup.diagnostic.message.clone());
        if let Some(driver) = cleanup.diagnostic.evidence.get("driverMessage") {
            self = self.evidence("cleanupDriverMessage", driver.clone());
        }
        self
    }
    pub(crate) fn evidence(
        mut self,
        key: &str,
        value: impl Into<mixture_core::error::EvidenceValue>,
    ) -> Self {
        self.diagnostic = Box::new((*self.diagnostic).with_evidence(key, value));
        self
    }
}
impl From<Diagnostic> for GpuOperationError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic: Box::new(diagnostic),
            compute_completed: false,
            adapter: None,
            device_loss: None,
            allocations: None,
        }
    }
}
impl fmt::Display for GpuOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.diagnostic.fmt(f)
    }
}
impl Error for GpuOperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.diagnostic.as_ref())
    }
}

/// Pop every scope before awaiting, including on operation failure.
/// wgpu pops synchronously and returns completion futures; this also balances
/// the browser scope stack before yielding to unrelated event-loop work.
/// Validation, internal, and allocation errors must not become panics.
pub(crate) async fn checked<T>(
    device: &wgpu::Device,
    stage: Stage,
    operation: &str,
    action: impl FnOnce() -> T,
) -> Result<T, GpuOperationError> {
    let memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let result = action();
    let validation = validation.pop();
    let internal = internal.pop();
    let memory = memory.pop();
    if let Some(error) = scoped_failure(
        stage,
        operation,
        validation.await,
        internal.await,
        memory.await,
    ) {
        return Err(error);
    }
    Ok(result)
}

// Scopes do not expose chronology across filters. Prefer the specific allocation
// cause over secondary internal/validation failures from the same action, and
// retain the other driver messages. A later context-loss notification never
// replaces this selected operation error.
fn scoped_failure(
    stage: Stage,
    operation: &str,
    validation: Option<wgpu::Error>,
    internal: Option<wgpu::Error>,
    memory: Option<wgpu::Error>,
) -> Option<GpuOperationError> {
    let mut errors = [
        ("secondaryOutOfMemoryMessage", memory),
        ("secondaryValidationMessage", validation),
        ("secondaryInternalMessage", internal),
    ]
    .into_iter()
    .filter_map(|(key, error)| error.map(|error| (key, error)));
    let (_, first) = errors.next()?;
    let mut first = GpuOperationError::from_wgpu(stage, operation, first);
    for (key, error) in errors {
        first = first.evidence(key, error.to_string());
    }
    Some(first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mixture_core::{DiagnosticReport, EvidenceValue};
    use std::io;

    fn oom() -> wgpu::Error {
        // A typed synthetic backend error exercises classification, not physical
        // memory exhaustion. Production uses this same mapper after native scopes.
        wgpu::Error::OutOfMemory {
            source: Box::new(io::Error::other("synthetic GPU allocation failure")),
        }
    }

    fn validation() -> wgpu::Error {
        wgpu::Error::Validation {
            source: Box::new(io::Error::other("original validation cause")),
            description: "invalid resource after the operation".into(),
        }
    }

    fn internal() -> wgpu::Error {
        wgpu::Error::Internal {
            source: Box::new(io::Error::other("original internal cause")),
            description: "secondary internal error".into(),
        }
    }

    fn loss() -> DeviceLoss {
        DeviceLoss {
            reason: DeviceLossReason::Unknown,
            message: "original device-loss notification".into(),
        }
    }

    #[test]
    fn operation_oom_preserves_stage_native_chain_and_numeric_cleanup_evidence() {
        for stage in [
            Stage::GpuShader,
            Stage::GpuPipeline,
            Stage::GpuExecution,
            Stage::Readback,
        ] {
            let error = GpuOperationError::from_wgpu(stage, "Could not allocate GPU data.", oom())
                .with_allocations(AllocationReport {
                    cumulative_bytes: 1024,
                    released_bytes: 1024,
                    ..Default::default()
                });
            assert_eq!(error.reason(), GpuFailureReason::OutOfMemory);
            assert_eq!(error.diagnostic().code, DiagnosticCode::GpuOutOfMemory);
            assert_eq!(error.diagnostic().stage, stage);
            assert_eq!(error.diagnostic().message, "Could not allocate GPU data.");
            let native = error.diagnostic().source().unwrap();
            assert!(native.downcast_ref::<wgpu::Error>().is_some());
            assert!(
                native
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .is_some()
            );
            assert_eq!(
                native.source().unwrap().to_string(),
                "synthetic GPU allocation failure"
            );
            let json =
                serde_json::to_value(DiagnosticReport::new([error.diagnostic().clone()])).unwrap();
            assert_eq!(json["ok"], false);
            assert_eq!(json["diagnostics"][0]["code"], "MIX_GPU_OUT_OF_MEMORY");
            assert_eq!(
                json["diagnostics"][0]["evidence"]["failureReason"],
                "outOfMemory"
            );
            assert_eq!(
                json["diagnostics"][0]["evidence"]["driverCause"],
                "synthetic GPU allocation failure"
            );
            assert_eq!(
                json["diagnostics"][0]["evidence"]["allocationReleasedBytes"].as_u64(),
                Some(1024)
            );
            assert_eq!(
                json["diagnostics"][0]["evidence"]["allocationLiveBytes"].as_u64(),
                Some(0)
            );
            assert!(json["diagnostics"][0].get("source").is_none());
            assert!(error.device_loss().is_none());
        }
    }

    #[test]
    fn operation_classification_never_parses_driver_or_host_allocation_messages() {
        let fake = wgpu::Error::Internal {
            source: Box::new(io::Error::other("OutOfMemory; device lost")),
            description: "out of memory".into(),
        };
        let native = GpuOperationError::from_wgpu(Stage::GpuExecution, "operation failed", fake);
        let host = GpuOperationError::source_error(
            Stage::Readback,
            "host allocation failed",
            io::Error::other("out of memory"),
        );
        for error in [native, host] {
            assert_eq!(error.reason(), GpuFailureReason::Other);
            assert!(error.device_loss().is_none());
            assert!(!error.diagnostic().evidence.contains_key("failureReason"));
        }
    }

    #[test]
    fn operation_scope_precedence_preserves_oom_and_other_driver_messages() {
        let error = scoped_failure(
            Stage::GpuExecution,
            "allocate",
            Some(validation()),
            Some(internal()),
            Some(oom()),
        )
        .unwrap();
        assert_eq!(error.reason(), GpuFailureReason::OutOfMemory);
        assert_eq!(
            error.diagnostic().evidence["secondaryValidationMessage"],
            EvidenceValue::Text("invalid resource after the operation".into())
        );
        assert_eq!(
            error.diagnostic().evidence["secondaryInternalMessage"],
            EvidenceValue::Text("secondary internal error".into())
        );
        // Without OOM, keep the existing validation-before-internal precedence.
        let error = scoped_failure(
            Stage::GpuShader,
            "shader",
            Some(validation()),
            Some(internal()),
            None,
        )
        .unwrap();
        assert_eq!(
            error.diagnostic().code,
            DiagnosticCode::GpuShaderValidationFailed
        );
        assert_eq!(
            error.diagnostic().evidence["driverCause"],
            EvidenceValue::Text("original validation cause".into())
        );
        assert!(scoped_failure(Stage::GpuExecution, "success", None, None, None).is_none());
    }

    #[test]
    fn operation_later_loss_keeps_first_failure_stage_reason_and_source() {
        for native in [oom(), validation()] {
            let error =
                GpuOperationError::from_wgpu(Stage::Readback, "first mapping failure", native)
                    .after_compute();
            let first = error.diagnostic().clone();
            let reason = error.reason();
            let error = error.with_device_loss(loss());
            assert_eq!(error.reason(), reason);
            assert_eq!(error.diagnostic().code, first.code);
            assert_eq!(error.diagnostic().stage, first.stage);
            assert_eq!(error.diagnostic().message, first.message);
            assert_eq!(
                error.diagnostic().source().unwrap().to_string(),
                first.source().unwrap().to_string()
            );
            assert_eq!(error.device_loss(), Some(&loss()));
            assert!(error.compute_completed());
            let json = serde_json::to_value(error.diagnostic()).unwrap();
            assert_eq!(json["evidence"]["deviceLost"], true);
            assert_eq!(json["evidence"]["deviceLostReason"], "unknown");
            assert_eq!(json["evidence"]["deviceLostMessage"], loss().message);
        }
    }

    #[test]
    fn operation_primary_loss_is_typed_and_preserves_its_notification() {
        let loss = DeviceLoss {
            reason: DeviceLossReason::Destroyed,
            message: String::new(),
        };
        let error = GpuOperationError::lost(Stage::Readback, loss.clone()).after_compute();
        assert_eq!(error.reason(), GpuFailureReason::DeviceLost);
        assert_eq!(error.diagnostic().stage, Stage::Readback);
        assert_eq!(error.device_loss(), Some(&loss));
        assert_eq!(
            error
                .diagnostic()
                .source()
                .unwrap()
                .downcast_ref::<DeviceLoss>(),
            Some(&loss)
        );
        let json = serde_json::to_value(error.diagnostic()).unwrap();
        assert_eq!(json["code"], "MIX_GPU_DEVICE_LOST");
        assert_eq!(json["evidence"]["failureReason"], "deviceLost");
        assert_eq!(json["evidence"]["deviceLostReason"], "destroyed");
        assert_eq!(json["evidence"]["deviceLostMessage"], "");
        assert!(error.compute_completed());
    }
}
