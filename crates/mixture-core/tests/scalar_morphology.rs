//! MAT-02 morphology admission and typed lowering, independent of GPU availability.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile,
    plan::{KernelInvocation, MorphologyAxis, MorphologyOperation},
};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/scalar-morphology/input.mix"
    ))
    .unwrap()
}
fn decode(value: &Value) -> MaterialDocument {
    MaterialDocument::decode(
        &serde_json::to_vec(value).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
}
#[test]
fn morphology_rejects_invalid_contracts_before_execution() {
    for params in [
        json!({"radius":-1}),
        json!({"radius":17}),
        json!({"radius":1.5}),
        json!({"axis":"z"}),
        json!({"operation":"blur"}),
        json!({"seed":0}),
        json!({"radius":null}),
    ] {
        let mut value = source();
        value["nodes"][1]["parameters"] = params;
        assert!(
            decode(&value)
                .into_validated(&SafetyLimits::default())
                .is_err()
        );
    }
    let mut value = source();
    value["edges"].as_array_mut().unwrap().remove(0);
    assert!(
        decode(&value)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
    let mut value = source();
    value["edges"][0]["from"] = json!({"nodeId":"color","portId":"color"});
    assert!(
        decode(&value)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
    let mut value = source();
    value["nodes"][1]["version"] = json!(2);
    assert!(
        decode(&value)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
    let invalid = serde_json::to_string(&source()).unwrap().replace(
        "\"type\":\"scalar-morphology\"",
        "\"type\":\"scalar-morphology\",\"parameters\":{\"radius\":1e999}",
    );
    assert!(MaterialDocument::decode(invalid.as_bytes(), &SafetyLimits::default()).is_err());
}
#[test]
fn morphology_lowering_hashing_and_slicing_follow_all_parameters() {
    let original = source();
    let document = decode(&original)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut request = CompileRequest {
        size: [19, 11],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let default = compile(&document, &request).unwrap();
    assert_eq!(default.passes().len(), 2);
    let kernel = &default.passes()[1].kernel;
    assert_eq!(kernel.inputs().count(), 1);
    assert_eq!(kernel.uniform_bytes(), 16);
    assert!(matches!(
        kernel,
        KernelInvocation::ScalarMorphology {
            operation: MorphologyOperation::Erode,
            axis: MorphologyAxis::X,
            radius: 2,
            ..
        }
    ));
    request.overrides =
        serde_json::from_value(json!({"operation":"erode","axis":"x","radius":2})).unwrap();
    assert_eq!(default.hash(), compile(&document, &request).unwrap().hash());
    for changes in [
        json!({"operation":"dilate"}),
        json!({"axis":"y"}),
        json!({"radius":0}),
        json!({"radius":16}),
    ] {
        request.overrides = serde_json::from_value(changes).unwrap();
        let changed = compile(&document, &request).unwrap();
        assert_ne!(default.hash(), changed.hash());
        assert_eq!(changed.passes().len(), 2);
        assert_eq!(changed.hash(), compile(&document, &request).unwrap().hash());
    }
    request.outputs = vec![OutputChannel::BaseColor];
    request.overrides.clear();
    let sliced = compile(&document, &request).unwrap();
    request.overrides.insert("radius".into(), json!(16));
    assert_eq!(sliced.hash(), compile(&document, &request).unwrap().hash());
    assert_eq!(
        serde_json::to_value(document.document()).unwrap(),
        serde_json::to_value(decode(&original)).unwrap()
    );
}
