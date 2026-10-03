//! Frozen MAT-03 Stage B observations of the real graph; no alternate renderer.
use super::painted_composition_tests::{periodic_shift, replay_normal};
use super::*;
use mixture_core::{CompileRequest, MaterialDocument, compile};
use serde_json::{Value, json};
const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/materials/woven-fabric/"
);
const WEAVE: [&str; 3] = [
    "n17-weave-height",
    "n09-weave-coverage",
    "n06-weave-warp-share",
];
const MODES: [&str; 3] = ["height", "coverage", "warp-share"];
const CHANNELS: [OutputChannel; 5] = [
    OutputChannel::BaseColor,
    OutputChannel::Normal,
    OutputChannel::Roughness,
    OutputChannel::Metallic,
    OutputChannel::Height,
];
fn inputs() -> (Value, Value) {
    let read = |name: &str| {
        serde_json::from_slice(&std::fs::read(format!("{ROOT}{name}")).unwrap()).unwrap()
    };
    (read("material.mix"), read("qualification-plan.json"))
}
fn context() -> GpuContext {
    let context = pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
    eprintln!("woven adapter: {}", json!(context.report().adapter()));
    if let Ok(name) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
        assert_eq!(json!(context.report().adapter())["name"], name);
    }
    context
}
fn controls(plan: &Value, changes: &Value) -> Value {
    let mut c = plan["defaults"].clone();
    for (k, v) in changes.as_object().unwrap() {
        c[k] = v.clone();
    }
    c
}
fn request(plan: &Value, c: &Value, size: [u32; 2]) -> CompileRequest {
    let mut values = c.clone();
    let detail = c["detailAmount"].as_f64().unwrap();
    values["roughnessMin"] = json!(c["yarnRoughness"].as_f64().unwrap() * (1. - 2. * detail));
    for axis in ["warp", "weft"] {
        values[format!("{axis}Dark")] = json!(
            c[format!("{axis}Color")]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, v)| if i == 3 {
                    1.
                } else {
                    v.as_f64().unwrap() * (1. - 4. * detail)
                })
                .collect::<Vec<_>>()
        );
    }
    CompileRequest {
        size,
        outputs: CHANNELS.to_vec(),
        overrides: plan["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                (
                    b["id"].as_str().unwrap().to_owned(),
                    values[b["control"].as_str().unwrap()].clone(),
                )
            })
            .collect(),
        ..Default::default()
    }
}
fn geometry(source: &Value, c: &Value, req: &CompileRequest, _plan: &Value) {
    // Typed lowering is checked independently of JSON source defaults.
    let document = MaterialDocument::decode(&serde_json::to_vec(source).unwrap(), &req.limits)
        .unwrap()
        .into_validated(&req.limits)
        .unwrap();
    let compiled = compile(&document, req).unwrap();
    let geometries: Vec<_> = compiled
        .passes()
        .iter()
        .filter(|p| p.kernel.id() == mixture_core::plan::KernelId::WeavePattern)
        .map(|p| serde_json::to_value(&p.kernel).unwrap())
        .collect();
    assert_eq!(geometries.len(), 3);
    let mut normalized = geometries.clone();
    for g in &mut normalized {
        g.as_object_mut().unwrap().remove("mode");
    }
    let float = |key: &str| c[key].as_f64().unwrap() as f32;
    let expected = json!({"id":"weavePattern","counts":[c["warpCount"],c["weftCount"]],"widths":[float("warpWidth"),float("weftWidth")],"bevel":float("bevel"),"crown":float("crown"),"underRatio":float("underRatio")});
    assert_eq!(
        normalized[0], expected,
        "public geometry controls not lowered"
    );
    assert_eq!(normalized[0], normalized[1], "geometry {}", json!(c));
    assert_eq!(normalized[1], normalized[2], "geometry {}", json!(c));
}
fn capture(
    context: &GpuContext,
    cache: &mut PipelineCache,
    source: &Value,
    req: &CompileRequest,
    alias: Option<&str>,
    raw: bool,
) -> Vec<Vec<u8>> {
    let mut source = source.clone();
    if let Some(node) = alias {
        let edge = source["edges"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["to"]["nodeId"] == "out" && e["to"]["portId"] == "height")
            .unwrap();
        edge["from"] = json!({"nodeId":node,"portId":"value"});
    }
    let doc = MaterialDocument::decode(&serde_json::to_vec(&source).unwrap(), &req.limits)
        .unwrap()
        .into_validated(&req.limits)
        .unwrap();
    let plan = compile(&doc, req).unwrap();
    assert_eq!(plan.passes().len(), 21);
    let kernels: Vec<_> = plan.passes().iter().map(|p| &p.kernel).collect();
    let slots: Vec<_> = plan
        .allocation()
        .resource_slots()
        .iter()
        .map(|s| s.index() as usize)
        .collect();
    let outputs: Vec<_> = CHANNELS
        .iter()
        .filter(|&&c| alias.is_none() || c == OutputChannel::Height)
        .map(|&c| {
            let o = plan.outputs().iter().find(|o| o.channel == c).unwrap();
            (
                o.resource.index() as usize,
                if raw {
                    ReadbackFormat::RawHalf
                } else {
                    ReadbackFormat::Rgba8(o.kind)
                },
            )
        })
        .collect();
    let result = pollster::block_on(execute_prepared(
        context,
        cache,
        plan.size(),
        &kernels,
        &outputs,
        &[],
        &slots,
    ))
    .unwrap();
    assert_eq!(result.allocations.live_bytes, 0);
    assert_eq!(
        result.allocations.released_bytes,
        result.allocations.cumulative_bytes
    );
    result.pixels
}
fn scalar(bytes: &[u8], i: usize) -> f32 {
    half::f16::from_bits(u16::from_le_bytes([bytes[8 * i], bytes[8 * i + 1]])).to_f32()
}
fn exact(a: &[u8], b: &[u8], size: [u32; 2], stride: usize, label: &str) {
    assert_eq!(a.len(), b.len(), "{label}");
    if let Some(i) = a.iter().zip(b).position(|(a, b)| a != b) {
        let p = i / stride;
        panic!(
            "{label} size {size:?} pixel ({},{}) actual {:?} expected {:?}",
            p % size[0] as usize,
            p / size[0] as usize,
            &a[p * stride..(p + 1) * stride],
            &b[p * stride..(p + 1) * stride]
        );
    }
}
fn fields(
    context: &GpuContext,
    cache: &mut PipelineCache,
    source: &Value,
    req: &CompileRequest,
) -> Vec<Vec<u8>> {
    WEAVE
        .iter()
        .map(|id| capture(context, cache, source, req, Some(id), true).remove(0))
        .collect()
}
fn normalized(f: &[Vec<u8>], size: [u32; 2], label: &str) {
    for i in 0..(size[0] * size[1]) as usize {
        let v: Vec<_> = f.iter().map(|b| scalar(b, i)).collect();
        assert!(
            v.iter().all(|v| v.is_finite() && (0.0..=1.).contains(v)),
            "{label} {size:?} pixel {i} fields {v:?}"
        );
        if v[1] == 0. {
            assert_eq!([v[0], v[2]], [0., 0.5], "{label} {size:?} pixel {i}");
        }
    }
}
fn sizes(p: &Value) -> Vec<[u32; 2]> {
    serde_json::from_value(p["sizes"].clone()).unwrap()
}
fn smooth(t: f64) -> f64 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
// Sparse crossing oracle only: no whole-image CPU rendering or normal computation.
fn center_heights(uv: [f64; 2], c: &Value) -> [f64; 2] {
    let cell = [
        uv[0] * c["warpCount"].as_f64().unwrap(),
        uv[1] * c["weftCount"].as_f64().unwrap(),
    ];
    let ij = cell.map(|x| x.floor() as i32);
    let local = cell.map(|x| x.fract() - 0.5);
    let width = [
        c["warpWidth"].as_f64().unwrap(),
        c["weftWidth"].as_f64().unwrap(),
    ];
    let crown = c["crown"].as_f64().unwrap();
    let r = c["underRatio"].as_f64().unwrap();
    let axial = cell.map(|x| x - 0.5);
    let k = axial.map(|x| x.floor() as i32);
    let s = axial.map(|x| smooth(x - x.floor()));
    let l = [
        if (ij[0] + k[1]).rem_euclid(2) == 0 {
            1. - s[1]
        } else {
            s[1]
        },
        if (ij[1] + k[0]).rem_euclid(2) == 0 {
            s[0]
        } else {
            1. - s[0]
        },
    ];
    std::array::from_fn(|a| {
        let q = (1. - (2. * local[a] / width[a]).powi(2)).max(0.);
        q * ((1. - crown) + crown * q) * (r + (1. - r) * l[a])
    })
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_woven_crossing_structure() {
    let (source, p) = inputs();
    let ctx = context();
    let mut cache = PipelineCache::default();
    let mut count = 0;
    for name in ["plain", "varied"] {
        let preset = p["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == name)
            .unwrap();
        let c = controls(&p, &preset["controls"]);
        for size in sizes(&p) {
            let req = request(&p, &c, size);
            geometry(&source, &c, &req, &p);
            let f = fields(&ctx, &mut cache, &source, &req);
            normalized(&f, size, name);
            for (mode, id) in WEAVE.iter().enumerate() {
                for instance in WEAVE {
                    let mut changed = source.clone();
                    changed["nodes"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|n| n["id"] == instance)
                        .unwrap()["parameters"]["mode"] = json!(MODES[mode]);
                    let observed =
                        capture(&ctx, &mut cache, &changed, &req, Some(instance), true).remove(0);
                    exact(
                        &observed,
                        &f[mode],
                        size,
                        8,
                        &format!("{name} same-mode {instance} {id}"),
                    );
                }
            }
            let nw = c["warpCount"].as_u64().unwrap() as u32;
            let nf = c["weftCount"].as_u64().unwrap() as u32;
            for j in 0..nf {
                for i in 0..nw {
                    let x = ((i as f64 + 0.5) / nw as f64 * size[0] as f64).floor() as u32;
                    let y = ((j as f64 + 0.5) / nf as f64 * size[1] as f64).floor() as u32;
                    let idx = (y * size[0] + x) as usize;
                    let upper = ((i + j) % 2) as usize;
                    let actual = [scalar(&f[0], idx), scalar(&f[1], idx), scalar(&f[2], idx)];
                    assert_eq!(
                        [actual[1], actual[2]],
                        [1., if upper == 0 { 1. } else { 0. }],
                        "{name} {size:?} crossing {i},{j} pixel {x},{y}"
                    );
                    let mut sum = 0.;
                    for [a, b] in [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75], [0.75, 0.75]] {
                        let h = center_heights(
                            [
                                (x as f64 + a) / size[0] as f64,
                                (y as f64 + b) / size[1] as f64,
                            ],
                            &c,
                        );
                        assert!(
                            h[upper] > h[1 - upper] && actual[0] as f64 > h[1 - upper],
                            "{name} {size:?} {x},{y} upper/lower {h:?}, H {}",
                            actual[0]
                        );
                        sum += h[upper];
                    }
                    assert!(
                        (actual[0] as f64 - sum / 4.).abs() <= 1. / 1024.,
                        "{name} {size:?} pixel {x},{y} H {} expected {}",
                        actual[0],
                        sum / 4.
                    );
                    let gx = i * size[0] / nw;
                    let gy = j * size[1] / nf;
                    let g = (gy * size[0] + gx) as usize;
                    assert_eq!(
                        [scalar(&f[0], g), scalar(&f[1], g), scalar(&f[2], g)],
                        [0., 0., 0.5],
                        "{name} {size:?} gap pixel {gx},{gy}"
                    );
                    count += 1;
                }
            }
        }
    }
    eprintln!("woven crossings passed {count}");
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_woven_flat() {
    let (s, p) = inputs();
    let ctx = context();
    let mut cache = PipelineCache::default();
    let c = controls(&p, &json!({"relief":0}));
    for size in sizes(&p) {
        let f = capture(&ctx, &mut cache, &s, &request(&p, &c, size), None, true);
        for (channel, expected) in [(4, [0f32, 0., 0., 1.]), (1, [0.5, 0.5, 1., 1.])] {
            let pixel: Vec<_> = expected
                .into_iter()
                .flat_map(|v| half::f16::from_f32(v).to_bits().to_le_bytes())
                .collect();
            exact(
                &f[channel],
                &pixel.repeat((size[0] * size[1]) as usize),
                size,
                8,
                &format!("flat channel {channel}"),
            );
        }
    }
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_woven_control_isolation() {
    let (s, p) = inputs();
    let ctx = context();
    let mut cache = PipelineCache::default();
    let size = [257, 129];
    let base = controls(&p, &json!({}));
    let req = request(&p, &base, size);
    let original = capture(&ctx, &mut cache, &s, &req, None, false);
    let old_fields = fields(&ctx, &mut cache, &s, &req);
    for variant in p["structuralProbes"]["isolation"]["variants"]
        .as_array()
        .unwrap()
    {
        let key = variant["control"].as_str().unwrap();
        let mut c = base.clone();
        c[key] = variant["value"].clone();
        let req = request(&p, &c, size);
        geometry(&s, &c, &req, &p);
        let a = capture(&ctx, &mut cache, &s, &req, None, false);
        let b = capture(&ctx, &mut cache, &s, &req, None, false);
        for (k, name) in ["baseColor", "normal", "roughness", "metallic", "height"]
            .into_iter()
            .enumerate()
        {
            exact(&a[k], &b[k], size, 4, &format!("repeat {key} {name}"));
            if variant["changed"]
                .as_array()
                .unwrap()
                .contains(&json!(name))
            {
                assert!(a[k] != original[k], "control {key} did not change {name}");
            } else {
                exact(
                    &a[k],
                    &original[k],
                    size,
                    4,
                    &format!("isolation {key} {name}"),
                );
            }
        }
        let f = fields(&ctx, &mut cache, &s, &req);
        normalized(&f, size, key);
        let geometry = p["structuralProbes"]["crossing"]["geometryControls"]
            .as_array()
            .unwrap()
            .contains(&json!(key));
        for i in 0..3 {
            if !geometry || i == 1 && ["crown", "underRatio"].contains(&key) {
                exact(
                    &f[i],
                    &old_fields[i],
                    size,
                    8,
                    &format!("control {key} {}", MODES[i]),
                );
            }
        }
        eprintln!("woven control passed {key}");
    }
}
fn replay_case(
    ctx: &GpuContext,
    cache: &mut PipelineCache,
    s: &Value,
    p: &Value,
    c: &Value,
    size: [u32; 2],
    name: &str,
) {
    let req = request(p, c, size);
    let f = capture(ctx, cache, s, &req, None, true);
    for offset in [[0, 0], [1, 0], [0, 1], [size[0] / 2, size[1] / 2]] {
        let actual = replay_normal(
            ctx,
            &periodic_shift(&f[4], size, offset),
            size,
            c["normalStrength"].as_f64().unwrap() as f32,
        );
        exact(
            &actual,
            &periodic_shift(&f[1], size, offset),
            size,
            8,
            &format!("{name} normal shift {offset:?}"),
        );
    }
    eprintln!("woven replay passed {name} {size:?}");
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_woven_normal_periodic() {
    let (s, p) = inputs();
    let ctx = context();
    let mut cache = PipelineCache::default();
    for preset in p["cases"].as_array().unwrap() {
        let c = controls(&p, &preset["controls"]);
        for size in sizes(&p) {
            replay_case(
                &ctx,
                &mut cache,
                &s,
                &p,
                &c,
                size,
                preset["id"].as_str().unwrap(),
            );
        }
    }
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_woven_stress() {
    let (s, p) = inputs();
    let ctx = context();
    let mut cache = PipelineCache::default();
    for preset in p["stress"].as_array().unwrap() {
        let c = controls(&p, &preset["controls"]);
        for size in sizes(preset) {
            let req = request(&p, &c, size);
            geometry(&s, &c, &req, &p);
            normalized(&fields(&ctx, &mut cache, &s, &req), size, "stress");
            replay_case(
                &ctx,
                &mut cache,
                &s,
                &p,
                &c,
                size,
                "stress outside default quality",
            );
        }
    }
}

// Monotonic finite binary16 ordering, including distinct adjacent -0/+0.
fn half_steps(a: u16, b: u16) -> u16 {
    assert!(half::f16::from_bits(a).is_finite() && half::f16::from_bits(b).is_finite());
    let rank = |v: u16| if v & 0x8000 != 0 { !v } else { v ^ 0x8000 };
    rank(a).abs_diff(rank(b))
}
fn weave_step_limit(size: [u32; 2], origin: [u32; 2]) -> u16 {
    if origin == [0, 0] || size.iter().all(|v| v.is_power_of_two()) {
        0
    } else {
        1
    }
}
#[test]
fn woven_periodic_amendment_keeps_exact_gates_and_bounds_zero_crossings() {
    for size in [[256, 256], [1024, 1024], [2048, 2048]] {
        for origin in [
            [0, 0],
            [size[0], 0],
            [0, size[1]],
            [size[0] + 3, size[1] + 5],
        ] {
            assert_eq!(weave_step_limit(size, origin), 0);
        }
    }
    assert_eq!(weave_step_limit([257, 129], [0, 0]), 0);
    assert_eq!(weave_step_limit([257, 129], [257, 0]), 1);
    assert_eq!(weave_step_limit([256, 129], [256, 0]), 1);
    for (a, b) in [
        (2149, 2150),
        (0, 1),
        (0x3bff, 0x3c00),
        (0x8000, 0),
        (0x8001, 0x8000),
    ] {
        assert_eq!(half_steps(a, b), 1);
        assert_eq!(half_steps(b, a), 1);
        assert_eq!(half_steps(a, a), 0);
    }
    assert_eq!(half_steps(0x8001, 1), 3);
    assert_eq!(half_steps(2148, 2150), 2);
}
#[derive(Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WeaveUlpStats {
    components: u64,
    differing_components: u64,
    differing_pixels: u64,
    max_half_steps: u16,
    max_absolute_difference: f64,
}
impl WeaveUlpStats {
    fn pixel(&mut self, actual: [u16; 4], expected: [u16; 4]) -> u16 {
        let mut maximum = 0;
        self.components += 4;
        self.differing_pixels += u64::from(actual != expected);
        for (a, b) in actual.into_iter().zip(expected) {
            let steps = half_steps(a, b);
            self.differing_components += u64::from(a != b);
            maximum = maximum.max(steps);
            self.max_half_steps = self.max_half_steps.max(steps);
            let delta = (f64::from(half::f16::from_bits(a).to_f32())
                - f64::from(half::f16::from_bits(b).to_f32()))
            .abs();
            self.max_absolute_difference = self.max_absolute_difference.max(delta);
        }
        maximum
    }
}
fn weave_periodic_receipt(context: &GpuContext, rows: &[Value], completed: bool) {
    let receipt = json!({"completed":completed,"ok":completed,"materialAccepted":false,"amendment":"2026-10-04-exact-where-exact","adapter":context.report().adapter(),"rows":rows});
    if let Ok(directory) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            std::path::Path::new(&directory).join("woven-weave-periodic.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires GPU; cargo xtask test-node weave-pattern or gpu-smoke"]
fn node_weave_pattern_gpu_woven_periodic() {
    let (source, plan) = inputs();
    let ctx = context();
    let mut cache = PipelineCache::default();
    let (production, entry) = crate::kernels::shader(mixture_core::plan::KernelId::WeavePattern);
    let anchor = "weave_box(vec2<f32>(gid.xy),vec2<f32>(size))";
    assert_eq!(production.matches(anchor).count(), 1);
    // Only the origin expression changes. Reserved float words 9/10 are zero at baseline.
    // No modulo here: the unchanged production function must establish periodicity.
    let instrumented = production.replace(
        anchor,
        "weave_box(vec2<f32>(gid.xy) + params.under_padding.yz,vec2<f32>(size))",
    );
    let mut comparisons = 0;
    let mut rows = Vec::new();
    assert_eq!(
        plan["structuralProbes"]["amendments"][0]["id"],
        "2026-10-04-exact-where-exact"
    );
    assert_eq!(
        plan["structuralProbes"]["amendments"][0]["after"]["maximumHalfSteps"],
        1
    );
    for preset in plan["cases"].as_array().unwrap() {
        let name = preset["id"].as_str().unwrap();
        let c = controls(&plan, &preset["controls"]);
        for size in sizes(&plan) {
            let req = request(&plan, &c, size);
            geometry(&source, &c, &req, &plan);
            let graph = fields(&ctx, &mut cache, &source, &req);
            for (mode, baseline) in graph.iter().enumerate() {
                assert!(
                    baseline.as_chunks::<8>().0.windows(2).any(|w| w[0] != w[1]),
                    "{name} {size:?} {} constant baseline",
                    MODES[mode]
                );
                let mut mode_stats = WeaveUlpStats::default();
                let mut origins_observed = 0;
                let f = |key: &str| (c[key].as_f64().unwrap() as f32).to_bits();
                for origin in [
                    [0, 0],
                    [size[0], 0],
                    [0, size[1]],
                    [size[0], size[1]],
                    [size[0] + 3, size[1] + 5],
                ] {
                    let words = [
                        c["warpCount"].as_u64().unwrap() as u32,
                        c["weftCount"].as_u64().unwrap() as u32,
                        mode as u32,
                        0,
                        f("warpWidth"),
                        f("weftWidth"),
                        f("bevel"),
                        f("crown"),
                        f("underRatio"),
                        (origin[0] as f32).to_bits(),
                        (origin[1] as f32).to_bits(),
                        0,
                    ];
                    let bytes: Vec<_> = words.into_iter().flat_map(u32::to_le_bytes).collect();
                    assert_eq!(bytes.len(), 48);
                    let pixels = super::periodic_scalar_readback::render(
                        &ctx,
                        size,
                        &bytes,
                        &instrumented,
                        entry,
                    );
                    let limit = weave_step_limit(size, origin);
                    let mut stats = WeaveUlpStats::default();
                    let mut failure = None;
                    for y in 0..size[1] {
                        for x in 0..size[0] {
                            let expected_index = (((y + origin[1]) % size[1]) * size[0]
                                + (x + origin[0]) % size[0])
                                as usize;
                            let expected: [u16; 4] = std::array::from_fn(|k| {
                                u16::from_le_bytes([
                                    baseline[expected_index * 8 + k * 2],
                                    baseline[expected_index * 8 + k * 2 + 1],
                                ])
                            });
                            let actual = pixels[(y * size[0] + x) as usize];
                            let steps = stats.pixel(actual, expected);
                            mode_stats.pixel(actual, expected);
                            if steps > limit && failure.is_none() {
                                failure = Some(
                                    json!({"pixel":[x,y],"actualHalf":actual,"expectedHalf":expected,"halfSteps":steps,"actual":actual.map(|v|half::f16::from_bits(v).to_f32()),"expected":expected.map(|v|half::f16::from_bits(v).to_f32())}),
                                );
                            }
                        }
                    }
                    origins_observed += 1;
                    let row = json!({"case":name,"size":size,"mode":MODES[mode],"origin":origin,"allowedHalfSteps":limit,"passed":failure.is_none(),"statistics":stats,"modeStatisticsThroughThisOrigin":mode_stats,"originsObserved":origins_observed,"modeComplete":origins_observed==5,"firstFailure":failure});
                    eprintln!("woven weave ULP row: {row}");
                    rows.push(row);
                    weave_periodic_receipt(&ctx, &rows, false);
                    assert!(
                        failure.is_none(),
                        "weave periodic case={name} size={size:?} mode={} origin={origin:?} limit={limit} firstFailure={failure:?}; full failing-image statistics retained",
                        MODES[mode]
                    );
                    comparisons += 1;
                }
            }
        }
    }
    assert_eq!(comparisons, 12 * 4 * 3 * 5);
    weave_periodic_receipt(&ctx, &rows, true);
    eprintln!(
        "woven weave periodic completed: {comparisons} amended comparisons including exact zero-origin graph identity"
    );
}
