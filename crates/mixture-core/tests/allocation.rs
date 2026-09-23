//! Independent expected schedules for the v3 physical-slot contract.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};
use serde_json::{Value, json};

fn node(id: &str, kind: &str) -> Value {
    json!({"id":id,"type":kind,"version":1})
}
fn edge(from: &str, to: &str, port: &str) -> Value {
    json!({"from":{"nodeId":from,"portId":"value"},"to":{"nodeId":to,"portId":port}})
}
fn source(nodes: Vec<Value>, edges: Vec<Value>) -> Value {
    let mut nodes = nodes;
    let mut edges = edges;
    nodes.push(node("zColor", "constant-color"));
    edges.push(json!({"from":{"nodeId":"zColor","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}));
    nodes.push(node("out", "material-output"));
    json!({"version":1,"nodes":nodes,"edges":edges})
}
fn plan(source: &Value, outputs: Vec<OutputChannel>) -> mixture_core::RenderPlan {
    let document = MaterialDocument::decode(
        &serde_json::to_vec(source).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap();
    let validated = document.into_validated(&SafetyLimits::default()).unwrap();
    compile(
        &validated,
        &CompileRequest {
            size: [19, 11],
            outputs,
            ..Default::default()
        },
    )
    .unwrap()
}
fn slots(plan: &mixture_core::RenderPlan) -> Vec<u32> {
    plan.allocation()
        .resource_slots()
        .iter()
        .map(|slot| slot.index())
        .collect()
}

#[test]
fn dead_scalar_slot_cannot_store_a_later_color_result() {
    let graph = json!({"version":1,"nodes":[node("a","constant-scalar"),node("b","levels"),node("c","gradient-map"),node("out","material-output")],
        "edges":[edge("a","b","in"),edge("b","c","in"),{"from":{"nodeId":"c","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}]});
    let compiled = plan(&graph, vec![OutputChannel::BaseColor]);
    assert_eq!(slots(&compiled), [0, 1, 2]);
    assert_eq!(compiled.allocation().last_uses(), [1, 2, 3]);
    assert_ne!(
        compiled.allocation().slots()[0].kind,
        compiled.allocation().slots()[2].kind
    );
}

#[test]
fn chain_reuses_only_after_the_consumer_and_retains_the_final_result() {
    let graph = source(
        vec![
            node("a", "constant-scalar"),
            node("b", "levels"),
            node("c", "levels"),
            node("d", "levels"),
        ],
        vec![
            edge("a", "b", "in"),
            edge("b", "c", "in"),
            edge("c", "d", "in"),
            edge("d", "out", "height"),
        ],
    );
    let compiled = plan(&graph, vec![OutputChannel::Height]);
    assert_eq!(compiled.version(), 3);
    assert_eq!(slots(&compiled), [0, 1, 0, 1]);
    assert_eq!(compiled.allocation().last_uses(), [1, 2, 3, 4]);
    assert_eq!(compiled.estimates().texture_count, 2);
    assert_eq!(compiled.estimates().logical_texture_bytes, 4 * 19 * 11 * 8);
    assert_eq!(compiled.estimates().texture_bytes, 2 * 19 * 11 * 8);
    assert!(
        compiled
            .hash_input()
            .unwrap()
            .starts_with(b"mixture-render-plan-v3\0")
    );
}

#[test]
fn diamond_repeated_bindings_alias_outputs_and_slicing_have_independent_schedules() {
    let mut graph = source(
        vec![
            node("a", "constant-scalar"),
            node("b", "levels"),
            node("c", "levels"),
            node("d", "scalar-blend"),
        ],
        vec![
            edge("a", "b", "in"),
            edge("a", "c", "in"),
            edge("b", "d", "a"),
            edge("c", "d", "b"),
            edge("d", "out", "height"),
            edge("b", "out", "roughness"),
            edge("b", "out", "metallic"),
        ],
    );
    let all = plan(
        &graph,
        vec![
            OutputChannel::Height,
            OutputChannel::Roughness,
            OutputChannel::Metallic,
        ],
    );
    assert_eq!(slots(&all), [0, 1, 2, 0]);
    assert_eq!(all.allocation().last_uses(), [2, 4, 3, 4]);
    assert_eq!(all.outputs()[0].resource, all.outputs()[1].resource);
    let sliced = plan(&graph, vec![OutputChannel::Roughness]);
    assert_eq!(slots(&sliced), [0, 1]);
    assert_eq!(sliced.allocation().last_uses(), [1, 2]);
    // Make both blend bindings refer to b; c must disappear from the selected slice.
    graph["edges"][3] = edge("b", "d", "b");
    let repeated = plan(&graph, vec![OutputChannel::Height]);
    assert_eq!(slots(&repeated), [0, 1, 0]);
    assert_eq!(repeated.allocation().last_uses(), [1, 2, 3]);
    let mut reversed = graph.clone();
    reversed["nodes"].as_array_mut().unwrap().reverse();
    reversed["edges"].as_array_mut().unwrap().reverse();
    assert_eq!(
        repeated.hash(),
        plan(&reversed, vec![OutputChannel::Height]).hash()
    );
}

#[test]
fn requested_defaults_pin_distinct_kinds_without_cross_kind_reuse() {
    let graph = source(
        vec![node("a", "constant-scalar")],
        vec![edge("a", "out", "height")],
    );
    let compiled = plan(
        &graph,
        vec![
            OutputChannel::Height,
            OutputChannel::Normal,
            OutputChannel::BaseColor,
        ],
    );
    assert_eq!(slots(&compiled), [0, 1, 2]);
    assert_eq!(compiled.allocation().last_uses(), [3, 3, 3]);
    assert_eq!(compiled.allocation().slots().len(), 3);
    assert_ne!(
        compiled.allocation().slots()[0].kind,
        compiled.allocation().slots()[1].kind
    );
    assert_ne!(
        compiled.allocation().slots()[1].kind,
        compiled.allocation().slots()[2].kind
    );
}

#[test]
fn frozen_painted_metal_fits_2k_without_raising_the_limit() {
    let document = MaterialDocument::decode(
        include_bytes!("../../../docs/evidence/perf-mat-before/material.mix"),
        &SafetyLimits::default(),
    )
    .unwrap();
    let validated = document.into_validated(&SafetyLimits::default()).unwrap();
    let mut request = CompileRequest {
        size: [2048, 2048],
        outputs: vec![
            OutputChannel::BaseColor,
            OutputChannel::Normal,
            OutputChannel::Roughness,
            OutputChannel::Metallic,
            OutputChannel::Height,
        ],
        ..Default::default()
    };
    request.overrides.insert("radiusX".into(), json!(8));
    request.overrides.insert("radiusY".into(), json!(8));
    let compiled = compile(&validated, &request).unwrap();
    assert_eq!(compiled.passes().len(), 23);
    assert_eq!(
        slots(&compiled),
        [
            0, 1, 0, 2, 0, 3, 4, 5, 4, 6, 7, 5, 8, 9, 10, 11, 4, 12, 6, 1, 0, 13, 2
        ]
    );
    assert_eq!(compiled.estimates().texture_count, 14);
    assert_eq!(compiled.estimates().peak_bytes, 503_316_944);
    request.limits.transient_bytes = 503_316_944;
    assert_eq!(
        compiled.hash(),
        compile(&validated, &request).unwrap().hash()
    );
    request.limits.transient_bytes -= 1;
    let error = compile(&validated, &request).unwrap_err();
    assert_eq!(
        error.report().diagnostics()[0].code,
        mixture_core::DiagnosticCode::LimitTransientBytesExceeded
    );
}
