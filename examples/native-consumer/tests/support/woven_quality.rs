//! Independent constant references and delivered-byte measurements, not a renderer.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile};
use mixture_wgpu::{RenderOutput, Renderer};
use serde_json::{Value, json};

fn pixels(output: &RenderOutput, channel: OutputChannel) -> &[u8] {
    output
        .channels()
        .iter()
        .find(|c| c.channel == channel)
        .unwrap()
        .pixels()
}

pub(super) fn endpoint_reference(gpu: &mut Renderer, preset: &str) -> Option<RenderOutput> {
    let value = if preset == "constant-high" { 1. } else { 0. };
    let mut nodes = vec![
        json!({"id":"color","type":"constant-color","version":1,"parameters":{"value":json!([value,value,value,1.])}}),
        json!({"id":"normal","type":"height-to-normal","version":1,"parameters":{"strength":1}}),
        json!({"id":"out","type":"material-output","version":1,"parameters":{}}),
    ];
    let edge = |from: &str, port: &str, to: &str, input: &str| json!({"from":{"nodeId":from,"portId":port},"to":{"nodeId":to,"portId":input}});
    let mut edges = vec![
        edge("color", "color", "out", "baseColor"),
        edge("height", "value", "normal", "in"),
        edge("normal", "normal", "out", "normal"),
    ];
    for (name, value) in [
        ("height", json!(0.)),
        ("roughness", json!(value)),
        ("metallic", json!(0.)),
    ] {
        nodes.push(
            json!({"id":name,"type":"constant-scalar","version":1,"parameters":{"value":value}}),
        );
        edges.push(edge(name, "value", "out", name));
    }
    let source = serde_json::to_vec(&json!({"version":1,"nodes":nodes,"edges":edges})).unwrap();
    let request = CompileRequest {
        size: [1, 1],
        outputs: vec![
            OutputChannel::BaseColor,
            OutputChannel::Height,
            OutputChannel::Normal,
            OutputChannel::Metallic,
            OutputChannel::Roughness,
        ],
        ..Default::default()
    };
    let document = MaterialDocument::decode(&source, &request.limits)
        .unwrap()
        .into_validated(&request.limits)
        .unwrap();
    Some(pollster::block_on(gpu.render(&compile(&document, &request).unwrap())).unwrap())
}

pub(super) fn check_endpoint(
    output: &RenderOutput,
    reference: &RenderOutput,
    preset: &str,
) -> Vec<Value> {
    output.channels().iter().filter(|c| c.channel == OutputChannel::Metallic || (preset == "flat" && [OutputChannel::Height,OutputChannel::Normal].contains(&c.channel)) || (preset == "neutral-normal" && c.channel == OutputChannel::Normal) || (preset.starts_with("constant-") && [OutputChannel::BaseColor,OutputChannel::Roughness].contains(&c.channel))).map(|channel| {
        let expected = pixels(reference, channel.channel);
        assert_eq!(expected.len(), 4);
        let (actual, remainder) = channel.pixels().as_chunks::<4>();
        assert!(remainder.is_empty());
        assert!(actual.iter().all(|pixel| pixel == expected), "endpoint {} differs from independent constant reference", channel.channel.as_str());
        json!({"channel":channel.channel.as_str(),"expectedRgba8":expected,"allPixelsExact":true})
    }).collect()
}

// Measurements on delivered bytes only, never a second renderer.
fn mean_error(low: &[u8], high: &[u8], width: usize, height: usize, factor: usize) -> [f64; 3] {
    assert!(width > 0 && height > 0 && factor > 0);
    assert_eq!(low.len(), width * height * 4);
    assert_eq!(high.len(), low.len() * factor * factor);
    let mut total = [0.; 3];
    for y in 0..height {
        for x in 0..width {
            for (c, total) in total.iter_mut().enumerate() {
                let mut sum = 0u32;
                for dy in 0..factor {
                    for dx in 0..factor {
                        sum += u32::from(
                            high[((y * factor + dy) * width * factor + x * factor + dx) * 4 + c],
                        );
                    }
                }
                *total += (f64::from(low[(y * width + x) * 4 + c])
                    - f64::from(sum) / (factor * factor) as f64)
                    .abs();
            }
        }
    }
    total.map(|v| v / (width * height) as f64)
}
pub(super) fn downsample(low: &RenderOutput, high: &RenderOutput, contract: &Value) -> Vec<Value> {
    let [w, h] = low.report().size;
    let [hw, hh] = high.report().size;
    assert_eq!(hw % w, 0);
    assert_eq!(hh % h, 0);
    assert_eq!(hw / w, hh / h);
    [OutputChannel::BaseColor,OutputChannel::Height].into_iter().map(|channel| {
        let error = mean_error(pixels(low,channel),pixels(high,channel),w as usize,h as usize,(hw/w) as usize);
        let limit=contract["maxDefaultAndVariedDownsampleMeanError"].as_f64().unwrap();
        json!({"channel":channel.as_str(),"from":[hw,hh],"to":[w,h],"box":hw/w,"componentMeanError":error,"limit":limit,"passed":error.iter().all(|v|v.is_finite()&&*v<=limit)})
    }).collect()
}
#[test]
fn fractional_box_and_rectangular_mapping() {
    for factor in [2, 4] {
        let low = [10, 20, 30, 255];
        let mut high = low.repeat(factor * factor);
        high[0] += 1;
        assert_eq!(
            mean_error(&low, &high, 1, 1, factor),
            [1. / (factor * factor) as f64, 0., 0.]
        );
        let low = [10, 20, 30, 255, 40, 50, 60, 255];
        let row = [10, 20, 30, 255]
            .repeat(factor)
            .into_iter()
            .chain([44, 50, 52, 255].repeat(factor))
            .collect::<Vec<_>>();
        assert_eq!(
            mean_error(&low, &row.repeat(factor), 2, 1, factor),
            [2., 0., 4.]
        );
    }
}
