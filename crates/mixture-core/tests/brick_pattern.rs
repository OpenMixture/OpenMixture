//! Brick use-case admission, defaults, compatibility and deterministic lowering.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, compile, plan::KernelInvocation,
};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/brick-pattern/input.mix"
    ))
    .unwrap()
}
fn plan(v: &Value, r: &CompileRequest) -> Result<mixture_core::RenderPlan, String> {
    let d = MaterialDocument::decode(&serde_json::to_vec(v).unwrap(), &r.limits)
        .map_err(|e| e.to_string())?
        .into_validated(&r.limits)
        .map_err(|e| e.to_string())?;
    compile(&d, r).map_err(|e| e.to_string())
}
#[test]
fn defaults_explicit_values_order_and_hash_agree() {
    let req = CompileRequest {
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let v = source();
    let p = plan(&v, &req).unwrap();
    assert_eq!(p.passes().len(), 1);
    assert!(
        matches!(p.passes()[0].kernel,KernelInvocation::BrickPattern{cells:[4,8],half_offset:true,gap,bevel} if gap==0.08 && bevel==0.08)
    );
    assert_eq!(p.passes()[0].kernel.uniform_bytes(), 32);
    assert_eq!(p.passes()[0].kernel.inputs().count(), 0);
    let mut explicit = v.clone();
    explicit["nodes"][0]["parameters"] =
        json!({"columns":4,"rows":8,"layout":"half-offset","gap":0.08,"bevel":0.08});
    explicit["nodes"].as_array_mut().unwrap().reverse();
    explicit["edges"].as_array_mut().unwrap().reverse();
    assert_eq!(p.hash(), plan(&explicit, &req).unwrap().hash());
    for (key, value) in [
        ("columns", json!(5)),
        ("rows", json!(6)),
        ("layout", json!("aligned")),
        ("gap", json!(0.1)),
        ("bevel", json!(0.1)),
    ] {
        let mut changed = req.clone();
        changed.overrides.insert(key.into(), value);
        assert_ne!(p.hash(), plan(&v, &changed).unwrap().hash());
    }
}
#[test]
fn reject_odd_staggered_rows_in_source_and_overrides_even_when_unused() {
    let mut req = CompileRequest {
        outputs: vec![OutputChannel::Metallic],
        ..Default::default()
    };
    let v = source();
    req.overrides.insert("rows".into(), json!(3));
    assert!(
        plan(&v, &req)
            .unwrap_err()
            .contains("MIX_PARAMETER_INVALID_VALUE")
    );
    req.overrides.insert("layout".into(), json!("aligned"));
    assert!(plan(&v, &req).is_ok());
    let mut invalid = v.clone();
    invalid["nodes"][0]["parameters"] = json!({"rows":3});
    assert!(plan(&invalid, &CompileRequest::default()).is_err());
    invalid["nodes"][0]["parameters"] = json!({});
    invalid["nodes"][0]["version"] = json!(2);
    assert!(
        plan(&invalid, &CompileRequest::default())
            .unwrap_err()
            .contains("MIX_NODE_UNSUPPORTED_VERSION")
    );
}
