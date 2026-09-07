//! Stage-specific failures and scoped native GPU operations.

use mixture_core::{Diagnostic, DiagnosticCode, Stage};
use std::{error::Error, fmt};

/// Failure in request validation, shader creation, execution, or readback.
#[derive(Debug)]
pub struct GpuOperationError(Box<Diagnostic>, bool);

impl GpuOperationError {
    /// The stable diagnostic, including its original native error chain.
    pub fn diagnostic(&self) -> &Diagnostic {
        &self.0
    }
    pub(crate) fn after_compute(mut self) -> Self {
        self.1 = true;
        self
    }
    pub(crate) fn compute_completed(&self) -> bool {
        self.1
    }

    pub(crate) fn in_plan(mut self, plan: &mixture_core::RenderPlan) -> Self {
        if let Some(mixture_core::EvidenceValue::Unsigned(index)) = self.0.evidence.get("passIndex")
            && let Ok(index) = usize::try_from(*index)
            && let Some(pass) = plan.passes().get(index)
        {
            let node = match &pass.origin {
                mixture_core::plan::PassOrigin::Node { node }
                | mixture_core::plan::PassOrigin::InputDefault { node, .. } => node,
            };
            self.0.node_id = Some(node.id.clone());
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
        error.0 = Box::new(
            (*error.0)
                .with_evidence("driverMessage", source.to_string())
                .with_source(source),
        );
        error
    }
    pub(crate) fn evidence(
        mut self,
        key: &str,
        value: impl Into<mixture_core::error::EvidenceValue>,
    ) -> Self {
        self.0 = Box::new((*self.0).with_evidence(key, value));
        self
    }
}
impl From<Diagnostic> for GpuOperationError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self(Box::new(diagnostic), false)
    }
}
impl fmt::Display for GpuOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl Error for GpuOperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

/// Pop every thread-local scope before awaiting, including on operation failure.
/// Native wgpu validation, internal, and allocation errors must not become panics.
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
    let errors = [validation.await, internal.await, memory.await];
    if let Some(error) = errors.into_iter().flatten().next() {
        return Err(GpuOperationError::source_error(stage, operation, error));
    }
    Ok(result)
}
