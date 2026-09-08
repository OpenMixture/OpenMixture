//! Validation of hostile graph topology and parameter bindings through public APIs.
use mixture_core::registry::PortDefault;
use mixture_core::{
    DiagnosticCode as Code, DiagnosticReport, Edge, Endpoint, InputSource, MaterialDocument,
    SafetyLimits,
};
use serde_json::{Value, json};
const CHECKER: &[u8] = include_bytes!("../../../fixtures/format/valid/checker.mix");
fn decode(bytes: &[u8]) -> MaterialDocument {
    MaterialDocument::decode(bytes, &SafetyLimits::default()).unwrap()
}
fn fixture() -> MaterialDocument {
    decode(CHECKER)
}
fn codes(document: &MaterialDocument) -> Vec<Code> {
    document
        .validate(&SafetyLimits::default())
        .diagnostics()
        .iter()
        .map(|d| d.code)
        .collect()
}
fn changed(edit: impl FnOnce(&mut Value)) -> MaterialDocument {
    let mut value: Value = serde_json::from_slice(CHECKER).unwrap();
    edit(&mut value);
    decode(&serde_json::to_vec(&value).unwrap())
}
#[test]
fn validation_all_m2_contracts_and_material_defaults_are_available_without_gpu() {
    let all = decode(include_bytes!("../../../fixtures/format/valid/all-m2.mix"));
    let validated = all.into_validated(&SafetyLimits::default()).unwrap();
    assert_eq!(validated.parameter("levels", "inputMin"), Some(json!(0.0)));
    assert_eq!(validated.parameter("levels", "gamma"), Some(json!(2.0)));
    assert!(matches!(
        validated.input_source("blend", "mask"),
        Some(InputSource::Connected { .. })
    ));
    let mut no_mask = validated.document().clone();
    no_mask.edges.retain(|edge| edge.to.port_id != "mask");
    assert_eq!(
        no_mask
            .into_validated(&SafetyLimits::default())
            .unwrap()
            .input_source("blend", "mask"),
        Some(InputSource::Default {
            value: PortDefault::Scalar(1.0)
        })
    );
    let simple = fixture().into_validated(&SafetyLimits::default()).unwrap();
    let channels = simple.material_channels();
    assert_eq!(channels.len(), 8);
    assert!(matches!(channels[0].input, InputSource::Connected { .. }));
    for (name, default) in [
        ("normal", PortDefault::Normal([0.5, 0.5, 1.0])),
        ("roughness", PortDefault::Scalar(1.0)),
        ("metallic", PortDefault::Scalar(0.0)),
        ("height", PortDefault::Scalar(0.0)),
        ("ambientOcclusion", PortDefault::Scalar(1.0)),
        ("opacity", PortDefault::Scalar(1.0)),
        ("emissive", PortDefault::Color([0.0, 0.0, 0.0, 1.0])),
    ] {
        assert_eq!(
            simple.input_source("out", name),
            Some(InputSource::Default { value: default })
        );
    }
    assert!(simple.parameter("missing", "cellsX").is_none());
    assert!(simple.input_source("checker", "color").is_none());
}
#[test]
fn validation_rejects_duplicate_invalid_unknown_and_unsupported_nodes() {
    for id in ["", "1node", "a b", "节点", &"a".repeat(65)] {
        assert!(
            codes(&changed(|v| v["nodes"][0]["id"] = json!(id))).contains(&Code::NodeInvalidId)
        );
    }
    assert!(
        codes(&decode(include_bytes!(
            "../../../fixtures/format/invalid/duplicate-ids.mix"
        )))
        .contains(&Code::NodeDuplicateId)
    );
    assert!(
        codes(&changed(|v| v["nodes"][0]["type"] = json!("transform-2d")))
            .contains(&Code::NodeUnknownType)
    );
    let unsupported = changed(|v| {
        v["nodes"][0]["version"] = json!(2);
        v["nodes"][0]["parameters"] = json!({"future":true});
    });
    let found = codes(&unsupported);
    assert!(found.contains(&Code::NodeUnsupportedVersion));
    assert!(
        !found.contains(&Code::ParameterUnknown),
        "do not interpret parameters using a different version"
    );
}
#[test]
fn validation_rejects_edge_identity_cardinality_endpoints_directions_and_kinds() {
    let duplicate = decode(include_bytes!(
        "../../../fixtures/format/invalid/duplicate-edges.mix"
    ));
    assert!(codes(&duplicate).contains(&Code::GraphDuplicateEdge));
    assert!(codes(&duplicate).contains(&Code::GraphMultipleInputs));
    for (pointer, value, expected) in [
        ("/edges/0/from/nodeId", "absent", Code::GraphUnknownNode),
        ("/edges/0/to/nodeId", "absent", Code::GraphUnknownNode),
        ("/edges/0/from/portId", "missing", Code::PortUnknown),
        ("/edges/0/to/portId", "missing", Code::PortUnknown),
    ] {
        assert!(
            codes(&changed(|v| *v.pointer_mut(pointer).unwrap() = json!(value)))
                .contains(&expected)
        );
    }
    let reversed = changed(|v| {
        let from = v["edges"][0]["from"].clone();
        v["edges"][0]["from"] = v["edges"][0]["to"].clone();
        v["edges"][0]["to"] = from;
    });
    assert_eq!(
        codes(&reversed)
            .iter()
            .filter(|code| **code == Code::PortUnknown)
            .count(),
        2
    );
    let mismatch = decode(include_bytes!(
        "../../../fixtures/format/invalid/type-mismatch.mix"
    ));
    assert!(codes(&mismatch).contains(&Code::PortTypeMismatch));
    assert!(codes(&mismatch).contains(&Code::PortRequiredConnection));
    let mut multiple = fixture();
    let mut second = multiple.nodes[0].clone();
    second.id = "second".into();
    multiple.nodes.push(second);
    let mut edge = multiple.edges[0].clone();
    edge.from.node_id = "second".into();
    multiple.edges.push(edge);
    assert!(codes(&multiple).contains(&Code::GraphMultipleInputs));
    assert!(!codes(&multiple).contains(&Code::GraphDuplicateEdge));
    let color_as_normal = changed(|v| {
        v["edges"].as_array_mut().unwrap().push(json!({"from":{"nodeId":"checker","portId":"color"},"to":{"nodeId":"out","portId":"normal"}}))
    });
    assert!(codes(&color_as_normal).contains(&Code::PortTypeMismatch));
}
#[test]
fn validation_cycles_are_deterministic_and_do_not_mislabel_downstream_nodes() {
    let document = decode(include_bytes!("../../../fixtures/format/invalid/cycle.mix"));
    let report = document.validate(&SafetyLimits::default());
    assert_eq!(
        serde_json::to_string_pretty(&report).unwrap(),
        include_str!("../../../fixtures/format/invalid/cycle.diagnostics.json").trim_end()
    );
    for _ in 0..4 {
        let mut reordered = document.clone();
        reordered.nodes.reverse();
        reordered.edges.reverse();
        assert_eq!(
            serde_json::to_value(reordered.validate(&SafetyLimits::default())).unwrap(),
            serde_json::to_value(&report).unwrap()
        );
    }
    let mut self_loop = document.clone();
    self_loop.edges.retain(|e| e.from.node_id == "checker");
    self_loop.edges.push(Edge {
        from: Endpoint {
            node_id: "a".into(),
            port_id: "value".into(),
        },
        to: Endpoint {
            node_id: "a".into(),
            port_id: "in".into(),
        },
    });
    let report = self_loop.validate(&SafetyLimits::default());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.code == Code::GraphCycle && d.evidence["cycle"] == "a -> a".into())
    );
}
#[test]
fn validation_requires_one_material_output_and_valid_required_inputs() {
    let missing = decode(include_bytes!(
        "../../../fixtures/format/invalid/missing-base-color.mix"
    ));
    assert!(codes(&missing).contains(&Code::PortRequiredConnection));
    let no_output = changed(|v| {
        v["nodes"].as_array_mut().unwrap().pop();
        v["edges"] = json!([]);
    });
    assert!(codes(&no_output).contains(&Code::GraphMaterialOutputCount));
    let extra_output = changed(|v| {
        v["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"extra","type":"material-output","version":1}))
    });
    assert!(codes(&extra_output).contains(&Code::GraphMaterialOutputCount));
    let no_levels_input = changed(|v| {
        v["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"l","type":"levels","version":1}))
    });
    assert!(
        codes(&no_levels_input).contains(&Code::PortRequiredConnection),
        "unreachable nodes are validated"
    );
}
#[test]
fn validation_checks_parameter_shapes_ranges_defaults_and_relations() {
    for value in [
        json!(0),
        json!(1025),
        json!(8.0),
        json!(-1),
        json!("8"),
        json!(null),
        json!(true),
        json!({}),
        json!([]),
    ] {
        let document = changed(|v| v["nodes"][0]["parameters"] = json!({"cellsX":value}));
        let report = document.validate(&SafetyLimits::default());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.code == Code::ParameterInvalidValue
                    && d.node_id.as_deref() == Some("checker")
                    && d.parameter_id.as_deref() == Some("cellsX"))
        );
    }
    for color in [
        json!([1, 1, 1]),
        json!([0, 0, 0, 1, 1]),
        json!([0, 0, 0, 2]),
        json!([0, 0, 0, null]),
        json!([0, 0, 0, "1"]),
    ] {
        assert!(
            codes(&changed(
                |v| v["nodes"][0]["parameters"] = json!({"colorA":color})
            ))
            .contains(&Code::ParameterInvalidValue)
        );
    }
    assert!(
        codes(&changed(
            |v| v["nodes"][0]["parameters"] = json!({"unknown":1})
        ))
        .contains(&Code::ParameterUnknown)
    );
    let mut all = decode(include_bytes!("../../../fixtures/format/valid/all-m2.mix"));
    all.nodes
        .iter_mut()
        .find(|n| n.id == "levels")
        .unwrap()
        .parameters
        .insert("inputMin".into(), json!(1.0));
    assert!(codes(&all).contains(&Code::ParameterInvalidValue));
    let levels = all.nodes.iter_mut().find(|n| n.id == "levels").unwrap();
    levels.parameters.insert("inputMin".into(), json!(0.0));
    levels.parameters.insert("outputMin".into(), json!(1.0));
    levels.parameters.insert("outputMax".into(), json!(0.0));
    assert!(
        all.validate(&SafetyLimits::default()).is_ok(),
        "output inversion is allowed"
    );
    all.nodes
        .iter_mut()
        .find(|n| n.id == "blend")
        .unwrap()
        .parameters
        .insert("mode".into(), json!("overlay"));
    assert!(codes(&all).contains(&Code::ParameterInvalidValue));
}
#[test]
fn validation_exposed_bindings_require_unique_names_unique_targets_and_mutable_parameters() {
    for (field, value) in [
        ("id", ""),
        ("id", "1bad"),
        ("nodeId", "absent"),
        ("parameterId", "version"),
        ("parameterId", "color"),
    ] {
        assert!(
            codes(&changed(|v| v["exposedParameters"][0][field] = json!(value)))
                .contains(&Code::ExposedParameterInvalid)
        );
    }
    for second in [
        json!({"id":"frequency","nodeId":"checker","parameterId":"cellsY"}),
        json!({"id":"alias","nodeId":"checker","parameterId":"cellsX"}),
    ] {
        assert!(
            codes(&changed(|v| v["exposedParameters"]
                .as_array_mut()
                .unwrap()
                .push(second)))
            .contains(&Code::ExposedParameterInvalid)
        );
    }
    assert!(
        fixture().validate(&SafetyLimits::default()).is_ok(),
        "defaulted mutable parameters can be exposed"
    );
}
#[test]
fn validation_reports_are_stable_under_all_source_collection_orders_and_do_not_repair() {
    let mut invalid = decode(include_bytes!("../../../fixtures/format/invalid/cycle.mix"));
    invalid.nodes.push(invalid.nodes[0].clone());
    invalid.edges.push(invalid.edges[0].clone());
    invalid
        .exposed_parameters
        .push(invalid.exposed_parameters[0].clone());
    let before = invalid.clone();
    let expected = serde_json::to_value(invalid.validate(&SafetyLimits::default())).unwrap();
    assert_eq!(invalid, before);
    for _ in 0..invalid.nodes.len() {
        invalid.nodes.rotate_left(1);
        invalid.edges.rotate_left(1);
        invalid.exposed_parameters.reverse();
        assert_eq!(
            serde_json::to_value(invalid.validate(&SafetyLimits::default())).unwrap(),
            expected
        );
    }
    assert!(invalid.into_validated(&SafetyLimits::default()).is_err());
}
#[test]
fn validation_rechecks_constructed_documents_and_explicit_collection_limits() {
    let doc = fixture();
    for limits in [
        SafetyLimits {
            nodes: 1,
            ..Default::default()
        },
        SafetyLimits {
            edges: 0,
            ..Default::default()
        },
        SafetyLimits {
            exposed_parameters: 0,
            ..Default::default()
        },
    ] {
        let report = doc.validate(&limits);
        assert!(!report.is_ok());
        assert_eq!(
            report.diagnostics().len(),
            1,
            "stop graph analysis after budget failure"
        );
    }
    let mut doc = doc;
    doc.version = 7;
    assert!(codes(&doc).contains(&Code::FormatUnsupportedVersion));
    let report: DiagnosticReport = fixture().validate(&SafetyLimits::default());
    assert!(report.is_ok());
}
