//! Independent public Rust consumer of the ENG-04 two-noise material.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile, plan::KernelId};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::{Value, json};
fn source() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/nodes/scalar-blend/two-noise.mix"
    ))
    .unwrap()
}
fn compile_source(v: &Value, weight: f64) -> mixture_core::RenderPlan {
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
    compile(&doc, &req).unwrap()
}
#[test]
fn scalar_blend_public_plan() {
    assert!(
        compile_source(&source(), 0.5)
            .passes()
            .iter()
            .any(|p| p.kernel.id() == KernelId::ScalarBlend)
    );
}
#[test]
#[ignore = "requires GPU; executed by cargo xtask gpu-smoke"]
fn scalar_blend_public_gpu() {
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
        let result = pollster::block_on(renderer.render(&plan)).unwrap();
        assert_eq!(result.report().allocations.live_bytes, 0);
        if weight == 0. || weight == 1. {
            let mut direct = source();
            for e in direct["edges"].as_array_mut().unwrap() {
                if e["from"]["nodeId"] == "combine" {
                    e["from"]["nodeId"] = json!(if weight == 0. { "a" } else { "b" });
                }
            }
            let reference =
                pollster::block_on(renderer.render(&compile_source(&direct, weight))).unwrap();
            for (a, b) in result.channels().iter().zip(reference.channels()) {
                assert_eq!(a.pixels(), b.pixels());
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
        rows.push(json!({"weight":weight,"planHash":plan.hash(),"range":[lo,hi],"heightSeamRatio":ratio,"execution":result.report()}));
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
    let path = std::env::var("MIXTURE_SCALAR_EVIDENCE")
        .unwrap_or_else(|_| "scalar-blend-evidence.json".into());
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&json!({"ok":true,"cases":rows,"ownedAfterDestroy":true}))
            .unwrap(),
    )
    .unwrap();
}
