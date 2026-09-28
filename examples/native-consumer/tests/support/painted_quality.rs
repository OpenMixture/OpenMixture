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

pub(super) fn endpoint_reference(
    gpu: &mut Renderer,
    preset: &str,
    defaults: &Value,
) -> Option<RenderOutput> {
    let (color, roughness, metallic, height) = match preset {
        "intact" => (
            "paintColor",
            "paintRoughness",
            0.,
            defaults["paintThickness"].as_f64().unwrap(),
        ),
        "exposed" => ("substrateColor", "substrateRoughness", 1., 0.),
        "rusted" => (
            "rustColor",
            "rustRoughness",
            0.,
            defaults["rustRelief"].as_f64().unwrap(),
        ),
        _ => return None,
    };
    let mut nodes = vec![
        json!({"id":"color","type":"constant-color","version":1,"parameters":{"value":defaults[color]}}),
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
        ("height", json!(height)),
        ("roughness", defaults[roughness].clone()),
        ("metallic", json!(metallic)),
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

pub(super) fn check_endpoint(output: &RenderOutput, reference: &RenderOutput) -> Vec<Value> {
    output.channels().iter().map(|channel| {
        let expected = pixels(reference, channel.channel);
        assert_eq!(expected.len(), 4);
        let (actual, remainder) = channel.pixels().as_chunks::<4>();
        assert!(remainder.is_empty());
        assert!(actual.iter().all(|pixel| pixel == expected), "endpoint {} differs from independent constant reference", channel.channel.as_str());
        json!({"channel":channel.channel.as_str(),"expectedRgba8":expected,"allPixelsExact":true})
    }).collect()
}

// Box-average the 16 corresponding high-resolution delivered samples without
// rounding the mean back to u8. Alpha is opaque; measure each RGB component.
fn mean_error(low: &[u8], high: &[u8], width: usize, height: usize) -> [f64; 3] {
    assert!(width > 0 && height > 0);
    assert_eq!(low.len(), width * height * 4);
    assert_eq!(high.len(), width * height * 64);
    let mut total = [0.; 3];
    for y in 0..height {
        for x in 0..width {
            for c in 0..3 {
                let mut sum = 0u32;
                for dy in 0..4 {
                    for dx in 0..4 {
                        sum += u32::from(high[((y * 4 + dy) * width * 4 + x * 4 + dx) * 4 + c]);
                    }
                }
                total[c] += (f64::from(low[(y * width + x) * 4 + c]) - f64::from(sum) / 16.).abs();
            }
        }
    }
    total.map(|v| v / (width * height) as f64)
}

pub(super) fn downsample(low: &RenderOutput, high: &RenderOutput, contract: &Value) -> Vec<Value> {
    assert_eq!(low.report().size, [256, 256]);
    assert_eq!(high.report().size, [1024, 1024]);
    assert_eq!(
        contract["downsampleChannels"],
        json!(["height", "baseColor"])
    );
    [OutputChannel::Height, OutputChannel::BaseColor].into_iter().map(|channel| {
        let error = mean_error(pixels(low, channel), pixels(high, channel), 256, 256);
        let limit = contract["maxDefaultDownsampleMeanError"].as_f64().unwrap();
        assert!(error.iter().all(|value| *value <= limit), "default downsample {}: {error:?} exceeds {limit}", channel.as_str());
        json!({"channel":channel.as_str(),"componentMeanError":error,"limit":limit,"passed":true,"filter":"exact 4x4 box mean of delivered RGB bytes"})
    }).collect()
}

#[test]
fn box_mean_preserves_fractional_samples_and_component_identity() {
    let low = [10, 20, 30, 255];
    let mut high = low.repeat(16);
    high[..4].copy_from_slice(&[26, 52, 78, 255]);
    assert_eq!(mean_error(&low, &high, 1, 1), [1., 2., 3.]);
    high[..4].copy_from_slice(&[11, 20, 30, 255]);
    assert_eq!(mean_error(&low, &high, 1, 1), [0.0625, 0., 0.]);
}

#[test]
fn box_mean_uses_each_rectangular_pixel_block() {
    let low = [10, 20, 30, 255, 40, 50, 60, 255];
    let row = [10, 20, 30, 255]
        .repeat(4)
        .into_iter()
        .chain([44, 50, 52, 255].repeat(4))
        .collect::<Vec<_>>();
    assert_eq!(mean_error(&low, &row.repeat(4), 2, 1), [2., 0., 4.]);
}

#[test]
#[should_panic]
fn box_mean_rejects_incomplete_image() {
    mean_error(&[0; 4], &[0; 63], 1, 1);
}
