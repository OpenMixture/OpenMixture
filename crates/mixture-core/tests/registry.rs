//! Contract consistency and parameter boundaries independent of pixel execution.
use mixture_core::registry::{BUILT_INS, ParameterKind, PortKind, node_contract};
use serde_json::json;
use std::collections::BTreeSet;
#[test]
fn registry_matches_reviewed_type_versions() {
    // This explicit list is a review gate, not a permanent node-count budget.
    // New types or versions must update it alongside their approved use case.
    let identities: Vec<_> = BUILT_INS.iter().map(|c| (c.type_id, c.version)).collect();
    assert_eq!(
        identities,
        [
            ("blend", 1),
            ("brick-pattern", 1),
            ("checker", 1),
            ("constant-color", 1),
            ("constant-scalar", 1),
            ("fractal-noise", 2),
            ("gradient-map", 1),
            ("height-to-normal", 1),
            ("image-input", 1),
            ("levels", 1),
            ("material-output", 1),
            ("scalar-blend", 1),
            ("scalar-mask-blend", 1),
            ("scalar-morphology", 1),
            ("scalar-subtract", 1),
            ("transform-2d", 1),
            ("warp", 1)
        ]
    );
}

#[test]
fn registry_contracts_have_valid_defaults_and_explicit_seed() {
    for contract in BUILT_INS {
        assert_eq!(
            contract
                .parameters
                .iter()
                .map(|p| p.id)
                .collect::<BTreeSet<_>>()
                .len(),
            contract.parameters.len()
        );
        for parameter in contract.parameters {
            if let Some(default) = parameter.default {
                assert!(
                    parameter.accepts(&default.value()),
                    "{} {} default",
                    contract.type_id,
                    parameter.id
                );
            } else {
                assert!(matches!(
                    (contract.type_id, parameter.id),
                    ("fractal-noise", "seed")
                        | ("brick-pattern", "seed")
                        | ("image-input", "resourceId")
                ));
            }
            match parameter.kind {
                ParameterKind::ResourceRef => {
                    assert!(parameter.accepts(&json!("heightSource")));
                    assert!(!parameter.accepts(&json!("../height.png")));
                }
                ParameterKind::Float { min, max } => {
                    assert!(parameter.accepts(&json!(min)));
                    assert!(parameter.accepts(&json!(max)));
                    assert!(!parameter.accepts(&json!(min - 1.0)));
                    assert!(!parameter.accepts(&json!(max + 1.0)));
                }
                ParameterKind::Integer { min, max } => {
                    assert!(parameter.accepts(&json!(min)));
                    assert!(parameter.accepts(&json!(max)));
                    assert!(!parameter.accepts(&json!(u64::from(max) + 1)));
                    assert!(!parameter.accepts(&json!(f64::from(min))));
                }
                ParameterKind::Color => {
                    assert!(parameter.accepts(&json!([0, 1, 0.5, 1])));
                    assert!(!parameter.accepts(&json!([0, 1, 0.5])));
                }
                ParameterKind::Enum { values } => {
                    for value in values {
                        assert!(parameter.accepts(&json!(value)));
                    }
                    assert!(!parameter.accepts(&json!("unknown-mode")));
                }
            }
            assert!(!parameter.accepts(&json!(null)));
            assert!(!parameter.accepts(&json!(true)));
        }
        for ports in [contract.inputs, contract.outputs] {
            assert_eq!(
                ports.iter().map(|p| p.id).collect::<BTreeSet<_>>().len(),
                ports.len()
            );
        }
    }
    assert!(node_contract("Checker").is_none());
    assert!(node_contract("unregistered-node").is_none());
    assert!(
        node_contract("fractal-noise")
            .unwrap()
            .parameter("seed")
            .unwrap()
            .default
            .is_none()
    );
    assert_eq!(
        node_contract("checker")
            .unwrap()
            .output("color")
            .unwrap()
            .kind,
        PortKind::Color
    );
    assert!(node_contract("checker").unwrap().input("color").is_none());
    assert!(node_contract("material-output").unwrap().outputs.is_empty());
}
