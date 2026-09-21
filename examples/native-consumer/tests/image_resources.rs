//! Independent public Rust consumer of the M6A-04 imported-height material.
use mixture_core::{
    CompileRequest, ImageBinding, MaterialDocument, OutputChannel, plan::KernelId, prepare,
};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!("image-input.mix")).unwrap()
}
fn compile_source(v: &Value, weight: f64) -> mixture_core::PreparedRender {
    let req = CompileRequest {
        size: [1024, 1024],
        outputs: vec![OutputChannel::Height, OutputChannel::Normal],
        overrides: [("detailWeight".into(), json!(weight))].into(),
        ..Default::default()
    };
    let doc = MaterialDocument::decode(&serde_json::to_vec(v).unwrap(), &req.limits)
        .unwrap()
        .into_validated(&req.limits)
        .unwrap();
    let triangle = |n: u32| {
        let p = n % 256;
        if p < 128 { p } else { 255 - p }
    };
    let pixels: Vec<u8> = (0..1024)
        .flat_map(|y| {
            (0..1024).flat_map(move |x| {
                let r = (triangle(x) + triangle(y)) as u8;
                [r, 255 - r, 17, if (x + y) % 2 == 1 { 91 } else { 0 }]
            })
        })
        .collect();
    prepare(
        &doc,
        &req,
        &[ImageBinding {
            id: "heightSource",
            width: 1024,
            height: 1024,
            format: "rgba8-linear",
            bytes_per_row: 4096,
            data: &pixels,
        }],
        &Default::default(),
    )
    .unwrap()
}
#[test]
fn image_resource_public_plan() {
    assert!(
        compile_source(&source(), 0.5)
            .plan()
            .passes()
            .iter()
            .any(|p| p.kernel.id() == KernelId::ImageInput)
    );
}
#[test]
#[ignore = "requires GPU; executed by cargo xtask gpu-smoke"]
fn image_resource_public_gpu() {
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
    let mut rows = Vec::new();
    let mut owned = Vec::new();
    let mut retained = Vec::new();
    for weight in [0., 0.25, 0.5, 1.] {
        let plan = compile_source(&source(), weight);
        let result = pollster::block_on(renderer.render_prepared(&plan)).unwrap();
        assert_eq!(result.report().allocations.live_bytes, 0);
        if weight == 0. || weight == 1. {
            let mut direct = source();
            for e in direct["edges"].as_array_mut().unwrap() {
                if e["from"]["nodeId"] == "mix" {
                    e["from"]["nodeId"] = json!(if weight == 0. { "image" } else { "detail" });
                }
            }
            let reference =
                pollster::block_on(renderer.render_prepared(&compile_source(&direct, weight)))
                    .unwrap();
            for (a, b) in result.channels().iter().zip(reference.channels()) {
                assert_eq!(a.pixels(), b.pixels());
            }
        }
        let mut comparisons = Vec::new();
        if let Ok(directory) = std::env::var("MIXTURE_RESOURCE_BROWSER_DIR") {
            for channel in result.channels() {
                let path = std::path::Path::new(&directory).join(format!(
                    "resource-{weight}-{}.png",
                    channel.channel.as_str()
                ));
                let mut reader =
                    png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()))
                        .read_info()
                        .unwrap();
                assert_eq!((reader.info().width, reader.info().height), (1024, 1024));
                assert_eq!(reader.info().color_type, png::ColorType::Rgba);
                assert_eq!(reader.info().bit_depth, png::BitDepth::Eight);
                let mut browser = vec![0; reader.output_buffer_size().unwrap()];
                reader.next_frame(&mut browser).unwrap();
                assert_eq!(browser.len(), channel.pixels().len());
                let max = browser
                    .iter()
                    .zip(channel.pixels())
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(
                    max <= 1,
                    "{} weight {weight}: max component delta {max}",
                    channel.channel.as_str()
                );
                let changed = browser
                    .iter()
                    .zip(channel.pixels())
                    .filter(|(a, b)| a != b)
                    .count();
                comparisons.push(json!({"channel":channel.channel.as_str(),"maxComponentDelta":max,"changedComponents":changed,"components":browser.len(),"limit":1}));
            }
        }
        let pixels = result
            .channels()
            .iter()
            .find(|c| c.channel == OutputChannel::Height)
            .unwrap()
            .pixels();
        let lo = *pixels.iter().step_by(4).min().unwrap();
        let hi = *pixels.iter().step_by(4).max().unwrap();
        assert!(hi - lo > 20);
        let mut interior = 0u64;
        let mut seam = 0u64;
        for y in 0..1024 {
            for x in 0..1024 {
                let p = pixels[(y * 1024 + x) * 4];
                if x > 0 {
                    interior += u64::from(p.abs_diff(pixels[(y * 1024 + x - 1) * 4]));
                }
                if y > 0 {
                    interior += u64::from(p.abs_diff(pixels[((y - 1) * 1024 + x) * 4]));
                }
            }
            seam += u64::from(pixels[y * 1024 * 4].abs_diff(pixels[(y * 1024 + 1023) * 4]));
            seam += u64::from(pixels[y * 4].abs_diff(pixels[(1023 * 1024 + y) * 4]));
        }
        let ratio = (seam as f64 / 2048.) / (interior as f64 / (2. * 1024. * 1023.));
        assert!(ratio < 2.0, "tile seam ratio {ratio}");
        if let Some(previous) = owned.last() {
            let previous: &Vec<u8> = previous;
            assert!(
                previous.iter().zip(pixels).filter(|(a, b)| a != b).count() > pixels.len() / 10
            );
        }
        owned.push(pixels.to_vec());
        rows.push(json!({"weight":weight,"browserComparison":comparisons,"planHash":plan.plan().hash(),"range":[lo,hi],"heightSeamRatio":ratio,"execution":result.report()}));
        retained.push(result);
    }
    drop(renderer);
    for (result, before) in retained.iter().zip(&owned) {
        assert_eq!(
            result
                .channels()
                .iter()
                .find(|c| c.channel == OutputChannel::Height)
                .unwrap()
                .pixels(),
            before
        );
    }
    assert_eq!(owned.len(), 4);
    assert!(owned.iter().all(|v| v.len() == 1024 * 1024 * 4));
    let path = std::env::var("MIXTURE_RESOURCE_EVIDENCE")
        .unwrap_or_else(|_| "resource-evidence.json".into());
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&json!({"ok":true,"cases":rows,"ownedAfterDestroy":true}))
            .unwrap(),
    )
    .unwrap();
}
