//! Fixed fixture/sentinel expectations, never a CPU implementation of graph pixels.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
const NODES: [&str; 13] = [
    "constant-scalar",
    "constant-color",
    "checker",
    "brick-pattern",
    "levels",
    "blend",
    "scalar-blend",
    "material-output",
    "fractal-noise",
    "gradient-map",
    "height-to-normal",
    "transform-2d",
    "warp",
];
#[path = "support/fixed_noise_probe.rs"]
mod fixed_noise_probe;
#[path = "support/normal_probe.rs"]
mod normal_probe;
#[path = "support/precision_probe.rs"]
mod precision_probe;
#[path = "support/resampling_probe.rs"]
mod resampling_probe;
#[path = "support/scalar_probe.rs"]
mod scalar_probe;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    node: String,
    source: String,
    cases: Vec<Case>,
    invalid_cases: Vec<Invalid>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    source: Option<String>,
    size: [u32; 2],
    outputs: Vec<String>,
    overrides: BTreeMap<String, Value>,
    pass_count: usize,
    samples: Vec<Sample>,
    golden: Option<Golden>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Golden {
    channel: String,
    file: String,
    hardware_tolerance: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    channel: String,
    xy: [u32; 2],
    rgba: [u8; 4],
    tolerance: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Invalid {
    source: Option<String>,
    overrides: BTreeMap<String, Value>,
    code: String,
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/nodes")
}
fn fixture(name: &str) -> Fixture {
    serde_json::from_slice(&std::fs::read(root().join(name).join("cases.json")).unwrap()).unwrap()
}
fn source(name: &str, path: &str) -> Vec<u8> {
    std::fs::read(root().join(name).join(path)).unwrap()
}
fn plan(
    bytes: &[u8],
    request: &CompileRequest,
) -> Result<mixture_core::RenderPlan, mixture_core::DiagnosticReport> {
    let document = MaterialDocument::decode(bytes, &request.limits)
        .and_then(|d| d.into_validated(&request.limits))
        .map_err(|e| e.report().clone())?;
    compile(&document, request).map_err(|e| e.report().clone())
}
fn request(case: &Case) -> CompileRequest {
    CompileRequest {
        size: case.size,
        outputs: case.outputs.iter().map(|s| s.parse().unwrap()).collect(),
        overrides: case.overrides.clone(),
        ..Default::default()
    }
}
fn options() -> GpuContextOptions {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("auto") => BackendPreference::Auto,
        Ok("metal") => BackendPreference::Metal,
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        other => panic!("invalid GPU test backend {other:?}"),
    };
    let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("0") => false,
        Ok("1") => true,
        other => panic!("invalid software policy {other:?}"),
    };
    GpuContextOptions {
        backend,
        software_adapter,
        ..Default::default()
    }
}
#[test]
fn node_fixtures_validate_defaults_boundaries_invalid_values_without_gpu() {
    for name in NODES {
        let fixture = fixture(name);
        assert_eq!(fixture.schema_version, 1);
        assert_eq!(fixture.node, name);
        assert!(fixture.cases.iter().any(|c| c.id == "defaults"));
        assert!(!fixture.invalid_cases.is_empty());
        for case in &fixture.cases {
            let plan = plan(
                &source(name, case.source.as_deref().unwrap_or(&fixture.source)),
                &request(case),
            )
            .unwrap();
            assert_eq!(plan.passes().len(), case.pass_count, "{name}/{}", case.id);
            for sample in &case.samples {
                assert!(sample.xy[0] < case.size[0] && sample.xy[1] < case.size[1]);
                assert!(
                    plan.outputs()
                        .iter()
                        .any(|o| o.channel.as_str() == sample.channel)
                );
            }
        }
        for invalid in fixture.invalid_cases {
            let request = CompileRequest {
                overrides: invalid.overrides,
                outputs: vec![OutputChannel::BaseColor, OutputChannel::Roughness],
                ..Default::default()
            };
            let error = plan(
                &source(name, invalid.source.as_deref().unwrap_or(&fixture.source)),
                &request,
            )
            .unwrap_err();
            assert!(
                error
                    .diagnostics()
                    .iter()
                    .any(|d| d.code.as_str() == invalid.code),
                "{name}: {error:?}"
            );
        }
    }
}
fn run_node(name: &str) {
    let fixture = fixture(name);
    let context = pollster::block_on(GpuContext::request(options())).unwrap();
    if name == "constant-color" {
        precision_probe::run(&context);
    }
    if name == "fractal-noise" {
        fixed_noise_probe::run(&context);
    }
    if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
        assert!(context.report().adapter().unwrap().name.contains(&expected));
    }
    let mut renderer = Renderer::new(context);
    let mut evidence = Vec::new();
    for case in &fixture.cases {
        let plan = plan(
            &source(name, case.source.as_deref().unwrap_or(&fixture.source)),
            &request(case),
        )
        .unwrap();
        let rendered = pollster::block_on(renderer.render(&plan)).unwrap();
        assert_eq!(rendered.report().pass_count, case.pass_count);
        assert_eq!(&rendered.report().plan_hash, plan.hash());
        assert_eq!(
            rendered.report().mapped_bytes,
            plan.estimates().cumulative_readback_bytes
        );
        for sample in &case.samples {
            let channel = rendered
                .channels()
                .iter()
                .find(|c| c.channel.as_str() == sample.channel)
                .unwrap();
            let offset = ((sample.xy[1] * case.size[0] + sample.xy[0]) * 4) as usize;
            let actual = &channel.pixels()[offset..offset + 4];
            for (got, expected) in actual.iter().zip(sample.rgba) {
                assert!(
                    got.abs_diff(expected) <= sample.tolerance,
                    "{name}/{} {:?}: actual {actual:?}, expected {:?}",
                    case.id,
                    sample.xy,
                    sample.rgba
                );
            }
        }
        if name == "checker" && case.id == "defaults" {
            assert_eq!(
                rendered.channels()[0].pixels(),
                include_bytes!("../../../fixtures/nodes/checker/checker-64.rgba")
            );
        }
        if let Some(golden) = &case.golden {
            let expected = std::fs::read(root().join(name).join(&golden.file)).unwrap();
            let actual = rendered
                .channels()
                .iter()
                .find(|c| c.channel.as_str() == golden.channel)
                .unwrap()
                .pixels();
            assert_eq!(actual.len(), expected.len(), "full node golden dimensions");
            let tolerance = if options().software_adapter {
                0
            } else {
                golden.hardware_tolerance
            };
            let maximum = actual
                .iter()
                .zip(&expected)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            assert!(
                maximum <= tolerance,
                "{name}/{} full golden max error {maximum} exceeds {tolerance}",
                case.id
            );
        }
        if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
            let path = Path::new(&directory).join(name).join(&case.id);
            std::fs::create_dir_all(&path).unwrap();
            for channel in rendered.channels() {
                std::fs::write(
                    path.join(format!("{}.rgba", channel.channel.as_str())),
                    channel.pixels(),
                )
                .unwrap();
            }
        }
        evidence.push(json!({"case":case.id,"ok":true,"sentinels":case.samples.len(),"execution":rendered.report()}));
    }
    if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            Path::new(&directory).join(format!("{name}.json")),
            serde_json::to_vec_pretty(&json!({"node":name,"ok":true,"cases":evidence})).unwrap(),
        )
        .unwrap();
    }
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node constant-scalar"]
fn node_constant_scalar_gpu() {
    run_node("constant-scalar");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node constant-color"]
fn node_constant_color_gpu() {
    run_node("constant-color");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node brick-pattern"]
fn node_brick_pattern_gpu() {
    run_node("brick-pattern");
    brick_invariants();
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node checker"]
fn node_checker_gpu() {
    run_node("checker");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node levels"]
fn node_levels_gpu() {
    run_node("levels");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node blend"]
fn node_blend_gpu() {
    run_node("blend");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node material-output"]
fn node_material_output_gpu() {
    run_node("material-output");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node fractal-noise"]
fn node_fractal_noise_gpu() {
    run_node("fractal-noise");
    noise_invariants("input.mix");
    noise_invariants("stable.mix");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node gradient-map"]
fn node_gradient_map_gpu() {
    run_node("gradient-map");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node height-to-normal"]
fn node_height_to_normal_gpu() {
    run_node("height-to-normal");
    let context = pollster::block_on(GpuContext::request(options())).unwrap();
    let evidence = normal_probe::run(&context);
    if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::write(
            Path::new(&directory).join("height-to-normal-relations.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires GPU; cargo xtask test-node transform-2d"]
fn node_transform_2d_gpu() {
    run_node("transform-2d");
    resampling_evidence("transform-2d");
    resampling_invariants("transform-2d");
}
#[test]
#[ignore = "requires GPU; cargo xtask test-node warp"]
fn node_warp_gpu() {
    run_node("warp");
    resampling_evidence("warp");
    resampling_invariants("warp");
}
fn resampling_evidence(name: &str) {
    let context = pollster::block_on(GpuContext::request(options())).unwrap();
    let evidence = resampling_probe::run(&context, name);
    if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::write(
            Path::new(&directory).join(format!("{name}-literal-probes.json")),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
    }
}
fn resampling_invariants(name: &str) {
    let bytes = source(name, "input.mix");
    let base = CompileRequest {
        size: [128, 64],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let mut renderer = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    let mut render = |request: &CompileRequest| {
        pollster::block_on(renderer.render(&plan(&bytes, request).unwrap()))
            .unwrap()
            .channels()[0]
            .pixels()
            .to_vec()
    };
    let original = render(&base);
    assert_eq!(render(&base), original, "warm cache repeats exactly");
    let mut changed = base.clone();
    if name == "transform-2d" {
        changed.overrides.insert("scaleX".into(), json!(8));
    } else {
        changed.overrides.insert("field".into(), json!(1));
        changed.overrides.insert("strengthX".into(), json!(0.125));
    }
    let transformed = render(&changed);
    assert!(
        transformed
            .iter()
            .zip(&original)
            .filter(|(a, b)| a != b)
            .count()
            > original.len() / 4,
        "nontrivial transform must change a substantial part of the real graph output"
    );
    if name == "transform-2d" {
        changed.overrides.insert("quarterTurns".into(), json!(1));
        assert_ne!(
            render(&changed),
            transformed,
            "rotation changes the anisotropic pattern"
        );
    } else {
        changed.overrides.insert("strengthX".into(), json!(0));
        assert_eq!(
            render(&changed),
            original,
            "zero strength is exact identity"
        );
    }
}

// Relations across full rendered outputs, not a second noise implementation.
fn noise_invariants(file: &str) {
    let bytes = source("fractal-noise", file);
    let base = CompileRequest {
        size: [129, 65],
        outputs: vec![OutputChannel::Height],
        ..Default::default()
    };
    let mut renderer = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    let mut render = |request: &CompileRequest| {
        let plan = plan(&bytes, request).unwrap();
        pollster::block_on(renderer.render(&plan))
            .unwrap()
            .channels()[0]
            .pixels()
            .to_vec()
    };
    let mut reference = None;
    for basis in ["value", "cellular"] {
        let mut base = base.clone();
        base.overrides.insert("basis".into(), json!(basis));
        let default = render(&base);
        assert_eq!(
            default,
            render(&base),
            "same seed must reproduce every pixel with a warm cache"
        );
        let mut previous_seed_output = None;
        for seed in [0_u32, u32::MAX, 16777216, 16777217] {
            let mut request = base.clone();
            request.overrides.insert("seed".into(), json!(seed));
            let output = render(&request);
            let changed = default
                .as_chunks::<4>()
                .0
                .iter()
                .zip(output.as_chunks::<4>().0.iter())
                .filter(|(a, b)| a != b)
                .count();
            assert!(
                changed > default.len() / 4 * 9 / 10,
                "{basis}: seed must change the spatial field"
            );
            if let Some(previous) = previous_seed_output {
                assert_ne!(
                    previous, output,
                    "adjacent high seeds must not collapse through f32"
                );
            }
            previous_seed_output = Some(output);
        }
        let mut first = base.clone();
        first.overrides.insert("octaves".into(), json!(1));
        let mut zero = base.clone();
        zero.overrides.insert("persistence".into(), json!(0));
        assert_eq!(
            render(&first),
            render(&zero),
            "zero persistence removes later octaves"
        );
        let unique = default
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0])
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            unique.len() > 100,
            "default must not collapse to a constant or binary field"
        );
        if let Some(value) = reference {
            assert_ne!(
                value, default,
                "bases must produce different spatial fields"
            );
        }
        reference = Some(default);
    }
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_slicing_cache_reuse_clear_and_output_ownership() {
    let bytes = include_bytes!("../../../examples/blend.mix");
    let full = plan(
        bytes,
        &CompileRequest {
            outputs: vec![OutputChannel::BaseColor, OutputChannel::Roughness],
            ..Default::default()
        },
    )
    .unwrap();
    let mut renderer = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    let a = pollster::block_on(renderer.render(&full)).unwrap();
    assert_eq!(a.report().pass_count, 5);
    assert_eq!(a.report().pipeline_cache.misses, 4);
    assert_eq!(a.report().pipeline_cache.hits, 1);
    let b = pollster::block_on(renderer.render(&full)).unwrap();
    assert_eq!(b.report().pipeline_cache.misses, 0);
    assert_eq!(b.report().pipeline_cache.hits, 5);
    assert_eq!(renderer.cached_pipeline_count(), 4);
    assert_eq!(a.channels()[0].pixels(), b.channels()[0].pixels());
    let sliced = plan(
        bytes,
        &CompileRequest {
            outputs: vec![OutputChannel::Roughness],
            ..Default::default()
        },
    )
    .unwrap();
    let c = pollster::block_on(renderer.render(&sliced)).unwrap();
    assert_eq!(c.report().pass_count, 1);
    assert_eq!(a.channels()[1].pixels(), c.channels()[0].pixels());
    renderer.clear_pipeline_cache();
    assert_eq!(renderer.cached_pipeline_count(), 0);
    let d = pollster::block_on(renderer.render(&sliced)).unwrap();
    assert_eq!(d.report().pipeline_cache.misses, 1);
    renderer.context().device().destroy();
    let failed = pollster::block_on(renderer.render(&sliced)).unwrap_err();
    assert!(matches!(
        failed.diagnostic().stage,
        mixture_core::Stage::GpuExecution | mixture_core::Stage::Readback
    ));
    drop(renderer);
    assert_eq!(a.channels()[1].pixels(), d.channels()[0].pixels());
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_device_limits_fail_before_allocating_and_context_remains_usable() {
    let context = pollster::block_on(GpuContext::request(options())).unwrap();
    let width = context.device().limits().max_texture_dimension_2d + 1;
    let mut request = CompileRequest {
        size: [width, 1],
        ..Default::default()
    };
    request.limits.output_dimension = u64::from(width);
    let bytes = source("constant-color", "input.mix");
    let large = plan(&bytes, &request).unwrap();
    let mut renderer = Renderer::new(context);
    let failed = pollster::block_on(renderer.render(&large)).unwrap_err();
    assert_eq!(failed.diagnostic().stage, mixture_core::Stage::GpuExecution);
    assert_eq!(renderer.cached_pipeline_count(), 0);
    assert_eq!(
        failed.diagnostic().evidence["planHash"],
        mixture_core::EvidenceValue::Text(large.hash().to_string())
    );
    request.size = [1, 1];
    pollster::block_on(renderer.render(&plan(&bytes, &request).unwrap())).unwrap();
}

fn device_loss_contract(warm_cache: bool) {
    let plan = plan(
        include_bytes!("../../../examples/blend.mix"),
        &CompileRequest {
            size: [33, 3],
            outputs: vec![OutputChannel::BaseColor, OutputChannel::Roughness],
            ..Default::default()
        },
    )
    .unwrap();
    let context = pollster::block_on(GpuContext::request(options())).unwrap();
    let mut renderer = Renderer::new(context);
    if warm_cache {
        pollster::block_on(renderer.render(&plan)).unwrap();
    }
    assert_eq!(
        renderer.cached_pipeline_count(),
        if warm_cache { 4 } else { 0 }
    );
    let acquisition = serde_json::to_value(renderer.context().report()).unwrap();
    renderer.context().device().destroy();
    let error = pollster::block_on(renderer.render(&plan)).unwrap_err();
    eprintln!(
        "destroyed device, warmCache={warm_cache}: {}",
        serde_json::to_string(error.diagnostic()).unwrap()
    );
    assert_eq!(error.diagnostic().code.as_str(), "MIX_GPU_DEVICE_LOST");
    assert_eq!(error.diagnostic().stage, mixture_core::Stage::GpuExecution);
    assert_eq!(error.reason(), mixture_wgpu::GpuFailureReason::DeviceLost);
    let loss = error.device_loss().unwrap().clone();
    assert_eq!(loss.reason, mixture_wgpu::DeviceLossReason::Destroyed);
    assert_eq!(renderer.context().device_loss(), Some(&loss));
    assert_eq!(renderer.cached_pipeline_count(), 0);
    assert_eq!(
        error.allocations(),
        Some(&mixture_wgpu::AllocationReport::default())
    );
    assert_eq!(
        serde_json::to_value(error.adapter().unwrap()).unwrap(),
        acquisition["adapter"]
    );
    assert_eq!(
        serde_json::to_value(renderer.context().report()).unwrap(),
        acquisition
    );
    assert_eq!(
        error.diagnostic().evidence["planHash"],
        mixture_core::EvidenceValue::Text(plan.hash().to_string())
    );
    for _ in 0..2 {
        let repeated = pollster::block_on(renderer.render(&plan)).unwrap_err();
        assert_eq!(repeated.device_loss(), Some(&loss));
        assert_eq!(
            repeated.allocations(),
            Some(&mixture_wgpu::AllocationReport::default())
        );
        assert_eq!(
            serde_json::to_value(repeated.diagnostic()).unwrap(),
            serde_json::to_value(error.diagnostic()).unwrap()
        );
        assert_eq!(renderer.cached_pipeline_count(), 0);
    }
    let mut independent = pollster::block_on(GpuContext::request(options())).unwrap();
    assert_eq!(
        pollster::block_on(independent.probe_checker()).verdict(),
        mixture_wgpu::DoctorVerdict::Healthy
    );
    assert!(independent.device_loss().is_none());
    drop(renderer);
    assert_eq!(error.device_loss(), Some(&loss));
    assert!(std::error::Error::source(error.diagnostic()).is_some());
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_device_loss_with_cold_cache_is_typed() {
    device_loss_contract(false);
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_device_loss_with_warm_cache_is_typed() {
    device_loss_contract(true);
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_cache_is_bounded_across_all_kernels_and_request_changes() {
    let mut renderer = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    let mut other = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    // Explicit reviewed kernels in the procedural fixture matrix. ImageInput is
    // covered by its independent resource suite, not this resource-free matrix.
    let expected: std::collections::BTreeSet<String> = [
        "Constant",
        "Checker",
        "BrickPattern",
        "Levels",
        "Blend",
        "ScalarBlend",
        "FractalNoise",
        "GradientMap",
        "HeightToNormal",
        "Transform2d",
        "Warp",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut rows = Vec::new();
    // Every focused contract case varies parameters/channels without new cache keys.
    // Existing node tests separately verify their pixels and boundary expectations.
    for name in NODES {
        let fixture = fixture(name);
        for case in fixture.cases.iter().take(2) {
            let bytes = source(name, case.source.as_deref().unwrap_or(&fixture.source));
            let request = CompileRequest {
                size: [33, 3],
                outputs: case.outputs.iter().map(|s| s.parse().unwrap()).collect(),
                overrides: case.overrides.clone(),
                ..Default::default()
            };
            let plan = plan(&bytes, &request).unwrap();
            let output = pollster::block_on(renderer.render(&plan)).unwrap();
            let report = output.report();
            for pass in plan.passes() {
                seen.insert(format!("{:?}", pass.kernel.id()));
            }
            assert_eq!(renderer.cached_pipeline_count(), seen.len());
            assert!(seen.is_subset(&expected));
            assert_eq!(report.allocations.live_bytes, 0);
            assert_eq!(
                report.allocations.released_bytes,
                report.allocations.cumulative_bytes
            );
            assert_eq!(report.allocations.reused_bytes, 0);
            // Identical request counters must reset rather than accumulate.
            let again = pollster::block_on(renderer.render(&plan)).unwrap();
            assert_eq!(again.report().allocations, report.allocations);
            assert_eq!(again.report().pipeline_cache.misses, 0);
            assert!(
                output
                    .channels()
                    .iter()
                    .zip(again.channels())
                    .all(|(a, b)| a.pixels() == b.pixels())
            );
            assert_eq!(other.cached_pipeline_count(), 0);
            rows.push(json!({"node":name,"case":case.id,"execution":report,"repeatedExecution":again.report()}));
        }
    }
    assert_eq!(seen, expected);
    renderer.clear_pipeline_cache();
    assert_eq!(renderer.cached_pipeline_count(), 0);
    let plan = plan(
        &source("constant-color", "input.mix"),
        &CompileRequest {
            size: [33, 3],
            ..Default::default()
        },
    )
    .unwrap();
    let output = pollster::block_on(renderer.render(&plan)).unwrap();
    assert_eq!(output.report().pipeline_cache.misses, 1);
    renderer.context().device().destroy();
    let failed = pollster::block_on(renderer.render(&plan)).unwrap_err();
    assert_eq!(failed.allocations().unwrap().live_bytes, 0);
    assert_eq!(failed.allocations().unwrap().cumulative_bytes, 0);
    assert_eq!(renderer.cached_pipeline_count(), 0);
    drop(renderer);
    let independent = pollster::block_on(other.render(&plan)).unwrap();
    assert_eq!(independent.report().pipeline_cache.misses, 1);
    assert_eq!(independent.report().allocations.live_bytes, 0);
    assert_eq!(
        output.channels()[0].pixels(),
        independent.channels()[0].pixels()
    );
    println!(
        "cache-bound-evidence: {}",
        json!({"kernelCount":seen.len(),"cases":rows,"failure":failed.diagnostic(),"independentExecution":independent.report(),"outputsSurviveDrop":true})
    );
}

#[test]
#[ignore = "requires GPU; cargo xtask test-node scalar-blend"]
fn node_scalar_blend_gpu() {
    run_node("scalar-blend");
    let context = pollster::block_on(GpuContext::request(options())).unwrap();
    let evidence = scalar_probe::run(&context);
    if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::write(
            Path::new(&directory).join("scalar-blend-saturation.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
    }
}

#[path = "support/image_probe.rs"]
mod image_probe;
#[test]
#[ignore = "requires GPU; cargo xtask test-node image-input"]
fn node_image_input_gpu() {
    image_probe::run();
}

fn brick_invariants() {
    let bytes = source("brick-pattern", "input.mix");
    let mut renderer = Renderer::new(pollster::block_on(GpuContext::request(options())).unwrap());
    let mut render = |size, overrides| {
        let request = CompileRequest {
            size,
            outputs: vec![OutputChannel::Height],
            overrides,
            ..Default::default()
        };
        pollster::block_on(renderer.render(&plan(&bytes, &request).unwrap()))
            .unwrap()
            .channels()[0]
            .pixels()
            .to_vec()
    };
    let original = render([128, 128], BTreeMap::new());
    assert_eq!(original, render([128, 128], BTreeMap::new()));
    let repeated = render(
        [256, 256],
        BTreeMap::from([("columns".into(), json!(8)), ("rows".into(), json!(16))]),
    );
    for y in 0..256 {
        for x in 0..256 {
            let a = (y * 256 + x) * 4;
            let b = ((y % 128) * 128 + x % 128) * 4;
            assert_eq!(
                &repeated[a..a + 4],
                &original[b..b + 4],
                "two by two tiling at {x},{y}"
            );
        }
    }
    for key in ["gap", "bevel"] {
        let wider = render([128, 128], BTreeMap::from([(key.into(), json!(0.2))]));
        assert!(
            wider.iter().zip(&original).all(|(a, b)| a <= b),
            "wider mortar/bevel cannot increase height"
        );
        assert!(wider.iter().zip(&original).any(|(a, b)| a < b));
    }
    for y in 0..112 {
        for x in 0..128 {
            let a = (y * 128 + x) * 4;
            let b = ((y + 16) * 128 + (x + 16) % 128) * 4;
            assert_eq!(original[a], original[b], "next row must shift half a brick");
        }
    }
}
