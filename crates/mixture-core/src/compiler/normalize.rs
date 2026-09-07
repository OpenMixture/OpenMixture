//! Resolve all default and override values into a separate validated source view.
use super::*;
use crate::{
    MaterialDocument,
    registry::{ParameterKind, node_contract},
};
use serde_json::json;

/// Immutable full source view with explicit normalized parameter defaults and
/// validated overrides. Unused branches remain here, but never enter the plan.
#[derive(Clone, Debug)]
pub struct NormalizedDocument {
    pub(super) inner: ValidatedDocument,
}
impl NormalizedDocument {
    /// Full normalized source in lexical node/edge/binding order. This is separate
    /// from both the caller's source and the requested-output slice.
    pub fn document(&self) -> &MaterialDocument {
        self.inner.document()
    }
}

/// Validate request policy and every override, including overrides on unrequested
/// branches. Then resolve defaults and normalize numeric representations without
/// changing the caller's validated source. Cross-parameter constraints are rechecked.
pub fn normalize(
    document: &ValidatedDocument,
    request: &CompileRequest,
) -> Result<NormalizedDocument, CompileError> {
    let request_report = request.validate();
    if !request_report.is_ok() {
        return Err(CompileError::new(
            request_report.diagnostics().iter().cloned(),
        ));
    }
    let source_report = document.document().validate(&request.limits);
    if !source_report.is_ok() {
        return Err(as_compile_error(&source_report, document, request));
    }
    let mut source = document.document().clone();
    let mut diagnostics = Vec::new();
    for (public_id, value) in &request.overrides {
        let Some(binding) = source
            .exposed_parameters
            .iter()
            .find(|binding| &binding.id == public_id)
        else {
            diagnostics.push(
                invalid("Override ID is not exposed by this document.")
                    .with_evidence("publicId", public_id.as_str()),
            );
            continue;
        };
        let Some(node) = source
            .nodes
            .iter_mut()
            .find(|node| node.id == binding.node_id)
        else {
            return Err(invariant("Validated override target node is missing."));
        };
        let Some(parameter) = node_contract(&node.type_id)
            .and_then(|contract| contract.parameter(&binding.parameter_id))
        else {
            return Err(invariant("Validated override target parameter is missing."));
        };
        if !parameter.accepts(value) {
            let mut diagnostic = Diagnostic::error(
                DiagnosticCode::ParameterInvalidValue,
                Stage::Compile,
                "Override does not satisfy the target parameter's type, range, or enum contract.",
            )
            .with_evidence("publicId", public_id.as_str())
            .with_evidence("expected", format!("{:?}", parameter.kind))
            .with_evidence("observed", value.to_string())
            .with_suggestion(
                "Use the exact JSON type and range declared by the exposed target parameter.",
            );
            diagnostic.node_id = Some(node.id.clone());
            diagnostic.parameter_id = Some(binding.parameter_id.clone());
            diagnostics.push(diagnostic);
        } else {
            node.parameters
                .insert(binding.parameter_id.clone(), value.clone());
        }
    }
    if !diagnostics.is_empty() {
        return Err(CompileError::new(diagnostics));
    }
    let report = source.validate(&request.limits);
    if !report.is_ok() {
        return Err(as_compile_error(&report, document, request));
    }
    for node in &mut source.nodes {
        let contract = node_contract(&node.type_id)
            .ok_or_else(|| invariant("Validated node contract is missing."))?;
        for parameter in contract.parameters {
            let value = node
                .parameters
                .get(parameter.id)
                .cloned()
                .unwrap_or_else(|| parameter.default.value());
            let canonical = match parameter.kind {
                ParameterKind::Float { .. } => {
                    json!(positive_zero(value.as_f64().ok_or_else(|| invariant(
                        "Validated float parameter is missing."
                    ))?))
                }
                ParameterKind::Color => {
                    let components = value
                        .as_array()
                        .ok_or_else(|| invariant("Validated color parameter is missing."))?;
                    Value::Array(
                        components
                            .iter()
                            .map(|v| {
                                v.as_f64().map(|n| json!(positive_zero(n))).ok_or_else(|| {
                                    invariant("Validated color component is missing.")
                                })
                            })
                            .collect::<Result<_, _>>()?,
                    )
                }
                ParameterKind::Integer { .. } | ParameterKind::Enum { .. } => value,
            };
            node.parameters.insert(parameter.id.into(), canonical);
        }
    }
    source.nodes.sort_by(|a, b| a.id.cmp(&b.id));
    source.edges.sort();
    source.exposed_parameters.sort();
    let inner = source
        .into_validated(&request.limits)
        .map_err(|error| as_compile_error(error.report(), document, request))?;
    Ok(NormalizedDocument { inner })
}
fn positive_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
fn as_compile_error(
    report: &DiagnosticReport,
    document: &ValidatedDocument,
    request: &CompileRequest,
) -> CompileError {
    CompileError::new(report.diagnostics().iter().cloned().map(|mut diagnostic| {
        diagnostic.stage = Stage::Compile;
        let changed: Vec<_> = document
            .document()
            .exposed_parameters
            .iter()
            .filter(|binding| {
                diagnostic.node_id.as_deref() == Some(binding.node_id.as_str())
                    && request.overrides.contains_key(&binding.id)
            })
            .map(|binding| binding.id.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !changed.is_empty() {
            diagnostic = diagnostic.with_evidence("publicIds", changed.join(","));
        }
        diagnostic
    }))
}
