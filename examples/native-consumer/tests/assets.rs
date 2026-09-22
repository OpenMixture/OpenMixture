//! External CPU consumer: only public crates and consumer-owned source/pixels.
use mixture_asset::{AssetLimits, AssetView, OwnedAsset, write};
use mixture_core::{CompileRequest, ImageBinding, MaterialDocument, OutputChannel};

const SOURCE: &[u8] = br#"{"version":1,"nodes":[
 {"id":"base","type":"constant-color","version":1},
 {"id":"height","type":"image-input","version":1,"parameters":{"resourceId":"Input"}},
 {"id":"out","type":"material-output","version":1}],"edges":[
 {"from":{"nodeId":"base","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},
 {"from":{"nodeId":"height","portId":"value"},"to":{"nodeId":"out","portId":"height"}}],
 "exposedParameters":[{"id":"source","nodeId":"height","parameterId":"resourceId"}]}"#;

#[test]
fn public_asset_roundtrip_matches_loose_core_and_owns_prepared_pixels() {
    let limits = AssetLimits::default();
    let mut pixels = vec![128, 34, 56, 0];
    let binding = ImageBinding {
        id: "Input",
        width: 1,
        height: 1,
        format: "rgba8-linear",
        bytes_per_row: 4,
        data: &pixels,
    };
    let mut bytes = write(SOURCE, &[binding], &limits).unwrap();
    assert_eq!(bytes, write(SOURCE, &[binding], &limits).unwrap());
    let request = CompileRequest {
        size: [1, 1],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let document = MaterialDocument::decode(SOURCE, &limits.safety)
        .unwrap()
        .into_validated(&limits.safety)
        .unwrap();
    let loose = mixture_core::prepare(&document, &request, &[binding], &limits.resources).unwrap();
    pixels.fill(0);
    drop(pixels);
    let view = AssetView::load(&bytes, &limits).unwrap();
    assert_eq!(view.source(), SOURCE);
    assert_eq!(view.image("Input"), Some([128, 34, 56, 0].as_slice()));
    assert_eq!(view.inspect().schema_version, 1);
    let snapshot = view.prepare(&request).unwrap();
    assert_eq!(snapshot.plan().hash(), loose.plan().hash());
    let mut forbidden = request.clone();
    forbidden
        .overrides
        .insert("source".into(), serde_json::json!("Input"));
    assert_eq!(
        view.prepare(&forbidden).unwrap_err().code(),
        "MIX_PACKAGE_RESOURCE_OVERRIDE"
    );
    let owned = OwnedAsset::copy_from(&bytes, &limits).unwrap();
    drop(view);
    bytes.fill(0);
    drop(bytes);
    assert_eq!(
        owned.prepare(&request).unwrap().plan().hash(),
        snapshot.plan().hash()
    );
    drop(owned);
    assert_eq!(snapshot.resources()[0].data(), [128, 34, 56, 0]);
    let bytes = write(
        SOURCE,
        &[ImageBinding {
            id: "Input",
            width: 1,
            height: 1,
            format: "rgba8-linear",
            bytes_per_row: 4,
            data: &[128, 34, 56, 0],
        }],
        &limits,
    )
    .unwrap();
    let ptr = bytes.as_ptr();
    let owned = OwnedAsset::from_vec(bytes, &limits).unwrap();
    assert_eq!(owned.as_bytes().as_ptr(), ptr);
}
