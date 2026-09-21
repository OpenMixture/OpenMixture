//! The consumer owns its source/pixels and uses only published Rust API surfaces.
use mixture_core::{
    CompileRequest, ImageBinding, MaterialDocument, OutputChannel, ResourceLimits, SafetyLimits,
    prepare,
};

#[test]
fn public_core_resource_preparation_owns_input_and_rejects_missing_bindings() {
    let source = br#"{"version":1,"nodes":[
      {"id":"color","type":"constant-color","version":1},
      {"id":"height","type":"image-input","version":1,"parameters":{"resourceId":"source"}},
      {"id":"out","type":"material-output","version":1}],
      "edges":[
        {"from":{"nodeId":"color","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},
        {"from":{"nodeId":"height","portId":"value"},"to":{"nodeId":"out","portId":"height"}}]}"#;
    let document = MaterialDocument::decode(source, &SafetyLimits::default())
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let request = CompileRequest {
        size: [1, 1],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let mut pixels = vec![128, 34, 56, 0];
    let prepared = prepare(
        &document,
        &request,
        &[ImageBinding {
            id: "source",
            width: 1,
            height: 1,
            format: "rgba8-linear",
            bytes_per_row: 4,
            data: &pixels,
        }],
        &ResourceLimits::default(),
    )
    .unwrap();
    pixels.fill(0);
    drop(pixels);
    assert_eq!(prepared.resources()[0].data(), &[128, 34, 56, 0]);
    assert_eq!(prepared.plan().version(), 2);
    assert_eq!(
        prepared.plan().image_resources()[0],
        *prepared.resources()[0].image()
    );
    let missing = prepare(&document, &request, &[], &ResourceLimits::default()).unwrap_err();
    assert_eq!(
        missing.report().diagnostics()[0].code.as_str(),
        "MIX_RESOURCE_MISSING"
    );
}
