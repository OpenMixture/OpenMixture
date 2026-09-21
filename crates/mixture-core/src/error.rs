//! Stable diagnostic data, deterministic reports, and native source errors.

use std::{cmp::Ordering, collections::BTreeMap, error::Error, fmt, sync::Arc};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

/// Initial diagnostic vocabulary. Reserved codes do not imply implemented runtime behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DiagnosticCode {
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_INVALID_BINDING")]
    ResourceInvalidBinding,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_DUPLICATE_ID")]
    ResourceDuplicateId,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_UNKNOWN_ID")]
    ResourceUnknownId,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_MISSING")]
    ResourceMissing,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_FORMAT_UNSUPPORTED")]
    ResourceFormatUnsupported,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_SIZE_MISMATCH")]
    ResourceSizeMismatch,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_LENGTH_MISMATCH")]
    ResourceLengthMismatch,
    /// External image resource contract failure.
    #[serde(rename = "MIX_RESOURCE_IDENTITY_MISMATCH")]
    ResourceIdentityMismatch,
    /// External image resource contract failure.
    #[serde(rename = "MIX_LIMIT_RESOURCE_COUNT_EXCEEDED")]
    LimitResourceCountExceeded,
    /// External image resource contract failure.
    #[serde(rename = "MIX_LIMIT_RESOURCE_PIXELS_EXCEEDED")]
    LimitResourcePixelsExceeded,
    /// External image resource contract failure.
    #[serde(rename = "MIX_LIMIT_RESOURCE_BYTES_EXCEEDED")]
    LimitResourceBytesExceeded,
    /// Input bytes are not valid UTF-8 (reserved for decoding).
    #[serde(rename = "MIX_PARSE_INVALID_UTF8")]
    ParseInvalidUtf8,
    /// Input is not valid JSON (reserved for decoding).
    #[serde(rename = "MIX_PARSE_INVALID_JSON")]
    ParseInvalidJson,
    /// The document format version is unsupported (reserved).
    #[serde(rename = "MIX_FORMAT_UNSUPPORTED_VERSION")]
    FormatUnsupportedVersion,
    /// Decoded input bytes exceed the configured maximum.
    #[serde(rename = "MIX_LIMIT_DECODED_BYTES_EXCEEDED")]
    LimitDecodedBytesExceeded,
    /// The node count exceeds the configured maximum.
    #[serde(rename = "MIX_LIMIT_NODES_EXCEEDED")]
    LimitNodesExceeded,
    /// The edge count exceeds the configured maximum.
    #[serde(rename = "MIX_LIMIT_EDGES_EXCEEDED")]
    LimitEdgesExceeded,
    /// The exposed parameter count exceeds the configured maximum.
    #[serde(rename = "MIX_LIMIT_EXPOSED_PARAMETERS_EXCEEDED")]
    LimitExposedParametersExceeded,
    /// An output axis exceeds the configured maximum.
    #[serde(rename = "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED")]
    LimitOutputDimensionExceeded,
    /// The requested output count exceeds the configured maximum.
    #[serde(rename = "MIX_LIMIT_REQUESTED_OUTPUTS_EXCEEDED")]
    LimitRequestedOutputsExceeded,
    /// Estimated transient bytes exceed the configured maximum.
    #[serde(rename = "MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED")]
    LimitTransientBytesExceeded,
    /// The node type is unknown (reserved).
    #[serde(rename = "MIX_NODE_UNKNOWN_TYPE")]
    NodeUnknownType,
    /// The node version is unsupported (reserved).
    #[serde(rename = "MIX_NODE_UNSUPPORTED_VERSION")]
    NodeUnsupportedVersion,
    /// A port does not exist on the node (reserved).
    #[serde(rename = "MIX_PORT_UNKNOWN")]
    PortUnknown,
    /// Connected port kinds are incompatible (reserved).
    #[serde(rename = "MIX_PORT_TYPE_MISMATCH")]
    PortTypeMismatch,
    /// The material graph contains a cycle (reserved).
    #[serde(rename = "MIX_GRAPH_CYCLE")]
    GraphCycle,
    /// A parameter value is invalid (reserved).
    #[serde(rename = "MIX_PARAMETER_INVALID_VALUE")]
    ParameterInvalidValue,
    /// The compilation request is invalid (reserved).
    #[serde(rename = "MIX_COMPILE_INVALID_REQUEST")]
    CompileInvalidRequest,
    /// No adapter satisfies the explicit policy (reserved).
    #[serde(rename = "MIX_GPU_ADAPTER_UNAVAILABLE")]
    GpuAdapterUnavailable,
    /// Requesting a GPU device failed (reserved).
    #[serde(rename = "MIX_GPU_DEVICE_REQUEST_FAILED")]
    GpuDeviceRequestFailed,
    /// GPU shader validation failed (reserved).
    #[serde(rename = "MIX_GPU_SHADER_VALIDATION_FAILED")]
    GpuShaderValidationFailed,
    /// GPU execution failed (reserved).
    #[serde(rename = "MIX_GPU_EXECUTION_FAILED")]
    GpuExecutionFailed,
    /// Reading an output back from the GPU failed (reserved).
    #[serde(rename = "MIX_READBACK_FAILED")]
    ReadbackFailed,
    /// Encoding an output failed (reserved).
    #[serde(rename = "MIX_ENCODING_FAILED")]
    EncodingFailed,
    /// JSON field structure does not conform to the source schema.
    #[serde(rename = "MIX_FORMAT_INVALID_DOCUMENT")]
    FormatInvalidDocument,
    /// A node ID violates the document identifier grammar.
    #[serde(rename = "MIX_NODE_INVALID_ID")]
    NodeInvalidId,
    /// A document contains multiple nodes with the same ID.
    #[serde(rename = "MIX_NODE_DUPLICATE_ID")]
    NodeDuplicateId,
    /// An edge refers to a node that does not exist.
    #[serde(rename = "MIX_GRAPH_UNKNOWN_NODE")]
    GraphUnknownNode,
    /// An edge identity occurs more than once.
    #[serde(rename = "MIX_GRAPH_DUPLICATE_EDGE")]
    GraphDuplicateEdge,
    /// A single-input port has multiple incoming edges.
    #[serde(rename = "MIX_GRAPH_MULTIPLE_INPUTS")]
    GraphMultipleInputs,
    /// A material document must contain exactly one material-output node.
    #[serde(rename = "MIX_GRAPH_MATERIAL_OUTPUT_COUNT")]
    GraphMaterialOutputCount,
    /// A required input has no valid connection.
    #[serde(rename = "MIX_PORT_REQUIRED_CONNECTION")]
    PortRequiredConnection,
    /// A parameter is not defined by the node contract.
    #[serde(rename = "MIX_PARAMETER_UNKNOWN")]
    ParameterUnknown,
    /// A public parameter ID or target binding is invalid or ambiguous.
    #[serde(rename = "MIX_EXPOSED_PARAMETER_INVALID")]
    ExposedParameterInvalid,
    /// Reading source bytes failed at the caller I/O boundary.
    #[serde(rename = "MIX_IO_READ_FAILED")]
    IoReadFailed,
    /// The explicit GPU context has received a device-loss notification.
    #[serde(rename = "MIX_GPU_DEVICE_LOST")]
    GpuDeviceLost,
    /// A typed GPU allocation error reported out-of-memory.
    #[serde(rename = "MIX_GPU_OUT_OF_MEMORY")]
    GpuOutOfMemory,
}

impl DiagnosticCode {
    /// The stable machine-readable spelling; independent of Rust debug formatting.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceInvalidBinding => "MIX_RESOURCE_INVALID_BINDING",
            Self::ResourceDuplicateId => "MIX_RESOURCE_DUPLICATE_ID",
            Self::ResourceUnknownId => "MIX_RESOURCE_UNKNOWN_ID",
            Self::ResourceMissing => "MIX_RESOURCE_MISSING",
            Self::ResourceFormatUnsupported => "MIX_RESOURCE_FORMAT_UNSUPPORTED",
            Self::ResourceSizeMismatch => "MIX_RESOURCE_SIZE_MISMATCH",
            Self::ResourceLengthMismatch => "MIX_RESOURCE_LENGTH_MISMATCH",
            Self::ResourceIdentityMismatch => "MIX_RESOURCE_IDENTITY_MISMATCH",
            Self::LimitResourceCountExceeded => "MIX_LIMIT_RESOURCE_COUNT_EXCEEDED",
            Self::LimitResourcePixelsExceeded => "MIX_LIMIT_RESOURCE_PIXELS_EXCEEDED",
            Self::LimitResourceBytesExceeded => "MIX_LIMIT_RESOURCE_BYTES_EXCEEDED",
            Self::ParseInvalidUtf8 => "MIX_PARSE_INVALID_UTF8",
            Self::ParseInvalidJson => "MIX_PARSE_INVALID_JSON",
            Self::FormatUnsupportedVersion => "MIX_FORMAT_UNSUPPORTED_VERSION",
            Self::LimitDecodedBytesExceeded => "MIX_LIMIT_DECODED_BYTES_EXCEEDED",
            Self::LimitNodesExceeded => "MIX_LIMIT_NODES_EXCEEDED",
            Self::LimitEdgesExceeded => "MIX_LIMIT_EDGES_EXCEEDED",
            Self::LimitExposedParametersExceeded => "MIX_LIMIT_EXPOSED_PARAMETERS_EXCEEDED",
            Self::LimitOutputDimensionExceeded => "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED",
            Self::LimitRequestedOutputsExceeded => "MIX_LIMIT_REQUESTED_OUTPUTS_EXCEEDED",
            Self::LimitTransientBytesExceeded => "MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED",
            Self::NodeUnknownType => "MIX_NODE_UNKNOWN_TYPE",
            Self::NodeUnsupportedVersion => "MIX_NODE_UNSUPPORTED_VERSION",
            Self::PortUnknown => "MIX_PORT_UNKNOWN",
            Self::PortTypeMismatch => "MIX_PORT_TYPE_MISMATCH",
            Self::GraphCycle => "MIX_GRAPH_CYCLE",
            Self::ParameterInvalidValue => "MIX_PARAMETER_INVALID_VALUE",
            Self::CompileInvalidRequest => "MIX_COMPILE_INVALID_REQUEST",
            Self::GpuAdapterUnavailable => "MIX_GPU_ADAPTER_UNAVAILABLE",
            Self::GpuDeviceRequestFailed => "MIX_GPU_DEVICE_REQUEST_FAILED",
            Self::GpuShaderValidationFailed => "MIX_GPU_SHADER_VALIDATION_FAILED",
            Self::GpuExecutionFailed => "MIX_GPU_EXECUTION_FAILED",
            Self::ReadbackFailed => "MIX_READBACK_FAILED",
            Self::EncodingFailed => "MIX_ENCODING_FAILED",
            Self::FormatInvalidDocument => "MIX_FORMAT_INVALID_DOCUMENT",
            Self::NodeInvalidId => "MIX_NODE_INVALID_ID",
            Self::NodeDuplicateId => "MIX_NODE_DUPLICATE_ID",
            Self::GraphUnknownNode => "MIX_GRAPH_UNKNOWN_NODE",
            Self::GraphDuplicateEdge => "MIX_GRAPH_DUPLICATE_EDGE",
            Self::GraphMultipleInputs => "MIX_GRAPH_MULTIPLE_INPUTS",
            Self::GraphMaterialOutputCount => "MIX_GRAPH_MATERIAL_OUTPUT_COUNT",
            Self::PortRequiredConnection => "MIX_PORT_REQUIRED_CONNECTION",
            Self::ParameterUnknown => "MIX_PARAMETER_UNKNOWN",
            Self::ExposedParameterInvalid => "MIX_EXPOSED_PARAMETER_INVALID",
            Self::IoReadFailed => "MIX_IO_READ_FAILED",
            Self::GpuDeviceLost => "MIX_GPU_DEVICE_LOST",
            Self::GpuOutOfMemory => "MIX_GPU_OUT_OF_MEMORY",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stage where a diagnostic was observed, ordered by the execution lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Stage {
    /// Reading document bytes as UTF-8/JSON.
    Parse,
    /// Checking format, graph, parameters, or request budgets.
    Validation,
    /// Building a backend-neutral plan and estimating resources.
    Compile,
    /// Selecting an adapter under explicit policy.
    GpuAdapter,
    /// Creating a device and queue.
    GpuDevice,
    /// Compiling or validating a shader.
    GpuShader,
    /// Creating a compute pipeline.
    GpuPipeline,
    /// Submitting or executing compute work.
    GpuExecution,
    /// Reading GPU outputs back.
    Readback,
    /// Encoding an output file.
    Encoding,
}

/// Severity order within a stage: errors, warnings, then informational observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    /// The requested operation cannot succeed.
    Error,
    /// An observation that does not invalidate the result.
    Warning,
    /// Informational evidence only.
    Info,
}

/// Flat, lossless evidence supported by the initial contract.
///
/// Values serialize as JSON booleans, unsigned integers, or strings. Floating-point,
/// negative, null, nested, and array values are deliberately not accepted yet.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvidenceValue {
    /// Boolean evidence, such as whether a capability was requested.
    Bool(bool),
    /// An exact, nonnegative count or byte total.
    Unsigned(u64),
    /// An identifier, description, or explicitly selected source-error detail.
    Text(String),
}

impl From<bool> for EvidenceValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<u64> for EvidenceValue {
    fn from(value: u64) -> Self {
        Self::Unsigned(value)
    }
}

impl From<String> for EvidenceValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for EvidenceValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// A GPU-independent, serializable observation with an optional native error chain.
///
/// Prefer [`DiagnosticReport`] when reporting several observations. Source errors are
/// available through [`Error::source`] but never serialized or used as ordering keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Diagnostic {
    /// Stable code, used by machines instead of the human-readable message.
    pub code: DiagnosticCode,
    /// Actual stage of failure; callers supply this explicitly.
    pub stage: Stage,
    /// Whether this observation invalidates the operation.
    pub severity: Severity,
    /// Concise human-readable explanation.
    pub message: String,
    /// Optional caller-supplied document path. No path is inferred or read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_path: Option<String>,
    /// Optional document-local node ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Optional port ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port_id: Option<String>,
    /// Optional parameter ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_id: Option<String>,
    /// Evidence with lexically ordered keys, independent of insertion order.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub evidence: BTreeMap<String, EvidenceValue>,
    /// Optional actionable remediation, without automatically changing the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip)]
    source: Option<Arc<dyn Error + Send + Sync + 'static>>,
}

impl Diagnostic {
    /// Construct an observation without context, evidence, or a source error.
    pub fn new(
        code: DiagnosticCode,
        stage: Stage,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            stage,
            severity,
            message: message.into(),
            document_path: None,
            node_id: None,
            port_id: None,
            parameter_id: None,
            evidence: BTreeMap::new(),
            suggestion: None,
            source: None,
        }
    }

    /// Construct a failure observation.
    pub fn error(code: DiagnosticCode, stage: Stage, message: impl Into<String>) -> Self {
        Self::new(code, stage, Severity::Error, message)
    }

    /// Add or explicitly replace evidence under one stable key.
    pub fn with_evidence(
        mut self,
        key: impl Into<String>,
        value: impl Into<EvidenceValue>,
    ) -> Self {
        self.evidence.insert(key.into(), value.into());
        self
    }

    /// Add an actionable suggestion without performing remediation.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Retain an actual source error and its chain for native human diagnostics.
    ///
    /// Cloning retains the same source. Deserializing JSON has no native source.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Arc::new(source));
        self
    }

    fn compare_for_report(&self, other: &Self) -> Ordering {
        // Every serialized field participates; sources do not. A tie therefore has
        // identical JSON even when native causes differ. Missing IDs sort first.
        (
            self.stage,
            self.severity,
            &self.document_path,
            &self.node_id,
            &self.port_id,
            &self.parameter_id,
            self.code.as_str(),
            &self.message,
            &self.evidence,
            &self.suggestion,
        )
            .cmp(&(
                other.stage,
                other.severity,
                &other.document_path,
                &other.node_id,
                &other.port_id,
                &other.parameter_id,
                other.code.as_str(),
                &other.message,
                &other.evidence,
                &other.suggestion,
            ))
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for Diagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

/// An immutable, deterministically ordered report with a derived success flag.
///
/// `ok` means no error-severity diagnostic, not evidence that rendering happened.
/// This is an output contract; deserialization is available for individual diagnostics.
#[derive(Clone, Debug)]
pub struct DiagnosticReport {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    /// Collect and order diagnostics; duplicates are retained as observations.
    pub fn new(diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        let mut diagnostics: Vec<_> = diagnostics.into_iter().collect();
        diagnostics.sort_by(Diagnostic::compare_for_report);
        Self { diagnostics }
    }

    /// True when there are no error-severity diagnostics, including an empty report.
    pub fn is_ok(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    /// The immutable ordered observations.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl Serialize for DiagnosticReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("DiagnosticReport", 2)?;
        state.serialize_field("ok", &self.is_ok())?;
        state.serialize_field("diagnostics", &self.diagnostics)?;
        state.end()
    }
}
