//! Frozen MAT-03 node admission, immutable lowering and full-graph validation.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile,
    plan::{KernelInvocation, WeaveMode},
};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/weave-pattern/input.mix"
    ))
    .unwrap()
}
fn decode(v: &Value) -> MaterialDocument {
    MaterialDocument::decode(&serde_json::to_vec(v).unwrap(), &SafetyLimits::default()).unwrap()
}
fn request() -> CompileRequest {
    CompileRequest {
        size: [257, 129],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    }
}
#[test]
fn weave_defaults_abi_hash_and_slicing() {
    let original = source();
    let doc = decode(&original)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let mut req = request();
    let initial = compile(&doc, &req).unwrap();
    assert_eq!(initial.passes().len(), 1);
    let pass = &initial.passes()[0];
    assert_eq!(pass.dispatch, [33, 17, 1]);
    assert_eq!(pass.kernel.inputs().count(), 0);
    assert_eq!(pass.kernel.uniform_bytes(), 48);
    assert!(matches!(
        pass.kernel,
        KernelInvocation::WeavePattern {
            counts: [8, 8],
            widths: [0.7, 0.7],
            bevel: 0.08,
            crown: 0.5,
            under_ratio: 0.5,
            mode: WeaveMode::Height
        }
    ));
    req.overrides=serde_json::from_value(json!({"mode":"height","warpCount":8,"weftCount":8,"warpWidth":0.7,"weftWidth":0.7,"bevel":0.08,"crown":0.5,"underRatio":0.5})).unwrap();
    assert_eq!(initial.hash(), compile(&doc, &req).unwrap().hash());
    for changes in [
        json!({"mode":"coverage"}),
        json!({"mode":"warp-share"}),
        json!({"warpCount":12}),
        json!({"weftCount":4}),
        json!({"warpWidth":0.55}),
        json!({"weftWidth":0.9}),
        json!({"bevel":0.02}),
        json!({"crown":1}),
        json!({"underRatio":0.75}),
    ] {
        req.overrides = serde_json::from_value(changes).unwrap();
        let changed = compile(&doc, &req).unwrap();
        assert_ne!(initial.hash(), changed.hash());
        assert_eq!(changed.hash(), compile(&doc, &req).unwrap().hash());
    }
    req.outputs = vec![OutputChannel::BaseColor];
    req.overrides.clear();
    let sliced = compile(&doc, &req).unwrap();
    req.overrides.insert("crown".into(), json!(0));
    assert_eq!(sliced.hash(), compile(&doc, &req).unwrap().hash());
    let mut reverse = original.clone();
    reverse["nodes"].as_array_mut().unwrap().reverse();
    reverse["edges"].as_array_mut().unwrap().reverse();
    assert_eq!(
        initial.hash(),
        compile(
            &decode(&reverse)
                .into_validated(&SafetyLimits::default())
                .unwrap(),
            &request()
        )
        .unwrap()
        .hash()
    );
    assert_eq!(
        serde_json::to_value(doc.document()).unwrap(),
        serde_json::to_value(decode(&original)).unwrap()
    );
}
#[test]
fn weave_rejects_invalid_parameters_in_source_unused_nodes_and_overrides() {
    let valid = decode(&source())
        .into_validated(&SafetyLimits::default())
        .unwrap();
    for (key, value) in [
        ("warpCount", json!(5)),
        ("weftCount", json!(31)),
        ("warpCount", json!(3)),
        ("weftCount", json!(33)),
        ("warpCount", json!(8.0)),
        ("weftCount", json!("8")),
        ("warpWidth", json!(0.54)),
        ("weftWidth", json!(0.91)),
        ("bevel", json!(0.019)),
        ("bevel", json!(0.121)),
        ("crown", json!(-0.01)),
        ("crown", json!(1.01)),
        ("underRatio", json!(0.24)),
        ("underRatio", json!(0.76)),
        ("mode", json!("Height")),
        ("mode", json!(null)),
        ("warpCount", json!(true)),
        ("seed", json!(0)),
    ] {
        let code = if key == "seed" {
            "MIX_PARAMETER_UNKNOWN"
        } else {
            "MIX_PARAMETER_INVALID_VALUE"
        };
        for unused in [false, true] {
            let mut s = source();
            s["nodes"][0]["parameters"] = json!({key:value});
            if unused {
                s["edges"].as_array_mut().unwrap().remove(0);
            }
            let err = decode(&s)
                .into_validated(&SafetyLimits::default())
                .unwrap_err();
            assert!(
                err.report()
                    .diagnostics()
                    .iter()
                    .any(|d| d.code.as_str() == code
                        && d.node_id.as_deref() == Some("weave")
                        && d.parameter_id.as_deref() == Some(key)),
                "{key}: {err:?}"
            );
        }
        if key != "seed" {
            let mut r = request();
            r.outputs = vec![OutputChannel::BaseColor];
            r.overrides.insert(key.into(), value);
            let err = compile(&valid, &r).unwrap_err();
            assert!(
                err.report()
                    .diagnostics()
                    .iter()
                    .any(|d| d.code.as_str() == code
                        && d.node_id.as_deref() == Some("weave")
                        && d.parameter_id.as_deref() == Some(key))
            );
        }
    }
    let mut s = source();
    s["nodes"][0]["version"] = json!(2);
    assert!(
        decode(&s)
            .into_validated(&SafetyLimits::default())
            .unwrap_err()
            .report()
            .diagnostics()
            .iter()
            .any(|d| d.code.as_str() == "MIX_NODE_UNSUPPORTED_VERSION")
    );
}
#[test]
fn weave_inclusive_boundaries_and_modes() {
    for mode in ["height", "coverage", "warp-share"] {
        for (n, w, b, c, r) in [
            (4, 0.55, 0.02, 0., 0.25),
            (32, 0.9, 0.12, 1., 0.75),
            (12, 0.7, 0.08, 0.5, 0.5),
        ] {
            let mut s = source();
            s["nodes"][0]["parameters"] = json!({"mode":mode,"warpCount":n,"weftCount":8,"warpWidth":w,"weftWidth":w,"bevel":b,"crown":c,"underRatio":r});
            let d = decode(&s).into_validated(&SafetyLimits::default()).unwrap();
            assert_eq!(compile(&d, &request()).unwrap().passes().len(), 1);
        }
    }
}
