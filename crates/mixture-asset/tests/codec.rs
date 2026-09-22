//! Independent fixture and public API regression coverage.
use mixture_asset::{AssetLimits, AssetView, OwnedAsset, write};
use mixture_core::{CompileRequest, ImageBinding, MaterialDocument, OutputChannel};

const VALID: &[u8] = include_bytes!("../../../fixtures/packages/mixpack-v1/valid.mixpack");
fn request(outputs: Vec<OutputChannel>) -> CompileRequest {
    CompileRequest {
        size: [2, 2],
        outputs,
        ..Default::default()
    }
}

#[test]
fn independent_malformed_corpus_and_exact_canonical_writer() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/packages/mixpack-v1");
    let cases: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    for case in cases.as_array().unwrap() {
        let name = case["file"].as_str().unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match case["expectedCode"].as_str() {
            None => {
                AssetView::load(&bytes, &AssetLimits::default()).unwrap();
            }
            Some(code) => assert_eq!(
                AssetView::load(&bytes, &AssetLimits::default())
                    .unwrap_err()
                    .code(),
                code,
                "{name}"
            ),
        }
    }
    let view = AssetView::load(VALID, &AssetLimits::default()).unwrap();
    let encoded = write(view.source(), &view.bindings(), &AssetLimits::default()).unwrap();
    assert_eq!(encoded, VALID);
    assert_eq!(
        view.inspect().package_sha256,
        "b229aa66620cbc18145e5a678d7124b0d50b8b2a5b86c2229fbf8992ca4be4b7"
    );
}

#[test]
fn every_truncation_and_header_byte_mutation_is_rejected_without_panics() {
    for end in 0..VALID.len() {
        assert!(
            AssetView::load(&VALID[..end], &AssetLimits::default()).is_err(),
            "prefix {end}"
        );
    }
    // Manifest 407 bytes, source 1235 bytes: headers at 0, 1024 and 3072.
    let view = AssetView::load(VALID, &AssetLimits::default()).unwrap();
    let image_header =
        view.image("heightSource").unwrap().as_ptr() as usize - VALID.as_ptr() as usize - 512;
    for header in [0, 1024, image_header] {
        for offset in header..header + 512 {
            let mut bytes = VALID.to_vec();
            bytes[offset] ^= 1;
            assert!(
                AssetView::load(&bytes, &AssetLimits::default()).is_err(),
                "offset {offset}"
            );
        }
    }
}

#[test]
fn borrows_moves_copies_and_snapshots_have_distinct_lifetimes() {
    let mut input = VALID.to_vec();
    let prepared = {
        let view = AssetView::load(&input, &AssetLimits::default()).unwrap();
        let image = view.image("heightSource").unwrap();
        assert!(image.as_ptr() as usize >= input.as_ptr() as usize);
        assert!((image.as_ptr() as usize + image.len()) <= (input.as_ptr() as usize + input.len()));
        let prepared = view
            .prepare(&request(vec![OutputChannel::Height, OutputChannel::Normal]))
            .unwrap();
        assert_ne!(prepared.resources()[0].data().as_ptr(), image.as_ptr());
        prepared
    };
    input.fill(0);
    drop(input);
    assert_eq!(prepared.resources()[0].data(), [0, 37, 91, 255].repeat(4));
    let input = VALID.to_vec();
    let ptr = input.as_ptr();
    let owned = OwnedAsset::from_vec(input, &AssetLimits::default()).unwrap();
    assert_eq!(owned.as_bytes().as_ptr(), ptr);
    let copied = OwnedAsset::copy_from(owned.as_bytes(), &AssetLimits::default()).unwrap();
    assert_ne!(copied.as_bytes().as_ptr(), ptr);
    assert_eq!(
        copied
            .prepare(&request(vec![OutputChannel::Height, OutputChannel::Normal]))
            .unwrap()
            .plan()
            .hash(),
        prepared.plan().hash()
    );
}

#[test]
fn loose_core_and_package_share_hashes_selection_and_override_errors() {
    let limits = AssetLimits::default();
    let view = AssetView::load(VALID, &limits).unwrap();
    let doc = MaterialDocument::decode(view.source(), &limits.safety)
        .unwrap()
        .into_validated(&limits.safety)
        .unwrap();
    for channel in [
        OutputChannel::BaseColor,
        OutputChannel::Height,
        OutputChannel::Normal,
    ] {
        for weight in [0.0, 0.25, 0.5, 1.0] {
            let mut req = request(vec![channel]);
            req.overrides
                .insert("detailWeight".into(), serde_json::json!(weight));
            let loose =
                mixture_core::prepare(&doc, &req, &view.bindings(), &limits.resources).unwrap();
            let packed = view.prepare(&req).unwrap();
            assert_eq!(loose.plan().hash(), packed.plan().hash());
            assert_eq!(
                packed.resources().len(),
                usize::from(channel != OutputChannel::BaseColor)
            );
            for (snapshot, binding) in packed.resources().iter().zip(view.bindings()) {
                assert_eq!(
                    snapshot.image(),
                    &mixture_core::resources::image_identity(
                        &binding,
                        &limits.safety,
                        &limits.resources
                    )
                    .unwrap()
                );
            }
        }
    }
    let mut req = request(vec![OutputChannel::BaseColor]);
    req.overrides
        .insert("heightSource".into(), serde_json::json!("heightSource"));
    assert_eq!(
        view.prepare(&req).unwrap_err().code(),
        "MIX_PACKAGE_RESOURCE_OVERRIDE"
    );
    for (key, value) in [
        ("heightSource", serde_json::json!(42)),
        ("missing", serde_json::json!(0)),
        ("detailWeight", serde_json::json!("bad")),
    ] {
        req.overrides.clear();
        req.overrides.insert(key.into(), value);
        let loose =
            mixture_core::prepare(&doc, &req, &view.bindings(), &limits.resources).unwrap_err();
        let packed = view.prepare(&req).unwrap_err();
        assert!(matches!(packed, mixture_asset::AssetError::Compile(_)));
        assert_eq!(packed.code(), loose.report().diagnostics()[0].code.as_str());
    }
    let mut wrong_size = request(vec![OutputChannel::BaseColor]);
    wrong_size.size = [1, 1];
    assert_eq!(
        view.prepare(&wrong_size).unwrap_err().code(),
        "MIX_RESOURCE_SIZE_MISMATCH"
    );
}

#[test]
fn budgets_apply_before_owned_copies_and_selected_capture() {
    let view = AssetView::load(VALID, &AssetLimits::default()).unwrap();
    let base = request(vec![OutputChannel::BaseColor]);
    let height = request(vec![OutputChannel::Height]);
    let scratch = view.preparation_buffer_bytes(&base).unwrap();
    assert_eq!(
        view.preparation_buffer_bytes(&height).unwrap(),
        scratch + 16
    );
    let mut limits = AssetLimits::default();
    limits.package.package_buffer_bytes = scratch;
    let limited = AssetView::load(VALID, &limits).unwrap();
    limited.prepare(&base).unwrap();
    assert_eq!(
        limited.prepare(&height).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    assert_eq!(
        OwnedAsset::copy_from(VALID, &limits).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    limits.package.package_buffer_bytes = VALID.len() as u64 + scratch + 16;
    let owned = OwnedAsset::copy_from(VALID, &limits).unwrap();
    owned.prepare(&height).unwrap();
    assert_eq!(
        owned.view().preparation_buffer_bytes(&height).unwrap(),
        owned.archive_capacity() as u64 + scratch + 16
    );
    let mut capacity = Vec::with_capacity(VALID.len() + 4096);
    capacity.extend_from_slice(VALID);
    assert_eq!(
        OwnedAsset::from_vec(capacity, &limits).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    limits = AssetLimits::default();
    limits.package.package_bytes = VALID.len() as u64 - 1;
    assert_eq!(
        AssetView::load(VALID, &limits).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    limits = AssetLimits::default();
    limits.resources.resource_bytes = 15;
    assert_eq!(
        AssetView::load(VALID, &limits).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
    limits = AssetLimits::default();
    limits.package.package_bytes += 1;
    assert_eq!(
        AssetView::load(VALID, &limits).unwrap_err().code(),
        "MIX_PACKAGE_LIMIT_EXCEEDED"
    );
}

#[test]
fn resource_free_raw_source_and_closed_full_graph() {
    let limits = AssetLimits::default();
    let source=br#" {"version":1,"nodes":[{"id":"base","type":"constant-color","version":1},{"id":"out","type":"material-output","version":1}],"edges":[{"from":{"nodeId":"base","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}]}
"#;
    let bytes = write(source, &[], &limits).unwrap();
    let view = AssetView::load(&bytes, &limits).unwrap();
    assert_eq!(view.source(), source);
    assert!(view.resources().is_empty());
    view.prepare(&request(vec![OutputChannel::Height])).unwrap();
    let image = ImageBinding {
        id: "unused",
        width: 2,
        height: 2,
        format: "rgba8-linear",
        bytes_per_row: 8,
        data: &[0; 16],
    };
    assert_eq!(
        write(source, &[image], &limits).unwrap_err().code(),
        "MIX_PACKAGE_INVALID"
    );
    let mut doc: serde_json::Value = serde_json::from_slice(source).unwrap();
    doc["nodes"].as_array_mut().unwrap().push(serde_json::json!({"id":"detached","type":"image-input","version":1,"parameters":{"resourceId":"unused"}}));
    let source = serde_json::to_vec(&doc).unwrap();
    assert_eq!(
        write(&source, &[], &limits).unwrap_err().code(),
        "MIX_PACKAGE_INVALID"
    );
    let bytes = write(&source, &[image], &limits).unwrap();
    let view = AssetView::load(&bytes, &limits).unwrap();
    assert!(
        view.prepare(&request(vec![OutputChannel::Height]))
            .unwrap()
            .resources()
            .is_empty()
    );
    assert_eq!(
        write(&source, &[image, image], &limits).unwrap_err().code(),
        "MIX_PACKAGE_INVALID"
    );
}
