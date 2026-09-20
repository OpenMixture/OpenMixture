//! ENG-04 public contract, validation and deterministic lowering.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile,
    plan::{KernelId, KernelInvocation},
};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/scalar-blend/input.mix"
    ))
    .unwrap()
}
fn plan(value: &Value, request: &CompileRequest) -> mixture_core::RenderPlan {
    let doc = MaterialDocument::decode(&serde_json::to_vec(value).unwrap(), &request.limits)
        .unwrap()
        .into_validated(&request.limits)
        .unwrap();
    compile(&doc, request).unwrap()
}
#[test]
fn scalar_contract_validation_and_endpoints() {
    let request = CompileRequest {
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    for weight in [0., 0.5, 1.] {
        let mut v = source();
        v["nodes"][3]["parameters"] = json!({"weight":weight});
        let p = plan(&v, &request);
        assert!(p.passes().iter().any(
            |p| matches!(p.kernel,KernelInvocation::ScalarBlend {weight:w,..} if w==weight as f32)
        ));
        assert_eq!(p.passes().len(), 3);
    }
    let mutations: Vec<(Value, &str)> = vec![
        (json!(-0.1), "MIX_PARAMETER_INVALID_VALUE"),
        (json!(1.1), "MIX_PARAMETER_INVALID_VALUE"),
        (json!("NaN"), "MIX_PARAMETER_INVALID_VALUE"),
        (json!(null), "MIX_PARAMETER_INVALID_VALUE"),
    ];
    for (weight, code) in mutations {
        let mut v = source();
        v["nodes"][3]["parameters"] = json!({"weight":weight});
        invalid(&v, code);
    }
    let mut v = source();
    v["nodes"][3]["version"] = json!(2);
    invalid(&v, "MIX_NODE_UNSUPPORTED_VERSION");
    let mut v = source();
    v["nodes"][3]["parameters"] = json!({"mask":0});
    invalid(&v, "MIX_PARAMETER_UNKNOWN");
    for index in [0, 1] {
        let mut v = source();
        v["edges"].as_array_mut().unwrap().remove(index);
        invalid(&v, "MIX_PORT_REQUIRED_CONNECTION");
    }
    let mut v = source();
    v["edges"][0]["from"] = json!({"nodeId":"color","portId":"color"});
    invalid(&v, "MIX_PORT_TYPE_MISMATCH");
    let text = serde_json::to_string(&source())
        .unwrap()
        .replace("0.25", "1e999");
    assert!(MaterialDocument::decode(text.as_bytes(), &SafetyLimits::default()).is_err());
}
fn invalid(value: &Value, code: &str) {
    let d = MaterialDocument::decode(
        &serde_json::to_vec(value).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap();
    let r = d.validate(&SafetyLimits::default());
    assert!(
        r.diagnostics().iter().any(|d| d.code.as_str() == code),
        "{r:?}"
    );
}
#[test]
fn scalar_plan_determinism_slicing_and_unused_override_validation() {
    let v = source();
    let mut req = CompileRequest {
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let a = plan(&v, &req);
    let mut reordered = v.clone();
    reordered["nodes"].as_array_mut().unwrap().reverse();
    reordered["edges"].as_array_mut().unwrap().reverse();
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&plan(&reordered, &req)).unwrap()
    );
    let mut explicit = v.clone();
    explicit["nodes"][3]["parameters"] = json!({"weight":0.5});
    assert_eq!(a.hash(), plan(&explicit, &req).hash());
    req.overrides.insert("weight".into(), json!(0.25));
    assert_ne!(a.hash(), plan(&v, &req).hash());
    req.outputs = vec![OutputChannel::BaseColor];
    assert!(
        plan(&v, &req)
            .passes()
            .iter()
            .all(|p| p.kernel.id() != KernelId::ScalarBlend)
    );
    req.overrides.insert("weight".into(), json!(2));
    let d = MaterialDocument::decode(&serde_json::to_vec(&v).unwrap(), &req.limits)
        .unwrap()
        .into_validated(&req.limits)
        .unwrap();
    assert!(compile(&d, &req).is_err());
    for w in [0., 1.] {
        let mut v = source();
        v["nodes"][3]["parameters"] = json!({"weight":w});
        v["nodes"][1]["parameters"] = json!({"value":2});
        invalid(&v, "MIX_PARAMETER_INVALID_VALUE");
    }
}
