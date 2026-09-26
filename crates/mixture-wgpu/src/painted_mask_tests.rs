//! Raw masks from the real material with a qualification-only height output alias.
use super::*;
use mixture_core::{CompileRequest, MaterialDocument, compile};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) const SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/evidence/perf-mat-before/material.mix"
);
pub(super) const CONTRACT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/materials/painted-metal/qualification-plan.json"
);

fn masks(context: &GpuContext, source: &[u8], changes: BTreeMap<String, Value>) -> Vec<Vec<f32>> {
    scalar_fields(
        context,
        source,
        changes,
        &[
            "exposure",
            "erodeY",
            "band",
            "rustMask",
            "rustSpread",
            "detail",
        ],
    )
}

pub(super) fn scalar_fields(
    context: &GpuContext,
    source: &[u8],
    changes: BTreeMap<String, Value>,
    nodes: &[&str],
) -> Vec<Vec<f32>> {
    let mut cache = PipelineCache::default();
    nodes
        .iter()
        .map(|node| {
            let mut source: Value = serde_json::from_slice(source).unwrap();
            let edge = source["edges"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|edge| edge["to"]["nodeId"] == "out" && edge["to"]["portId"] == "height")
                .unwrap();
            edge["from"] = json!({"nodeId":node,"portId":"value"});
            let request = CompileRequest {
                size: [257, 129],
                outputs: vec![
                    OutputChannel::BaseColor,
                    OutputChannel::Normal,
                    OutputChannel::Roughness,
                    OutputChannel::Metallic,
                    OutputChannel::Height,
                ],
                overrides: changes.clone(),
                ..Default::default()
            };
            let document =
                MaterialDocument::decode(&serde_json::to_vec(&source).unwrap(), &request.limits)
                    .unwrap()
                    .into_validated(&request.limits)
                    .unwrap();
            let plan = compile(&document, &request).unwrap();
            assert_eq!(
                plan.passes().len(),
                23,
                "retain the complete material computation"
            );
            let kernels: Vec<_> = plan.passes().iter().map(|p| &p.kernel).collect();
            let slots: Vec<_> = plan
                .allocation()
                .resource_slots()
                .iter()
                .map(|s| s.index() as usize)
                .collect();
            let output = plan
                .outputs()
                .iter()
                .find(|o| o.channel == OutputChannel::Height)
                .unwrap();
            let result = pollster::block_on(execute_prepared(
                context,
                &mut cache,
                plan.size(),
                &kernels,
                &[(output.resource.index() as usize, ReadbackFormat::RawHalf)],
                &[],
                &slots,
            ))
            .unwrap();
            assert_eq!(result.allocations.live_bytes, 0);
            assert_eq!(
                result.allocations.released_bytes,
                result.allocations.cumulative_bytes
            );
            let (pixels, tail) = result.pixels[0].as_chunks::<8>();
            assert!(tail.is_empty());
            assert_eq!(pixels.len(), 257 * 129);
            pixels
                .iter()
                .map(|pixel| {
                    let value =
                        half::f16::from_bits(u16::from_le_bytes([pixel[0], pixel[1]])).to_f32();
                    assert!(value.is_finite() && (0.0..=1.0).contains(&value));
                    value
                })
                .collect()
        })
        .collect()
}

fn relations(fields: &[Vec<f32>]) {
    let [wear, interior, band, rust, spread, detail] = fields else {
        panic!("six masks required")
    };
    for index in 0..wear.len() {
        assert!(interior[index] <= wear[index]);
        assert!(band[index] <= spread[index] && spread[index] <= wear[index]);
        assert!(rust[index] <= spread[index]);
        assert!((0.5..=1.).contains(&detail[index]));
        assert!(
            rust[index] <= wear[index],
            "R>W at {index}: {} > {}",
            rust[index],
            wear[index]
        );
        let expected = half::f16::from_f32((wear[index] - interior[index]).max(0.)).to_f32();
        assert_eq!(band[index], expected, "half-rounded W-I at {index}");
    }
}

fn bits(fields: &[Vec<f32>]) -> Vec<Vec<u32>> {
    fields
        .iter()
        .map(|field| field.iter().map(|value| value.to_bits()).collect())
        .collect()
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_painted_raw_mask_relations_and_control_causality() {
    let source = std::fs::read(SOURCE)
        .expect("run this ignored material qualification from its source checkout");
    let contract: Value = serde_json::from_slice(&std::fs::read(CONTRACT).unwrap()).unwrap();
    assert_eq!(contract["causality"]["size"], json!([257, 129]));
    let context = pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
    let mut defaults = BTreeMap::new();
    // Match the frozen caller's independent axis mapping, not source's 1K radii.
    defaults.insert("radiusX".into(), json!(1));
    defaults.insert("radiusY".into(), json!(1));
    let baseline = masks(&context, &source, defaults.clone());
    relations(&baseline);
    assert!(
        baseline[0].iter().any(|v| *v > 0. && *v < 1.),
        "fractional wear must be exercised"
    );
    assert_eq!(
        bits(&baseline),
        bits(&masks(&context, &source, defaults.clone())),
        "seed repeat"
    );
    let mut cases = 2;
    let mut previous: Option<Vec<Vec<f32>>> = None;
    for width in contract["causality"]["edgeWidths"].as_array().unwrap() {
        let width = width.as_u64().unwrap();
        let mut changes = defaults.clone();
        for (axis, size) in [("radiusX", 257), ("radiusY", 129)] {
            let radius = if width == 0 {
                0
            } else {
                ((width * size + 512) / 1024).max(1)
            };
            changes.insert(axis.into(), json!(radius));
        }
        let actual = masks(&context, &source, changes);
        relations(&actual);
        assert_eq!(actual[0], baseline[0], "edge width moved W");
        if width == 0 {
            assert_eq!(actual[0], actual[1]);
            assert!(actual[2].iter().all(|v| *v == 0.));
        }
        if let Some(before) = previous {
            assert!(
                before[2].iter().zip(&actual[2]).all(|(a, b)| a <= b),
                "band not monotonic"
            );
        }
        previous = Some(actual);
        cases += 1;
    }
    for control in ["rustAmount", "rustFill"] {
        let key = if control == "rustAmount" {
            "rustAmounts"
        } else {
            "rustFills"
        };
        let mut previous: Option<Vec<Vec<f32>>> = None;
        for value in contract["causality"][key].as_array().unwrap() {
            let mut changes = defaults.clone();
            changes.insert(control.into(), value.clone());
            let actual = masks(&context, &source, changes);
            relations(&actual);
            assert_eq!(actual[..3], baseline[..3], "rust controls moved W/I/B");
            if control == "rustFill" && value == &json!(0) {
                assert_eq!(actual[4], actual[2], "zero fill must use the edge band");
            }
            if control == "rustFill" && value == &json!(1) {
                assert_eq!(actual[4], actual[0], "full fill must use wear");
            }
            if control == "rustAmount" && value == &json!(0) {
                assert!(actual[3].iter().all(|v| *v == 0.));
            }
            if let Some(before) = previous {
                assert!(
                    before[3].iter().zip(&actual[3]).all(|(a, b)| a <= b),
                    "rust not monotonic"
                );
            }
            previous = Some(actual);
            cases += 1;
        }
    }
    let mut previous: Option<Vec<Vec<f32>>> = None;
    for amount in contract["causality"]["exposureAmounts"].as_array().unwrap() {
        let amount = amount.as_f64().unwrap();
        let endpoint = amount == 0. || amount == 1.;
        let low = if endpoint { 0. } else { 0.8 * (1. - amount) };
        let mut changes = defaults.clone();
        for (key, value) in [
            ("exposureInputMin", low),
            ("exposureInputMax", if endpoint { 1. } else { low + 0.2 }),
            ("exposureOutputMin", if endpoint { amount } else { 0. }),
            ("exposureOutputMax", if endpoint { amount } else { 1. }),
        ] {
            changes.insert(key.into(), json!(value));
        }
        let actual = masks(&context, &source, changes);
        relations(&actual);
        if endpoint {
            assert!(actual[0].iter().all(|v| f64::from(*v) == amount));
        }
        if let Some(before) = previous {
            assert!(
                before[0].iter().zip(&actual[0]).all(|(a, b)| a <= b),
                "exposure not monotonic"
            );
        }
        previous = Some(actual);
        cases += 1;
    }
    let mut changes = defaults;
    changes.insert(
        "detailSeed".into(),
        json!(contract["defaults"]["detailSeed"].as_u64().unwrap() + 1),
    );
    let changed = masks(&context, &source, changes.clone());
    relations(&changed);
    assert_eq!(changed[..3], baseline[..3], "detail seed moved W/I/B");
    assert_ne!(changed[3], baseline[3], "detail seed must affect rust");
    assert_ne!(changed[5], baseline[5], "detail seed must affect detail");
    assert_eq!(bits(&changed), bits(&masks(&context, &source, changes)));
    cases += 2;
    eprintln!(
        "painted raw masks: {}",
        json!({"ok":true,"materialAccepted":false,"cases":cases,
        "pixelsPerMask":257*129,"masks":["W","I","B","R","S","D"],"tolerance":0,"adapter":context.report().adapter()})
    );
}
