//! Stage A public woven material matrix; structural and human acceptance are separate.
#[path = "support/woven_quality.rs"]
mod woven_quality;
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

// Retain every delivered channel and, when supplied, compare the browser PNG.
fn save_and_compare(
    output: &RenderOutput,
    id: &str,
    size: [u32; 2],
    destination: &Path,
    browser_directory: Option<&str>,
) -> Vec<Value> {
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
        if let Some(directory) = browser_directory {
            let other = read_png(&Path::new(directory).join(filename), size);
            let max = other
                .iter()
                .zip(c.pixels())
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            comparisons.push(json!({"channel":c.channel.as_str(),"maxComponentDelta":max}));
        }
    }
    comparisons
}

fn same(a: &RenderOutput, b: &RenderOutput) {
    assert_eq!(a.channels().len(), b.channels().len());
    for (a, b) in a.channels().iter().zip(b.channels()) {
        assert_eq!(a.channel, b.channel);
        assert_eq!(a.pixels(), b.pixels());
    }
}

#[test]
#[ignore = "requires release GPU and fresh MAT-03 evidence directory"]
fn woven_material_public_matrix() {
    if cfg!(debug_assertions) {
        panic!("timings require release mode");
    }
    let root = std::env::var("MIXTURE_WOVEN_ROOT").unwrap();
    let inputs = std::env::var("MIXTURE_WOVEN_REQUESTS").unwrap();
    let destination = std::env::var("MIXTURE_WOVEN_EVIDENCE").unwrap();
    let destination = Path::new(&destination);
    std::fs::create_dir(destination).unwrap();
    let receipt = destination.join("native.json");
    std::fs::write(&receipt, b"{\"ok\":false,\"completed\":false}").unwrap();
    let source = std::fs::read(Path::new(&inputs).join("material.mix")).unwrap();
    assert_eq!(
        source,
        std::fs::read(Path::new(&root).join("fixtures/materials/woven-fabric/material.mix"))
            .unwrap()
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(Path::new(&inputs).join("requests.json")).unwrap())
            .unwrap();
    let contract: Value = serde_json::from_slice(
        &std::fs::read(
            Path::new(&root).join("fixtures/materials/woven-fabric/qualification-plan.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let revision = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(revision.status.success());
    assert_eq!(
        manifest["sourceRevision"],
        String::from_utf8(revision.stdout).unwrap().trim()
    );
    assert_eq!(
        manifest["workingTreeStatus"], "",
        "qualification requires clean source"
    );
    assert_eq!(
        std::fs::read(Path::new(&inputs).join("qualification-plan.json")).unwrap(),
        std::fs::read(
            Path::new(&root).join("fixtures/materials/woven-fabric/qualification-plan.json")
        )
        .unwrap()
    );
    assert_eq!(manifest["kind"], "mat03-material-requests");
    assert_eq!(manifest["recipeRevision"], contract["recipeRevision"]);
    assert_eq!(manifest["rows"].as_array().unwrap().len(), 48);
    assert_eq!(manifest["stress"].as_array().unwrap().len(), 3);
    let archive = mixture_asset::write(&source, &[], &Default::default()).unwrap();
    assert_eq!(
        archive,
        mixture_asset::write(&source, &[], &Default::default()).unwrap()
    );
    std::fs::write(destination.join("material.mixpack"), &archive).unwrap();
    let asset = mixture_asset::AssetView::load(&archive, &Default::default()).unwrap();
    assert_eq!(asset.source(), source);
    let browser_directory = std::env::var("MIXTURE_WOVEN_BROWSER").ok();
    let browser: Option<Value> = browser_directory.as_ref().map(|p| {
        serde_json::from_slice(&std::fs::read(Path::new(p).join("woven-browser.json")).unwrap())
            .unwrap()
    });
    let mut rows = Vec::new();
    let mut timing = Vec::new();
    let mut downsample = Vec::new();
    let mut retained = std::collections::BTreeMap::new();
    for (index, case) in manifest["rows"]
        .as_array()
        .unwrap()
        .iter()
        .chain(manifest["stress"].as_array().unwrap())
        .enumerate()
    {
        let id = case["id"].as_str().unwrap();
        let preset = case["preset"].as_str().unwrap_or("stress");
        let size: [u32; 2] = serde_json::from_value(case["request"]["size"].clone()).unwrap();
        if index < 48 {
            assert_eq!(case["preset"], contract["cases"][index / 4]["id"]);
            assert_eq!(case["request"]["size"], contract["sizes"][index % 4]);
        } else {
            assert_eq!(case["stress"], contract["stress"][0]["id"]);
            assert_eq!(
                case["request"]["size"],
                contract["stress"][0]["sizes"][index - 48]
            );
        }
        assert_eq!(case["request"]["channels"], contract["channels"]);
        let request = CompileRequest {
            size,
            outputs: vec![
                OutputChannel::BaseColor,
                OutputChannel::Normal,
                OutputChannel::Roughness,
                OutputChannel::Metallic,
                OutputChannel::Height,
            ],
            overrides: serde_json::from_value(case["request"]["overrides"].clone()).unwrap(),
            ..Default::default()
        };
        let document = MaterialDocument::decode(&source, &request.limits)
            .unwrap()
            .into_validated(&request.limits)
            .unwrap();
        let plan = compile(&document, &request).unwrap();
        assert_eq!(plan.version(), 3);
        assert_eq!(plan.passes().len(), 21);
        assert!(plan.passes().len() as u64 <= contract["maxPasses"].as_u64().unwrap());
        let mut gpu = renderer();
        let start = Instant::now();
        let output = pollster::block_on(gpu.render(&plan)).unwrap();
        let cold = start.elapsed().as_secs_f64() * 1000.;
        let expected = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER").unwrap();
        assert!(
            output.report().adapter.name.contains(&expected),
            "unexpected adapter"
        );
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
        let timed = ["plain", "varied"].contains(&preset) && [1024, 2048].contains(&size[0]);
        let samples = if timed {
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
        let mut sliced_request = request.clone();
        sliced_request.outputs = vec![OutputChannel::Height];
        let sliced =
            pollster::block_on(gpu.render(&compile(&document, &sliced_request).unwrap())).unwrap();
        assert_eq!(
            sliced.channels()[0].pixels(),
            output
                .channels()
                .iter()
                .find(|c| c.channel == OutputChannel::Height)
                .unwrap()
                .pixels()
        );
        let reference = woven_quality::endpoint_reference(&mut gpu, preset).unwrap();
        let endpoints = woven_quality::check_endpoint(&output, &reference, preset);
        drop(gpu);
        let comparisons =
            save_and_compare(&output, id, size, destination, browser_directory.as_deref());
        let parity = comparisons.iter().all(|c| {
            c["maxComponentDelta"].as_u64().unwrap()
                <= contract["maxCrossRuntimeComponentError"].as_u64().unwrap()
        });
        if let Some(browser) = &browser {
            let row = &browser["rows"][index];
            assert_eq!(row["id"], id);
            assert_eq!(row["size"], json!(size));
            assert_eq!(row["overrides"], case["request"]["overrides"]);
            assert_eq!(row["planHash"], plan.hash().as_str());
            assert_eq!(
                row["allocation"],
                serde_json::to_value(plan.allocation()).unwrap()
            );
        }
        let mut timing_pass = true;
        if timed {
            let software = std::env::var("MIXTURE_GPU_SOFTWARE").unwrap() == "1";
            let b = &contract["timing"][if software { "software" } else { "hardware" }];
            let backend = std::env::var("MIXTURE_GPU_BACKEND").unwrap();
            let matched = if software {
                output.report().adapter.name.contains("SwiftShader")
                    && std::env::var("MIXTURE_SWIFTSHADER_COMMIT").is_ok()
            } else {
                output.report().adapter.name == b["adapter"].as_str().unwrap()
                    && b["backends"].as_array().unwrap().contains(&json!(backend))
            };
            let mut ordered = warm.clone();
            ordered.sort_by(f64::total_cmp);
            let median = ordered[ordered.len() / 2];
            // Software-adapter budgets are recorded but never gate a run; the adapter must still match.
            let within = median
                <= b[if size[0] == 1024 {
                    "warmMedian1024Ms"
                } else {
                    "warmMedian2048Ms"
                }]
                .as_f64()
                .unwrap()
                && (size[0] != 1024 || cold <= b["cold1024Ms"].as_f64().unwrap());
            timing_pass = matched && (software || within);
            timing.push(json!({"id":id,"size":size,"adapter":output.report().adapter,"backend":backend,"coldMs":cold,"warmMs":warm,"warmMedianMs":median,"budgetPolicy":if software {"software"} else {"hardware"},"budgetPassed":matched && within,"matchedAdapter":matched}));
        }
        let mut quality_pass = true;
        if ["plain", "varied"].contains(&preset) && [1024, 2048].contains(&size[0]) {
            let low_size = if size[0] == 1024 { 256 } else { 1024 };
            let low = retained.get(&(preset.to_owned(), low_size)).unwrap();
            let measurements = woven_quality::downsample(low, &output, &contract);
            quality_pass = measurements.iter().all(|m| m["passed"] == true);
            downsample.push(json!({"preset":preset,"channels":measurements}));
        }
        if preset == "stress" && size == [1024, 1024] {
            let low = retained.get(&("stress".to_owned(), 256)).unwrap();
            downsample.push(json!({"preset":"stress","qualityScope":case["qualityScope"],"gated":false,"channels":woven_quality::downsample(low,&output,&contract)}));
        }
        rows.push(json!({"id":id,"size":size,"overrides":case["request"]["overrides"],"qualityScope":case["qualityScope"],"planHash":plan.hash().as_str(),"passes":plan.passes().len(),"report":output.report(),"repeatExact":true,"packageExact":true,"slicedExact":true,"ownedAfterDestroy":true,"endpoints":endpoints,"comparisons":comparisons}));
        let ok = parity && timing_pass && quality_pass;
        std::fs::write(&receipt,serde_json::to_vec_pretty(&json!({"ok":ok&&rows.len()==51,"completed":rows.len()==51,"lastRow":id,"failedGates":{"parity":!parity,"timing":!timing_pass,"downsample":!quality_pass},"materialAccepted":false,"browserCompared":browser.is_some(),"requestManifest":manifest,"rows":rows,"timing":timing,"downsample":downsample})).unwrap()).unwrap();
        assert!(
            ok,
            "frozen MAT-03 gate failed at {id}; retained receipt; stop without relaxing gates"
        );
        if ["plain", "varied", "stress"].contains(&preset) && [256, 1024].contains(&size[0]) {
            retained.insert((preset.to_owned(), size[0]), output);
        }
    }
    assert_eq!(rows.len(), 51);
    assert_eq!(timing.len(), 4);
    assert_eq!(downsample.len(), 5);
}
