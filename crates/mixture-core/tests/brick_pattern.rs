//! MAT-01b public contracts: periodic constraints, slicing and immutable lowering.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile, plan::KernelInvocation,
};
use serde_json::{Value, json};

fn source() -> Value {
    json!({
        "version": 1,
        "nodes": [
            {"id":"brick","type":"brick-pattern","version":1,"parameters":{"seed":4294967295_u32}},
            {"id":"color","type":"constant-color","version":1},
            {"id":"out","type":"material-output","version":1}
        ],
        "edges":[
            {"from":{"nodeId":"brick","portId":"value"},"to":{"nodeId":"out","portId":"height"}},
            {"from":{"nodeId":"color","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}
        ],
        "exposedParameters": [
            {"id":"seed","nodeId":"brick","parameterId":"seed"},
            {"id":"rows","nodeId":"brick","parameterId":"rows"},
            {"id":"offset","nodeId":"brick","parameterId":"rowOffset"},
            {"id":"bevel","nodeId":"brick","parameterId":"bevel"}
        ]
    })
}
fn decode(source: &Value) -> MaterialDocument {
    MaterialDocument::decode(
        &serde_json::to_vec(source).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
}
fn request() -> CompileRequest {
    CompileRequest {
        size: [257, 129],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    }
}

#[test]
fn brick_defaults_lower_exact_seed_and_account_one_pass() {
    let source = source();
    let document = decode(&source)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let plan = compile(&document, &request()).unwrap();
    assert_eq!(plan.passes().len(), 1);
    assert_eq!(plan.passes()[0].kernel.uniform_bytes(), 48);
    assert_eq!(plan.passes()[0].kernel.inputs().count(), 0);
    assert_eq!(plan.passes()[0].dispatch, [33, 17, 1]);
    assert!(matches!(
        plan.passes()[0].kernel,
        KernelInvocation::BrickPattern {
            cells: [8, 8],
            seed: u32::MAX,
            row_offset: 0.5,
            mortar: [0.08, 0.08],
            bevel: 0.08,
            variation: 0.15
        }
    ));
    assert_eq!(
        serde_json::to_value(document.document()).unwrap(),
        serde_json::to_value(decode(&source)).unwrap()
    );
}

#[test]
fn brick_periodicity_is_validated_after_atomic_overrides() {
    let document = decode(&source())
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut request = request();
    request.overrides.insert("rows".into(), json!(3));
    let error = compile(&document, &request).unwrap_err();
    assert!(
        error
            .report()
            .diagnostics()
            .iter()
            .any(|d| d.code.as_str() == "MIX_PARAMETER_INVALID_VALUE"
                && d.parameter_id.as_deref() == Some("rows"))
    );
    request.overrides.insert("offset".into(), json!(0));
    let legal = compile(&document, &request).unwrap();
    assert!(matches!(
        legal.passes()[0].kernel,
        KernelInvocation::BrickPattern {
            cells: [8, 3],
            row_offset: 0.0,
            ..
        }
    ));
    let original = compile(&document, &self::request()).unwrap();
    assert_ne!(legal.hash(), original.hash());
}

#[test]
fn brick_invalid_parameters_and_missing_seed_fail_even_when_unused() {
    for (name, value) in [
        ("seed", json!(-1)),
        ("seed", json!(4294967296_u64)),
        ("seed", json!(8.0)),
        ("columns", json!(0)),
        ("rows", json!(65)),
        ("rows", json!(3)),
        ("rowOffset", json!(1.01)),
        ("mortarX", json!(0.46)),
        ("mortarY", json!(-0.01)),
        ("bevel", json!(0.26)),
        ("variation", json!(1.01)),
    ] {
        let mut input = source();
        input["nodes"][0]["parameters"][name] = value;
        assert!(
            decode(&input)
                .into_validated(&SafetyLimits::default())
                .is_err(),
            "{name}"
        );
    }
    let mut input = source();
    input["nodes"][0]["parameters"]
        .as_object_mut()
        .unwrap()
        .remove("seed");
    input["edges"].as_array_mut().unwrap().remove(0);
    assert!(
        decode(&input)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
    input["nodes"][0]["parameters"]["seed"] = json!(0);
    input["nodes"][0]["version"] = json!(2);
    assert!(
        decode(&input)
            .into_validated(&SafetyLimits::default())
            .is_err()
    );
}

#[test]
fn brick_hash_and_slicing_follow_effective_parameters() {
    let document = decode(&source())
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut req = request();
    let initial = compile(&document, &req).unwrap();
    req.overrides.insert("seed".into(), json!(u32::MAX));
    assert_eq!(initial.hash(), compile(&document, &req).unwrap().hash());
    req.overrides.insert("seed".into(), json!(17));
    assert_ne!(initial.hash(), compile(&document, &req).unwrap().hash());
    req.outputs = vec![OutputChannel::BaseColor];
    let sliced = compile(&document, &req).unwrap();
    req.overrides.insert("bevel".into(), json!(0.25));
    assert_eq!(sliced.hash(), compile(&document, &req).unwrap().hash());
    let mut reordered = source();
    reordered["nodes"].as_array_mut().unwrap().reverse();
    reordered["edges"].as_array_mut().unwrap().reverse();
    let reversed = decode(&reordered)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    assert_eq!(
        initial.hash(),
        compile(&reversed, &request()).unwrap().hash()
    );
}
