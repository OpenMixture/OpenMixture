//! Frozen @2 structural observations through production WGSL and public graph execution.
//! The analytical function below is a sparse test oracle, never an image executor.
use half::f16;
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};
use mixture_wgpu::{GpuContext, Renderer};
use serde_json::{Value, json};
const SHADER: &str = concat!(
    include_str!("../../shaders/precision.wgsl"),
    "\n",
    include_str!("../../shaders/nodes/weave-pattern-v2.wgsl")
);
const ALL: [[u32; 2]; 4] = [[256, 256], [1024, 1024], [2048, 2048], [257, 129]];
fn selected(cpu: bool) -> Vec<[u32; 2]> {
    ALL.into_iter()
        .filter(|s| !cpu || *s != [2048, 2048])
        .collect()
}
#[test]
fn hardware_and_software_sizes_are_explicit() {
    assert_eq!(selected(false), ALL);
    assert_eq!(selected(true), [[256, 256], [1024, 1024], [257, 129]]);
}
fn n(p: &Value, k: &str) -> f32 {
    p[k].as_f64().unwrap() as f32
}
fn words(p: &Value, mode: u32, origin: [u32; 2]) -> Vec<u8> {
    [
        n(p, "warpCount") as u32,
        n(p, "weftCount") as u32,
        mode,
        0,
        n(p, "warpWidth").to_bits(),
        n(p, "weftWidth").to_bits(),
        n(p, "bevel").to_bits(),
        n(p, "crown").to_bits(),
        n(p, "underRatio").to_bits(),
        (origin[0] as f32).to_bits(),
        (origin[1] as f32).to_bits(),
        0,
    ]
    .into_iter()
    .flat_map(u32::to_le_bytes)
    .collect()
}
fn raw(
    ctx: &GpuContext,
    p: &Value,
    mode: u32,
    size: [u32; 2],
    origin: [u32; 2],
    shader: &str,
) -> Vec<[u16; 4]> {
    super::periodic_scalar_readback::render(
        ctx,
        size,
        &words(p, mode, origin),
        shader,
        "weave_pattern",
    )
}
fn f(v: u16) -> f32 {
    f16::from_bits(v).to_f32()
}
fn half(v: f32) -> f32 {
    f16::from_f32(v).to_f32()
}
fn save(receipt: &Value) {
    if let Ok(dir) = std::env::var("MIXTURE_NODE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            std::path::Path::new(&dir).join("weave-pattern-v2-probes.json"),
            serde_json::to_vec_pretty(receipt).unwrap(),
        )
        .unwrap();
    }
}
fn edge(a: &str, ap: &str, b: &str, bp: &str) -> Value {
    json!({"from":{"nodeId":a,"portId":ap},"to":{"nodeId":b,"portId":bp}})
}
fn graph(p: &Value) -> MaterialDocument {
    let mut h = p.clone();
    h["mode"] = json!("height");
    let mut c = p.clone();
    c["mode"] = json!("coverage");
    let mut s = p.clone();
    s["mode"] = json!("warp-share");
    let doc = json!({"version":1,"nodes":[{"id":"h","type":"weave-pattern","version":2,"parameters":h},{"id":"c","type":"weave-pattern","version":2,"parameters":c},{"id":"s","type":"weave-pattern","version":2,"parameters":s},{"id":"r","type":"constant-color","version":1,"parameters":{"value":[1,0,0,1]}},{"id":"g","type":"constant-color","version":1,"parameters":{"value":[0,1,0,1]}},{"id":"b","type":"constant-color","version":1,"parameters":{"value":[0,0,1,1]}},{"id":"yarn","type":"blend","version":1},{"id":"surface","type":"blend","version":1},{"id":"relief","type":"levels","version":1,"parameters":{"outputMax":0.025}},{"id":"normal","type":"height-to-normal","version":1,"parameters":{"strength":0.5}},{"id":"out","type":"material-output","version":1}],"edges":[edge("g","color","yarn","a"),edge("r","color","yarn","b"),edge("s","value","yarn","mask"),edge("b","color","surface","a"),edge("yarn","color","surface","b"),edge("c","value","surface","mask"),edge("surface","color","out","baseColor"),edge("h","value","relief","in"),edge("relief","value","normal","in"),edge("normal","normal","out","normal")]});
    MaterialDocument::decode(&serde_json::to_vec(&doc).unwrap(), &SafetyLimits::default()).unwrap()
}
fn width(p: &Value, size: [u32; 2], pixels: &[u8]) -> Value {
    let mut results = Vec::new();
    for warp in [true, false] {
        let (trans, along, nc, mc, w) = if warp {
            (
                size[0],
                size[1],
                n(p, "warpCount") as u32,
                n(p, "weftCount") as u32,
                n(p, "warpWidth"),
            )
        } else {
            (
                size[1],
                size[0],
                n(p, "weftCount") as u32,
                n(p, "warpCount") as u32,
                n(p, "weftWidth"),
            )
        };
        let mut values = Vec::new();
        for i in 0..nc {
            let core: Vec<_> = (0..trans)
                .filter(|x| {
                    let u = (*x as f64 + 0.5) / trans as f64 * nc as f64;
                    u.floor() as u32 == i
                        && (u.fract() - 0.5).abs() <= (w as f64 - n(p, "bevel") as f64) / 2.
                })
                .collect();
            assert!(!core.is_empty());
            for y in 0..along {
                let j = ((y as f64 + 0.5) / along as f64 * mc as f64).floor() as u32;
                if (i + j) % 2 != u32::from(!warp) {
                    continue;
                }
                let good = core
                    .iter()
                    .filter(|x| {
                        let k = if warp {
                            y * size[0] + **x
                        } else {
                            **x * size[0] + y
                        } as usize
                            * 4;
                        if warp {
                            pixels[k] > pixels[k + 1]
                        } else {
                            pixels[k + 1] > pixels[k]
                        }
                    })
                    .count();
                values.push(good as f64 / core.len() as f64);
            }
        }
        values.sort_by(f64::total_cmp);
        results.push(json!({"axis":if warp{"warp"}else{"weft"},"min":values[0],"median":(values[(values.len()-1)/2]+values[values.len()/2])/2.,"scanlines":values.len()}));
    }
    json!(results)
}
fn smooth(t: f64) -> f64 {
    let s = t.clamp(0., 1.);
    s * s * (3. - 2. * s)
}
fn frac(x: f64) -> f64 {
    x - x.floor()
}
// Sparse closed-form oracle used only at explicitly selected boundary pixels.
fn analytical(uv: [f64; 2], p: &Value) -> [f64; 3] {
    let a = |k| p[k].as_f64().unwrap();
    let x = frac(uv[0]) * a("warpCount");
    let y = frac(uv[1]) * a("weftCount");
    let i = x.floor() as i32;
    let j = y.floor() as i32;
    let u = frac(x) - 0.5;
    let v = frac(y) - 0.5;
    let aw = smooth((a("warpWidth") / 2. - u.abs()) / a("bevel"));
    let af = smooth((a("weftWidth") / 2. - v.abs()) / a("bevel"));
    let profile = |q: f64| q * (1. - a("crown") + a("crown") * q);
    let tw = profile((1. - (2. * u / a("warpWidth")).powi(2)).max(0.));
    let tf = profile((1. - (2. * v / a("weftWidth")).powi(2)).max(0.));
    let sy = smooth(frac(y - 0.5));
    let sx = smooth(frac(x - 0.5));
    let lw = if (i + (y - 0.5).floor() as i32).rem_euclid(2) == 0 {
        1. - sy
    } else {
        sy
    };
    let lf = if (j + (x - 0.5).floor() as i32).rem_euclid(2) == 0 {
        sx
    } else {
        1. - sx
    };
    let hw = tw * (a("underRatio") + (1. - a("underRatio")) * lw);
    let hf = tf * (a("underRatio") + (1. - a("underRatio")) * lf);
    let zw = aw * hw;
    let zf = af * hf;
    let d = if (i + j).rem_euclid(2) == 0 { 1. } else { 0. };
    [
        zw + zf - zw * zf,
        aw + (1. - aw) * af,
        aw * (1. - af * (1. - d)),
    ]
}
fn expected_height(pixel: [u32; 2], size: [u32; 2], p: &Value) -> f32 {
    let mut h = 0.;
    for offset in [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75], [0.75, 0.75]] {
        h += analytical(
            [
                (pixel[0] as f64 + offset[0]) / size[0] as f64,
                (pixel[1] as f64 + offset[1]) / size[1] as f64,
            ],
            p,
        )[0];
    }
    half(half((h / 4.) as f32) * 0.025)
}
fn continuity(ctx: &GpuContext, p: &Value) -> Value {
    let nw = n(p, "warpCount") as f64;
    let nf = n(p, "weftCount") as f64;
    let mut uvs = Vec::new();
    for u in [
        0.,
        0.5 / nw,
        1. / nw,
        (0.5 + n(p, "warpWidth") as f64 / 2.) / nw,
    ] {
        for v in [
            0.,
            0.5 / nf,
            1. / nf,
            (0.5 + n(p, "weftWidth") as f64 / 2.) / nf,
        ] {
            for axis in 0..2 {
                for sign in [-1., 1.] {
                    let mut uv = [u, v];
                    uv[axis] += sign * 1e-5;
                    uvs.push(uv);
                }
            }
        }
    }
    let mut body = String::from("var uv=vec2<f32>(0.0);switch gid.y*32u+gid.x {");
    for (i, uv) in uvs.iter().enumerate() {
        body.push_str(&format!(
            "case {i}u: {{uv=vec2<f32>({:.9},{:.9});}}",
            uv[0], uv[1]
        ));
    }
    body.push_str("default:{} } let s=weave_sample(uv);textureStore(output,vec2<i32>(gid.xy),mixture_half4(vec4<f32>(s.fields[params.counts_mode.z],0.0,0.0,1.0)));");
    let shader = format!(
        "{SHADER}\n@compute @workgroup_size(8,8,1) fn probe(@builtin(global_invocation_id) gid:vec3<u32>) {{if any(gid.xy>=textureDimensions(output)){{return;}}{body}}}"
    );
    let pixels: Vec<_> = (0..3)
        .map(|field| {
            super::periodic_scalar_readback::render(
                ctx,
                [32, 2],
                &words(p, field, [0, 0]),
                &shader,
                "probe",
            )
        })
        .collect();
    let mut maximum = 0f32;
    for (i, uv) in uvs.iter().enumerate() {
        let want = analytical(*uv, p);
        for (k, w) in want.iter().enumerate() {
            let got = f(pixels[k][i][0]);
            assert!(
                (got - *w as f32).abs() <= 1. / 1024.,
                "sparse case={p} uv={uv:?} field={k} actual={got} expected={w}"
            );
        }
        if i % 2 == 1 {
            let d = (f(pixels[0][i][0]) - f(pixels[0][i - 1][0])).abs();
            maximum = maximum.max(d);
            assert!(d <= 0.01, "C1 case={p} uv={uv:?} delta={d}");
        }
    }
    json!({"samples":uvs.len(),"maximumHeightDelta":maximum,"sparseAnalyticalTolerance":1./1024.})
}
fn normal_oracle(p: &Value, pixels: &[u8]) -> f32 {
    let size = [1024, 1024];
    let mut maximum = 0f32;
    let nw = n(p, "warpCount") as f64;
    let nf = n(p, "weftCount") as f64;
    for u in [
        0.,
        0.5 / nw,
        1. / nw,
        (0.5 + n(p, "warpWidth") as f64 / 2.) / nw,
    ] {
        for v in [
            0.,
            0.5 / nf,
            1. / nf,
            (0.5 + n(p, "weftWidth") as f64 / 2.) / nf,
        ] {
            for delta in [-1i32, 0, 1] {
                let x = ((u * 1024.).floor() as i32 + delta).rem_euclid(1024) as u32;
                let y = ((v * 1024.).floor() as i32 + delta).rem_euclid(1024) as u32;
                let du = (expected_height([(x + 1) % 1024, y], size, p)
                    - expected_height([(x + 1023) % 1024, y], size, p))
                    * 512.
                    * 0.5;
                let dv = (expected_height([x, (y + 1) % 1024], size, p)
                    - expected_height([x, (y + 1023) % 1024], size, p))
                    * 512.
                    * 0.5;
                let len = (du * du + dv * dv + 1.).sqrt();
                for (k, want) in [-du / len, dv / len, 1. / len].into_iter().enumerate() {
                    let want = half(want * 0.5 + 0.5);
                    let got = pixels[((y * 1024 + x) * 4) as usize + k] as f32 / 255.;
                    let delta = (got - want).abs();
                    assert!(
                        delta <= 4. / 255.,
                        "normal case={p} pixel={x},{y} component={k} got={got} want={want}"
                    );
                    maximum = maximum.max(delta);
                }
            }
        }
    }
    maximum
}
pub(super) fn run(renderer: &mut Renderer) -> Value {
    let plan: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/nodes/weave-pattern-v2/acceptance-plan.json"
    ))
    .unwrap();
    assert_eq!(plan["frozen"], true);
    let cpu = renderer.context().report().adapter().unwrap().device_type == "Cpu";
    let sizes = selected(cpu);
    assert!(cpu || sizes.contains(&[2048, 2048]));
    let mut receipt = json!({"ok":false,"adapter":renderer.context().report().adapter(),"executedSizes":sizes,"rows":[],"omitted":[]});
    let anchor = "weave_box(vec2<f32>(gid.xy),vec2<f32>(size))";
    assert_eq!(SHADER.matches(anchor).count(), 1);
    let shifted = SHADER.replace(
        anchor,
        "weave_box(vec2<f32>(gid.xy)+params.under_padding.yz,vec2<f32>(size))",
    );
    for case in plan["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let p = &case["parameters"];
        let continuity = continuity(renderer.context(), p);
        receipt["rows"]
            .as_array_mut()
            .unwrap()
            .push(json!({"case":id,"probe":"continuity","result":continuity}));
        save(&receipt);
        if cpu {
            receipt["omitted"].as_array_mut().unwrap().push(json!({"case":id,"size":[2048,2048],"status":"notRunOnSoftware","amendment":"2026-10-04-software-size-scope"}));
        }
        for size in sizes.iter().copied() {
            if size[0] == 1024 || size[0] == 2048 {
                let graph = graph(p).into_validated(&SafetyLimits::default()).unwrap();
                let request = CompileRequest {
                    size,
                    outputs: vec![OutputChannel::BaseColor, OutputChannel::Normal],
                    ..Default::default()
                };
                let render_plan = compile(&graph, &request).unwrap();
                let out = pollster::block_on(renderer.render(&render_plan)).unwrap();
                let color = out
                    .channels()
                    .iter()
                    .find(|c| c.channel == OutputChannel::BaseColor)
                    .unwrap();
                let m = width(p, size, color.pixels());
                receipt["rows"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"case":id,"size":size,"probe":"visible-core","metric":m}));
                save(&receipt);
                for axis in m.as_array().unwrap() {
                    assert_eq!(axis["min"], 1., "visible-core {id} {size:?} {axis}");
                    assert_eq!(axis["median"], 1.);
                }
                if size[0] == 1024 {
                    let normal = out
                        .channels()
                        .iter()
                        .find(|c| c.channel == OutputChannel::Normal)
                        .unwrap();
                    let error = normal_oracle(p, normal.pixels());
                    receipt["rows"].as_array_mut().unwrap().push(
                        json!({"case":id,"probe":"normal-boundaries","maximumEncodedError":error}),
                    );
                }
            }
            let odd = size == [257, 129];
            let zero_only = odd
                && !plan["gates"]["periodic"]["oddCases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v == id);
            let ctx = renderer.context();
            let bases: Vec<_> = (0..3)
                .map(|mode| raw(ctx, p, mode, size, [0, 0], SHADER))
                .collect();
            for mode in 0..3 {
                assert!(
                    bases[mode].windows(2).any(|w| w[0] != w[1]),
                    "constant baseline {id} {size:?} {mode}"
                );
                for origin in [
                    [0, 0],
                    [size[0], 0],
                    [0, size[1]],
                    [size[0], size[1]],
                    [size[0] + 3, size[1] + 5],
                ] {
                    if zero_only && origin != [0, 0] {
                        continue;
                    }
                    let got = raw(ctx, p, mode as u32, size, origin, &shifted);
                    let weighted = odd && mode == 2 && origin != [0, 0];
                    let coverage = if weighted {
                        Some(raw(ctx, p, 1, size, origin, &shifted))
                    } else {
                        None
                    };
                    let mut max_steps = 0;
                    let mut changed = 0;
                    let mut max_p = 0f32;
                    let mut failure = None;
                    for y in 0..size[1] {
                        for x in 0..size[0] {
                            let at = (y * size[0] + x) as usize;
                            let from = (((y + origin[1]) % size[1]) * size[0]
                                + (x + origin[0]) % size[0])
                                as usize;
                            let actual = got[at];
                            let want = bases[mode][from];
                            let steps = actual[0].abs_diff(want[0]);
                            max_steps = max_steps.max(steps);
                            changed += usize::from(steps > 0);
                            if let Some(c) = &coverage {
                                max_p = max_p.max(
                                    (f(actual[0]) * f(c[at][0])
                                        - f(want[0]) * f(bases[1][from][0]))
                                    .abs(),
                                );
                            }
                            let limit = if odd && origin != [0, 0] { 1 } else { 0 };
                            if (!weighted && steps > limit) || actual[1..] != want[1..] {
                                failure.get_or_insert(json!({"pixel":[x,y],"actual":actual,"expected":want,"steps":steps,"limit":limit}));
                            }
                        }
                    }
                    let row = json!({"case":id,"size":size,"mode":mode,"origin":origin,"qualityScope":if weighted{"observation-only odd share; not a pass"}else{"gated"},"passed":if weighted{Value::Null}else{json!(failure.is_none())},"changedPixels":changed,"maxHalfSteps":max_steps,"maxVisibleWeightDelta":max_p,"firstFailure":failure});
                    eprintln!("weave v2 {row}");
                    receipt["rows"].as_array_mut().unwrap().push(row);
                    save(&receipt);
                    assert!(
                        failure.is_none(),
                        "frozen weave v2 translation failed {id} {size:?} mode={mode} origin={origin:?}: {failure:?}"
                    );
                }
            }
        }
    }
    receipt["ok"] = json!(true);
    save(&receipt);
    receipt
}
