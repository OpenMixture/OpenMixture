//! Compile validated material graphs into deterministic, GPU-independent plans.

mod lower;
mod normalize;

use crate::{
    Diagnostic, DiagnosticCode, DiagnosticReport, LimitKind, SafetyLimits, Stage,
    ValidatedDocument,
    plan::{OutputChannel, RenderPlan},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

pub use normalize::{NormalizedDocument, normalize};

/// Explicit compilation policy. Defaults request only baseColor at 64x64.
#[derive(Clone, Debug)]
pub struct CompileRequest {
    /// Output width and height, positive and within the supplied limits.
    pub size: [u32; 2],
    /// Nonempty unique material channels; request order has no semantic effect.
    pub outputs: Vec<OutputChannel>,
    /// Public exposed IDs to exact JSON values; undeclared IDs are rejected.
    pub overrides: BTreeMap<String, Value>,
    /// Caller-owned decoding/graph/request/allocation ceilings.
    pub limits: SafetyLimits,
}
impl Default for CompileRequest {
    fn default() -> Self {
        Self {
            size: [64, 64],
            outputs: vec![OutputChannel::BaseColor],
            overrides: BTreeMap::new(),
            limits: SafetyLimits::default(),
        }
    }
}
impl CompileRequest {
    /// Check request shape and directly measurable budgets before normalization.
    pub fn validate(&self) -> DiagnosticReport {
        let mut diagnostics = Vec::new();
        for (axis, dimension) in ["width", "height"].into_iter().zip(self.size) {
            if dimension == 0 {
                diagnostics.push(
                    invalid("Output dimensions must be positive.").with_evidence("axis", axis),
                );
            }
            if let Err(error) = self
                .limits
                .check(LimitKind::OutputDimension, u64::from(dimension))
            {
                diagnostics.push(error.diagnostic(Stage::Compile).with_evidence("axis", axis));
            }
        }
        if self.outputs.is_empty() {
            diagnostics.push(invalid("At least one material output must be requested."));
        }
        let mut unique = BTreeSet::new();
        for output in &self.outputs {
            if !unique.insert(output) {
                diagnostics.push(
                    invalid("Requested material channels must be unique.")
                        .with_evidence("channel", output.as_str()),
                );
            }
        }
        for (kind, count) in [
            (LimitKind::RequestedOutputs, self.outputs.len()),
            (LimitKind::ExposedParameters, self.overrides.len()),
        ] {
            if let Err(error) = self.limits.check(kind, count as u64) {
                diagnostics.push(error.diagnostic(Stage::Compile));
            }
        }
        DiagnosticReport::new(diagnostics)
    }
}

/// Immutable compiler failure report with stable codes, stages, and native sources.
#[derive(Clone, Debug)]
pub struct CompileError {
    report: DiagnosticReport,
}
impl CompileError {
    pub(crate) fn new(diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        Self {
            report: DiagnosticReport::new(diagnostics),
        }
    }
    /// All independently discovered compile-request or normalization failures.
    pub fn report(&self) -> &DiagnosticReport {
        &self.report
    }
}
impl From<Diagnostic> for CompileError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self::new([diagnostic])
    }
}
impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.report.diagnostics().first() {
            Some(first) => write!(
                f,
                "{first} ({} diagnostic(s))",
                self.report.diagnostics().len()
            ),
            None => f.write_str("Compilation failed."),
        }
    }
}
impl Error for CompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.report
            .diagnostics()
            .first()
            .map(|d| d as &(dyn Error + 'static))
    }
}

/// Apply validated overrides, slice requested dependencies, order passes, lower
/// typed parameters/resources, estimate allocations, and hash the immutable plan.
/// No GPU state, filesystem path, clock, or executor is consulted.
pub fn compile(
    document: &ValidatedDocument,
    request: &CompileRequest,
) -> Result<RenderPlan, CompileError> {
    let normalized = normalize(document, request)?;
    lower::compile(&normalized, request)
}
pub(crate) fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::CompileInvalidRequest, Stage::Compile, message)
        .with_suggestion("Use supported unique outputs, positive bounded dimensions, and declared parameter overrides.")
}
pub(crate) fn serialization_error(source: serde_json::Error) -> CompileError {
    invalid("Could not serialize the deterministic plan body.")
        .with_source(source)
        .into()
}
pub(crate) fn invariant(message: &str) -> CompileError {
    invalid(message).with_suggestion("The validated graph could not be lowered by the supported plan version; inspect the node contract and report this invariant failure.").into()
}
