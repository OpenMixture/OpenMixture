use mixture_core::{CompileError, DocumentError, Severity};
use serde::Serialize;
use std::{collections::BTreeMap, error::Error, fmt};

/// Stable package transport codes; Core failures keep their own [`mixture_core::DiagnosticCode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PackageCode {
    /// Framing, metadata or closure failure.
    Invalid,
    /// Asset version other than v1.
    UnsupportedVersion,
    /// Measured value above the explicit package, graph or resource policy.
    LimitExceeded,
    /// A bounded byte buffer could not be reserved.
    AllocationFailed,
    /// Payload bytes differ from their manifest digest.
    ContentMismatch,
    /// Package v1 rejects every `resourceRef` override.
    ResourceOverride,
}
impl PackageCode {
    /// Every code, in declaration order.
    pub const ALL: [Self; 6] = [
        Self::Invalid,
        Self::UnsupportedVersion,
        Self::LimitExceeded,
        Self::AllocationFailed,
        Self::ContentMismatch,
        Self::ResourceOverride,
    ];
    /// The single spelling used by display and JSON serialization.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "MIX_PACKAGE_INVALID",
            Self::UnsupportedVersion => "MIX_PACKAGE_UNSUPPORTED_VERSION",
            Self::LimitExceeded => "MIX_PACKAGE_LIMIT_EXCEEDED",
            Self::AllocationFailed => "MIX_PACKAGE_ALLOCATION_FAILED",
            Self::ContentMismatch => "MIX_PACKAGE_CONTENT_MISMATCH",
            Self::ResourceOverride => "MIX_PACKAGE_RESOURCE_OVERRIDE",
        }
    }
}
impl Serialize for PackageCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
impl fmt::Display for PackageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Transport-only diagnostic. Core failures retain their own typed reports.
///
/// Serializes with the same field names as [`mixture_core::Diagnostic`], so adapters
/// project it directly instead of rebuilding the report shape.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDiagnostic {
    /// Stable MIX_PACKAGE_* code.
    pub code: PackageCode,
    /// Always package; delegated Core failures retain their original stage.
    pub stage: &'static str,
    /// Package failures always invalidate the operation.
    pub severity: Severity,
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
            Self::Package(p) => p.code.as_str(),
            Self::Document(e) => e
                .report()
                .diagnostics()
                .first()
                .map_or(PackageCode::Invalid.as_str(), |d| d.code.as_str()),
            Self::Compile(e) => e
                .report()
                .diagnostics()
                .first()
                .map_or(PackageCode::Invalid.as_str(), |d| d.code.as_str()),
        }
    }
    pub(crate) fn new(code: PackageCode, message: impl Into<String>) -> Self {
        Self::Package(PackageDiagnostic {
            code,
            stage: "package",
            severity: Severity::Error,
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
    AssetError::new(PackageCode::Invalid, message)
}
pub(crate) fn limit(name: &str, observed: u64, configured: u64) -> Result<(), AssetError> {
    if observed > configured {
        Err(
            AssetError::new(PackageCode::LimitExceeded, "Package policy exceeded.")
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
        PackageCode::AllocationFailed,
        "Cannot reserve the bounded byte buffer.",
    )
}
