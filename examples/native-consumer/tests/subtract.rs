//! Independent public consumer verifies subtraction, amplification and browser parity.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::{Value, json};

#[test]
#[ignore = "requires GPU; invoked by check-subtract.mjs"]
fn public_subtraction_retains_small_edges_and_browser_parity() {
    let source: Value = serde_json::from_slice(include_bytes!("scalar-subtract.mix")).unwrap();
    let mut amplified = source.clone();
    amplified["nodes"].as_array_mut().unwrap().push(json!({"id":"amplify","type":"levels","version":1,"parameters":{"inputMin":0,"inputMax":1.0/4096.0}}));
    for edge in amplified["edges"].as_array_mut().unwrap() {
        if edge["from"]["nodeId"] == "mix" {
            edge["from"]["nodeId"] = json!("amplify");
        }
    }
    amplified["edges"].as_array_mut().unwrap().push(
        json!({"from":{"nodeId":"mix","portId":"value"},"to":{"nodeId":"amplify","portId":"in"}}),
    );
    let documents = [source, amplified].map(|v| {
        MaterialDocument::decode(&serde_json::to_vec(&v).unwrap(), &SafetyLimits::default())
            .unwrap()
            .into_validated(&SafetyLimits::default())
            .unwrap()
    });
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
    let browser: Option<Value> = std::env::var("MIXTURE_SUBTRACT_BROWSER")
        .ok()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap());
    let mut rows = Vec::new();
    let cases = [
        (0.0, 0.0, 0u8),
        (1.0, 0.0, 255),
        (0.0, 1.0, 0),
        (1.0, 1.0, 0),
        (0.5, 0.5, 0),
        (0.5, 0.5 - 1.0 / 4096.0, 0),
        (0.25, 0.75, 0),
        (0.75, 0.25, 128),
    ];
    for size in [[1, 1], [1, 17], [17, 1], [19, 11]] {
        for (id, (a, b, gray)) in cases.into_iter().enumerate() {
            for (amplify, document) in documents.iter().enumerate() {
                let request = CompileRequest {
                    size,
                    outputs: vec![OutputChannel::Height],
                    overrides: serde_json::from_value(json!({"a":a,"b":b})).unwrap(),
                    ..Default::default()
                };
                let plan = compile(document, &request).unwrap();
                let output = pollster::block_on(renderer.render(&plan)).unwrap();
                let repeat = pollster::block_on(renderer.render(&plan)).unwrap();
                let pixels = output.channels()[0].pixels();
                assert_eq!(pixels, repeat.channels()[0].pixels());
                assert_eq!(output.report().allocations.live_bytes, 0);
                let want = if amplify == 0 {
                    gray
                } else if [1, 5, 7].contains(&id) {
                    255
                } else {
                    0
                };
                for pixel in pixels.as_chunks::<4>().0 {
                    assert_eq!(pixel, &[want, want, want, 255]);
                }
                if let Some(browser) = &browser {
                    let row = &browser["rows"][rows.len()];
                    assert_eq!(row["size"], json!(size));
                    assert_eq!(row["id"], json!(id));
                    assert_eq!(row["a"].as_f64(), Some(a));
                    assert_eq!(row["b"].as_f64(), Some(b));
                    assert_eq!(row["amplified"], json!(amplify == 1));
                    assert_eq!(row["planHash"], json!(plan.hash().as_str()));
                    let other: Vec<u8> = serde_json::from_value(row["pixels"].clone()).unwrap();
                    assert_eq!(pixels, other);
                }
                rows.push(json!({"size":size,"id":id,"a":a,"b":b,"amplified":amplify==1,"planHash":plan.hash().as_str(),"pixels":pixels,"repeatExact":true}));
            }
        }
    }
    assert_eq!(rows.len(), 64);
    if let Some(browser) = &browser {
        assert_eq!(browser["rows"].as_array().unwrap().len(), rows.len());
    }
    if let Ok(path) = std::env::var("MIXTURE_SUBTRACT_EVIDENCE") {
        std::fs::write(path,serde_json::to_vec_pretty(&json!({"ok":true,"adapter":adapter,"rows":rows,"browserCompared":browser.is_some(),"maxComponentDelta":if browser.is_some(){Some(0)}else{None}})).unwrap()).unwrap();
    }
}
