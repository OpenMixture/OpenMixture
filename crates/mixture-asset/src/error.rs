use mixture_core::{CompileError, DocumentError};
use serde::Serialize;
use std::{collections::BTreeMap, error::Error, fmt};

/// Transport-only diagnostic. Core failures retain their own typed reports.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDiagnostic {
    /// Stable MIX_PACKAGE_* code.
    pub code: &'static str,
    /// Always package; delegated Core failures retain their original stage.
    pub stage: &'static str,
    /// Concise explanation.
    pub message: String,
    /// Measured values, entry/offset or configured policy.
    pub evidence: BTreeMap<String, serde_json::Value>,
    /// Actionable correction.
    pub suggestion: &'static str,
}
/// Typed package or original Core failure; no generic wrapper replaces diagnostics.
#[derive(Debug)]
pub enum AssetError {
    /// Package framing, policy or content failure.
    Package(PackageDiagnostic),
    /// Original graph decoding/validation failure.
    Document(DocumentError),
    /// Original preparation/override failure.
    Compile(CompileError),
}
impl AssetError {
    /// First stable diagnostic code.
    pub fn code(&self) -> &str {
        match self {
            Self::Package(p) => p.code,
            Self::Document(e) => e
                .report()
                .diagnostics()
                .first()
                .map_or("MIX_PACKAGE_INVALID", |d| d.code.as_str()),
            Self::Compile(e) => e
                .report()
                .diagnostics()
                .first()
                .map_or("MIX_PACKAGE_INVALID", |d| d.code.as_str()),
        }
    }
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self::Package(PackageDiagnostic {
            code,
            stage: "package",
            message: message.into(),
            evidence: BTreeMap::new(),
            suggestion: "Supply a canonical mixpack v1 asset and an explicit policy within its documented ceilings.",
        })
    }
    pub(crate) fn evidence(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        if let Self::Package(p) = &mut self {
            p.evidence.insert(key.into(), value.into());
        }
        self
    }
}
impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Package(p) => write!(f, "{}: {}", p.code, p.message),
            Self::Document(e) => e.fmt(f),
            Self::Compile(e) => e.fmt(f),
        }
    }
}
impl Error for AssetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Document(e) => Some(e),
            Self::Compile(e) => Some(e),
            _ => None,
        }
    }
}
impl From<DocumentError> for AssetError {
    fn from(e: DocumentError) -> Self {
        Self::Document(e)
    }
}
impl From<CompileError> for AssetError {
    fn from(e: CompileError) -> Self {
        Self::Compile(e)
    }
}
pub(crate) fn invalid(message: impl Into<String>) -> AssetError {
    AssetError::new("MIX_PACKAGE_INVALID", message)
}
pub(crate) fn limit(name: &str, observed: u64, configured: u64) -> Result<(), AssetError> {
    if observed > configured {
        Err(
            AssetError::new("MIX_PACKAGE_LIMIT_EXCEEDED", "Package policy exceeded.")
                .evidence("limit", name)
                .evidence("observed", observed)
                .evidence("configured", configured),
        )
    } else {
        Ok(())
    }
}
pub(crate) fn allocation() -> AssetError {
    AssetError::new(
        "MIX_PACKAGE_ALLOCATION_FAILED",
        "Cannot reserve the bounded byte buffer.",
    )
}
