//! The consumer owns its source/pixels and uses only published Rust API surfaces.
use mixture_core::{
    CompileRequest, ImageBinding, MaterialDocument, OutputChannel, ResourceLimits, SafetyLimits,
    prepare,
};

const SOURCE: &[u8] = br#"{"version":1,"nodes":[
      {"id":"color","type":"constant-color","version":1},
      {"id":"height","type":"image-input","version":1,"parameters":{"resourceId":"source"}},
      {"id":"out","type":"material-output","version":1}],
      "edges":[
        {"from":{"nodeId":"color","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},
        {"from":{"nodeId":"height","portId":"value"},"to":{"nodeId":"out","portId":"height"}}]}"#;
#[test]
fn public_core_resource_preparation_owns_input_and_rejects_missing_bindings() {
    let document = MaterialDocument::decode(SOURCE, &SafetyLimits::default())
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
    let adapted = mixture_core::prepare_from(
        &document,
        &request,
        &[mixture_core::AdapterImageBinding {
            id: "source",
            width: 1,
            height: 1,
            format: "rgba8-linear",
            bytes_per_row: 4,
            data: pixels.as_slice(),
        }],
        &ResourceLimits::default(),
    )
    .unwrap();
    assert_eq!(adapted.plan().hash(), prepared.plan().hash());
    pixels.fill(0);
    drop(pixels);
    assert_eq!(prepared.resources()[0].data(), &[128, 34, 56, 0]);
    assert_eq!(prepared.plan().version(), 3);
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

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn public_native_prepared_image_survives_input_and_renderer_drop() {
    use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("metal") => BackendPreference::Metal,
        Ok("dx12") => BackendPreference::Dx12,
        _ => BackendPreference::Auto,
    };
    let mut renderer = Renderer::new(
        pollster::block_on(GpuContext::request(GpuContextOptions {
            backend,
            software_adapter: std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() == Ok("1"),
            ..Default::default()
        }))
        .unwrap(),
    );
    let doc = MaterialDocument::decode(SOURCE, &SafetyLimits::default())
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
    let request = CompileRequest {
        size: [65, 3],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let mut bytes: Vec<_> = (0..195)
        .flat_map(|n| [(n % 256) as u8, 219, 73, 0])
        .collect();
    let prepared = prepare(
        &doc,
        &request,
        &[ImageBinding {
            id: "source",
            width: 65,
            height: 3,
            format: "rgba8-linear",
            bytes_per_row: 260,
            data: &bytes,
        }],
        &ResourceLimits::default(),
    )
    .unwrap();
    bytes.fill(255);
    drop(bytes);
    let error = pollster::block_on(renderer.render(prepared.plan())).unwrap_err();
    assert_eq!(error.diagnostic().code.as_str(), "MIX_RESOURCE_MISSING");
    assert_eq!(renderer.cached_pipeline_count(), 0);
    let mut outputs = Vec::new();
    for _ in 0..8 {
        let output = pollster::block_on(renderer.render_prepared(&prepared)).unwrap();
        let a = &output.report().allocations;
        assert_eq!(a.resource_count, 1);
        assert_eq!(a.resource_upload_bytes, 780);
        assert_eq!(a.resource_texture_bytes, 780);
        assert_eq!(a.resource_staging_bytes, 1536);
        assert_eq!(a.live_bytes, 0);
        assert_eq!(a.released_bytes, a.cumulative_bytes);
        outputs.push(output);
    }
    assert_eq!(renderer.cached_pipeline_count(), 1);
    drop(prepared);
    drop(renderer);
    for output in outputs {
        for (n, pixel) in output.channels()[0]
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
        {
            assert_eq!(*pixel, [n as u8, n as u8, n as u8, 255]);
        }
    }
}
