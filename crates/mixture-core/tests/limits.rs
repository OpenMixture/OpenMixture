//! Public-API boundary and rejection tests for conservative resource limits.

use std::error::Error;

use mixture_core::{
    DiagnosticCode as Code, EvidenceValue, LimitExceeded, LimitKind, SafetyLimits, Stage,
};

const LIMITS: [(LimitKind, u64, Code); 7] = [
    (
        LimitKind::DecodedBytes,
        2_097_152,
        Code::LimitDecodedBytesExceeded,
    ),
    (LimitKind::Nodes, 128, Code::LimitNodesExceeded),
    (LimitKind::Edges, 512, Code::LimitEdgesExceeded),
    (
        LimitKind::ExposedParameters,
        64,
        Code::LimitExposedParametersExceeded,
    ),
    (
        LimitKind::OutputDimension,
        2048,
        Code::LimitOutputDimensionExceeded,
    ),
    (
        LimitKind::RequestedOutputs,
        8,
        Code::LimitRequestedOutputsExceeded,
    ),
    (
        LimitKind::TransientBytes,
        536_870_912,
        Code::LimitTransientBytesExceeded,
    ),
];

#[test]
fn limits_defaults_match_architecture_and_reviewed_snapshot() {
    let limits = SafetyLimits::default();
    for (kind, expected, _) in LIMITS {
        assert_eq!(limits.maximum(kind), expected);
    }
    let json = serde_json::to_string_pretty(&limits).expect("serialize defaults");
    assert_eq!(
        json,
        include_str!("snapshots/limits-defaults.json").trim_end()
    );
    assert_eq!(
        serde_json::from_str::<SafetyLimits>(&json).expect("decode limits"),
        limits
    );
}

#[test]
fn limits_accept_below_and_at_boundary_but_reject_above_with_exact_evidence() {
    let limits = SafetyLimits::default();
    for (kind, maximum, code) in LIMITS {
        for observed in [0, maximum - 1, maximum] {
            limits.check(kind, observed).expect("within the ceiling");
        }
        let error = limits
            .check(kind, maximum + 1)
            .expect_err("over the ceiling");
        assert_eq!(
            error,
            LimitExceeded {
                kind,
                configured: maximum,
                observed: maximum + 1
            }
        );
        let diagnostic = error.diagnostic(Stage::Validation);
        assert_eq!(diagnostic.code, code);
        assert_eq!(
            diagnostic.evidence["limit"],
            EvidenceValue::from(kind.as_str())
        );
        assert_eq!(
            diagnostic.evidence["configured"],
            EvidenceValue::Unsigned(maximum)
        );
        assert_eq!(
            diagnostic.evidence["observed"],
            EvidenceValue::Unsigned(maximum + 1)
        );
        assert!(diagnostic.suggestion.is_some());
        assert!(
            diagnostic
                .source()
                .expect("typed source retained")
                .downcast_ref::<LimitExceeded>()
                .is_some()
        );
    }
    assert_eq!(
        limits,
        SafetyLimits::default(),
        "checks must never mutate policy"
    );
}

#[test]
fn limits_failure_json_matches_snapshot_and_preserves_callers_stage() {
    let violation = SafetyLimits::default()
        .check(LimitKind::Nodes, 129)
        .expect_err("node limit exceeded");
    assert_eq!(
        serde_json::to_string_pretty(&violation.diagnostic(Stage::Validation))
            .expect("serialize limit error"),
        include_str!("snapshots/limits-exceeded.json").trim_end()
    );
    assert_eq!(violation.diagnostic(Stage::Compile).stage, Stage::Compile);
}

#[test]
fn limits_overrides_are_explicit_and_zero_is_not_replaced_by_a_default() {
    let configured = SafetyLimits {
        nodes: 256,
        edges: 0,
        ..SafetyLimits::default()
    };
    assert!(
        SafetyLimits::default()
            .check(LimitKind::Nodes, 200)
            .is_err()
    );
    configured
        .check(LimitKind::Nodes, 200)
        .expect("explicit higher ceiling");
    configured
        .check(LimitKind::Edges, 0)
        .expect("zero usage with zero budget");
    assert_eq!(
        configured
            .check(LimitKind::Edges, 1)
            .expect_err("zero budget enforced")
            .configured,
        0
    );
    assert_eq!(
        configured.decoded_bytes,
        SafetyLimits::default().decoded_bytes
    );
    let json = serde_json::to_string(&configured).expect("serialize override");
    assert_eq!(
        serde_json::from_str::<SafetyLimits>(&json).expect("decode override"),
        configured
    );
}

#[test]
fn limits_extreme_counts_do_not_overflow_or_truncate() {
    let limits = SafetyLimits {
        transient_bytes: u64::MAX,
        ..SafetyLimits::default()
    };
    limits
        .check(LimitKind::TransientBytes, u64::MAX)
        .expect("exact maximum accepted");
    let violation = limits
        .check(LimitKind::DecodedBytes, u64::MAX)
        .expect_err("huge input rejected");
    let json =
        serde_json::to_string(&violation.diagnostic(Stage::Parse)).expect("serialize exact count");
    assert!(json.contains("18446744073709551615"));
    assert_eq!(violation.observed, u64::MAX);
}

#[test]
fn limits_config_rejects_missing_unknown_negative_fractional_and_overflow_fields() {
    assert!(serde_json::from_str::<SafetyLimits>("{}").is_err());
    for invalid in [
        "-1",
        "1.5",
        "null",
        "true",
        "\"128\"",
        "18446744073709551616",
    ] {
        let json = include_str!("snapshots/limits-defaults.json")
            .replace("\"nodes\": 128", &format!("\"nodes\": {invalid}"));
        assert!(
            serde_json::from_str::<SafetyLimits>(&json).is_err(),
            "reject {invalid}"
        );
    }
    let mut config = serde_json::to_value(SafetyLimits::default()).expect("serialize defaults");
    config["typo"] = serde_json::json!(128);
    assert!(serde_json::from_value::<SafetyLimits>(config).is_err());
}

#[test]
fn limits_output_dimension_is_checked_independently_for_each_axis() {
    let limits = SafetyLimits::default();
    for [width, height] in [[2048, 2049], [2049, 2048]] {
        let violations: Vec<_> = [width, height]
            .into_iter()
            .filter_map(|axis| limits.check(LimitKind::OutputDimension, axis).err())
            .collect();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].observed, 2049);
    }
}
