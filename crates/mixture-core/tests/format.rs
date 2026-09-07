//! Consumer-level decoding, rejection, resource-boundary, and round-trip tests.
use mixture_core::{
    DiagnosticCode as Code, EvidenceValue, LimitExceeded, MaterialDocument, SafetyLimits, Stage,
};
use serde_json::{Value, json};
use std::error::Error;
const CHECKER: &[u8] = include_bytes!("../../../fixtures/format/valid/checker.mix");
const ALL: &[u8] = include_bytes!("../../../fixtures/format/valid/all-m2.mix");
fn decode(bytes: &[u8]) -> MaterialDocument {
    MaterialDocument::decode(bytes, &SafetyLimits::default()).unwrap()
}
fn reject(bytes: &[u8], code: Code) {
    let error = MaterialDocument::decode(bytes, &SafetyLimits::default()).unwrap_err();
    assert_eq!(error.report().diagnostics()[0].code, code, "{error}");
    assert!(!error.report().is_ok());
}
#[test]
fn format_round_trip_is_canonical_without_mutation_or_default_insertion() {
    let original = decode(ALL);
    let before = original.clone();
    let canonical = original.to_json().unwrap();
    let mut reordered = original.clone();
    reordered.nodes.reverse();
    reordered.edges.reverse();
    reordered.exposed_parameters.reverse();
    assert_eq!(reordered.to_json().unwrap(), canonical);
    assert_eq!(decode(&canonical).to_json().unwrap(), canonical);
    assert_eq!(original, before);
    let checker = decode(CHECKER);
    let encoded: Value = serde_json::from_slice(&checker.to_json().unwrap()).unwrap();
    assert_eq!(encoded["nodes"][0]["parameters"], json!({}));
    let validated = checker
        .clone()
        .into_validated(&SafetyLimits::default())
        .unwrap();
    assert_eq!(validated.document(), &checker);
    assert_eq!(validated.parameter("checker", "cellsX"), Some(json!(8)));
}
#[test]
fn format_rejects_invalid_utf8_json_trailing_data_and_nonfinite_numbers() {
    reject(&[0xff], Code::ParseInvalidUtf8);
    let error = MaterialDocument::decode(&[0xff], &SafetyLimits::default()).unwrap_err();
    assert!(
        error.report().diagnostics()[0]
            .source()
            .unwrap()
            .is::<std::str::Utf8Error>()
    );
    for bytes in [
        b"".as_slice(),
        b"{",
        b"{\"version\":1,}",
        b"{ /*comment*/ }",
        b"{\"version\":NaN}",
    ] {
        reject(bytes, Code::ParseInvalidJson);
    }
    let mut trailing = CHECKER.to_vec();
    trailing.extend(b" {}");
    reject(&trailing, Code::ParseInvalidJson);
    for number in ["NaN", "Infinity", "1e400"] {
        let raw = format!(
            r#"{{"version":1,"nodes":[{{"id":"n","type":"constant-scalar","version":1,"parameters":{{"value":{number}}}}}],"edges":[]}}"#
        );
        reject(raw.as_bytes(), Code::ParseInvalidJson);
    }
    let raw = format!(
        r#"{{"version":1,"nodes":[{{"id":"n","type":"constant-scalar","version":1,"parameters":{{"value":{}0{}}}}}],"edges":[]}}"#,
        "[".repeat(256),
        "]".repeat(256)
    );
    reject(raw.as_bytes(), Code::ParseInvalidJson);
}
#[test]
fn format_rejects_duplicate_keys_at_every_object_boundary() {
    let cases = [
        r#"{"version":1,"version":1,"nodes":[],"edges":[]}"#,
        r#"{"version":1,"nodes":[],"edges":[],"edges":[]}"#,
        r#"{"version":1,"nodes":[{"id":"a","id":"b","type":"checker","version":1}],"edges":[]}"#,
        r#"{"version":1,"nodes":[{"id":"a","type":"checker","version":1,"parameters":{"cellsX":8,"cells\u0058":9}}],"edges":[]}"#,
        r#"{"version":1,"nodes":[{"id":"a","type":"checker","version":1,"parameters":{"x":{"a":1,"a":2}}}],"edges":[]}"#,
        r#"{"version":1,"nodes":[],"edges":[{"from":{"nodeId":"a","portId":"color","portId":"value"},"to":{"nodeId":"b","portId":"in"}}]}"#,
        r#"{"version":1,"nodes":[],"edges":[],"exposedParameters":[{"id":"x","id":"y","nodeId":"a","parameterId":"cellsX"}]}"#,
    ];
    for raw in cases {
        reject(raw.as_bytes(), Code::FormatInvalidDocument);
    }
}
#[test]
fn format_rejects_unknown_fields_missing_fields_and_wrong_structural_types() {
    let original: Value = serde_json::from_slice(CHECKER).unwrap();
    for pointer in [
        "",
        "/nodes/0",
        "/edges/0",
        "/edges/0/from",
        "/exposedParameters/0",
    ] {
        let mut value = original.clone();
        value.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
        reject(
            &serde_json::to_vec(&value).unwrap(),
            Code::FormatInvalidDocument,
        );
    }
    for key in ["version", "nodes", "edges"] {
        let mut value = original.clone();
        value.as_object_mut().unwrap().remove(key);
        reject(
            &serde_json::to_vec(&value).unwrap(),
            Code::FormatInvalidDocument,
        );
    }
    for (pointer, bad) in [
        ("/version", json!(1.0)),
        ("/version", json!(-1)),
        ("/version", json!(4294967296_u64)),
        ("/nodes", Value::Null),
        ("/nodes/0/version", json!("1")),
        ("/nodes/0/parameters", Value::Null),
        ("/exposedParameters", Value::Null),
    ] {
        let mut value = original.clone();
        if pointer.ends_with("parameters") {
            value["nodes"][0]["parameters"] = bad;
        } else {
            *value.pointer_mut(pointer).unwrap() = bad;
        }
        reject(
            &serde_json::to_vec(&value).unwrap(),
            Code::FormatInvalidDocument,
        );
    }
    for version in [0, 2, u32::MAX] {
        let mut value = original.clone();
        value["version"] = json!(version);
        reject(
            &serde_json::to_vec(&value).unwrap(),
            Code::FormatUnsupportedVersion,
        );
    }
    let error = MaterialDocument::decode(b"{}", &SafetyLimits::default()).unwrap_err();
    let diagnostic = &error.report().diagnostics()[0];
    assert!(diagnostic.source().unwrap().is::<serde_json::Error>());
    assert!(diagnostic.evidence.contains_key("line"));
}
#[test]
fn format_byte_limit_is_checked_before_utf8_or_json_and_accepts_equality() {
    let mut limits = SafetyLimits {
        decoded_bytes: CHECKER.len() as u64,
        ..Default::default()
    };
    MaterialDocument::decode(CHECKER, &limits).unwrap();
    limits.decoded_bytes -= 1;
    let error = MaterialDocument::decode(CHECKER, &limits).unwrap_err();
    let diagnostic = &error.report().diagnostics()[0];
    assert_eq!(diagnostic.code, Code::LimitDecodedBytesExceeded);
    assert_eq!(
        diagnostic.evidence["observed"],
        EvidenceValue::Unsigned(CHECKER.len() as u64)
    );
    assert!(diagnostic.source().unwrap().is::<LimitExceeded>());
    limits.decoded_bytes = 0;
    assert_eq!(
        MaterialDocument::decode(&[0xff], &limits)
            .unwrap_err()
            .report()
            .diagnostics()[0]
            .code,
        Code::LimitDecodedBytesExceeded
    );
}
#[test]
fn format_collection_limits_stop_before_constructing_an_extra_item() {
    let limits = SafetyLimits {
        nodes: 0,
        edges: 0,
        exposed_parameters: 0,
        ..Default::default()
    };
    for (field, code) in [
        ("nodes", Code::LimitNodesExceeded),
        ("edges", Code::LimitEdgesExceeded),
        ("exposedParameters", Code::LimitExposedParametersExceeded),
    ] {
        let mut value = json!({"version":1,"nodes":[],"edges":[],"exposedParameters":[]});
        // null would fail typed decoding. The collection ceiling takes precedence.
        value[field] = json!([null]);
        let error =
            MaterialDocument::decode(&serde_json::to_vec(&value).unwrap(), &limits).unwrap_err();
        let diagnostic = &error.report().diagnostics()[0];
        assert_eq!(diagnostic.code, code);
        assert_eq!(diagnostic.stage, Stage::Parse);
        assert_eq!(diagnostic.evidence["observed"], EvidenceValue::Unsigned(1));
    }
    let empty = br#"{"version":1,"nodes":[],"edges":[]}"#;
    MaterialDocument::decode(empty, &limits).unwrap();
    let exact = SafetyLimits {
        nodes: 2,
        edges: 1,
        exposed_parameters: 1,
        ..Default::default()
    };
    MaterialDocument::decode(CHECKER, &exact).unwrap();
    let mut value: Value = serde_json::from_slice(CHECKER).unwrap();
    value["nodes"] = Value::Array(
        (0..129)
            .map(|i| json!({"id":format!("n{i}"),"type":"checker","version":1}))
            .collect(),
    );
    assert_eq!(
        MaterialDocument::decode(
            &serde_json::to_vec(&value).unwrap(),
            &SafetyLimits::default()
        )
        .unwrap_err()
        .report()
        .diagnostics()[0]
            .code,
        Code::LimitNodesExceeded
    );
    MaterialDocument::decode(
        &serde_json::to_vec(&value).unwrap(),
        &SafetyLimits {
            nodes: 129,
            ..Default::default()
        },
    )
    .unwrap();
}

#[test]
fn format_numeric_parameters_do_not_drift_during_serialization_round_trips() {
    let mut document = decode(ALL);
    // A legal gamma value with a known rounding edge in the fast JSON parser.
    let gamma = 51.248178375505404_f64;
    document
        .nodes
        .iter_mut()
        .find(|node| node.id == "levels")
        .unwrap()
        .parameters
        .insert("gamma".into(), json!(gamma));
    let bytes = document.to_json().unwrap();
    let decoded = decode(&bytes)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    assert_eq!(
        decoded
            .parameter("levels", "gamma")
            .unwrap()
            .as_f64()
            .unwrap()
            .to_bits(),
        gamma.to_bits()
    );
    assert_eq!(decoded.document().to_json().unwrap(), bytes);
}
