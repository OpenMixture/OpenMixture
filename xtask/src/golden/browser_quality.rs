//! Browser-only measurements of supplied textures. No graph execution or CPU renderer.
use super::{
    files,
    pixels::{self, Image},
};
use crate::TaskResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn calibrate(root: &Path, output: &Path) -> TaskResult {
    let p = Policy::load(root)?;
    std::fs::create_dir(output)?;
    files::write_json(
        &output.join("calibration.json"),
        &json!({"ok":false,"status":"incomplete"}),
    )?;
    let mut records = Vec::new();
    let mut all_ok = true;
    for channel in ["baseColor", "normal", "height", "roughness"] {
        let source = Image {
            size: 64,
            pixels: if channel == "normal" {
                [128, 128, 255, 255].repeat(4096)
            } else {
                [128, 128, 128, 255].repeat(4096)
            },
        };
        for name in [
            "identity",
            "balanced-one-code",
            "sparse-one-code",
            "uniform-bias",
            "local-bias",
            "wrap-bias",
            "two-code-spike",
            "seam-stripe",
            "lost-detail",
            "invalid-alpha",
        ] {
            let expected = ["identity", "balanced-one-code", "sparse-one-code"].contains(&name);
            let mut before = source.clone();
            let mut after = source.clone();
            for y in 0..64usize {
                for x in 0..64usize {
                    let i = (y * 64 + x) * 4;
                    let delta: i16 = match name {
                        "balanced-one-code" => {
                            if (x + y) % 2 == 0 {
                                -1
                            } else {
                                1
                            }
                        }
                        "sparse-one-code" => i16::from(x == 32 && y == 32),
                        "uniform-bias" => 1,
                        "local-bias" => i16::from((13..29).contains(&x) && (13..29).contains(&y)),
                        "wrap-bias" => i16::from(!(8..56).contains(&x) && !(8..56).contains(&y)),
                        "two-code-spike" => {
                            if x == 32 && y == 32 {
                                2
                            } else {
                                0
                            }
                        }
                        "seam-stripe" => i16::from(x < 8),
                        _ => 0,
                    };
                    for c in 0..if channel == "normal" { 1 } else { 3 } {
                        after.pixels[i + c] = (i16::from(source.pixels[i + c]) + delta) as u8;
                        if name == "lost-detail" {
                            before.pixels[i + c] = if x % 2 == 0 { 124 } else { 132 };
                        }
                    }
                    if name == "invalid-alpha" && x == 0 && y == 0 {
                        after.pixels[i + 3] = 254;
                    }
                }
            }
            let measured = compare(&before, &after, channel, &p);
            let matched = measured["ok"] == expected;
            all_ok &= matched;
            pixels::sheet(
                &output.join(format!("{channel}-{name}.png")),
                &[(
                    name.to_owned(),
                    vec![
                        ("SOURCE".into(), Some(before.clone())),
                        ("PERTURBED".into(), Some(after.clone())),
                        ("DIFF X4".into(), Some(pixels::difference(&before, &after))),
                    ],
                )],
            )?;
            records.push(json!({"channel":channel,"case":name,"expectedPass":expected,"matched":matched,"metrics":measured}));
        }
    }
    files::write_json(
        &output.join("calibration.json"),
        &json!({"schemaVersion":1,"ok":all_ok,"source":"procedural texture perturbations; no browser output inputs","profile":p,"profileSha256":files::digest(&root.join("docs/browser-quality-v2.json"))?,"cases":records}),
    )?;
    if !all_ok {
        return Err("independent browser quality controls failed".into());
    }
    println!(
        "40 independent browser quality controls passed: {}",
        output.display()
    );
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Policy {
    schema_version: u32,
    id: String,
    max_component_delta: u8,
    window_size: usize,
    window_stride: usize,
    max_window_bias: f64,
    max_normal_angle_degrees: f64,
    max_normal_length_delta: f64,
    max_normal_diffuse_delta: f64,
    max_normal_highlight_delta: f64,
    normal_highlight_exponent: i32,
    max_height_gradient_delta: f64,
    max_roughness_response_delta: f64,
    roughness_floor: f64,
}

impl Policy {
    pub(super) fn load(root: &Path) -> TaskResult<Self> {
        let policy: Self = files::json(&root.join("docs/browser-quality-v2.json"))?;
        // This is a versioned, frozen profile, not a caller-controlled tolerance override.
        let frozen: Value =
            serde_json::from_str(include_str!("../../../docs/browser-quality-v2.json"))?;
        if serde_json::to_value(&policy)? != frozen {
            return Err("browser quality profile differs from compiled policy".into());
        }
        Ok(policy)
    }
}

fn normal(pixel: &[u8]) -> ([f64; 3], f64) {
    let v = [pixel[0], pixel[1], pixel[2]].map(|v| f64::from(v) * 2.0 / 255.0 - 1.0);
    let length = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    (v.map(|x| x / length), length)
}

// Fixed normalized directional response samples. These measure existing outputs;
// they neither execute .mix nodes nor stand in for an engine-specific material preview.
const DIRECTIONS: [[f64; 3]; 5] = [
    [0.0, 0.0, 1.0],
    [0.6, 0.0, 0.8],
    [-0.6, 0.0, 0.8],
    [0.0, 0.6, 0.8],
    [0.0, -0.6, 0.8],
];

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

// Peak-normalized GGX NDF cross-section with alpha = roughness^2.
// A shape sensitivity probe, not a full BRDF, lighting model or perceptual metric.
fn roughness_response(roughness: f64, cosine: f64, floor: f64) -> f64 {
    let a2 = roughness.max(floor).powi(4);
    (a2 / (1.0 - cosine * cosine + a2 * cosine * cosine)).powi(2)
}

pub(super) fn compare(before: &Image, after: &Image, channel: &str, p: &Policy) -> Value {
    let size = before.size as usize;
    if size == 0
        || before.size != after.size
        || before.pixels.len() != size * size * 4
        || before.pixels.len() != after.pixels.len()
        || !["baseColor", "normal", "height", "roughness"].contains(&channel)
    {
        return json!({"ok":false,"reason":"invalid comparison input"});
    }
    let mut maximum = 0u8;
    let mut alpha_ok = true;
    let mut scalar_ok = true;
    let mut max_angle = 0.0f64;
    let mut max_length = 0.0f64;
    let mut max_diffuse = 0.0f64;
    let mut max_highlight = 0.0f64;
    let mut max_response = 0.0f64;
    let scalar = channel == "height" || channel == "roughness";
    let mut differences = vec![[0i16; 3]; size * size];
    for (i, (a, b)) in before
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(after.pixels.as_chunks::<4>().0.iter())
        .enumerate()
    {
        alpha_ok &= a[3] == 255 && b[3] == 255;
        if scalar {
            scalar_ok &= a[0] == a[1] && a[1] == a[2] && b[0] == b[1] && b[1] == b[2];
        }
        for c in 0..3 {
            maximum = maximum.max(a[c].abs_diff(b[c]));
            differences[i][c] = i16::from(b[c]) - i16::from(a[c]);
        }
        if channel == "normal" {
            let (na, la) = normal(a);
            let (nb, lb) = normal(b);
            max_angle = max_angle.max(dot(na, nb).clamp(-1.0, 1.0).acos().to_degrees());
            max_length = max_length.max((la - lb).abs());
            for direction in DIRECTIONS {
                let da = dot(na, direction).max(0.0);
                let db = dot(nb, direction).max(0.0);
                max_diffuse = max_diffuse.max((da - db).abs());
                max_highlight = max_highlight.max(
                    (da.powi(p.normal_highlight_exponent) - db.powi(p.normal_highlight_exponent))
                        .abs(),
                );
            }
        }
        if channel == "roughness" {
            // Include near-specular directions; low roughness is intentionally sensitive.
            for cosine in [0.5, 0.8, 0.95, 0.99, 0.999, 0.9999, 0.99999, 1.0] {
                max_response = max_response.max(
                    (roughness_response(f64::from(a[0]) / 255.0, cosine, p.roughness_floor)
                        - roughness_response(f64::from(b[0]) / 255.0, cosine, p.roughness_floor))
                    .abs(),
                );
            }
        }
    }
    // Overlapping periodic windows cover both tile interiors and wrap seams.
    let mut bias = 0.0f64;
    for y in (0..size).step_by(p.window_stride) {
        for x in (0..size).step_by(p.window_stride) {
            let width = p.window_size.min(size);
            let mut sum = [0i64; 3];
            for dy in 0..width {
                for dx in 0..width {
                    let d = differences[((y + dy) % size) * size + (x + dx) % size];
                    for c in 0..3 {
                        sum[c] += i64::from(d[c]);
                    }
                }
            }
            for value in sum {
                bias = bias.max(value.abs() as f64 / (width * width) as f64);
            }
        }
    }
    let mut gradient = 0.0f64;
    let mut seam_gradient = 0.0f64;
    if channel == "height" {
        for y in 0..size {
            for x in 0..size {
                let d = differences[y * size + x][0];
                for (next, seam) in [
                    (y * size + (x + 1) % size, x + 1 == size),
                    (((y + 1) % size) * size + x, y + 1 == size),
                ] {
                    let delta = f64::from((d - differences[next][0]).abs()) / 255.0;
                    gradient = gradient.max(delta);
                    if seam {
                        seam_gradient = seam_gradient.max(delta);
                    }
                }
            }
        }
    }
    let numerical_ok = maximum <= p.max_component_delta && bias <= p.max_window_bias;
    let response_ok = match channel {
        "normal" => {
            max_angle <= p.max_normal_angle_degrees
                && max_length <= p.max_normal_length_delta
                && max_diffuse <= p.max_normal_diffuse_delta
                && max_highlight <= p.max_normal_highlight_delta
        }
        "height" => gradient <= p.max_height_gradient_delta,
        "roughness" => max_response <= p.max_roughness_response_delta,
        _ => true,
    };
    json!({"ok":alpha_ok && scalar_ok && numerical_ok && response_ok,
        "profile":p.id,"encodingOk":alpha_ok && scalar_ok,"numericalOk":numerical_ok,"responseOk":response_ok,
        "maxComponentDelta":maximum,"maxWindowBias":bias,
        "normal":if channel == "normal" {json!({"maxAngleDegrees":max_angle,"maxLengthDelta":max_length,"maxDiffuseDelta":max_diffuse,"maxHighlightDelta":max_highlight})}else{Value::Null},
        "height":if channel == "height" {json!({"maxGradientDelta":gradient,"maxSeamGradientDelta":seam_gradient,"units":"normalized height per texel; no world-space displacement promise"})}else{Value::Null},
        "roughness":if channel == "roughness" {json!({"maxResponseDelta":max_response,"response":"peak-normalized GGX NDF samples; not a complete BRDF"})}else{Value::Null}})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> Policy {
        serde_json::from_str(include_str!("../../../docs/browser-quality-v2.json")).unwrap()
    }
    fn image(pixel: [u8; 4]) -> Image {
        Image {
            size: 64,
            pixels: pixel.repeat(64 * 64),
        }
    }

    #[test]
    fn broad_balanced_quantization_is_not_a_changed_pixel_count_failure() {
        for channel in ["baseColor", "normal", "height", "roughness"] {
            let a = image(if channel == "normal" {
                [128, 128, 255, 255]
            } else {
                [128, 128, 128, 255]
            });
            let mut b = a.clone();
            for (i, pixel) in b.pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let value = if (i / 64 + i % 64) % 2 == 0 { 127 } else { 129 };
                pixel[0] = value;
                if channel != "normal" {
                    pixel[1] = value;
                    pixel[2] = value;
                }
            }
            let result = compare(&a, &b, channel, &policy());
            assert_eq!(result["ok"], true, "{channel}: {result}");
        }
    }
    #[test]
    fn localized_bias_spikes_seams_and_erased_detail_are_rejected() {
        let a = image([128, 128, 128, 255]);
        for (name, region) in [
            ("global bias", 0),
            ("local patch", 1),
            ("seam stripe", 2),
            ("spike", 3),
            ("boundary patch", 4),
        ] {
            let mut b = a.clone();
            for y in 0..64 {
                for x in 0..64 {
                    let change = match region {
                        0 => true,
                        1 => x < 16 && y < 16,
                        2 => x < 8,
                        3 => x == 31 && y == 31,
                        _ => !(8..56).contains(&x) && !(8..56).contains(&y),
                    };
                    if change {
                        for c in 0..3 {
                            b.pixels[(y * 64 + x) * 4 + c] += if region == 3 { 2 } else { 1 };
                        }
                    }
                }
            }
            assert_eq!(compare(&a, &b, "height", &policy())["ok"], false, "{name}");
        }
        let mut detailed = a.clone();
        for (i, pixel) in detailed
            .pixels
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .enumerate()
        {
            let v = if i % 2 == 0 { 124 } else { 132 };
            pixel[..3].fill(v);
        }
        assert_eq!(compare(&detailed, &a, "height", &policy())["ok"], false);
    }
    #[test]
    fn channel_encoding_and_normal_rotation_cannot_hide_in_average_error() {
        let a = image([128, 128, 255, 255]);
        let b = image([141, 128, 254, 255]);
        assert_eq!(compare(&a, &b, "normal", &policy())["responseOk"], false);
        let a = image([128, 128, 128, 255]);
        let mut b = a.clone();
        b.pixels[3] = 254;
        assert_eq!(compare(&a, &b, "baseColor", &policy())["encodingOk"], false);
        b = a.clone();
        b.pixels[1] = 129;
        assert_eq!(compare(&a, &b, "height", &policy())["encodingOk"], false);
    }
    #[test]
    fn measured_geometry_and_response_have_independent_analytic_cases() {
        let (n, length) = normal(&[128, 128, 255, 255]);
        assert!((length - (1.0 + 2.0 / 255.0f64.powi(2)).sqrt()).abs() < 1e-12);
        assert!((dot(n, n) - 1.0).abs() < 1e-12);
        assert_eq!(roughness_response(1.0, 0.5, 0.04), 1.0);
        assert_eq!(roughness_response(0.5, 1.0, 0.04), 1.0);
        assert!((roughness_response(0.5, 0.0, 0.04) - 1.0 / 256.0).abs() < 1e-12);
        let a = image([128, 128, 128, 255]);
        let mut b = a.clone();
        b.pixels[..3].fill(129);
        let result = compare(&a, &b, "height", &policy());
        assert!(
            (result["height"]["maxGradientDelta"].as_f64().unwrap() - 1.0 / 255.0).abs() < 1e-12
        );
        assert_eq!(result["maxWindowBias"], 1.0 / 256.0);
    }

    #[test]
    fn sharp_roughness_response_can_reject_a_sparse_single_code_change() {
        let a = image([20, 20, 20, 255]);
        let mut b = a.clone();
        b.pixels[..3].fill(21);
        let result = compare(&a, &b, "roughness", &policy());
        assert_eq!(result["numericalOk"], true);
        assert_eq!(result["responseOk"], false);
        assert_eq!(result["ok"], false);
        // A different output use must not inherit the roughness response test.
        assert_eq!(compare(&a, &b, "height", &policy())["ok"], true);
    }

    #[test]
    fn swapping_reference_preserves_all_error_verdicts() {
        let a = image([128, 128, 128, 255]);
        let mut b = a.clone();
        b.pixels[..3].fill(129);
        for channel in ["baseColor", "normal", "height", "roughness"] {
            assert_eq!(
                compare(&a, &b, channel, &policy()),
                compare(&b, &a, channel, &policy())
            );
        }
    }
}
