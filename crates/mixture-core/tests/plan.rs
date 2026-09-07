//! Public-consumer regressions for deterministic compilation, slicing, and budgets.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel as Channel, RenderPlan, SafetyLimits, Stage,
    ValidatedDocument, compile, normalize,
    plan::{KernelId, KernelInvocation, PassOrigin, TextureFormat},
    registry::PortKind,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CHECKER: &[u8] = include_bytes!("../../../examples/checker.mix");
const ALL: &[u8] = include_bytes!("../../../fixtures/format/valid/all-m2.mix");
fn document(bytes: &[u8]) -> ValidatedDocument {
    let limits = SafetyLimits::default();
    MaterialDocument::decode(bytes, &limits)
        .unwrap()
        .into_validated(&limits)
        .unwrap()
}
fn value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}
fn edited(value: &Value) -> ValidatedDocument {
    document(&serde_json::to_vec(value).unwrap())
}
fn request(outputs: &[Channel]) -> CompileRequest {
    CompileRequest {
        outputs: outputs.to_vec(),
        ..Default::default()
    }
}
fn bytes(plan: &RenderPlan) -> Vec<u8> {
    serde_json::to_vec(plan).unwrap()
}
fn nodes(plan: &RenderPlan) -> Vec<&str> {
    plan.passes()
        .iter()
        .filter_map(|p| match &p.origin {
            PassOrigin::Node { node } => Some(node.id.as_str()),
            _ => None,
        })
        .collect()
}
fn assert_failure(doc: &ValidatedDocument, request: &CompileRequest, code: &str) {
    let error = compile(doc, request).unwrap_err();
    assert!(
        error
            .report()
            .diagnostics()
            .iter()
            .all(|d| d.stage == Stage::Compile)
    );
    assert!(
        error
            .report()
            .diagnostics()
            .iter()
            .any(|d| d.code.as_str() == code),
        "{error:?}"
    );
}
#[test]
fn reviewed_plan_and_hash_snapshots() {
    for (source, outputs, expected) in [
        (
            CHECKER,
            vec![Channel::BaseColor],
            include_str!("snapshots/plan-checker.json"),
        ),
        (
            CHECKER,
            Channel::ALL.to_vec(),
            include_str!("snapshots/plan-defaults.json"),
        ),
        (
            ALL,
            vec![Channel::BaseColor, Channel::Roughness],
            include_str!("snapshots/plan-all-m2.json"),
        ),
        (
            ALL,
            vec![Channel::Roughness],
            include_str!("snapshots/plan-sliced.json"),
        ),
    ] {
        let plan = compile(&document(source), &request(&outputs)).unwrap();
        assert_eq!(
            format!("{}\n", serde_json::to_string_pretty(&plan).unwrap()),
            expected
        );
        let hash_input = plan.hash_input().unwrap();
        assert!(hash_input.starts_with(b"mixture-render-plan-v1\0"));
        let digest = Sha256::digest(hash_input);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(plan.hash().as_str(), format!("sha256:{hex}"));
    }
}
#[test]
fn stable_kahn_order_and_typed_resources_are_producer_first() {
    let plan = compile(&document(ALL), &request(&Channel::ALL)).unwrap();
    assert_eq!(nodes(&plan), ["checker", "mask", "levels", "tint", "blend"]);
    for (index, pass) in plan.passes().iter().enumerate() {
        assert_eq!(pass.id.index() as usize, index);
        assert_eq!(pass.output.index() as usize, index);
        assert_eq!(pass.output_desc.format, TextureFormat::Rgba16Float);
        for input in pass.kernel.inputs() {
            assert!(input.index() < pass.output.index());
        }
        match &pass.kernel {
            KernelInvocation::Levels { input, .. } => assert_eq!(
                plan.passes()[input.index() as usize].output_desc.kind,
                PortKind::Scalar
            ),
            KernelInvocation::Blend { a, b, mask, .. } => {
                for color in [a, b] {
                    assert_eq!(
                        plan.passes()[color.index() as usize].output_desc.kind,
                        PortKind::Color
                    );
                }
                assert_eq!(
                    plan.passes()[mask.index() as usize].output_desc.kind,
                    PortKind::Scalar
                );
            }
            _ => {}
        }
    }
    assert_eq!(
        plan.outputs().iter().map(|o| o.channel).collect::<Vec<_>>(),
        Channel::ALL
    );
}
#[test]
fn unrequested_branches_and_their_valid_overrides_do_not_enter_plan_or_hash() {
    let doc = document(ALL);
    let mut req = request(&[Channel::Roughness]);
    let baseline = compile(&doc, &req).unwrap();
    assert_eq!(nodes(&baseline), ["mask"]);
    req.overrides.insert("frequency".into(), json!(32));
    req.overrides.insert("contrast".into(), json!(3));
    assert_eq!(bytes(&baseline), bytes(&compile(&doc, &req).unwrap()));
    let mut source = value(ALL);
    source["nodes"][0]["parameters"]["cellsY"] = json!(123);
    assert_eq!(
        bytes(&baseline),
        bytes(&compile(&edited(&source), &req).unwrap())
    );
    req.overrides.insert("frequency".into(), json!(0));
    assert_failure(&doc, &req, "MIX_PARAMETER_INVALID_VALUE");
}
#[test]
fn channel_aliases_share_a_resource_and_charge_sequential_readback() {
    let mut source = value(ALL);
    source["edges"].as_array_mut().unwrap().push(
        json!({"from":{"nodeId":"mask","portId":"value"},"to":{"nodeId":"out","portId":"height"}}),
    );
    let plan = compile(
        &edited(&source),
        &request(&[Channel::Height, Channel::Roughness]),
    )
    .unwrap();
    assert_eq!(plan.passes().len(), 1);
    assert_eq!(plan.outputs()[0].resource, plan.outputs()[1].resource);
    assert_eq!(
        plan.estimates().cumulative_bytes - plan.estimates().peak_bytes,
        plan.estimates().readback_buffer_bytes
    );
}
#[test]
fn repeated_producer_bindings_and_missing_mask_lower_without_duplicate_node_passes() {
    let mut source = value(ALL);
    let edges = source["edges"].as_array_mut().unwrap();
    edges.retain(|edge| !(edge["to"]["nodeId"] == "blend" && edge["to"]["portId"] == "mask"));
    edges
        .iter_mut()
        .find(|e| e["to"]["nodeId"] == "blend" && e["to"]["portId"] == "b")
        .unwrap()["from"]["nodeId"] = json!("checker");
    let plan = compile(&edited(&source), &CompileRequest::default()).unwrap();
    assert_eq!(nodes(&plan), ["checker", "blend"]);
    assert_eq!(plan.passes().len(), 3);
    assert!(
        matches!(&plan.passes()[1].origin, PassOrigin::InputDefault {node,port} if node.id == "blend" && port == "mask")
    );
    assert!(
        matches!(plan.passes()[1].kernel, KernelInvocation::Constant {value} if value == [1.,0.,0.,1.])
    );
    assert!(
        matches!(plan.passes()[2].kernel, KernelInvocation::Blend {a,b,mask,..} if a == b && mask.index() == 1)
    );
}
#[test]
fn requested_defaults_are_real_typed_resources_without_a_sink_pass() {
    let plan = compile(&document(CHECKER), &request(&Channel::ALL)).unwrap();
    assert_eq!(plan.passes().len(), 8);
    assert_eq!(nodes(&plan), ["checker"]);
    assert!(
        matches!(plan.passes()[1].kernel, KernelInvocation::Constant {value} if value == [0.5,0.5,1.,1.])
    );
    assert_eq!(plan.passes()[1].output_desc.kind, PortKind::Normal);
    for output in &plan.outputs()[1..] {
        assert!(matches!(
            output.input,
            mixture_core::InputSource::Default { .. }
        ));
    }
    let normal = compile(&document(CHECKER), &request(&[Channel::Normal])).unwrap();
    assert!(nodes(&normal).is_empty());
    assert_eq!(normal.passes().len(), 1);
}
#[test]
fn normalization_is_immutable_explicit_and_order_independent() {
    let doc = document(ALL);
    let before = doc.document().to_json().unwrap();
    let mut req = request(&[Channel::Roughness, Channel::BaseColor]);
    req.overrides.insert("frequency".into(), json!(23));
    let normalized = normalize(&doc, &req).unwrap();
    let checker = normalized
        .document()
        .nodes
        .iter()
        .find(|n| n.id == "checker")
        .unwrap();
    assert_eq!(checker.parameters["cellsX"], 23);
    assert_eq!(checker.parameters["colorA"], json!([0., 0., 0., 1.]));
    assert_eq!(doc.document().to_json().unwrap(), before);
    let expected = bytes(&compile(&doc, &req).unwrap());
    let mut source = value(ALL);
    for field in ["nodes", "edges", "exposedParameters"] {
        source[field].as_array_mut().unwrap().reverse();
    }
    req.outputs.reverse();
    assert_eq!(expected, bytes(&compile(&edited(&source), &req).unwrap()));
}
#[test]
fn explicit_defaults_numeric_spelling_signed_zero_and_noop_overrides_are_equivalent() {
    let doc = document(CHECKER);
    let req = CompileRequest::default();
    let expected = bytes(&compile(&doc, &req).unwrap());
    let normalized = normalize(&doc, &req).unwrap();
    assert_eq!(
        expected,
        bytes(&compile(&document(&normalized.document().to_json().unwrap()), &req).unwrap())
    );
    let mut source = value(CHECKER);
    source["nodes"][0]["parameters"] = json!({"colorA":[-0.0,0,0.0,1],"colorB":[1,1.0,1,1]});
    assert_eq!(expected, bytes(&compile(&edited(&source), &req).unwrap()));
    let mut req = req;
    req.overrides.insert("frequency".into(), json!(8));
    req.limits.transient_bytes += 1;
    assert_eq!(expected, bytes(&compile(&doc, &req).unwrap()));
}
#[test]
fn effective_overrides_match_explicit_source_and_semantic_changes_change_hash() {
    let doc = document(CHECKER);
    let baseline = compile(&doc, &CompileRequest::default()).unwrap();
    let mut req = CompileRequest::default();
    req.overrides.insert("frequency".into(), json!(16));
    let changed = compile(&doc, &req).unwrap();
    assert_ne!(baseline.hash(), changed.hash());
    let mut source = value(CHECKER);
    source["nodes"][0]["parameters"] = json!({"cellsX":16});
    assert_eq!(
        bytes(&changed),
        bytes(&compile(&edited(&source), &CompileRequest::default()).unwrap())
    );
    for req in [
        CompileRequest {
            size: [65, 64],
            ..Default::default()
        },
        request(&[Channel::BaseColor, Channel::Opacity]),
    ] {
        assert_ne!(baseline.hash(), compile(&doc, &req).unwrap().hash());
    }
    source["nodes"][0]["parameters"] = json!({"colorA":[0.25,0,0,1]});
    assert_ne!(
        baseline.hash(),
        compile(&edited(&source), &CompileRequest::default())
            .unwrap()
            .hash()
    );
}
#[test]
fn odd_rectangular_sizes_have_checked_aligned_memory_and_dispatch() {
    let mut req = CompileRequest {
        size: [65, 3],
        ..Default::default()
    };
    let doc = document(CHECKER);
    let plan = compile(&doc, &req).unwrap();
    assert_eq!(plan.passes()[0].dispatch, [9, 1, 1]);
    assert_eq!(plan.estimates().texture_bytes, 1560);
    assert_eq!(plan.estimates().uniform_bytes, 48);
    assert_eq!(plan.estimates().padded_bytes_per_row, 768);
    assert_eq!(plan.estimates().readback_buffer_bytes, 2304);
    assert_eq!(plan.estimates().peak_bytes, 3912);
    req.outputs.push(Channel::Roughness);
    let plan = compile(&doc, &req).unwrap();
    assert_eq!(plan.estimates().peak_bytes, 5488);
    assert_eq!(plan.estimates().cumulative_bytes, 7792);
    req.limits.transient_bytes = 5488;
    assert!(compile(&doc, &req).is_ok());
    req.limits.transient_bytes -= 1;
    assert_failure(&doc, &req, "MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED");
}
#[test]
fn invalid_request_shapes_and_stricter_policies_fail_at_compile_stage() {
    let doc = document(CHECKER);
    for size in [[0, 64], [64, 0]] {
        assert_failure(
            &doc,
            &CompileRequest {
                size,
                ..Default::default()
            },
            "MIX_COMPILE_INVALID_REQUEST",
        );
    }
    assert_failure(
        &doc,
        &CompileRequest {
            size: [2049, 64],
            ..Default::default()
        },
        "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED",
    );
    for outputs in [vec![], vec![Channel::BaseColor, Channel::BaseColor]] {
        assert_failure(&doc, &request(&outputs), "MIX_COMPILE_INVALID_REQUEST");
    }
    let mut req = CompileRequest::default();
    req.limits.requested_outputs = 0;
    assert_failure(&doc, &req, "MIX_LIMIT_REQUESTED_OUTPUTS_EXCEEDED");
    req = CompileRequest::default();
    req.limits.nodes = 1;
    assert_failure(&doc, &req, "MIX_LIMIT_NODES_EXCEEDED");
    assert!("BaseColor".parse::<Channel>().is_err());
}
#[test]
fn overrides_require_exposed_ids_and_exact_parameter_types() {
    let doc = document(CHECKER);
    for (id, value, code) in [
        ("cellsX", json!(16), "MIX_COMPILE_INVALID_REQUEST"),
        ("frequency", json!(8.0), "MIX_PARAMETER_INVALID_VALUE"),
        ("frequency", json!(0), "MIX_PARAMETER_INVALID_VALUE"),
        ("frequency", json!(1025), "MIX_PARAMETER_INVALID_VALUE"),
        ("frequency", json!("16"), "MIX_PARAMETER_INVALID_VALUE"),
        ("frequency", Value::Null, "MIX_PARAMETER_INVALID_VALUE"),
    ] {
        let mut req = CompileRequest::default();
        req.overrides.insert(id.into(), value);
        assert_failure(&doc, &req, code);
    }
}
#[test]
fn enum_color_and_cross_parameter_overrides_are_validated_after_atomic_application() {
    let mut source = value(ALL);
    source["exposedParameters"].as_array_mut().unwrap().extend([
        json!({"id":"minimum","nodeId":"levels","parameterId":"inputMin"}),
        json!({"id":"maximum","nodeId":"levels","parameterId":"inputMax"}),
        json!({"id":"mode","nodeId":"blend","parameterId":"mode"}),
        json!({"id":"color","nodeId":"tint","parameterId":"value"}),
    ]);
    source["nodes"][3]["parameters"]["inputMax"] = json!(0.4);
    let doc = edited(&source);
    let mut req = CompileRequest::default();
    req.overrides.insert("minimum".into(), json!(0.5));
    assert_failure(&doc, &req, "MIX_PARAMETER_INVALID_VALUE");
    req.overrides.insert("maximum".into(), json!(0.6));
    assert!(compile(&doc, &req).is_ok());
    for (id, v) in [
        ("mode", json!("overlay")),
        ("color", json!([1, 0, 0])),
        ("color", json!([1, 0, 0, 2])),
    ] {
        let mut req = req.clone();
        req.overrides.insert(id.into(), v);
        assert_failure(&doc, &req, "MIX_PARAMETER_INVALID_VALUE");
    }
    req.overrides.insert("mode".into(), json!("screen"));
    req.overrides.insert("color".into(), json!([1, 0, 0, 1]));
    assert!(compile(&doc, &req).is_ok());
}
#[test]
fn f32_collapse_is_rejected_and_equivalent_effective_values_share_hashes() {
    let mut source = value(ALL);
    source["nodes"][3]["parameters"]["inputMin"] = json!(0.5);
    source["nodes"][3]["parameters"]["inputMax"] = json!(0.500000001);
    let doc = edited(&source); // valid f64 source; invalid selected f32 invocation
    assert_failure(
        &doc,
        &CompileRequest::default(),
        "MIX_PARAMETER_INVALID_VALUE",
    );
    assert!(compile(&doc, &request(&[Channel::Roughness])).is_ok());
    let a = compile(&document(ALL), &CompileRequest::default()).unwrap();
    source = value(ALL);
    source["nodes"][2]["parameters"]["value"] = json!(0.500000001);
    assert_eq!(
        a.hash(),
        compile(&edited(&source), &CompileRequest::default())
            .unwrap()
            .hash()
    );
}
#[test]
fn extreme_allocation_and_coordinate_arithmetic_returns_errors_without_panicking() {
    let doc = document(CHECKER);
    let mut req = request(&[Channel::Normal]);
    req.size = [u32::MAX, u32::MAX];
    req.limits.output_dimension = u64::MAX;
    req.limits.transient_bytes = u64::MAX;
    assert_failure(&doc, &req, "MIX_COMPILE_INVALID_REQUEST");
    req.outputs = vec![Channel::BaseColor];
    assert_failure(&doc, &req, "MIX_COMPILE_INVALID_REQUEST");
}
#[test]
fn transient_budget_is_computed_after_slicing_a_large_valid_graph() {
    let mut source = value(CHECKER);
    source["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"scalar","type":"constant-scalar","version":1}));
    let mut prior = "scalar".to_owned();
    for index in 0..20 {
        let id = format!("levels{index:02}");
        source["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":id,"type":"levels","version":1}));
        source["edges"].as_array_mut().unwrap().push(
            json!({"from":{"nodeId":prior,"portId":"value"},"to":{"nodeId":id,"portId":"in"}}),
        );
        prior = id;
    }
    source["edges"].as_array_mut().unwrap().push(json!({"from":{"nodeId":prior,"portId":"value"},"to":{"nodeId":"out","portId":"roughness"}}));
    let doc = edited(&source);
    let mut req = CompileRequest {
        size: [2048, 2048],
        ..request(&[Channel::Roughness])
    };
    assert_failure(&doc, &req, "MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED");
    req.outputs = vec![Channel::Normal];
    assert_eq!(
        compile(&doc, &req).unwrap().passes()[0].kernel.id(),
        KernelId::Constant
    );
}
