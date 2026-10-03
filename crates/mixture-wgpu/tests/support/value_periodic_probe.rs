//! Unwrapped sampling-origin probe using production fixed value-noise v2 and its ABI.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};

pub(super) fn run(context: &GpuContext) -> Value {
    run_parameters(
        context,
        &[
            (8, 3, 1729),
            (8, 3, u32::MAX),
            (32, 2, 65537),
            (32, 2, u32::MAX),
            (64, 3, 1729),
        ],
        "selected MAT-02 inputs; unwrapped production fixed v2 sampling",
    )
}
pub(super) fn run_woven(context: &GpuContext) -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/materials/woven-fabric");
    let plan: Value =
        serde_json::from_slice(&std::fs::read(root.join("qualification-plan.json")).unwrap())
            .unwrap();
    let source: Value =
        serde_json::from_slice(&std::fs::read(root.join("material.mix")).unwrap()).unwrap();
    for id in ["n00-weftNoise", "n03-warpNoise"] {
        let node = source["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"] == id)
            .unwrap();
        assert_eq!(node["version"], 2);
        assert_eq!(node["parameters"]["scale"], 4);
        assert_eq!(node["parameters"]["octaves"], 2);
        assert_eq!(node["parameters"]["persistence"], 0.5);
        assert_eq!(node["parameters"]["basis"], "value");
        let key = if id == "n00-weftNoise" {
            "weftSeed"
        } else {
            "warpSeed"
        };
        assert_eq!(node["parameters"]["seed"], plan["defaults"][key]);
    }
    assert_eq!(
        plan["structuralProbes"]["noisePeriodicity"]["parameters"],
        json!([[4, 2, 1729], [4, 2, 65537], [4, 2, u32::MAX], [4, 2, 0]])
    );
    assert_eq!(
        plan["structuralProbes"]["noisePeriodicity"]["sizes"],
        json!([[256, 256], [1024, 1024], [2048, 2048], [257, 129]])
    );
    run_parameters(
        context,
        &[(4, 2, 1729), (4, 2, 65537), (4, 2, u32::MAX), (4, 2, 0)],
        "frozen MAT-03 grain inputs; unwrapped production fixed v2 sampling",
    )
}
fn run_parameters(context: &GpuContext, parameters: &[(u32, u32, u32)], scope: &str) -> Value {
    eprintln!(
        "periodic input adapter: {}",
        json!(context.report().adapter())
    );
    if let Ok(name) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
        assert_eq!(json!(context.report().adapter())["name"], name);
    }
    let mut cases = Vec::new();
    for size in [[256, 256], [1024, 1024], [2048, 2048], [257, 129]] {
        for &parameters in parameters {
            let original = render(context, size, parameters, None);
            assert_eq!(
                original,
                render(context, size, parameters, Some([0, 0])),
                "zero-origin instrumentation changed production baseline"
            );
            assert!(
                original.windows(2).any(|p| p[0] != p[1]),
                "nonconstant noise required"
            );
            for origin in [[size[0], 0], [0, size[1]], [size[0] + 3, size[1] + 5]] {
                let shifted = render(context, size, parameters, Some(origin));
                for y in 0..size[1] {
                    for x in 0..size[0] {
                        let before = (((y + origin[1]) % size[1]) * size[0]
                            + (x + origin[0]) % size[0])
                            as usize;
                        let after = (y * size[0] + x) as usize;
                        assert_eq!(
                            shifted[after], original[before],
                            "unwrapped v2 sampling {size:?} {parameters:?} origin {origin:?} pixel {x},{y}"
                        );
                    }
                }
                cases.push(json!({"size":size,"scale":parameters.0,"octaves":parameters.1,"seed":parameters.2,"persistence":0.5,"origin":origin,"rawHalfExact":true}));
            }
        }
    }
    assert_eq!(cases.len(), parameters.len() * 12);
    json!({"ok":true,"materialAccepted":false,"adapter":context.report().adapter(),"scope":scope,"cases":cases})
}

fn render(
    context: &GpuContext,
    size: [u32; 2],
    parameters: (u32, u32, u32),
    origin: Option<[u32; 2]>,
) -> Vec<[u16; 4]> {
    let (scale, octaves, seed) = parameters;
    let shift = origin.unwrap_or([0, 0]);
    let mut bytes: Vec<u8> = [seed, scale, octaves, 2]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    bytes.extend(
        [0.5, shift[0] as f32, shift[1] as f32, 0.]
            .into_iter()
            .flat_map(f32::to_le_bytes),
    );
    let production = concat!(
        include_str!("../../shaders/precision.wgsl"),
        "\n",
        include_str!("../../shaders/nodes/fractal-noise.wgsl")
    );
    let shader = if origin.is_some() {
        let anchor = "fixed_fractal(id.xy, size)";
        assert_eq!(production.matches(anchor).count(), 1);
        // Intentionally no modulo here: lattice wrapping must establish periodicity.
        production.replace(
            anchor,
            "fixed_fractal(id.xy + vec2<u32>(u32(parameters._pad0), u32(parameters._pad1)), size)",
        )
    } else {
        production.to_owned()
    };
    super::periodic_scalar_readback::render(context, size, &bytes, &shader, "fractal_noise")
}
