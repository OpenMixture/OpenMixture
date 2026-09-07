//! Backend-neutral material graph semantics for Mixture.
//!
//! PR-002 provides structured diagnostics and explicit, conservative safety limits.
//! Document decoding, graph validation, node contracts, and compilation are not
//! implemented yet. This library has no GPU, CLI, browser, or image dependencies.
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

pub mod error;
pub mod limits;

pub use error::{Diagnostic, DiagnosticCode, DiagnosticReport, EvidenceValue, Severity, Stage};
pub use limits::{LimitExceeded, LimitKind, SafetyLimits};
