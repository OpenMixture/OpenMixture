//! Versioned source data, bounded decoding, and canonical serialization.

mod decode;

use crate::{Diagnostic, DiagnosticCode, DiagnosticReport, SafetyLimits, Stage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, error::Error, fmt};

/// The only supported source document version. Node versions are independent.
pub const FORMAT_VERSION: u32 = 1;

/// Editable source data. Use `decode` for untrusted bytes and `into_validated`
/// before consuming graph semantics. This type is deliberately not Deserialize:
/// callers must supply the byte and collection limits at the decoding boundary.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialDocument {
    /// Required top-level format version.
    pub version: u32,
    /// Source nodes, in author order until canonical serialization.
    pub nodes: Vec<Node>,
    /// Directed output-to-input connections.
    pub edges: Vec<Edge>,
    /// Public parameter bindings; omitted on input means an empty list.
    pub exposed_parameters: Vec<ExposedParameter>,
}

/// One versioned node. Omitted parameters use the versioned contract defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    /// Unique document-local identifier.
    pub id: String,
    /// Stable kebab-case built-in type ID; JSON field is `type`.
    #[serde(rename = "type")]
    pub type_id: String,
    /// Required node version, independent from the document version.
    pub version: u32,
    /// Explicit parameter values, with unique keys and lexical serialization.
    #[serde(default, deserialize_with = "decode::parameters")]
    pub parameters: BTreeMap<String, Value>,
}

/// A named port on a node. Its direction is determined by its edge position.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Endpoint {
    /// Node identifier.
    pub node_id: String,
    /// Input or output port identifier.
    pub port_id: String,
}

/// An edge has identity `(from.nodeId, from.portId, to.nodeId, to.portId)`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    /// Source output port.
    pub from: Endpoint,
    /// Destination input port.
    pub to: Endpoint,
}

/// One public name bound to one mutable node parameter; no aliases in v1.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExposedParameter {
    /// Unique public identifier.
    pub id: String,
    /// Target node identifier.
    pub node_id: String,
    /// Target parameter identifier, including parameters using a default.
    pub parameter_id: String,
}

/// Parse or validation failure with immutable, deterministically ordered evidence.
#[derive(Clone, Debug)]
pub struct DocumentError {
    report: DiagnosticReport,
}
impl DocumentError {
    pub(crate) fn new(diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        Self {
            report: DiagnosticReport::new(diagnostics),
        }
    }
    /// All safely discoverable failures, preserving native sources.
    pub fn report(&self) -> &DiagnosticReport {
        &self.report
    }
}
impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(first) = self.report.diagnostics().first() {
            write!(
                f,
                "{first} ({} diagnostic(s))",
                self.report.diagnostics().len()
            )
        } else {
            f.write_str("Document validation failed.")
        }
    }
}
impl Error for DocumentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.report
            .diagnostics()
            .first()
            .map(|d| d as &(dyn Error + 'static))
    }
}
impl From<Diagnostic> for DocumentError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self::new([diagnostic])
    }
}

impl MaterialDocument {
    /// Decode UTF-8 JSON under explicit limits. Reject duplicate/unknown fields,
    /// unsupported versions, oversized collections, and trailing data. Graph
    /// semantics are checked separately by `validate` or `into_validated`.
    pub fn decode(bytes: &[u8], limits: &SafetyLimits) -> Result<Self, DocumentError> {
        decode::document(bytes, limits)
    }

    /// Serialize in lexical node/edge/public-ID order without changing this source.
    /// Explicit parameters remain explicit; defaults are not inserted. Array order
    /// inside parameter values is meaningful and is preserved.
    pub fn to_json(&self) -> Result<Vec<u8>, DocumentError> {
        let mut canonical = self.clone();
        canonical
            .nodes
            .sort_by(|a, b| (&a.id, &a.type_id, a.version).cmp(&(&b.id, &b.type_id, b.version)));
        canonical.edges.sort();
        canonical.exposed_parameters.sort();
        serde_json::to_vec_pretty(&canonical).map_err(|source| {
            Diagnostic::error(
                DiagnosticCode::FormatInvalidDocument,
                Stage::Parse,
                "Could not serialize the material document.",
            )
            .with_source(source)
            .into()
        })
    }
}
