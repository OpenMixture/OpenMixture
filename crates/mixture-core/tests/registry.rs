//! Contract consistency and parameter boundaries independent of pixel execution.
use mixture_core::registry::{BUILT_INS, ParameterKind, PortKind, node_contract};
use serde_json::json;
use std::collections::BTreeSet;
#[test]
fn registry_has_exactly_the_six_version_one_contracts_with_valid_unique_defaults() {
    let names: Vec<_> = BUILT_INS.iter().map(|c| c.type_id).collect();
    assert_eq!(
        names,
        [
            "blend",
            "checker",
            "constant-color",
            "constant-scalar",
            "levels",
            "material-output"
        ]
    );
    for contract in BUILT_INS {
        assert_eq!(contract.version, 1);
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
            assert!(
                parameter.accepts(&parameter.default.value()),
                "{} {} default",
                contract.type_id,
                parameter.id
            );
            match parameter.kind {
                ParameterKind::Float { min, max } => {
                    assert!(parameter.accepts(&json!(min)));
                    assert!(parameter.accepts(&json!(max)));
                    assert!(!parameter.accepts(&json!(min - 1.0)));
                    assert!(!parameter.accepts(&json!(max + 1.0)));
                }
                ParameterKind::Integer { min, max } => {
                    assert!(parameter.accepts(&json!(min)));
                    assert!(parameter.accepts(&json!(max)));
                    assert!(!parameter.accepts(&json!(max + 1)));
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
    assert!(node_contract("fractal-noise").is_none());
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
