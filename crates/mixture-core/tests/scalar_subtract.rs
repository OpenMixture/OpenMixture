//! Bounded subtract admission: required kinds, exact bindings, slicing and deterministic plans.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile, plan::KernelInvocation,
};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/scalar-subtract/input.mix"
    ))
    .unwrap()
}
fn decode(v: &Value) -> MaterialDocument {
    MaterialDocument::decode(&serde_json::to_vec(v).unwrap(), &SafetyLimits::default()).unwrap()
}
#[test]
fn subtract_rejects_missing_inputs_wrong_kinds_versions_and_parameters() {
    for index in [0, 1] {
        let mut v = source();
        v["edges"].as_array_mut().unwrap().remove(index);
        assert!(decode(&v).into_validated(&SafetyLimits::default()).is_err());
    }
    for index in [0, 1] {
        let mut v = source();
        v["edges"][index]["from"] = json!({"nodeId":"color","portId":"color"});
        assert!(decode(&v).into_validated(&SafetyLimits::default()).is_err());
    }
    let mut v = source();
    v["nodes"][2]["version"] = json!(2);
    assert!(decode(&v).into_validated(&SafetyLimits::default()).is_err());
    let mut v = source();
    v["nodes"][2]["parameters"] = json!({"weight":1});
    assert!(decode(&v).into_validated(&SafetyLimits::default()).is_err());
}
#[test]
fn subtract_preserves_order_shared_bindings_and_source() {
    let original = source();
    let doc = decode(&original)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut request = CompileRequest {
        size: [19, 11],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let plan = compile(&doc, &request).unwrap();
    assert_eq!(plan.passes().len(), 3);
    let kernel = &plan.passes()[2].kernel;
    assert!(matches!(kernel, KernelInvocation::ScalarSubtract { .. }));
    assert_eq!(kernel.uniform_bytes(), 16);
    let inputs: Vec<_> = kernel.inputs().collect();
    assert_ne!(inputs[0], inputs[1]);
    let mut swapped = source();
    swapped["edges"][0]["from"]["nodeId"] = json!("b");
    swapped["edges"][1]["from"]["nodeId"] = json!("a");
    let other = compile(
        &decode(&swapped)
            .into_validated(&SafetyLimits::default())
            .unwrap(),
        &request,
    )
    .unwrap();
    assert_ne!(plan.hash(), other.hash());
    let reversed: Vec<_> = other.passes()[2].kernel.inputs().collect();
    assert_eq!(inputs, vec![reversed[1], reversed[0]]);
    let mut shared = source();
    shared["edges"][1]["from"]["nodeId"] = json!("a");
    let same = compile(
        &decode(&shared)
            .into_validated(&SafetyLimits::default())
            .unwrap(),
        &request,
    )
    .unwrap();
    assert_eq!(same.passes().len(), 2);
    let inputs: Vec<_> = same.passes()[1].kernel.inputs().collect();
    assert_eq!(inputs[0], inputs[1]);
    request.outputs = vec![OutputChannel::BaseColor];
    let sliced = compile(&doc, &request).unwrap();
    request.overrides.insert("a".into(), json!(1));
    assert_eq!(sliced.hash(), compile(&doc, &request).unwrap().hash());
    assert_eq!(
        serde_json::to_value(doc.document()).unwrap(),
        serde_json::to_value(decode(&original)).unwrap()
    );
}
