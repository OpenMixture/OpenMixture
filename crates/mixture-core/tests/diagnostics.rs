//! Public-API regression coverage for the diagnostic wire contract.

use std::{error::Error, fmt, io};

use mixture_core::{
    Diagnostic, DiagnosticCode as Code, DiagnosticReport, EvidenceValue, Severity, Stage,
};

fn mismatch() -> Diagnostic {
    let mut diagnostic = Diagnostic::error(
        Code::PortTypeMismatch,
        Stage::Validation,
        "Cannot connect a color output to a scalar input used as height.",
    )
    .with_evidence("targetKind", "scalar")
    .with_evidence("sourceKind", "color")
    .with_suggestion("Connect a scalar output or add an explicit supported conversion node.");
    diagnostic.document_path = Some("examples/sample.mix".into());
    diagnostic.node_id = Some("normal".into());
    diagnostic.port_id = Some("height".into());
    diagnostic.parameter_id = Some("strength".into());
    diagnostic
}

#[test]
fn diagnostics_report_matches_reviewed_json_snapshot() {
    let parse = Diagnostic::error(
        Code::ParseInvalidJson,
        Stage::Parse,
        "Expected a JSON object.",
    )
    .with_evidence("offset", 7_u64)
    .with_evidence("recoverable", false)
    .with_source(io::Error::other("native parser detail"));
    let report = DiagnosticReport::new([mismatch(), parse]);
    let actual = serde_json::to_string_pretty(&report).expect("serialize report");
    assert_eq!(
        actual,
        include_str!("snapshots/diagnostics-report.json").trim_end()
    );
}

#[test]
fn diagnostics_context_round_trips_and_optional_fields_are_omitted() {
    let diagnostic = mismatch();
    let encoded = serde_json::to_string(&diagnostic).expect("serialize diagnostic");
    let decoded: Diagnostic = serde_json::from_str(&encoded).expect("decode diagnostic");
    assert_eq!(
        serde_json::to_string(&decoded).expect("serialize decoded"),
        encoded
    );
    assert_eq!(decoded.parameter_id.as_deref(), Some("strength"));
    let bare = Diagnostic::error(Code::GraphCycle, Stage::Validation, "Cycle detected.");
    assert_eq!(
        serde_json::to_value(&bare).expect("serialize minimal diagnostic"),
        serde_json::json!({"code":"MIX_GRAPH_CYCLE", "stage":"validation", "severity":"error", "message":"Cycle detected."})
    );
}

#[test]
fn diagnostics_invalid_wire_values_are_rejected() {
    let original = serde_json::to_value(mismatch()).expect("serialize diagnostic");
    for (field, value) in [
        ("code", serde_json::json!("MIX_MADE_UP")),
        ("stage", serde_json::json!("fallback")),
        ("severity", serde_json::json!("healthy")),
        ("unexpected", serde_json::json!(true)),
        ("source", serde_json::json!("a forged native source")),
    ] {
        let mut input = original.clone();
        input[field] = value;
        assert!(
            serde_json::from_value::<Diagnostic>(input).is_err(),
            "reject {field}"
        );
    }
    for value in [
        "-1",
        "0.5",
        "null",
        "[]",
        "{}",
        "18446744073709551616",
        "NaN",
        "Infinity",
    ] {
        assert!(
            serde_json::from_str::<EvidenceValue>(value).is_err(),
            "reject {value}"
        );
    }
}

#[test]
fn diagnostics_evidence_preserves_integer_precision_and_key_order() {
    let left = mismatch()
        .with_evidence("zCount", u64::MAX)
        .with_evidence("aFlag", true);
    let right = mismatch()
        .with_evidence("aFlag", true)
        .with_evidence("zCount", u64::MAX);
    let encoded = serde_json::to_string(&left).expect("serialize exact evidence");
    assert_eq!(
        encoded,
        serde_json::to_string(&right).expect("serialize reversed evidence")
    );
    assert!(encoded.contains("18446744073709551615"));
    let decoded: Diagnostic = serde_json::from_str(&encoded).expect("decode exact evidence");
    assert_eq!(
        decoded.evidence["zCount"],
        EvidenceValue::Unsigned(u64::MAX)
    );
}

#[test]
fn diagnostics_order_is_independent_of_input_order_including_ties() {
    let base = mismatch();
    let mut alternate_path = base.clone();
    alternate_path.document_path = Some("a.mix".into());
    let mut alternate_node = base.clone();
    alternate_node.node_id = Some("another-node".into());
    let mut alternate_port = base.clone();
    alternate_port.port_id = Some("another-port".into());
    let mut alternate_parameter = base.clone();
    alternate_parameter.parameter_id = Some("another-parameter".into());
    let mut alternate_code = base.clone();
    alternate_code.code = Code::GraphCycle;
    let mut alternate_message = base.clone();
    alternate_message.message = "Another message.".into();
    let alternate_evidence = base.clone().with_evidence("additional", 1_u64);
    let alternate_suggestion = base.clone().with_suggestion("Another suggestion.");
    let mut warning = base.clone();
    warning.severity = Severity::Warning;
    let parse = Diagnostic::error(Code::ParseInvalidJson, Stage::Parse, "Parse failed.");
    let mut inputs = vec![
        base.clone(),
        alternate_path,
        alternate_node,
        alternate_port,
        alternate_parameter,
        alternate_code,
        alternate_message,
        alternate_evidence,
        alternate_suggestion,
        warning,
        parse,
        base.with_source(io::Error::other("different native cause")),
    ];
    let report = DiagnosticReport::new(inputs.clone());
    let expected = serde_json::to_string(&report).expect("serialize ordered report");
    assert_eq!(report.diagnostics()[0].stage, Stage::Parse);
    assert_eq!(
        report.diagnostics()[1].document_path.as_deref(),
        Some("a.mix")
    );
    assert_eq!(
        report
            .diagnostics()
            .last()
            .expect("nonempty report")
            .severity,
        Severity::Warning
    );
    for _ in 0..inputs.len() {
        inputs.rotate_left(1);
        for permutation in [inputs.clone(), inputs.iter().rev().cloned().collect()] {
            assert_eq!(
                serde_json::to_string(&DiagnosticReport::new(permutation))
                    .expect("serialize permutation"),
                expected
            );
        }
    }
}

#[test]
fn diagnostics_report_success_is_derived_from_severity() {
    assert_eq!(
        serde_json::to_string(&DiagnosticReport::new([])).expect("serialize empty report"),
        r#"{"ok":true,"diagnostics":[]}"#
    );
    for severity in [Severity::Warning, Severity::Info] {
        let observation = Diagnostic::new(
            Code::CompileInvalidRequest,
            Stage::Compile,
            severity,
            "Contract-test observation.",
        );
        assert!(DiagnosticReport::new([observation.clone()]).is_ok());
        assert!(!DiagnosticReport::new([observation, mismatch()]).is_ok());
    }
}

#[derive(Debug)]
struct ParserFailure(io::Error);

impl fmt::Display for ParserFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("parser failed")
    }
}

impl Error for ParserFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn diagnostics_preserve_real_source_chains_without_leaking_them_into_json() {
    let diagnostic = Diagnostic::error(Code::ParseInvalidJson, Stage::Parse, "Invalid document.")
        .with_source(ParserFailure(io::Error::other("sensitive native detail")));
    let source = diagnostic.source().expect("source retained");
    assert!(source.downcast_ref::<ParserFailure>().is_some());
    assert!(
        source
            .source()
            .expect("nested source retained")
            .downcast_ref::<io::Error>()
            .is_some()
    );
    assert_eq!(
        diagnostic.to_string(),
        "MIX_PARSE_INVALID_JSON: Invalid document."
    );
    assert!(diagnostic.clone().source().is_some());
    let encoded = serde_json::to_string(&diagnostic).expect("serialize diagnostic");
    assert!(!encoded.contains("native detail"));
    assert!(!encoded.contains("source"));
    let decoded: Diagnostic = serde_json::from_str(&encoded).expect("decode diagnostic");
    assert!(decoded.source().is_none());
}

#[test]
fn diagnostics_codes_and_stage_spellings_match_the_wire_vocabulary() {
    let codes = [
        Code::ParseInvalidUtf8,
        Code::ParseInvalidJson,
        Code::FormatUnsupportedVersion,
        Code::LimitDecodedBytesExceeded,
        Code::LimitNodesExceeded,
        Code::LimitEdgesExceeded,
        Code::LimitExposedParametersExceeded,
        Code::LimitOutputDimensionExceeded,
        Code::LimitRequestedOutputsExceeded,
        Code::LimitTransientBytesExceeded,
        Code::NodeUnknownType,
        Code::NodeUnsupportedVersion,
        Code::PortUnknown,
        Code::PortTypeMismatch,
        Code::GraphCycle,
        Code::ParameterInvalidValue,
        Code::CompileInvalidRequest,
        Code::GpuAdapterUnavailable,
        Code::GpuDeviceRequestFailed,
        Code::GpuShaderValidationFailed,
        Code::GpuExecutionFailed,
        Code::ReadbackFailed,
        Code::EncodingFailed,
        Code::FormatInvalidDocument,
        Code::NodeInvalidId,
        Code::NodeDuplicateId,
        Code::GraphUnknownNode,
        Code::GraphDuplicateEdge,
        Code::GraphMultipleInputs,
        Code::GraphMaterialOutputCount,
        Code::PortRequiredConnection,
        Code::ParameterUnknown,
        Code::ExposedParameterInvalid,
        Code::IoReadFailed,
    ];
    for code in codes {
        assert_eq!(
            serde_json::to_value(code).expect("serialize code"),
            code.as_str()
        );
        assert_eq!(
            serde_json::from_value::<Code>(serde_json::json!(code.as_str())).expect("decode code"),
            code
        );
    }
    let stages = [
        Stage::Parse,
        Stage::Validation,
        Stage::Compile,
        Stage::GpuAdapter,
        Stage::GpuDevice,
        Stage::GpuShader,
        Stage::GpuPipeline,
        Stage::GpuExecution,
        Stage::Readback,
        Stage::Encoding,
    ];
    let vocabulary = serde_json::json!({ "codes": codes.as_slice(), "stages": stages, "severities": [Severity::Error, Severity::Warning, Severity::Info] });
    assert_eq!(
        serde_json::to_string_pretty(&vocabulary).expect("serialize vocabulary"),
        include_str!("snapshots/diagnostics-vocabulary.json").trim_end()
    );
}
