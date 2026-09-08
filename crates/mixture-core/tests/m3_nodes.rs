//! Public API checks for the new vertical slice, with no GPU or pixel implementation.
use mixture_core::{
    CompileRequest, DiagnosticCode, MaterialDocument, OutputChannel, SafetyLimits, compile,
    plan::{KernelId, KernelInvocation, NoiseBasis},
};
use serde_json::{Value, json};

fn source() -> Value {
    json!({
        "version":1,
        "nodes":[
            {"id":"grain","type":"fractal-noise","version":1,"parameters":{"seed":4294967295_u32}},
            {"id":"color","type":"gradient-map","version":1},
            {"id":"normal","type":"height-to-normal","version":1},
            {"id":"out","type":"material-output","version":1}
        ],
        "edges":[
            {"from":{"nodeId":"grain","portId":"value"},"to":{"nodeId":"color","portId":"in"}},
            {"from":{"nodeId":"grain","portId":"value"},"to":{"nodeId":"normal","portId":"in"}},
            {"from":{"nodeId":"grain","portId":"value"},"to":{"nodeId":"out","portId":"height"}},
            {"from":{"nodeId":"color","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},
            {"from":{"nodeId":"normal","portId":"normal"},"to":{"nodeId":"out","portId":"normal"}}
        ],
        "exposedParameters":[
            {"id":"seed","nodeId":"grain","parameterId":"seed"},
            {"id":"scale","nodeId":"grain","parameterId":"scale"},
            {"id":"basis","nodeId":"grain","parameterId":"basis"},
            {"id":"strength","nodeId":"normal","parameterId":"strength"}
        ]
    })
}
fn decode(value: &Value) -> MaterialDocument {
    MaterialDocument::decode(
        &serde_json::to_vec(value).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
}

#[test]
fn m3_seed_is_required_even_on_an_unused_branch_and_never_coerced_or_defaulted() {
    let mut value = source();
    value["nodes"][0]["parameters"] = json!({});
    let report = decode(&value).validate(&SafetyLimits::default());
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.parameter_id.as_deref() == Some("seed"))
        .unwrap();
    assert_eq!(diagnostic.code, DiagnosticCode::ParameterInvalidValue);
    assert_eq!(diagnostic.node_id.as_deref(), Some("grain"));
    assert_eq!(diagnostic.evidence["present"], false.into());
    for seed in [
        json!(-1),
        json!(1.0),
        json!(4294967296_u64),
        json!("1"),
        json!(null),
    ] {
        value["nodes"][0]["parameters"]["seed"] = seed;
        assert!(!decode(&value).validate(&SafetyLimits::default()).is_ok());
    }
    for seed in [0_u32, u32::MAX] {
        value["nodes"][0]["parameters"]["seed"] = json!(seed);
        assert!(decode(&value).validate(&SafetyLimits::default()).is_ok());
    }
    value["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"unused","type":"fractal-noise","version":1}));
    let report = decode(&value).validate(&SafetyLimits::default());
    assert!(report.diagnostics().iter().any(
        |d| d.node_id.as_deref() == Some("unused") && d.parameter_id.as_deref() == Some("seed")
    ));
}

#[test]
fn m3_typed_lowering_preserves_seed_bits_inputs_and_output_slicing() {
    let document = decode(&source())
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let request = CompileRequest {
        size: [129, 65],
        outputs: vec![
            OutputChannel::BaseColor,
            OutputChannel::Normal,
            OutputChannel::Height,
        ],
        ..Default::default()
    };
    let plan = compile(&document, &request).unwrap();
    assert_eq!(plan.passes().len(), 3);
    assert_eq!(
        plan.passes()
            .iter()
            .map(|p| p.kernel.id())
            .collect::<Vec<_>>(),
        [
            KernelId::FractalNoise,
            KernelId::GradientMap,
            KernelId::HeightToNormal
        ]
    );
    assert!(matches!(
        plan.passes()[0].kernel,
        KernelInvocation::FractalNoise {
            seed: u32::MAX,
            scale: 8,
            octaves: 4,
            persistence: 0.5,
            basis: NoiseBasis::Value
        }
    ));
    for pass in &plan.passes()[1..] {
        assert_eq!(
            pass.kernel.inputs().collect::<Vec<_>>(),
            [plan.passes()[0].output]
        );
    }
    assert_eq!(plan.estimates().uniform_bytes, 80);
    let only_height = compile(
        &document,
        &CompileRequest {
            outputs: vec![OutputChannel::Height],
            ..request.clone()
        },
    )
    .unwrap();
    assert_eq!(only_height.passes().len(), 1);
    let mut changed = request.clone();
    for seed in [16777216_u32, 16777217_u32, u32::MAX - 1, u32::MAX] {
        changed.overrides.insert("seed".into(), json!(seed));
        let changed_plan = compile(&document, &changed).unwrap();
        assert!(
            matches!(changed_plan.passes()[0].kernel, KernelInvocation::FractalNoise { seed:actual,.. } if actual==seed)
        );
        if seed != u32::MAX {
            assert_ne!(changed_plan.hash(), plan.hash());
        }
    }
    for (name, value) in [
        ("basis", json!("cellular")),
        ("scale", json!(16)),
        ("strength", json!(0.0)),
    ] {
        changed = request.clone();
        changed.overrides.insert(name.into(), value);
        assert_ne!(compile(&document, &changed).unwrap().hash(), plan.hash());
    }
    let mut reordered = source();
    reordered["nodes"].as_array_mut().unwrap().reverse();
    reordered["edges"].as_array_mut().unwrap().reverse();
    let reordered = decode(&reordered)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    assert_eq!(compile(&reordered, &request).unwrap().hash(), plan.hash());
}

#[test]
fn m3_new_ports_require_explicit_exact_kind_connections() {
    let mut value = source();
    value["edges"][1]["from"] = json!({"nodeId":"color","portId":"color"});
    let report = decode(&value).validate(&SafetyLimits::default());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::PortTypeMismatch)
    );
    value = source();
    value["edges"].as_array_mut().unwrap().remove(0);
    let report = decode(&value).validate(&SafetyLimits::default());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::PortRequiredConnection
                && d.node_id.as_deref() == Some("color"))
    );
}
