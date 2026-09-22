//! MAT-01c public Scalar mask contracts and deterministic lowering.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile, plan::KernelInvocation,
};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/scalar-mask-blend/input.mix"
    ))
    .unwrap()
}
fn decode(input: &Value) -> MaterialDocument {
    MaterialDocument::decode(
        &serde_json::to_vec(input).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
}
#[test]
fn mask_is_required_even_at_zero_opacity_and_requires_scalar_kind() {
    let mut input = source();
    input["nodes"][3]["parameters"] = json!({"opacity":0});
    input["edges"].as_array_mut().unwrap().remove(2);
    assert!(
        decode(&input)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
    let mut input = source();
    input["edges"][2]["from"] = json!({"nodeId":"color","portId":"color"});
    assert!(
        decode(&input)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
    input = source();
    input["nodes"][3]["version"] = json!(2);
    assert!(
        decode(&input)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
}
#[test]
fn mask_lowering_preserves_three_bindings_endpoints_and_old_source() {
    let input = source();
    let document = decode(&input)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut request = CompileRequest {
        size: [65, 3],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let original = compile(&document, &request).unwrap();
    assert_eq!(original.passes().len(), 4);
    let pass = &original.passes()[3];
    assert_eq!(pass.kernel.inputs().count(), 3);
    assert_eq!(pass.kernel.uniform_bytes(), 16);
    assert!(matches!(
        pass.kernel,
        KernelInvocation::ScalarMaskBlend { opacity: 1.0, .. }
    ));
    request.overrides.insert("opacity".into(), json!(1));
    assert_eq!(
        original.hash(),
        compile(&document, &request).unwrap().hash()
    );
    request.overrides.insert("opacity".into(), json!(0));
    let endpoint = compile(&document, &request).unwrap();
    assert_eq!(endpoint.passes().len(), 4);
    assert_ne!(original.hash(), endpoint.hash());
    request.overrides.insert("opacity".into(), json!(1.01));
    assert!(compile(&document, &request).is_err());
    assert_eq!(
        serde_json::to_value(document.document()).unwrap(),
        serde_json::to_value(decode(&input)).unwrap()
    );
}
#[test]
fn mask_shared_producers_and_unused_overrides_preserve_determinism() {
    let mut input = source();
    input["edges"][2]["from"]["nodeId"] = json!("a");
    let document = decode(&input)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut request = CompileRequest {
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let plan = compile(&document, &request).unwrap();
    assert_eq!(plan.passes().len(), 3);
    let bindings: Vec<_> = plan.passes()[2].kernel.inputs().collect();
    assert_eq!(bindings[0], bindings[2]);
    request.outputs = vec![OutputChannel::BaseColor];
    let sliced = compile(&document, &request).unwrap();
    request.overrides.insert("opacity".into(), json!(0));
    assert_eq!(sliced.hash(), compile(&document, &request).unwrap().hash());
}
