//! Public consumer: no renderer-internal helpers or CPU pixel implementation.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::{Value, json};
#[test]
#[ignore = "requires GPU; invoked by check-weave.mjs"]
fn public_weave_modes_and_browser_parity() {
    let edge =
        |a, ap, b, bp| json!({"from":{"nodeId":a,"portId":ap},"to":{"nodeId":b,"portId":bp}});
    let names = [
        "mode",
        "warpCount",
        "weftCount",
        "warpWidth",
        "weftWidth",
        "bevel",
        "crown",
        "underRatio",
    ];
    let source = json!({"version":1,"nodes":[{"id":"weave","type":"weave-pattern","version":1},{"id":"color","type":"constant-color","version":1},{"id":"out","type":"material-output","version":1}],"edges":[edge("weave","value","out","height"),edge("color","color","out","baseColor")],"exposedParameters":names.iter().map(|id|json!({"id":id,"nodeId":"weave","parameterId":id})).collect::<Vec<_>>()});
    let document = MaterialDocument::decode(
        &serde_json::to_vec(&source).unwrap(),
        &SafetyLimits::default(),
    )
    .unwrap()
    .into_validated(&SafetyLimits::default())
    .unwrap();
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        Ok("metal") => BackendPreference::Metal,
        Ok("auto") | Err(std::env::VarError::NotPresent) => BackendPreference::Auto,
        other => panic!("invalid backend {other:?}"),
    };
    let software = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Ok("1") => true,
        Ok("0") | Err(std::env::VarError::NotPresent) => false,
        other => panic!("invalid software policy {other:?}"),
    };
    let context = pollster::block_on(GpuContext::request(GpuContextOptions {
        backend,
        software_adapter: software,
        ..Default::default()
    }))
    .unwrap();
    let adapter = serde_json::to_value(context.report().adapter()).unwrap();
    let mut renderer = Renderer::new(context);

    if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
        assert!(adapter.to_string().contains(&expected), "{adapter}");
    }
    let browser_path = std::env::var("MIXTURE_WEAVE_BROWSER")
        .ok()
        .map(std::path::PathBuf::from);
    let browser: Option<Value> = browser_path
        .as_ref()
        .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap());
    let mut rows = Vec::new();
    let mut maximum = 0u8;
    for (case, size, geometry) in [
        ("default-256", [256, 256], json!({})),
        ("default-1024", [1024, 1024], json!({})),
        ("default-2048", [2048, 2048], json!({})),
        (
            "unequal",
            [257, 129],
            json!({"warpCount":12,"weftCount":8,"warpWidth":0.55,"weftWidth":0.9}),
        ),
        (
            "low",
            [256, 256],
            json!({"warpCount":4,"weftCount":4,"warpWidth":0.55,"weftWidth":0.55,"bevel":0.02,"crown":0,"underRatio":0.25}),
        ),
        (
            "high",
            [256, 256],
            json!({"warpCount":32,"weftCount":32,"warpWidth":0.9,"weftWidth":0.9,"bevel":0.12,"crown":1,"underRatio":0.75}),
        ),
    ] {
        for mode in ["height", "coverage", "warp-share"] {
            let mut overrides = geometry.clone();
            overrides["mode"] = json!(mode);
            let request = CompileRequest {
                size,
                outputs: vec![OutputChannel::Height],
                overrides: serde_json::from_value(overrides).unwrap(),
                ..Default::default()
            };
            let plan = compile(&document, &request).unwrap();
            assert_eq!(plan.passes().len(), 1);
            let result = pollster::block_on(renderer.render(&plan)).unwrap();
            let repeat = pollster::block_on(renderer.render(&plan)).unwrap();
            let actual = result.channels()[0].pixels();
            assert_eq!(actual, repeat.channels()[0].pixels());
            assert_eq!(result.report().allocations.live_bytes, 0);
            let mut delta = 0;
            if let Some(browser) = &browser {
                let row = &browser["rows"][rows.len()];
                assert_eq!(row["case"], json!(case));
                assert_eq!(row["mode"], json!(mode));
                assert_eq!(row["size"], json!(size));
                assert_eq!(row["planHash"], json!(plan.hash().as_str()));
                let filename = row["file"].as_str().unwrap();
                assert!(!filename.contains('/') && !filename.contains('\\'));
                let expected = std::fs::read(
                    browser_path
                        .as_ref()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join(filename),
                )
                .unwrap();
                assert_eq!(actual.len(), expected.len());
                delta = actual
                    .iter()
                    .zip(&expected)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap();
                assert!(delta <= 1, "{case}/{mode}: {delta}");
                maximum = maximum.max(delta);
            }
            rows.push(json!({"case":case,"mode":mode,"size":size,"planHash":plan.hash().as_str(),"repeatExact":true,"maxComponentDelta":delta}));
        }
    }
    assert_eq!(rows.len(), 18);
    if let Some(b) = &browser {
        assert_eq!(b["rows"].as_array().unwrap().len(), 18);
    }
    if let Ok(path) = std::env::var("MIXTURE_WEAVE_EVIDENCE") {
        std::fs::write(path,serde_json::to_vec_pretty(&json!({"ok":true,"adapter":adapter,"rows":rows,"browserCompared":browser.is_some(),"maxComponentDelta":maximum})).unwrap()).unwrap();
    }
}
