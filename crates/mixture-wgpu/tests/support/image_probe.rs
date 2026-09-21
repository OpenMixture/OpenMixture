//! Literal bytes and algebraic endpoint tests through the production executor.
use super::*;
use mixture_core::{ImageBinding, PreparedRender, ResourceLimits, prepare};
fn document(direct: Option<&str>) -> Value {
    let mut v: Value = serde_json::from_slice(include_bytes!(
        "../../../../fixtures/nodes/image-input/height.mix"
    ))
    .unwrap();
    if let Some(node) = direct {
        for edge in v["edges"].as_array_mut().unwrap() {
            if edge["from"]["nodeId"] == "mix" {
                edge["from"]["nodeId"] = json!(node);
            }
        }
    }
    v
}
fn prepared(v: &Value, size: [u32; 2], bytes: &[u8], weight: f64) -> PreparedRender {
    let req = CompileRequest {
        size,
        outputs: vec![OutputChannel::Normal, OutputChannel::Height],
        overrides: [("detailWeight".into(), json!(weight))].into(),
        ..Default::default()
    };
    let doc = MaterialDocument::decode(&serde_json::to_vec(v).unwrap(), &req.limits)
        .unwrap()
        .into_validated(&req.limits)
        .unwrap();
    prepare(
        &doc,
        &req,
        &[ImageBinding {
            id: "heightSource",
            width: size[0],
            height: size[1],
            format: "rgba8-linear",
            bytes_per_row: u64::from(size[0]) * 4,
            data: bytes,
        }],
        &ResourceLimits::default(),
    )
    .unwrap()
}
fn channel(output: &mixture_wgpu::RenderOutput, id: OutputChannel) -> &[u8] {
    output
        .channels()
        .iter()
        .find(|c| c.channel == id)
        .unwrap()
        .pixels()
}
fn clean(output: &mixture_wgpu::RenderOutput) {
    let a = &output.report().allocations;
    assert_eq!(a.live_bytes, 0);
    assert_eq!(a.released_bytes, a.cumulative_bytes);
    assert!(a.peak_bytes <= output.report().estimates.peak_bytes);
    assert_eq!(a.resource_count, output.report().estimates.resource_count);
    assert_eq!(
        a.resource_upload_bytes,
        output.report().estimates.resource_upload_bytes
    );
    assert_eq!(
        a.resource_texture_bytes,
        output.report().estimates.resource_texture_bytes
    );
    assert_eq!(
        a.resource_staging_bytes,
        output.report().estimates.resource_staging_bytes
    );
}
pub fn run() {
    let mut renderer = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    let mut evidence = Vec::new();
    for size in [[1, 1], [1, 256], [256, 1], [2, 2], [65, 3], [1024, 1024]] {
        let bytes: Vec<_> = (0..size[0] * size[1])
            .flat_map(|i| {
                let r = if size == [2, 2] {
                    [0, 64, 128, 255][i as usize]
                } else {
                    (i % 256) as u8
                };
                [r, 255 - r, 17, if i % 2 == 0 { 0 } else { 91 }]
            })
            .collect();
        let mut mutable = bytes.clone();
        let p = prepared(&document(Some("image")), size, &mutable, 0.);
        mutable.fill(0);
        drop(mutable);
        for iteration in 0..3 {
            let out = pollster::block_on(renderer.render_prepared(&p)).unwrap();
            clean(&out);
            for (expected, actual) in bytes.as_chunks::<4>().0.iter().zip(
                channel(&out, OutputChannel::Height)
                    .as_chunks::<4>()
                    .0
                    .iter(),
            ) {
                assert_eq!(*actual, [expected[0], expected[0], expected[0], 255]);
            }
            if size == [1, 1] || size == [2, 2] {
                assert!(
                    channel(&out, OutputChannel::Normal)
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .all(|p| *p == [128, 128, 255, 255])
                );
            }
            evidence.push(json!({"size":size,"iteration":iteration,"execution":out.report()}));
        }
        let original = pollster::block_on(renderer.render_prepared(&p)).unwrap();
        if size == [65, 3] {
            let h = channel(&original, OutputChannel::Height);
            assert_eq!(h[0], 0);
            assert_eq!(h[64 * 4], 64); // No automatic seam repair.
            let n = channel(&original, OutputChannel::Normal);
            assert!(n[0] > 128 && n[4] < 128); // Wrapped discontinuity versus interior +X slope.
            assert!(n[1] < 128 && n[(65 + 1) * 4 + 1] > 128); // Top-left orientation, tangent +Y up.
        }
        let mut other = bytes.clone();
        for pixel in other.as_chunks_mut::<4>().0.iter_mut() {
            pixel[1] ^= 255;
            pixel[2] ^= 255;
            pixel[3] ^= 255;
        }
        let q = prepared(&document(Some("image")), size, &other, 0.);
        assert_ne!(p.plan().hash(), q.plan().hash());
        let out = pollster::block_on(renderer.render_prepared(&q)).unwrap();
        for (a, b) in original.channels().iter().zip(out.channels()) {
            assert_eq!(a.pixels(), b.pixels());
        }
        let cached = renderer.cached_pipeline_count();
        let err = pollster::block_on(renderer.render(p.plan())).unwrap_err();
        assert_eq!(err.diagnostic().code.as_str(), "MIX_RESOURCE_MISSING");
        assert!(err.allocations().is_none());
        assert_eq!(cached, renderer.cached_pipeline_count());
        // Shared ID references create two image passes, but only one upload.
        let mut shared = document(Some("image"));
        shared["nodes"].as_array_mut().unwrap().push(json!({"id":"second","type":"image-input","version":1,"parameters":{"resourceId":"heightSource"}}));
        for e in shared["edges"].as_array_mut().unwrap() {
            if e["to"]["nodeId"] == "normal" {
                e["from"]["nodeId"] = json!("second");
            }
        }
        let out =
            pollster::block_on(renderer.render_prepared(&prepared(&shared, size, &bytes, 0.)))
                .unwrap();
        clean(&out);
        assert_eq!(out.report().allocations.resource_count, 1);
        assert_eq!(
            channel(&out, OutputChannel::Height),
            channel(&original, OutputChannel::Height)
        );
        // Same ID replacement must use new bytes without growing the kernel cache.
        other
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|p| p[0] = 255 - p[0]);
        let replaced = prepared(&document(Some("image")), size, &other, 0.);
        let out = pollster::block_on(renderer.render_prepared(&replaced)).unwrap();
        clean(&out);
        for (input, pixel) in other.as_chunks::<4>().0.iter().zip(
            channel(&out, OutputChannel::Height)
                .as_chunks::<4>()
                .0
                .iter(),
        ) {
            assert_eq!(*pixel, [input[0], input[0], input[0], 255]);
        }
        assert_eq!(cached, renderer.cached_pipeline_count());
    }
    let size = [1024, 1024];
    // Frozen periodic 64-texel triangular tile: edges agree; asymmetric y phase.
    let bytes: Vec<u8> = (0..1024u32 * 1024)
        .flat_map(|i| {
            let x = i % 1024 % 64;
            let y = i / 1024 % 64;
            let r = ((x.min(63 - x) * 5 + y.min(63 - y) * 3) % 256) as u8;
            [r, 19, 201, 0]
        })
        .collect();
    let mut heights = Vec::new();
    for weight in [0., 0.25, 0.5, 1.] {
        let p = prepared(&document(None), size, &bytes, weight);
        let out = pollster::block_on(renderer.render_prepared(&p)).unwrap();
        clean(&out);
        if weight == 0. || weight == 1. {
            let direct = prepared(
                &document(Some(if weight == 0. { "image" } else { "detail" })),
                size,
                &bytes,
                weight,
            );
            let expected = pollster::block_on(renderer.render_prepared(&direct)).unwrap();
            for (a, b) in out.channels().iter().zip(expected.channels()) {
                assert_eq!(a.pixels(), b.pixels());
            }
        }
        for c in out.channels() {
            assert!(c.pixels().as_chunks::<4>().0.iter().all(|p| p[3] == 255));
        }
        if let Ok(dir) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            for (name, id) in [
                ("height", OutputChannel::Height),
                ("normal", OutputChannel::Normal),
            ] {
                std::fs::write(
                    Path::new(&dir).join(format!("image-input-{weight}-{name}.rgba")),
                    channel(&out, id),
                )
                .unwrap();
            }
        }
        heights.push(channel(&out, OutputChannel::Height).to_vec());
        evidence.push(json!({"weight":weight,"execution":out.report()}));
    }
    for pair in heights.windows(2) {
        assert!(pair[0].iter().zip(&pair[1]).filter(|(a, b)| a != b).count() > 1024);
    }
    for i in (0..heights[0].len()).step_by(4) {
        for (index, weight) in [(1, 0.25), (2, 0.5)] {
            let expected =
                f64::from(heights[0][i]) * (1. - weight) + f64::from(heights[3][i]) * weight;
            assert!((f64::from(heights[index][i]) - expected).abs() <= 1.);
        }
    }
    assert_eq!(renderer.cached_pipeline_count(), 4);
    let p = prepared(&document(Some("image")), [1, 1], &[64, 1, 2, 0], 0.);
    let owned = pollster::block_on(renderer.render_prepared(&p)).unwrap();
    renderer.context().device().destroy();
    for _ in 0..3 {
        let error = pollster::block_on(renderer.render_prepared(&p)).unwrap_err();
        assert_eq!(error.allocations().unwrap().live_bytes, 0);
        assert_eq!(error.allocations().unwrap().cumulative_bytes, 0);
        assert!(error.device_loss().is_some());
    }
    assert_eq!(renderer.cached_pipeline_count(), 0);
    drop(renderer);
    drop(p);
    assert_eq!(channel(&owned, OutputChannel::Height), [64, 64, 64, 255]);
    if let Ok(dir) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            Path::new(&dir).join("image-input.json"),
            serde_json::to_vec_pretty(
                &json!({"ok":true,"cases":evidence,"ownedOutputAfterDestroy":true}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
