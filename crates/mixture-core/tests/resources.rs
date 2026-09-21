//! Public resource preparation: no GPU and no caller-trusted digests.
use mixture_core::{
    registry::{ParameterKind, node_contract},
    *,
};
use serde_json::{Value, json};

fn source() -> Value {
    json!({"version":1,"nodes":[
        {"id":"color","type":"constant-color","version":1},
        {"id":"a","type":"image-input","version":1,"parameters":{"resourceId":"first"}},
        {"id":"b","type":"image-input","version":1,"parameters":{"resourceId":"second"}},
        {"id":"unused","type":"image-input","version":1,"parameters":{"resourceId":"unused"}},
        {"id":"mix","type":"scalar-blend","version":1},
        {"id":"out","type":"material-output","version":1}],
        "edges":[
            {"from":{"nodeId":"color","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},
            {"from":{"nodeId":"a","portId":"value"},"to":{"nodeId":"mix","portId":"a"}},
            {"from":{"nodeId":"b","portId":"value"},"to":{"nodeId":"mix","portId":"b"}},
            {"from":{"nodeId":"mix","portId":"value"},"to":{"nodeId":"out","portId":"height"}}],
        "exposedParameters":[
            {"id":"source","nodeId":"a","parameterId":"resourceId"},
            {"id":"unusedSource","nodeId":"unused","parameterId":"resourceId"},
            {"id":"weight","nodeId":"mix","parameterId":"weight"}]})
}
fn document(value: &Value) -> ValidatedDocument {
    MaterialDocument::decode(
        &serde_json::to_vec(value).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
    .into_validated(&SafetyLimits::default())
    .unwrap()
}
fn request() -> CompileRequest {
    CompileRequest {
        size: [2, 2],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    }
}
fn binding<'a>(id: &'a str, data: &'a [u8]) -> ImageBinding<'a> {
    ImageBinding {
        id,
        width: 2,
        height: 2,
        format: "rgba8-linear",
        bytes_per_row: 8,
        data,
    }
}
fn codes(error: CompileError) -> Vec<String> {
    error
        .report()
        .diagnostics()
        .iter()
        .map(|d| d.code.as_str().into())
        .collect()
}
fn fails(doc: &ValidatedDocument, req: &CompileRequest, inputs: &[ImageBinding<'_>], code: &str) {
    let error = prepare(doc, req, inputs, &ResourceLimits::default()).unwrap_err();
    assert!(
        error
            .report()
            .diagnostics()
            .iter()
            .all(|d| d.stage == Stage::Compile)
    );
    assert!(codes(error).iter().any(|c| c == code), "missing {code}");
}

#[test]
fn resource_ref_is_required_typed_exposable_and_roundtrips() {
    let contract = node_contract("image-input").unwrap();
    let p = contract.parameter("resourceId").unwrap();
    assert!(matches!(p.kind, ParameterKind::ResourceRef));
    assert!(p.default.is_none());
    for bad in [
        json!(null),
        json!(3),
        json!(true),
        json!([]),
        json!(""),
        json!("_x"),
        json!("file.png"),
        json!("../x"),
        json!("https://a"),
        json!("高度"),
        json!("x".repeat(65)),
    ] {
        let mut s = source();
        s["nodes"][3]["parameters"]["resourceId"] = bad;
        assert!(
            !MaterialDocument::decode(&serde_json::to_vec(&s).unwrap(), &SafetyLimits::default())
                .unwrap()
                .validate(&SafetyLimits::default())
                .is_ok()
        );
    }
    assert!(p.accepts(&json!("A".repeat(64))));
    let doc = document(&source());
    let encoded = doc.document().to_json().unwrap();
    let roundtrip = MaterialDocument::decode(&encoded, &SafetyLimits::default()).unwrap();
    assert_eq!(roundtrip.to_json().unwrap(), encoded);
    let mut missing = source();
    missing["nodes"][1]["parameters"] = json!({});
    assert!(
        !MaterialDocument::decode(
            &serde_json::to_vec(&missing).unwrap(),
            &SafetyLimits::default()
        )
        .unwrap()
        .validate(&SafetyLimits::default())
        .is_ok()
    );
    let mut wrong_version = source();
    wrong_version["nodes"][1]["version"] = json!(2);
    assert!(
        !MaterialDocument::decode(
            &serde_json::to_vec(&wrong_version).unwrap(),
            &SafetyLimits::default()
        )
        .unwrap()
        .validate(&SafetyLimits::default())
        .is_ok()
    );
}

#[test]
fn missing_resources_follow_slice_but_overrides_validate_full_graph() {
    let doc = document(&source());
    let mut req = request();
    let missing = prepare(&doc, &req, &[], &Default::default()).unwrap_err();
    let diagnostics = missing.report().diagnostics();
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].node_id.as_deref(), Some("a"));
    assert_eq!(diagnostics[1].node_id.as_deref(), Some("b"));
    assert!(
        diagnostics
            .iter()
            .all(|d| d.parameter_id.as_deref() == Some("resourceId"))
    );
    assert_eq!(codes(compile(&doc, &req).unwrap_err()), codes(missing));
    req.outputs = vec![OutputChannel::BaseColor];
    let plan = compile(&doc, &req).unwrap();
    assert!(plan.image_resources().is_empty());
    req.overrides.insert("unusedSource".into(), json!("../bad"));
    assert!(compile(&doc, &req).is_err());
    req = request();
    req.overrides.insert("weight".into(), json!(0));
    fails(
        &doc,
        &req,
        &[binding("first", &[0; 16])],
        "MIX_RESOURCE_MISSING",
    );
    req.overrides.insert("source".into(), json!("second"));
    let prepared = prepare(
        &doc,
        &req,
        &[binding("second", &[1; 16])],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(prepared.resources().len(), 1);
    assert_eq!(prepared.plan().estimates().resource_count, 1);
    fails(
        &doc,
        &req,
        &[binding("first", &[0; 16]), binding("second", &[1; 16])],
        "MIX_RESOURCE_UNKNOWN_ID",
    );
}

#[test]
fn rejects_duplicate_unknown_format_size_stride_and_length_without_repair() {
    let doc = document(&source());
    let req = request();
    let bytes = [0; 16];
    let a = binding("first", &bytes);
    let b = binding("second", &bytes);
    for (bad, code) in [
        (
            ImageBinding { id: "second", ..a },
            "MIX_RESOURCE_DUPLICATE_ID",
        ),
        (ImageBinding { id: "First", ..a }, "MIX_RESOURCE_UNKNOWN_ID"),
        (
            ImageBinding {
                id: "../first",
                ..a
            },
            "MIX_RESOURCE_INVALID_BINDING",
        ),
        (
            ImageBinding {
                format: "rgba8-srgb",
                ..a
            },
            "MIX_RESOURCE_FORMAT_UNSUPPORTED",
        ),
        (ImageBinding { width: 0, ..a }, "MIX_RESOURCE_SIZE_MISMATCH"),
        (ImageBinding { width: 3, ..a }, "MIX_RESOURCE_SIZE_MISMATCH"),
        (
            ImageBinding {
                bytes_per_row: 256,
                ..a
            },
            "MIX_RESOURCE_LENGTH_MISMATCH",
        ),
        (
            ImageBinding {
                data: &bytes[..15],
                ..a
            },
            "MIX_RESOURCE_LENGTH_MISMATCH",
        ),
        (
            ImageBinding {
                data: &[0; 17],
                ..a
            },
            "MIX_RESOURCE_LENGTH_MISMATCH",
        ),
    ] {
        fails(&doc, &req, &[bad, b], code);
    }
    let mut unused = binding("unused", &bytes);
    unused.format = "unknown";
    fails(
        &doc,
        &req,
        &[a, b, unused],
        "MIX_RESOURCE_FORMAT_UNSUPPORTED",
    );
    let first = serde_json::to_value(
        prepare(&doc, &req, &[a, a, b], &Default::default())
            .unwrap_err()
            .report(),
    )
    .unwrap();
    let second = serde_json::to_value(
        prepare(&doc, &req, &[b, a, a], &Default::default())
            .unwrap_err()
            .report(),
    )
    .unwrap();
    assert_eq!(first, second);
}

#[test]
fn budgets_are_inclusive_include_unused_inputs_and_never_raise_limits() {
    let doc = document(&source());
    let req = request();
    let inputs = [binding("first", &[0; 16]), binding("second", &[1; 16])];
    let exact = ResourceLimits {
        resource_count: 2,
        resource_pixels: 8,
        resource_bytes: 32,
    };
    let prepared = prepare(&doc, &req, &inputs, &exact).unwrap();
    let estimate = prepared.plan().estimates();
    assert_eq!(
        (
            estimate.resource_texture_bytes,
            estimate.resource_staging_bytes
        ),
        (32, 1024)
    );
    assert_eq!(estimate.resource_upload_bytes, 32);
    for (limits, code) in [
        (
            ResourceLimits {
                resource_count: 1,
                ..exact
            },
            "MIX_LIMIT_RESOURCE_COUNT_EXCEEDED",
        ),
        (
            ResourceLimits {
                resource_pixels: 7,
                ..exact
            },
            "MIX_LIMIT_RESOURCE_PIXELS_EXCEEDED",
        ),
        (
            ResourceLimits {
                resource_bytes: 31,
                ..exact
            },
            "MIX_LIMIT_RESOURCE_BYTES_EXCEEDED",
        ),
    ] {
        assert!(codes(prepare(&doc, &req, &inputs, &limits).unwrap_err()).contains(&code.into()));
    }
    let mut all = inputs.to_vec();
    all.push(binding("unused", &[2; 16]));
    assert!(prepare(&doc, &req, &all, &exact).is_err());
    let mut cap = req.clone();
    cap.limits.transient_bytes = estimate.peak_bytes;
    assert!(prepare(&doc, &cap, &inputs, &exact).is_ok());
    cap.limits.transient_bytes -= 1;
    assert!(
        codes(prepare(&doc, &cap, &inputs, &exact).unwrap_err())
            .contains(&"MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED".into())
    );
    let mut sliced = req.clone();
    sliced.outputs = vec![OutputChannel::BaseColor];
    let zero = ResourceLimits {
        resource_count: 0,
        resource_pixels: 0,
        resource_bytes: 0,
    };
    assert!(prepare(&doc, &sliced, &[], &zero).is_ok());
    assert!(prepare(&doc, &sliced, &inputs, &zero).is_err());
    // Overflow is detected using descriptor arithmetic, without a huge allocation.
    let huge = ImageBinding {
        width: u32::MAX,
        height: u32::MAX,
        data: &[],
        ..inputs[0]
    };
    assert!(
        codes(prepare(&doc, &req, &[huge], &Default::default()).unwrap_err())
            .contains(&"MIX_RESOURCE_INVALID_BINDING".into())
    );
    assert!(
        serde_json::from_value::<ResourceLimits>(
            json!({"resourceCount":8,"resourcePixels":8,"resourceBytes":8,"unknown":1})
        )
        .is_err()
    );
}

#[test]
fn snapshot_and_plan_identity_are_content_bound_sorted_and_slice_local() {
    let doc = document(&source());
    let req = request();
    let mut pixels: Vec<u8> = (0..16).collect();
    let first = prepare(
        &doc,
        &req,
        &[binding("second", &[7; 16]), binding("first", &pixels)],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(first.plan()).unwrap(),
        serde_json::from_str::<Value>(include_str!("snapshots/plan-v2-images.json")).unwrap()
    );
    pixels.fill(255);
    assert_eq!(first.resources()[0].data(), &(0u8..16).collect::<Vec<_>>());
    assert_eq!(first.resources()[0].image().id, "first");
    assert!(!format!("{first:?}").contains("data: ["));
    let second = prepare(
        &doc,
        &req,
        &[
            binding("first", &(0u8..16).collect::<Vec<_>>()),
            binding("second", &[7; 16]),
            binding("unused", &pixels),
        ],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(first.plan().hash(), second.plan().hash());
    let changed = prepare(
        &doc,
        &req,
        &[binding("first", &pixels), binding("second", &[7; 16])],
        &Default::default(),
    )
    .unwrap();
    assert_ne!(first.plan().hash(), changed.plan().hash());
    // Only alpha changes; exact content identity still changes.
    let mut alpha: Vec<u8> = (0..16).collect();
    alpha[3] = 0;
    let changed = prepare(
        &doc,
        &req,
        &[binding("first", &alpha), binding("second", &[7; 16])],
        &Default::default(),
    )
    .unwrap();
    assert_ne!(first.plan().hash(), changed.plan().hash());
    let mut reordered = source();
    reordered["nodes"].as_array_mut().unwrap().reverse();
    reordered["edges"].as_array_mut().unwrap().reverse();
    let equivalent = prepare(
        &document(&reordered),
        &req,
        &[
            binding("first", &(0u8..16).collect::<Vec<_>>()),
            binding("second", &[7; 16]),
        ],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(first.plan().hash(), equivalent.plan().hash());
    assert_eq!(first.plan().version(), 2);
    assert!(
        first
            .plan()
            .hash_input()
            .unwrap()
            .starts_with(b"mixture-render-plan-v2\0")
    );
    // Frozen independently using Node crypto, not the production digest function.
    assert_eq!(
        first.resources()[0].image().content_digest,
        "781dde1290998ab7c47a68780691e96c9b703cea9b2d91bee1514c210ff8afdb"
    );
}

#[test]
fn odd_dimensions_identity_and_policy_are_explicit() {
    let mut source = source();
    source["nodes"][2]["parameters"]["resourceId"] = json!("first");
    let document = document(&source);
    let mut request = request();
    request.size = [65, 3];
    let bytes = vec![128; 65 * 3 * 4];
    let image = ImageBinding {
        id: "first",
        width: 65,
        height: 3,
        format: "rgba8-linear",
        bytes_per_row: 260,
        data: &bytes,
    };
    let prepared = prepare(&document, &request, &[image], &Default::default()).unwrap();
    assert_eq!(prepared.plan().estimates().resource_staging_bytes, 1536);
    assert_eq!(prepared.plan().estimates().resource_upload_bytes, 780);
    let permissive = ResourceLimits {
        resource_count: 99,
        resource_pixels: 9999,
        resource_bytes: 99999,
    };
    assert_eq!(
        prepared.plan().hash(),
        prepare(&document, &request, &[image], &permissive)
            .unwrap()
            .plan()
            .hash()
    );
    request.size = [3, 65];
    let swapped = ImageBinding {
        width: 3,
        height: 65,
        bytes_per_row: 12,
        ..image
    };
    let other = prepare(&document, &request, &[swapped], &permissive).unwrap();
    assert_ne!(
        prepared.resources()[0].image().content_digest,
        other.resources()[0].image().content_digest
    );
    assert_ne!(prepared.plan().hash(), other.plan().hash());
}

#[test]
fn approved_height_fixture_prepares_both_outputs_without_a_gpu() {
    let source = include_bytes!("../../../fixtures/nodes/image-input/height.mix");
    let doc = MaterialDocument::decode(source, &SafetyLimits::default())
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let pixels = [0, 11, 22, 0, 64, 33, 44, 1, 128, 55, 66, 2, 255, 77, 88, 3];
    let mut req = request();
    req.outputs = vec![OutputChannel::Height, OutputChannel::Normal];
    for weight in [0., 0.25, 0.5, 1.] {
        req.overrides.insert("detailWeight".into(), json!(weight));
        let result = prepare(
            &doc,
            &req,
            &[binding("heightSource", &pixels)],
            &Default::default(),
        )
        .unwrap();
        assert_eq!(result.resources().len(), 1);
        assert_eq!(result.plan().outputs().len(), 2);
        assert_eq!(result.plan().passes().len(), 4);
    }
}

#[test]
fn adapter_capture_is_selected_bounded_and_hashed_by_core() {
    use std::cell::Cell;
    struct Source<'a>(&'a Cell<usize>, &'a [u8]);
    impl ImageData for Source<'_> {
        fn byte_len(&self) -> usize {
            self.1.len()
        }
        fn copy_to(&self, target: &mut [u8]) -> Result<(), CompileError> {
            self.0.set(self.0.get() + 1);
            target.copy_from_slice(self.1);
            Ok(())
        }
    }
    let calls = Cell::new(0);
    let pixels = [19; 16];
    let input = |id| AdapterImageBinding {
        id,
        width: 2,
        height: 2,
        format: "rgba8-linear",
        bytes_per_row: 8,
        data: Source(&calls, &pixels),
    };
    let doc = document(&source());
    let req = request();
    for inputs in [
        vec![input("first")],
        vec![input("first"), input("first")],
        vec![input("first"), input("second"), input("unknown")],
    ] {
        assert!(prepare_from(&doc, &req, &inputs, &Default::default()).is_err());
        assert_eq!(calls.get(), 0);
    }
    let inputs = [input("first"), input("second"), input("unused")];
    assert!(
        prepare_from(
            &doc,
            &req,
            &inputs,
            &ResourceLimits {
                resource_bytes: 47,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(calls.get(), 0);
    let adapted = prepare_from(&doc, &req, &inputs, &Default::default()).unwrap();
    assert_eq!(calls.get(), 2);
    let borrowed = prepare(
        &doc,
        &req,
        &[binding("first", &pixels), binding("second", &pixels)],
        &Default::default(),
    )
    .unwrap();
    assert_eq!(adapted.plan().hash(), borrowed.plan().hash());
}
