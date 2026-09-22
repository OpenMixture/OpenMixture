//! MAT-01 public API measurements. This receipt alone does not accept the material.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, RenderPlan, compile};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, RenderOutput, Renderer};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, time::Instant};

fn request(controls: &Value, changes: &Value, size: [u32; 2]) -> CompileRequest {
    let mut request = CompileRequest {
        size,
        outputs: vec![
            OutputChannel::BaseColor,
            OutputChannel::Normal,
            OutputChannel::Roughness,
            OutputChannel::Height,
        ],
        ..Default::default()
    };
    for (control, value) in changes.as_object().unwrap() {
        for id in controls[control].as_array().unwrap() {
            request
                .overrides
                .insert(id.as_str().unwrap().into(), value.clone());
        }
    }
    request
}

fn plan(source: &[u8], controls: &Value, changes: &Value, size: [u32; 2]) -> RenderPlan {
    let request = request(controls, changes, size);
    let document = MaterialDocument::decode(source, &request.limits)
        .unwrap()
        .into_validated(&request.limits)
        .unwrap();
    compile(&document, &request).unwrap()
}

fn renderer() -> Renderer {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        Ok("metal") => BackendPreference::Metal,
        other => panic!("explicit MIXTURE_GPU_BACKEND required: {other:?}"),
    };
    let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Ok("1") => true,
        Ok("0") => false,
        other => panic!("explicit MIXTURE_GPU_SOFTWARE=0/1 required: {other:?}"),
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

fn pixels(output: &RenderOutput, channel: OutputChannel) -> &[u8] {
    output
        .channels()
        .iter()
        .find(|c| c.channel == channel)
        .unwrap()
        .pixels()
}

// Compare delivered byte channels, not a reimplementation of any pixel kernel.
fn downsample_error(low: &[u8], high: &[u8], size: usize) -> [f64; 3] {
    assert_eq!(low.len(), size * size * 4);
    assert_eq!(high.len(), size * size * 64);
    let mut total = [0.; 3];
    for y in 0..size {
        for x in 0..size {
            for c in 0..3 {
                let mut sum = 0u32;
                for dy in 0..4 {
                    for dx in 0..4 {
                        sum += u32::from(high[((y * 4 + dy) * size * 4 + x * 4 + dx) * 4 + c]);
                    }
                }
                total[c] += (f64::from(low[(y * size + x) * 4 + c]) - f64::from(sum) / 16.).abs();
            }
        }
    }
    total.map(|v| v / (size * size) as f64)
}

fn structural_checks(source: &[u8], controls: &Value, gpu: &mut Renderer) -> Value {
    let render = |gpu: &mut Renderer, changes: Value| {
        pollster::block_on(gpu.render(&plan(source, controls, &changes, [256, 256]))).unwrap()
    };
    let base = render(gpu, json!({}));
    let height = pixels(&base, OutputChannel::Height);
    let mut checked = Vec::new();
    for (control, value) in [("mortarX", 0.2), ("mortarY", 0.2), ("bevel", 0.2)] {
        let changed = render(gpu, json!({control:value}));
        let changed_height = pixels(&changed, OutputChannel::Height);
        assert!(
            changed_height
                .iter()
                .step_by(4)
                .zip(height.iter().step_by(4))
                .all(|(a, b)| a <= b)
        );
        assert!(
            changed_height
                .iter()
                .step_by(4)
                .zip(height.iter().step_by(4))
                .any(|(a, b)| a < b)
        );
        checked.push(control);
    }
    for (control, changed_channels, stable_channels) in [
        (
            "heightVariation",
            vec![OutputChannel::Height, OutputChannel::Normal],
            vec![OutputChannel::BaseColor, OutputChannel::Roughness],
        ),
        (
            "colorVariation",
            vec![OutputChannel::BaseColor],
            vec![
                OutputChannel::Height,
                OutputChannel::Normal,
                OutputChannel::Roughness,
            ],
        ),
    ] {
        let changed = render(gpu, json!({control:0.8}));
        for channel in changed_channels {
            assert_ne!(
                pixels(&base, channel),
                pixels(&changed, channel),
                "{control}: expected effect"
            );
        }
        for channel in stable_channels {
            assert_eq!(
                pixels(&base, channel),
                pixels(&changed, channel),
                "{control}: leaked effect"
            );
        }
        checked.push(control);
    }
    let zero = render(gpu, json!({"variation":0,"seed":0}));
    let zero_other = render(gpu, json!({"variation":0,"seed":4294967295_u32}));
    for channel in zero.channels() {
        assert_eq!(
            channel.pixels(),
            pixels(&zero_other, channel.channel),
            "zero variation seed independence"
        );
    }
    let other_seed = render(gpu, json!({"seed":4294967295_u32}));
    assert_eq!(
        pixels(&base, OutputChannel::Roughness),
        pixels(&other_seed, OutputChannel::Roughness)
    );
    assert_ne!(height, pixels(&other_seed, OutputChannel::Height));
    let regular = render(gpu, json!({"rowOffset":0}));
    let regular_height = pixels(&regular, OutputChannel::Height);
    // Even rows retain phase, odd rows move by half a cell.
    assert_eq!(
        &height[16 * 256 * 4..17 * 256 * 4],
        &regular_height[16 * 256 * 4..17 * 256 * 4]
    );
    assert_ne!(
        &height[48 * 256 * 4..49 * 256 * 4],
        &regular_height[48 * 256 * 4..49 * 256 * 4]
    );
    let runs = |values: Vec<bool>| {
        values
            .iter()
            .enumerate()
            .filter(|(i, v)| **v && (*i == 0 || !values[*i - 1]))
            .count()
    };
    for control in ["columns", "rows"] {
        let changed = render(gpu, json!({control:16}));
        let values = pixels(&changed, OutputChannel::Height);
        let occupied = if control == "columns" {
            (0..256).map(|x| values[(16 * 256 + x) * 4] > 128).collect()
        } else {
            (0..256)
                .map(|y| (0..256).any(|x| values[(y * 256 + x) * 4] > 128))
                .collect()
        };
        assert_eq!(runs(occupied), 16, "{control} count");
        checked.push(control);
    }
    json!({"monotoneAndIndependentControls":checked,"seedChangesHeightPreservesMask":true,
        "zeroVariationSeedIndependent":true,"alternatingRowPhase":true})
}

fn save(output: &RenderOutput, directory: &Path, stem: &str) {
    for channel in output.channels() {
        let file = std::fs::File::create(
            directory.join(format!("{stem}-{}.png", channel.channel.as_str())),
        )
        .unwrap();
        let mut encoder = png::Encoder::new(file, channel.size[0], channel.size[1]);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        if channel.channel == OutputChannel::BaseColor {
            encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        } else {
            encoder.set_source_gamma(png::ScaledFloat::new(1.));
        }
        encoder
            .write_header()
            .unwrap()
            .write_image_data(channel.pixels())
            .unwrap();
    }
}

fn compare_browser(output: &RenderOutput, directory: &Path, stem: &str) -> Vec<Value> {
    output.channels().iter().map(|channel| {
        let file = std::fs::File::open(directory.join(format!("{stem}-{}.png", channel.channel.as_str()))).unwrap();
        let mut reader = png::Decoder::new(std::io::BufReader::new(file)).read_info().unwrap();
        assert_eq!([reader.info().width, reader.info().height], channel.size);
        assert_eq!(reader.info().color_type, png::ColorType::Rgba);
        assert_eq!(reader.info().bit_depth, png::BitDepth::Eight);
        assert_eq!(reader.output_buffer_size().unwrap(), channel.pixels().len());
        let mut bytes = vec![0; channel.pixels().len()];
        reader.next_frame(&mut bytes).unwrap();
        let max = bytes.iter().zip(channel.pixels()).map(|(a,b)| a.abs_diff(*b)).max().unwrap();
        json!({"channel":channel.channel.as_str(),"maxComponentDelta":max,"components":bytes.len()})
    }).collect()
}

#[test]
fn downsample_metric_detects_each_rgb_component_and_ignores_alpha() {
    let low = [10, 20, 30, 0];
    let high = [10, 20, 30, 255].repeat(16);
    assert_eq!(downsample_error(&low, &high, 1), [0.; 3]);
    assert_eq!(downsample_error(&[15, 20, 22, 255], &high, 1), [5., 0., 8.]);
}

#[test]
#[ignore = "explicit MAT-01 fixture/evidence directories and GPU policy required"]
fn brick_material_public_gpu_matrix() {
    let fixture = std::env::var("MIXTURE_BRICK_FIXTURE_DIR").unwrap();
    let fixture = Path::new(&fixture);
    let destination = std::env::var("MIXTURE_BRICK_EVIDENCE_DIR").unwrap();
    let destination = Path::new(&destination);
    // Never overwrite an earlier receipt, including a failed one.
    std::fs::create_dir(destination).unwrap();
    let report_path = destination.join("native-matrix.json");
    std::fs::write(&report_path, br#"{"ok":false,"completed":false}"#).unwrap();
    let source = std::fs::read(fixture.join("material.mix")).unwrap();
    let controls: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("controls.json")).unwrap()).unwrap();
    let matrix: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("qualification-plan.json")).unwrap())
            .unwrap();
    for name in ["material.mix", "controls.json", "qualification-plan.json"] {
        std::fs::copy(fixture.join(name), destination.join(name)).unwrap();
    }
    let asset_limits = mixture_asset::AssetLimits::default();
    let archive = mixture_asset::write(&source, &[], &asset_limits).unwrap();
    assert_eq!(
        archive,
        mixture_asset::write(&source, &[], &asset_limits).unwrap()
    );
    std::fs::write(destination.join("material.mixpack"), &archive).unwrap();
    let asset = mixture_asset::AssetView::load(&archive, &asset_limits).unwrap();
    assert_eq!(asset.source(), source);
    let mut gpu = renderer();
    let browser_directory = std::env::var("MIXTURE_BRICK_BROWSER_DIR").ok();
    let mut rows = Vec::new();
    let mut defaults = BTreeMap::new();
    for case in matrix["cases"].as_array().unwrap() {
        for size in matrix["sizes"].as_array().unwrap() {
            let size = [
                size[0].as_u64().unwrap() as u32,
                size[1].as_u64().unwrap() as u32,
            ];
            let plan = plan(&source, &controls, &case["overrides"], size);
            let output = pollster::block_on(gpu.render(&plan)).unwrap();
            if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
                assert!(
                    output.report().adapter.name.contains(&expected),
                    "unexpected adapter"
                );
            }
            let repeat = pollster::block_on(gpu.render(&plan)).unwrap();
            let prepared = asset
                .prepare(&request(&controls, &case["overrides"], size))
                .unwrap();
            assert_eq!(prepared.plan().hash(), plan.hash());
            let packaged = pollster::block_on(gpu.render_prepared(&prepared)).unwrap();
            assert_eq!(output.channels().len(), 4);
            for channel in output.channels() {
                assert_eq!(
                    channel.pixels(),
                    pixels(&repeat, channel.channel),
                    "repeated pixels"
                );
                assert_eq!(
                    channel.pixels(),
                    pixels(&packaged, channel.channel),
                    "package pixels"
                );
            }
            assert!(output.report().pass_count as u64 <= matrix["maxPasses"].as_u64().unwrap());
            assert!(
                output.report().allocations.peak_bytes
                    <= matrix["maxDescriptorPeakBytes"].as_u64().unwrap()
            );
            assert_eq!(output.report().allocations.live_bytes, 0);
            let id = case["id"].as_str().unwrap();
            save(
                &output,
                destination,
                &format!("{id}-{}x{}", size[0], size[1]),
            );
            let browser = browser_directory.as_ref().map(|path| {
                compare_browser(
                    &output,
                    Path::new(path),
                    &format!("{id}-{}x{}", size[0], size[1]),
                )
            });
            rows.push(json!({"case":id,"size":size,"repeatExact":true,"packageExact":true,"execution":output.report(),"browserComparison":browser}));
            if id == "default" && matches!(size, [256, 256] | [1024, 1024]) {
                defaults.insert(size[0], output);
            }
        }
    }
    let structure = structural_checks(&source, &controls, &mut gpu);
    let mut downsampling = Vec::new();
    for channel in [OutputChannel::Height, OutputChannel::BaseColor] {
        let error = downsample_error(
            pixels(&defaults[&256], channel),
            pixels(&defaults[&1024], channel),
            256,
        );
        downsampling.push(json!({"channel":channel.as_str(),"meanRgbError":error}));
    }
    let mut timing = Vec::new();
    for size in [1024, 2048] {
        let mut gpu = renderer();
        let plan = plan(&source, &controls, &json!({}), [size, size]);
        let mut times = Vec::new();
        let mut adapter = Value::Null;
        for _ in 0..=matrix["timing"]["warmSamples"].as_u64().unwrap() {
            let start = Instant::now();
            let output = pollster::block_on(gpu.render(&plan)).unwrap();
            times.push(start.elapsed().as_secs_f64() * 1000.);
            adapter = serde_json::to_value(&output.report().adapter).unwrap();
        }
        let mut warm = times[1..].to_vec();
        warm.sort_by(f64::total_cmp);
        let budget = ["hardware", "software"].into_iter().find_map(|policy| {
            let budget = &matrix["timing"][policy];
            adapter["name"]
                .as_str()
                .unwrap()
                .contains(budget["adapter"].as_str().unwrap())
                .then_some(budget)
        });
        let budget_passed = budget.map(|budget| {
            let warm_key = if size == 1024 {
                "warmMedian1024Ms"
            } else {
                "warmMedian2048Ms"
            };
            warm[warm.len() / 2] <= budget[warm_key].as_f64().unwrap()
                && (size != 1024 || times[0] <= budget["cold1024Ms"].as_f64().unwrap())
        });
        timing.push(json!({"size":size,"adapter":adapter,"coldMs":times[0],"warmMs":&times[1..],"warmMedianMs":warm[warm.len()/2],"budgetPassed":budget_passed}));
    }
    let downsample_ok = downsampling.iter().all(|v| {
        v["meanRgbError"].as_array().unwrap().iter().all(|x| {
            x.as_f64().unwrap() <= matrix["maxDefaultDownsampleMeanError"].as_f64().unwrap()
        })
    });
    let timing_ok = timing.iter().all(|row| row["budgetPassed"] != false);
    let browser_ok = rows.iter().all(|row| {
        row["browserComparison"]
            .as_array()
            .is_none_or(|comparisons| {
                comparisons.iter().all(|v| {
                    v["maxComponentDelta"].as_u64().unwrap()
                        <= matrix["maxCrossRuntimeComponentError"].as_u64().unwrap()
                })
            })
    });
    std::fs::write(report_path, serde_json::to_vec_pretty(&json!({
        "schemaVersion":1,"completed":true,"ok":downsample_ok && timing_ok && browser_ok,"debugAssertions":cfg!(debug_assertions),"browserCompared":browser_directory.is_some(),
        "scope":"native matrix, repeatability, package roundtrip, descriptor bounds, downsampling and matched-adapter timing budgets",
        "materialAccepted":false,"cases":rows,"structure":structure,"downsampling":downsampling,"timing":timing
    })).unwrap()).unwrap();
    assert!(
        downsample_ok,
        "default downsample error exceeds frozen gate; see receipt"
    );
    assert!(
        timing_ok,
        "frozen timing budget failed; start PERF-MAT using retained measurements"
    );
    assert!(
        browser_ok,
        "Native/browser component error exceeds frozen gate; see receipt"
    );
}
