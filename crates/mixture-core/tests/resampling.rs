//! Typed contracts/lowering for periodic scalar resampling; pixels belong to WGSL.
use mixture_core::{
    CompileRequest, DiagnosticCode, MaterialDocument, OutputChannel, SafetyLimits, compile,
    plan::{KernelId, KernelInvocation},
};
use serde_json::{Value, json};

fn source(name: &str) -> Value {
    serde_json::from_slice(match name {
        "transform-2d" => {
            include_bytes!("../../../fixtures/nodes/transform-2d/input.mix").as_slice()
        }
        "warp" => include_bytes!("../../../fixtures/nodes/warp/input.mix").as_slice(),
        _ => panic!("unknown resampling fixture"),
    })
    .unwrap()
}
fn decode(value: &Value) -> MaterialDocument {
    MaterialDocument::decode(
        &serde_json::to_vec(value).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
}
fn request() -> CompileRequest {
    CompileRequest {
        size: [129, 65],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    }
}

#[test]
fn resampling_defaults_lower_to_typed_parameters_and_ordered_input_bindings() {
    for name in ["transform-2d", "warp"] {
        let document = decode(&source(name))
            .into_validated(&SafetyLimits::default())
            .unwrap();
        let plan = compile(&document, &request()).unwrap();
        let kernel = &plan.passes().last().unwrap().kernel;
        match kernel {
            KernelInvocation::Transform2d {
                input,
                scale,
                quarter_turns,
                offset,
            } => {
                assert_eq!(*scale, [1, 1]);
                assert_eq!(*quarter_turns, 0);
                assert_eq!(*offset, [0., 0.]);
                assert_eq!(*input, plan.passes()[0].output);
                assert_eq!(kernel.inputs().collect::<Vec<_>>(), [*input]);
                assert_eq!(kernel.uniform_bytes(), 32);
            }
            KernelInvocation::Warp {
                input,
                displacement,
                strength,
            } => {
                assert_eq!(*strength, [0.05, 0.]);
                let noise = plan
                    .passes()
                    .iter()
                    .find(|p| p.kernel.id() == KernelId::FractalNoise)
                    .unwrap();
                let field = plan
                    .passes()
                    .iter()
                    .find(|p| p.kernel.id() == KernelId::Constant)
                    .unwrap();
                assert_eq!(*input, noise.output);
                assert_eq!(*displacement, field.output);
                assert_eq!(kernel.inputs().collect::<Vec<_>>(), [*input, *displacement]);
                assert_eq!(kernel.uniform_bytes(), 16);
            }
            _ => panic!("fixture must end with the resampling pass"),
        }
        assert_eq!(plan.estimates().uniform_bytes, 64);
        let color = compile(
            &document,
            &CompileRequest {
                outputs: vec![OutputChannel::BaseColor],
                ..request()
            },
        )
        .unwrap();
        assert_eq!(
            color.passes().len(),
            1,
            "unrequested resampling branch is sliced"
        );
        assert_eq!(color.passes()[0].kernel.id(), KernelId::Constant);
    }
}

#[test]
fn resampling_parameters_change_hashes_and_source_order_does_not() {
    for (name, changes) in [
        (
            "transform-2d",
            vec![
                ("scaleX", json!(64)),
                ("scaleY", json!(3)),
                ("quarterTurns", json!(1)),
                ("offsetX", json!(-1)),
                ("offsetY", json!(0.25)),
            ],
        ),
        (
            "warp",
            vec![("strengthX", json!(-1)), ("strengthY", json!(1))],
        ),
    ] {
        let document = decode(&source(name))
            .into_validated(&SafetyLimits::default())
            .unwrap();
        let original = compile(&document, &request()).unwrap();
        for (parameter, value) in changes {
            let mut changed = request();
            changed.overrides.insert(parameter.into(), value);
            assert_ne!(
                compile(&document, &changed).unwrap().hash(),
                original.hash(),
                "{name}/{parameter}"
            );
        }
        let mut reversed = source(name);
        reversed["nodes"].as_array_mut().unwrap().reverse();
        reversed["edges"].as_array_mut().unwrap().reverse();
        reversed["exposedParameters"]
            .as_array_mut()
            .unwrap()
            .reverse();
        let reversed = decode(&reversed)
            .into_validated(&SafetyLimits::default())
            .unwrap();
        assert_eq!(
            compile(&reversed, &request()).unwrap().hash(),
            original.hash()
        );
    }
}

#[test]
fn resampling_requires_exact_scalar_connections_and_preserves_repeated_warp_bindings() {
    for (name, port) in [
        ("transform-2d", "in"),
        ("warp", "in"),
        ("warp", "displacement"),
    ] {
        let mut missing = source(name);
        missing["edges"]
            .as_array_mut()
            .unwrap()
            .retain(|edge| edge["to"] != json!({"nodeId":"sample","portId":port}));
        let report = decode(&missing).validate(&SafetyLimits::default());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.code == DiagnosticCode::PortRequiredConnection
                    && d.node_id.as_deref() == Some("sample")
                    && d.port_id.as_deref() == Some(port))
        );
        let mut mismatch = source(name);
        let edge = mismatch["edges"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|edge| edge["to"] == json!({"nodeId":"sample","portId":port}))
            .unwrap();
        edge["from"] = json!({"nodeId":"color","portId":"color"});
        let report = decode(&mismatch).validate(&SafetyLimits::default());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.code == DiagnosticCode::PortTypeMismatch)
        );
    }
    let mut repeated = source("warp");
    let edge = repeated["edges"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|edge| edge["to"]["portId"] == "displacement")
        .unwrap();
    edge["from"] = json!({"nodeId":"source","portId":"value"});
    let document = decode(&repeated)
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let plan = compile(&document, &request()).unwrap();
    assert_eq!(plan.passes().len(), 2);
    assert_eq!(
        plan.passes()[1].kernel.inputs().collect::<Vec<_>>(),
        [plan.passes()[0].output; 2]
    );
}
