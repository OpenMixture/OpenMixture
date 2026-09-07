//! Backend-neutral material graph semantics for Mixture.
//!
//! Strict .mix v1 decoding, six versioned node contracts, graph validation, and
//! structured diagnostics with explicit safety limits. Compilation is future work.
//! This library has no GPU, CLI, browser, or image dependencies.
//!
//! ```
//! use mixture_core::{DiagnosticReport, LimitKind, SafetyLimits, Stage};
//!
//! let limits = SafetyLimits::default();
//! let violation = limits.check(LimitKind::Nodes, 129).unwrap_err();
//! let report = DiagnosticReport::new([violation.diagnostic(Stage::Validation)]);
//! assert!(!report.is_ok());
//! assert_eq!(report.diagnostics()[0].code.as_str(), "MIX_LIMIT_NODES_EXCEEDED");
//! ```

//! Decode and validate untrusted source bytes without accessing a GPU:
//!
//! ```
//! use mixture_core::{MaterialDocument, SafetyLimits};
//! let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/checker.mix"));
//! let limits = SafetyLimits::default();
//! let document = MaterialDocument::decode(bytes, &limits)?;
//! let validated = document.into_validated(&limits)?;
//! assert_eq!(validated.material_channels().len(), 8);
//! let encoded = validated.document().to_json()?;
//! assert!(MaterialDocument::decode(&encoded, &limits)?.validate(&limits).is_ok());
//! # Ok::<(), mixture_core::DocumentError>(())
//! ```

pub mod document;
pub mod error;
pub mod limits;
mod nodes;
pub mod registry;
pub mod validation;

pub use error::{Diagnostic, DiagnosticCode, DiagnosticReport, EvidenceValue, Severity, Stage};
pub use limits::{LimitExceeded, LimitKind, SafetyLimits};

pub use document::{
    DocumentError, Edge, Endpoint, ExposedParameter, FORMAT_VERSION, MaterialDocument, Node,
};
pub use validation::{InputSource, MaterialChannel, ValidatedDocument};
