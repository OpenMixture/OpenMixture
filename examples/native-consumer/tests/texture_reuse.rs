//! Public release-mode PERF-MAT execution; material aesthetics remain separate.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, RenderOutput, Renderer};
use serde_json::{Value, json};
use std::{path::Path, time::Instant};

fn renderer() -> Renderer {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        Ok("metal") => BackendPreference::Metal,
        other => panic!("explicit GPU backend required: {other:?}"),
    };
    let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        other => panic!("explicit GPU policy required: {other:?}"),
    };
    Renderer::new(
        pollster::block_on(GpuContext::request(GpuContextOptions {
            backend,
            software_adapter,
            ..Default::default()
        }))
        .unwrap(),
    )
}

fn read_png(path: &Path, size: [u32; 2]) -> Vec<u8> {
    let mut reader = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()))
        .read_info()
        .unwrap();
    assert_eq!([reader.info().width, reader.info().height], size);
    assert_eq!(reader.info().color_type, png::ColorType::Rgba);
    assert_eq!(reader.info().bit_depth, png::BitDepth::Eight);
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    bytes.truncate(info.buffer_size());
    assert_eq!(bytes.len(), size[0] as usize * size[1] as usize * 4);
    bytes
}

fn same(a: &RenderOutput, b: &RenderOutput) {
    assert_eq!(a.channels().len(), b.channels().len());
    for (a, b) in a.channels().iter().zip(b.channels()) {
        assert_eq!(a.channel, b.channel);
        assert_eq!(a.pixels(), b.pixels());
    }
}

#[test]
#[ignore = "requires release GPU and fresh PERF-MAT evidence directory"]
fn public_reuse_preserves_outputs_packages_and_2k_budget() {
    if cfg!(debug_assertions) {
        panic!("timings require release mode");
    }
    let root = std::env::var("MIXTURE_REUSE_ROOT").unwrap();
    let root = Path::new(&root);
    let destination = std::env::var("MIXTURE_REUSE_EVIDENCE").unwrap();
    let destination = Path::new(&destination);
    std::fs::create_dir(destination).unwrap();
    let receipt = destination.join("native.json");
    std::fs::write(&receipt, b"{\"ok\":false,\"completed\":false}").unwrap();
    let source = std::fs::read(root.join("docs/evidence/perf-mat-before/material.mix")).unwrap();
    let contract: Value = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/materials/painted-metal/qualification-plan.json"))
            .unwrap(),
    )
    .unwrap();
    let archive = mixture_asset::write(&source, &[], &Default::default()).unwrap();
    assert_eq!(
        archive,
        mixture_asset::write(&source, &[], &Default::default()).unwrap()
    );
    std::fs::write(destination.join("material.mixpack"), &archive).unwrap();
    let asset = mixture_asset::AssetView::load(&archive, &Default::default()).unwrap();
    assert_eq!(asset.source(), source);
    let browser_directory = std::env::var("MIXTURE_REUSE_BROWSER").ok();
    let browser: Option<Value> = browser_directory.as_ref().map(|p| {
        serde_json::from_slice(&std::fs::read(Path::new(p).join("reuse-browser.json")).unwrap())
            .unwrap()
    });
    let cases = [
        (
            "odd",
            [65, 3],
            json!({"radiusX":1,"radiusY":1,"macroSeed":4294967295_u32}),
        ),
        (
            "intact",
            [257, 129],
            json!({"radiusX":1,"radiusY":1,"exposureInputMin":0,"exposureInputMax":1,"exposureOutputMin":0,"exposureOutputMax":0}),
        ),
        ("default-1k", [1024, 1024], json!({})),
        ("default-2k", [2048, 2048], json!({"radiusX":8,"radiusY":8})),
    ];
    let mut rows = Vec::new();
    let mut timing = Vec::new();
    let mut before_exact = None;
    for (id, size, overrides) in cases {
        let request = CompileRequest {
            size,
            outputs: vec![
                OutputChannel::BaseColor,
                OutputChannel::Normal,
                OutputChannel::Roughness,
                OutputChannel::Metallic,
                OutputChannel::Height,
            ],
            overrides: serde_json::from_value(overrides.clone()).unwrap(),
            ..Default::default()
        };
        let document = MaterialDocument::decode(&source, &request.limits)
            .unwrap()
            .into_validated(&request.limits)
            .unwrap();
        let plan = compile(&document, &request).unwrap();
        assert_eq!(plan.version(), 3);
        assert_eq!(plan.passes().len(), 23);
        assert_eq!(plan.estimates().texture_count, 14);
        let mut gpu = renderer();
        let start = Instant::now();
        let output = pollster::block_on(gpu.render(&plan)).unwrap();
        let cold = start.elapsed().as_secs_f64() * 1000.;
        if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
            assert!(
                output.report().adapter.name.contains(&expected),
                "unexpected adapter"
            );
        }
        let a = &output.report().allocations;
        assert_eq!(a.texture_count, plan.estimates().texture_count);
        assert_eq!(a.peak_bytes, plan.estimates().peak_bytes);
        assert_eq!(
            a.reused_bytes,
            plan.estimates().logical_texture_bytes - plan.estimates().texture_bytes
        );
        assert!(a.reused_bytes > 0);
        assert_eq!(a.live_bytes, 0);
        assert_eq!(a.released_bytes, a.cumulative_bytes);
        assert!(a.peak_bytes <= contract["maxDescriptorPeakBytes"].as_u64().unwrap());
        if size == [2048, 2048] {
            assert_eq!(a.peak_bytes, 503_316_944);
        }
        let samples = if size[0] >= 1024 {
            contract["timing"]["warmSamples"].as_u64().unwrap()
        } else {
            1
        };
        let mut warm = Vec::new();
        for _ in 0..samples {
            let start = Instant::now();
            let repeat = pollster::block_on(gpu.render(&plan)).unwrap();
            warm.push(start.elapsed().as_secs_f64() * 1000.);
            same(&output, &repeat);
            assert_eq!(repeat.report().allocations, *a);
        }
        let prepared = asset.prepare(&request).unwrap();
        assert_eq!(prepared.plan().hash(), plan.hash());
        same(
            &output,
            &pollster::block_on(gpu.render_prepared(&prepared)).unwrap(),
        );
        let mut partial_request = request.clone();
        partial_request.outputs = vec![OutputChannel::Height];
        let partial = compile(&document, &partial_request).unwrap();
        let sliced = pollster::block_on(gpu.render(&partial)).unwrap();
        assert_eq!(
            sliced.channels()[0].pixels(),
            output
                .channels()
                .iter()
                .find(|c| c.channel == OutputChannel::Height)
                .unwrap()
                .pixels()
        );
        if size[0] >= 1024 {
            let budget = ["hardware", "software"].into_iter().find_map(|key| {
                let b = &contract["timing"][key];
                output
                    .report()
                    .adapter
                    .name
                    .contains(b["adapter"].as_str().unwrap())
                    .then_some(b)
            });
            let mut ordered = warm.clone();
            ordered.sort_by(f64::total_cmp);
            let median = ordered[ordered.len() / 2];
            let passed = budget.map(|b| {
                median
                    <= b[if size[0] == 1024 {
                        "warmMedian1024Ms"
                    } else {
                        "warmMedian2048Ms"
                    }]
                    .as_f64()
                    .unwrap()
                    && (size[0] != 1024 || cold <= b["cold1024Ms"].as_f64().unwrap())
            });
            timing.push(json!({"size":size,"adapter":output.report().adapter,"coldMs":cold,"warmMs":warm,"warmMedianMs":median,"budgetPassed":passed}));
        }
        drop(gpu);
        let mut comparisons = Vec::new();
        for c in output.channels() {
            let filename = format!("{id}-{}.png", c.channel.as_str());
            let file = std::fs::File::create(destination.join(&filename)).unwrap();
            let mut encoder = png::Encoder::new(file, size[0], size[1]);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(c.pixels())
                .unwrap();
            if let Some(directory) = &browser_directory {
                let other = read_png(&Path::new(directory).join(filename), size);
                let max = other
                    .iter()
                    .zip(c.pixels())
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                comparisons.push(json!({"channel":c.channel.as_str(),"maxComponentDelta":max}));
            }
            if id == "default-1k"
                && let Ok(directory) = std::env::var("MIXTURE_REUSE_BEFORE")
            {
                assert!(
                    output
                        .report()
                        .adapter
                        .name
                        .contains("NVIDIA GeForce GT 1030")
                );
                assert_eq!(std::env::var("MIXTURE_GPU_BACKEND").unwrap(), "vulkan");
                let other = read_png(
                    &Path::new(&directory).join(format!("{}-1024.png", c.channel.as_str())),
                    size,
                );
                assert_eq!(other, c.pixels(), "same-adapter before/after pixels");
                before_exact = Some(true);
            }
        }
        if let Some(browser) = &browser {
            let row = &browser["rows"][rows.len()];
            assert_eq!(row["id"], id);
            assert_eq!(row["size"], json!(size));
            assert_eq!(row["overrides"], overrides);
            assert_eq!(row["planHash"], plan.hash().as_str());
            assert_eq!(
                row["allocation"],
                serde_json::to_value(plan.allocation()).unwrap()
            );
        }
        rows.push(json!({"id":id,"size":size,"planHash":plan.hash().as_str(),"report":output.report(),"repeatExact":true,"packageExact":true,"slicedExact":true,"ownedAfterDestroy":true,"comparisons":comparisons}));
    }
    if let Some(browser) = &browser {
        assert_eq!(browser["rows"].as_array().unwrap().len(), rows.len());
    }
    let ok = timing.iter().all(|v| v["budgetPassed"] != false)
        && rows.iter().all(|r| {
            r["comparisons"].as_array().unwrap().iter().all(|c| {
                c["maxComponentDelta"].as_u64().unwrap()
                    <= contract["maxCrossRuntimeComponentError"].as_u64().unwrap()
            })
        });
    std::fs::write(receipt, serde_json::to_vec_pretty(&json!({"ok":ok,"completed":true,"debugAssertions":cfg!(debug_assertions),"browserCompared":browser.is_some(),"beforeExact":before_exact,"rows":rows,"timing":timing,"materialAccepted":false})).unwrap()).unwrap();
    assert!(ok, "retained PERF-MAT timing or parity failure");
}
