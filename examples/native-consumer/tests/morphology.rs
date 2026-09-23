//! Independent public Native consumer, sharing no renderer-internal test helpers.
use mixture_core::{
    CompileRequest, ImageBinding, MaterialDocument, OutputChannel, ResourceLimits, SafetyLimits,
    prepare,
};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, Renderer};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[test]
#[ignore = "requires GPU; invoked by check-morphology.mjs"]
fn public_morphology_supports_and_browser_parity() {
    let edge =
        |a, ap, b, bp| json!({"from":{"nodeId":a,"portId":ap},"to":{"nodeId":b,"portId":bp}});
    let source = json!({"version":1,"nodes":[
        {"id":"image","type":"image-input","version":1,"parameters":{"resourceId":"source"}},
        {"id":"morph","type":"scalar-morphology","version":1},
        {"id":"color","type":"constant-color","version":1},
        {"id":"out","type":"material-output","version":1}],
        "edges":[edge("image","value","morph","in"),edge("morph","value","out","height"),edge("color","color","out","baseColor")],
        "exposedParameters":[{"id":"operation","nodeId":"morph","parameterId":"operation"},{"id":"axis","nodeId":"morph","parameterId":"axis"},{"id":"radius","nodeId":"morph","parameterId":"radius"}]});
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
    let browser: Option<Value> = std::env::var("MIXTURE_MORPHOLOGY_BROWSER")
        .ok()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap());
    let mut rows = Vec::new();
    for size in [[1u32, 1u32], [1, 17], [17, 1], [19, 11]] {
        let [w, h] = size;
        let fields: Vec<BTreeSet<u32>> = vec![
            [0].into(),
            [(h / 2) * w + w / 2].into(),
            (0..h).map(|y| y * w + w / 2).collect(),
            [0, w - 1, (h - 1) * w, w * h - 1].into(),
        ];
        for (field, support) in fields.iter().enumerate() {
            let pixels: Vec<u8> = (0..w * h)
                .flat_map(|i| [if support.contains(&i) { 255 } else { 0 }, 31, 199, 0])
                .collect();
            for operation in ["erode", "dilate"] {
                for axis in ["x", "y"] {
                    for radius in [0i32, 1, 16] {
                        let mut expected = support.clone();
                        for k in -radius..=radius {
                            let moved: BTreeSet<_> = support
                                .iter()
                                .map(|&i| {
                                    let x = (i % w) as i32;
                                    let y = (i / w) as i32;
                                    let nx = (x + if axis == "x" { k } else { 0 })
                                        .rem_euclid(w as i32)
                                        as u32;
                                    let ny = (y + if axis == "y" { k } else { 0 })
                                        .rem_euclid(h as i32)
                                        as u32;
                                    ny * w + nx
                                })
                                .collect();
                            expected = if operation == "erode" {
                                expected.intersection(&moved).copied().collect()
                            } else {
                                expected.union(&moved).copied().collect()
                            };
                        }
                        let request = CompileRequest {
                            size,
                            outputs: vec![OutputChannel::Height],
                            overrides: serde_json::from_value(
                                json!({"operation":operation,"axis":axis,"radius":radius}),
                            )
                            .unwrap(),
                            ..Default::default()
                        };
                        let prepared = prepare(
                            &document,
                            &request,
                            &[ImageBinding {
                                id: "source",
                                width: w,
                                height: h,
                                format: "rgba8-linear",
                                bytes_per_row: w as u64 * 4,
                                data: &pixels,
                            }],
                            &ResourceLimits::default(),
                        )
                        .unwrap();
                        let output =
                            pollster::block_on(renderer.render_prepared(&prepared)).unwrap();
                        let repeat =
                            pollster::block_on(renderer.render_prepared(&prepared)).unwrap();
                        let actual = output.channels()[0].pixels();
                        assert_eq!(actual, repeat.channels()[0].pixels());
                        assert_eq!(output.report().allocations.live_bytes, 0);
                        for (i, pixel) in actual.chunks_exact(4).enumerate() {
                            let value = if expected.contains(&(i as u32)) {
                                255
                            } else {
                                0
                            };
                            assert_eq!(pixel, &[value, value, value, 255]);
                        }
                        if let Some(browser) = &browser {
                            let row = &browser["rows"][rows.len()];
                            assert_eq!(row["size"], json!(size));
                            assert_eq!(row["field"], json!(field));
                            assert_eq!(row["operation"], json!(operation));
                            assert_eq!(row["axis"], json!(axis));
                            assert_eq!(row["radius"], json!(radius));
                            assert_eq!(row["planHash"], json!(prepared.plan().hash().as_str()));
                            let browser_pixels: Vec<u8> =
                                serde_json::from_value(row["pixels"].clone()).unwrap();
                            assert_eq!(actual, browser_pixels);
                        }
                        rows.push(json!({"size":size,"field":field,"operation":operation,"axis":axis,"radius":radius,"planHash":prepared.plan().hash().as_str(),"pixels":actual,"repeatExact":true}));
                    }
                }
            }
        }
    }
    assert_eq!(rows.len(), 192);
    if let Some(browser) = &browser {
        assert_eq!(browser["rows"].as_array().unwrap().len(), rows.len());
    }
    if let Ok(path) = std::env::var("MIXTURE_MORPHOLOGY_EVIDENCE") {
        std::fs::write(path,serde_json::to_vec_pretty(&json!({"ok":true,"adapter":adapter,"rows":rows,"browserCompared":browser.is_some(),"maxComponentDelta":if browser.is_some(){Some(0)}else{None}})).unwrap()).unwrap();
    }
}
