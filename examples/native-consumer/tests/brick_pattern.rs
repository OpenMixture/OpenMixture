//! Independent public consumer with optional mandatory browser comparison.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::{Value, json};
fn cases() -> Vec<(&'static str, [u32; 2], Value)> {
    vec![
        ("offset", [1024, 1024], json!({})),
        ("aligned", [1024, 1024], json!({"layout":"aligned"})),
        ("wide", [1024, 1024], json!({"gap":0.2})),
        (
            "rect",
            [65, 3],
            json!({"columns":1,"rows":1,"layout":"aligned"}),
        ),
    ]
}
fn plan(size: [u32; 2], overrides: Value) -> mixture_core::RenderPlan {
    let request = CompileRequest {
        size,
        outputs: vec![OutputChannel::Height, OutputChannel::Normal],
        overrides: serde_json::from_value(overrides).unwrap(),
        ..Default::default()
    };
    let doc = MaterialDocument::decode(include_bytes!("brick-pattern.mix"), &request.limits)
        .unwrap()
        .into_validated(&request.limits)
        .unwrap();
    compile(&doc, &request).unwrap()
}
#[test]
fn public_brick_plan() {
    for (_, size, o) in cases() {
        assert_eq!(plan(size, o).passes().len(), 2);
    }
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn public_brick_pixels() {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        Ok("metal") => BackendPreference::Metal,
        _ => BackendPreference::Auto,
    };
    let context = pollster::block_on(GpuContext::request(GpuContextOptions {
        backend,
        software_adapter: std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() == Ok("1"),
        ..Default::default()
    }))
    .unwrap();
    let mut renderer = Renderer::new(context);
    let mut rows = vec![];
    let mut retained = vec![];
    for (id, size, o) in cases() {
        let plan = plan(size, o);
        let result = pollster::block_on(renderer.render(&plan)).unwrap();
        assert_eq!(result.report().allocations.live_bytes, 0);
        let repeated = pollster::block_on(renderer.render(&plan)).unwrap();
        let mut comparisons = vec![];
        for (channel, again) in result.channels().iter().zip(repeated.channels()) {
            assert_eq!(channel.pixels(), again.pixels());
            if let Ok(directory) = std::env::var("MIXTURE_BRICK_BROWSER_DIR") {
                let path = std::path::Path::new(&directory)
                    .join(format!("brick-{id}-{}.png", channel.channel.as_str()));
                let mut reader =
                    png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()))
                        .read_info()
                        .unwrap();
                assert_eq!([reader.info().width, reader.info().height], size);
                let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
                let info = reader.next_frame(&mut pixels).unwrap();
                assert_eq!(info.color_type, png::ColorType::Rgba);
                assert_eq!(pixels.len(), channel.pixels().len());
                let max = pixels
                    .iter()
                    .zip(channel.pixels())
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(
                    max <= 1,
                    "brick {id} {}: delta {max}",
                    channel.channel.as_str()
                );
                comparisons
                    .push(json!({"channel":channel.channel.as_str(),"maxComponentDelta":max}));
            }
        }
        rows.push(json!({"id":id,"planHash":plan.hash(),"browserComparison":comparisons,"execution":result.report()}));
        retained.push(result);
    }
    drop(renderer);
    assert!(
        retained
            .iter()
            .all(|r| r.channels().iter().all(|c| !c.pixels().is_empty()))
    );
    if let Ok(path) = std::env::var("MIXTURE_BRICK_EVIDENCE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({"ok":true,"cases":rows})).unwrap(),
        )
        .unwrap();
    }
}
