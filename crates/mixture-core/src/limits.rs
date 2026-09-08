//! Explicit upper bounds. These checks do not parse documents or allocate GPU resources.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{Diagnostic, DiagnosticCode, Stage};

/// Resource category checked against [`SafetyLimits`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LimitKind {
    /// Decoded JSON bytes.
    DecodedBytes,
    /// Graph nodes.
    Nodes,
    /// Graph edges.
    Edges,
    /// Publicly exposed parameters.
    ExposedParameters,
    /// Output dimension, checked separately for each axis.
    OutputDimension,
    /// Requested material output channels.
    RequestedOutputs,
    /// Estimated transient GPU bytes.
    TransientBytes,
}

impl LimitKind {
    /// The stable evidence key corresponding to a field in [`SafetyLimits`].
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DecodedBytes => "decodedBytes",
            Self::Nodes => "nodes",
            Self::Edges => "edges",
            Self::ExposedParameters => "exposedParameters",
            Self::OutputDimension => "outputDimension",
            Self::RequestedOutputs => "requestedOutputs",
            Self::TransientBytes => "transientBytes",
        }
    }

    fn code(self) -> DiagnosticCode {
        match self {
            Self::DecodedBytes => DiagnosticCode::LimitDecodedBytesExceeded,
            Self::Nodes => DiagnosticCode::LimitNodesExceeded,
            Self::Edges => DiagnosticCode::LimitEdgesExceeded,
            Self::ExposedParameters => DiagnosticCode::LimitExposedParametersExceeded,
            Self::OutputDimension => DiagnosticCode::LimitOutputDimensionExceeded,
            Self::RequestedOutputs => DiagnosticCode::LimitRequestedOutputsExceeded,
            Self::TransientBytes => DiagnosticCode::LimitTransientBytesExceeded,
        }
    }
}

/// Caller-owned maximums with conservative v1 defaults, in platform-independent units.
///
/// Every field is required when deserializing; use [`Default`] explicitly for defaults.
/// Zero is a valid upper bound. Lower bounds and graph correctness belong to
/// request/document validation. Changing a field never changes other limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SafetyLimits {
    /// Maximum decoded JSON bytes; default 2 MiB.
    pub decoded_bytes: u64,
    /// Maximum graph nodes; default 128.
    pub nodes: u64,
    /// Maximum graph edges; default 512.
    pub edges: u64,
    /// Maximum exposed parameters; default 64.
    pub exposed_parameters: u64,
    /// Maximum output dimension per axis; default 2048.
    pub output_dimension: u64,
    /// Maximum requested material channels; default 8.
    pub requested_outputs: u64,
    /// Maximum estimated transient GPU bytes; default 512 MiB.
    pub transient_bytes: u64,
}

impl Default for SafetyLimits {
    fn default() -> Self {
        Self {
            decoded_bytes: 2 * 1024 * 1024,
            nodes: 128,
            edges: 512,
            exposed_parameters: 64,
            output_dimension: 2048,
            requested_outputs: 8,
            transient_bytes: 512 * 1024 * 1024,
        }
    }
}

impl SafetyLimits {
    /// Return the configured ceiling for one category.
    pub const fn maximum(&self, kind: LimitKind) -> u64 {
        match kind {
            LimitKind::DecodedBytes => self.decoded_bytes,
            LimitKind::Nodes => self.nodes,
            LimitKind::Edges => self.edges,
            LimitKind::ExposedParameters => self.exposed_parameters,
            LimitKind::OutputDimension => self.output_dimension,
            LimitKind::RequestedOutputs => self.requested_outputs,
            LimitKind::TransientBytes => self.transient_bytes,
        }
    }

    /// Compare a measured value to a ceiling, accepting equality without changing policy.
    ///
    /// # Errors
    /// Returns the exact configured and observed values when `observed` exceeds the limit.
    pub fn check(&self, kind: LimitKind, observed: u64) -> Result<(), LimitExceeded> {
        let configured = self.maximum(kind);
        if observed > configured {
            Err(LimitExceeded {
                kind,
                configured,
                observed,
            })
        } else {
            Ok(())
        }
    }
}

/// A limit failure with exact evidence, independent of where the check took place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitExceeded {
    /// The resource category that exceeded its ceiling.
    pub kind: LimitKind,
    /// The explicit ceiling supplied to the check.
    pub configured: u64,
    /// The value actually checked, without truncation or clamping.
    pub observed: u64,
}

impl LimitExceeded {
    /// Attach the caller's actual stage and produce a structured error diagnostic.
    pub fn diagnostic(&self, stage: Stage) -> Diagnostic {
        Diagnostic::error(self.kind.code(), stage, self.to_string())
            .with_evidence("limit", self.kind.as_str())
            .with_evidence("configured", self.configured)
            .with_evidence("observed", self.observed)
            .with_suggestion("Reduce the request to the configured limit; raise limits only through an explicit caller policy.")
            .with_source(*self)
    }
}

impl fmt::Display for LimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} limit exceeded: configured {}, observed {}",
            self.kind.as_str(),
            self.configured,
            self.observed
        )
    }
}

impl Error for LimitExceeded {}
