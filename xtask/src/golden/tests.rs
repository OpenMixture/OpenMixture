use super::*;
use model::{Change, Structure};
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn directional_metrics_distinguish_axes_without_histogram_shortcuts() {
    let row = [20_u8, 50, 110, 180, 220, 180, 110, 50];
    let vertical = Image {
        size: 8,
        pixels: row
            .repeat(8)
            .into_iter()
            .flat_map(|v| [v, v, v, 255])
            .collect(),
    };
    let horizontal = Image {
        size: 8,
        pixels: row
            .into_iter()
            .flat_map(|v| [v, v, v, 255].repeat(8))
            .collect(),
    };
    let rule = |axis| Structure::Directional {
        axis,
        min_energy_ratio: 4.,
        min_span: 100,
        min_std_dev: 20.,
        min_neighbor_correlation: 0.6,
        max_seam_ratio: 2.,
    };
    assert_eq!(vertical.statistics(), horizontal.statistics());
    for (image, axis, wrong) in [
        (
            &vertical,
            model::GrainAxis::Vertical,
            model::GrainAxis::Horizontal,
        ),
        (
            &horizontal,
            model::GrainAxis::Horizontal,
            model::GrainAxis::Vertical,
        ),
    ] {
        assert_eq!(pixels::structure(image, &rule(axis))["ok"], true);
        assert_eq!(pixels::structure(image, &rule(wrong))["ok"], false);
    }
    let mut scrambled = vertical.clone();
    for row in scrambled
        .pixels
        .as_chunks_mut::<32>()
        .0
        .iter_mut()
        .skip(1)
        .step_by(2)
    {
        row.rotate_left(16);
    }
    assert_eq!(vertical.statistics(), scrambled.statistics());
    assert_eq!(
        pixels::structure(&scrambled, &rule(model::GrainAxis::Vertical))["ok"],
        false
    );
    let measured = pixels::structure(&uniform(8, 128), &rule(model::GrainAxis::Vertical));
    assert_eq!(measured["ok"], false);
    assert_eq!(measured["directionality"]["crossToAlongRatio"], 0.);

    let root = crate::workspace_root().unwrap();
    let (_, mut contract) = material(&root, "leather").unwrap();
    contract.cases[0]
        .checks
        .insert("height".into(), rule(model::GrainAxis::Vertical));
    assert!(contract.validate("leather").is_ok());
    if let Structure::Directional {
        min_energy_ratio, ..
    } = contract.cases[0].checks.get_mut("height").unwrap()
    {
        *min_energy_ratio = 1.;
    }
    assert!(contract.validate("leather").is_err());
    let mut encoded = serde_json::to_value(rule(model::GrainAxis::Vertical)).unwrap();
    encoded["axis"] = json!("diagonal");
    assert!(serde_json::from_value::<Structure>(encoded).is_err());
}

#[test]
fn spatial_metrics_reject_scrambling_and_an_extra_wrap_seam() {
    // Literal smooth periodic signal for measurement tests; no node implementation.
    let row = [20_u8, 50, 110, 180, 220, 180, 110, 50];
    let image = Image {
        size: 8,
        pixels: row
            .repeat(8)
            .into_iter()
            .flat_map(|v| [v, v, v, 255])
            .collect(),
    };
    let rule = Structure::Spatial {
        min_span: 100,
        min_std_dev: 20.,
        min_neighbor_correlation: 0.6,
        max_seam_ratio: 2.,
    };
    assert_eq!(pixels::structure(&image, &rule)["ok"], true);
    let mut scrambled = image.clone();
    for row in scrambled
        .pixels
        .as_chunks_mut::<32>()
        .0
        .iter_mut()
        .skip(1)
        .step_by(2)
    {
        row.rotate_left(4 * 4);
    }
    assert_eq!(
        image.statistics(),
        scrambled.statistics(),
        "identical histograms must not imply spatial acceptance"
    );
    assert_eq!(pixels::structure(&scrambled, &rule)["ok"], false);
    let mut seam = image.clone();
    for row in seam.pixels.as_chunks_mut::<32>().0 {
        row[7 * 4..].copy_from_slice(&[255, 255, 255, 255]);
    }
    let measured = pixels::structure(&seam, &rule);
    assert!(
        measured["measurements"]["seam"]["ratio"][0]
            .as_f64()
            .unwrap()
            > 2.
    );
    assert_eq!(measured["ok"], false);
    let fine = Image {
        size: 8,
        pixels: [20_u8, 110, 220, 110, 20, 110, 220, 110]
            .repeat(8)
            .into_iter()
            .flat_map(|v| [v, v, v, 255])
            .collect(),
    };
    let change = Change::NormalizedGradientEnergy {
        min_ratio: 2.,
        max_ratio: 5.,
        min_pixel_ratio: 0.5,
    };
    assert_eq!(pixels::causality(&image, &fine, &change)["ok"], true);
    assert_eq!(pixels::causality(&fine, &image, &change)["ok"], false);
    assert_eq!(pixels::causality(&image, &image, &change)["ok"], false);
    let mut darker = image.clone();
    for pixel in darker.pixels.as_chunks_mut::<4>().0 {
        for channel in &mut pixel[..3] {
            *channel /= 2;
        }
    }
    let measurement = pixels::causality(&image, &darker, &change);
    assert!(
        (measurement["redNormalizedGradientEnergy"]["ratio"]
            .as_f64()
            .unwrap()
            - 1.0)
            .abs()
            < 1e-12,
        "contrast alone must not count as additional spatial detail"
    );
    assert_eq!(measurement["ok"], false);
}

#[test]
fn normal_checks_reject_invalid_length_and_flipped_height_direction() {
    let heights = [20_u8, 50, 110, 180, 220, 180, 110, 50];
    let height = Image {
        size: 8,
        pixels: heights
            .repeat(8)
            .into_iter()
            .flat_map(|v| [v, v, v, 255])
            .collect(),
    };
    let normal = Image {
        size: 8,
        pixels: [
            [128, 128, 255, 255],
            [70, 128, 242, 255],
            [70, 128, 242, 255],
            [70, 128, 242, 255],
            [128, 128, 255, 255],
            [185, 128, 242, 255],
            [185, 128, 242, 255],
            [185, 128, 242, 255],
        ]
        .repeat(8)
        .into_iter()
        .flatten()
        .collect(),
    };
    let rule = Structure::Normal {
        max_length_error: 0.015,
        min_mean_tilt: 0.05,
        max_mean_tilt: 0.2,
        // This eight-sample literal has only four slope transitions per repeat.
        max_seam_ratio: 3.,
    };
    assert_eq!(pixels::structure(&normal, &rule)["ok"], true);
    assert_eq!(pixels::structure(&uniform(8, 128), &rule)["ok"], false);
    let relation = model::Relationship::HeightNormalDirection {
        min_sign_agreement: 0.98,
        min_measured_ratio: 0.3,
    };
    let mut images = BTreeMap::from([
        ("height".to_string(), height),
        ("normal".to_string(), normal.clone()),
    ]);
    assert_eq!(pixels::relationship(&images, &relation)["ok"], true);
    let mut flipped = normal;
    for p in flipped.pixels.as_chunks_mut::<4>().0 {
        p[0] = 255 - p[0];
    }
    images.insert("normal".to_string(), flipped);
    assert_eq!(pixels::relationship(&images, &relation)["ok"], false);
    images.insert("height".to_string(), uniform(8, 128));
    assert_eq!(
        pixels::relationship(&images, &relation)["ok"],
        false,
        "no usable height slope must not vacuously pass"
    );
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        // Wall-clock resolution can be coarser than concurrent test creation.
        static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
        let directory = env::temp_dir().join(format!(
            "mixture-golden-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn uniform(size: u32, value: u8) -> Image {
    Image {
        size,
        pixels: [value, value, value, 255].repeat((size * size) as usize),
    }
}

#[test]
fn golden_metrics_measure_rgba_errors_and_pixel_ratios_with_exact_tolerance_boundaries() {
    let before = uniform(2, 0);
    let mut after = before.clone();
    after.pixels[0] = 4;
    after.pixels[7] = 253; // Alpha differences count, even when RGB is unchanged.
    let t = Tolerance {
        max_absolute: 4,
        mean_absolute: 0.375,
        pixel_threshold: 2,
        max_changed_pixel_ratio: 0.25,
    };
    let comparison = pixels::compare(&before, &after, t);
    assert_eq!(comparison["ok"], true);
    assert_eq!(comparison["changedPixelRatio"], 0.5);
    assert_eq!(comparison["aboveThresholdPixelRatio"], 0.25);
    assert_eq!(comparison["maxByComponent"], json!([4, 0, 0, 2]));
    assert_eq!(comparison["meanAbsolute"], 0.375);
    assert_eq!(
        pixels::compare(&before, &after, Tolerance::EXACT)["ok"],
        false
    );
    for t in [
        Tolerance {
            max_absolute: 3,
            ..t
        },
        Tolerance {
            mean_absolute: 0.374,
            ..t
        },
        Tolerance {
            max_changed_pixel_ratio: 0.24,
            ..t
        },
    ] {
        assert_eq!(pixels::compare(&before, &after, t)["ok"], false);
    }
}

#[test]
fn alternating_structure_accepts_intentional_wrap_jumps_but_rejects_seams_and_degeneracy() {
    // An explicitly specified tiny test tile, not a CPU node implementation.
    let rows = [
        "00110011", "00110011", "11001100", "11001100", "00110011", "00110011", "11001100",
        "11001100",
    ];
    let image = Image {
        size: 8,
        pixels: rows
            .iter()
            .flat_map(|row| {
                row.bytes().flat_map(|b| {
                    if b == b'0' {
                        [20, 30, 40, 255]
                    } else {
                        [220, 230, 240, 255]
                    }
                })
            })
            .collect(),
    };
    let rule = Structure::Alternating {
        cells: [4, 4],
        min_contrast: 100,
        max_balance_error: 0.0,
    };
    let shape = pixels::structure(&image, &rule);
    assert_eq!(shape["ok"], true);
    assert_eq!(shape["seam"]["wrapMeanAbsolute"], json!([200.0, 200.0]));
    assert_eq!(shape["seam"]["maxExcess"], 0);
    let mut broken = image.clone();
    broken.pixels[..4].copy_from_slice(&[220, 230, 240, 255]);
    assert_eq!(pixels::structure(&broken, &rule)["ok"], false);
    assert_eq!(pixels::structure(&uniform(8, 20), &rule)["ok"], false);
    assert_eq!(
        pixels::structure(
            &image,
            &Structure::Alternating {
                cells: [8, 4],
                min_contrast: 100,
                max_balance_error: 0.0
            }
        )["ok"],
        false
    );
}

#[test]
fn uniform_default_and_parameter_causality_gates_are_not_nonempty_pixel_checks() {
    let before = uniform(4, 40);
    let after = uniform(4, 140);
    assert_eq!(
        pixels::structure(
            &before,
            &Structure::Uniform {
                rgba: [40, 40, 40, 255],
                tolerance: 0
            }
        )["ok"],
        true
    );
    assert_eq!(
        pixels::structure(
            &after,
            &Structure::Uniform {
                rgba: [40, 40, 40, 255],
                tolerance: 1
            }
        )["ok"],
        false
    );
    assert_eq!(
        pixels::causality(&before, &before, &Change::Unchanged)["ok"],
        true
    );
    assert_eq!(
        pixels::causality(&before, &after, &Change::Unchanged)["ok"],
        false
    );
    assert_eq!(
        pixels::causality(
            &before,
            &after,
            &Change::Changed {
                min_pixel_ratio: 1.0
            }
        )["ok"],
        true
    );
    assert_eq!(
        pixels::causality(
            &before,
            &before,
            &Change::Changed {
                min_pixel_ratio: 0.5
            }
        )["ok"],
        false
    );
    assert_eq!(
        pixels::causality(&before, &after, &Change::MeanIncreases { min_delta: 0.3 })["ok"],
        true
    );
    assert_eq!(
        pixels::causality(&after, &before, &Change::MeanIncreases { min_delta: 0.3 })["ok"],
        false
    );
}

#[test]
fn material_fixture_contract_and_variants_are_strict_and_cover_all_channels() {
    let root = crate::workspace_root().unwrap();
    let (directory, acceptance) = material(&root, "glazed-ceramic").unwrap();
    for case in &acceptance.cases {
        if let Some(id) = &case.variant {
            let variant: Variant =
                files::json(&directory.join(format!("variants/{id}.json"))).unwrap();
            assert!(!variant.overrides.is_empty());
        }
    }
    for mutation in ["unknown", "missingChannel", "oddCells", "noCausality"] {
        let mut value = serde_json::to_value(&acceptance).unwrap();
        match mutation {
            "unknown" => {
                value["sizeTypo"] = json!(64);
            }
            "missingChannel" => {
                value["channels"].as_object_mut().unwrap().remove("height");
            }
            "oddCells" => {
                value["cases"][0]["checks"]["baseColor"]["cells"] = json!([7, 8]);
            }
            _ => {
                value["cases"][1]["changes"] = json!({});
            }
        }
        let decoded = serde_json::from_value::<Acceptance>(value);
        assert!(
            decoded.is_err() || decoded.unwrap().validate("glazed-ceramic").is_err(),
            "{mutation}"
        );
    }
}

#[test]
fn bounded_png_decoding_rejects_wrong_dimensions_metadata_and_truncated_files() {
    let temp = Temp::new();
    let path = temp.0.join("image.png");
    pixels::sheet(
        &path,
        &[("test".into(), vec![("test".into(), Some(uniform(4, 20)))])],
    )
    .unwrap();
    assert!(Image::read(&path, 4, "rgba8-srgb").is_err());
    let mut data = Vec::new();
    let mut encoder = png::Encoder::new(&mut data, 4, 4);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::new(1.0));
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&uniform(4, 20).pixels).unwrap();
    writer.finish().unwrap();
    fs::write(&path, &data).unwrap();
    assert!(Image::read(&path, 4, "rgba8-linear").is_ok());
    assert!(Image::read(&path, 4, "rgba8-srgb").is_err());
    fs::write(&path, &data[..data.len() / 2]).unwrap();
    assert!(Image::read(&path, 4, "rgba8-linear").is_err());
}

// Minimal local fixture tree to test acceptance guards without cargo, a driver, or Git writes.
fn candidate_fixture() -> (Temp, PathBuf, PathBuf, Acceptance, Candidate) {
    let temp = Temp::new();
    let root = crate::workspace_root().unwrap();
    let (actual, mut acceptance) = material(&root, "glazed-ceramic").unwrap();
    let directory = temp.0.join("fixtures/materials/glazed-ceramic");
    fs::create_dir_all(directory.join("variants")).unwrap();
    for file in [
        "material.mix",
        "acceptance.json",
        "variants/fine-tiles.json",
        "variants/matte.json",
    ] {
        fs::copy(actual.join(file), directory.join(file)).unwrap();
    }
    // Synthetic uniform sentinels exercise the file/update boundary without rendering nodes.
    for case in &mut acceptance.cases {
        let value = if case.id == "fine-tiles" { 60 } else { 40 };
        case.checks.insert(
            "baseColor".into(),
            Structure::Uniform {
                rgba: [value, value, value, 255],
                tolerance: 0,
            },
        );
    }
    files::write_json(&directory.join("acceptance.json"), &acceptance).unwrap();
    // Mirror the fingerprint's required project paths, with fixed test-only contents.
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/mixture-core/Cargo.toml",
        "crates/mixture-wgpu/Cargo.toml",
        "crates/mixture-cli/Cargo.toml",
        "xtask/Cargo.toml",
        ".github/scripts/setup-swiftshader.sh",
    ] {
        let path = temp.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"test fixture").unwrap();
    }
    let review_root = review_root(&temp.0, "glazed-ceramic").unwrap();
    let review = review_root.join("software-1-1");
    fs::create_dir(&review).unwrap();
    for case in &acceptance.cases {
        for channel in CHANNELS {
            let path = review.join(format!("{}/{channel}.png", case.id));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let Structure::Uniform { rgba, .. } = case.checks[channel] else {
                panic!("test fixture uses uniform sentinels")
            };
            let mut encoder = png::Encoder::new(
                fs::File::create(path).unwrap(),
                acceptance.size,
                acceptance.size,
            );
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            if channel == "baseColor" {
                encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
            } else {
                encoder.set_source_gamma(png::ScaledFloat::new(1.0));
            }
            let mut writer = encoder.write_header().unwrap();
            writer
                .write_image_data(&rgba.repeat((acceptance.size * acceptance.size) as usize))
                .unwrap();
            writer.finish().unwrap();
        }
    }
    files::write_json(&review.join("report.json"),&json!({"schemaVersion":1,"material":"glazed-ceramic","policy":"software","machineChecksPassed":true,
        "softwareSource":{"revision":model::REVISION},"adapter":{"name":"SwiftShader test fixture","backend":"Vulkan","deviceType":"Cpu"},
        "cases":acceptance.cases.iter().map(|c|json!({"id":c.id,"planHash":format!("sha256:{}","0".repeat(64))})).collect::<Vec<_>>()})).unwrap();
    let candidate = Candidate {
        schema_version: 1,
        material: "glazed-ceramic".into(),
        inputs: files::inputs(&temp.0, &directory).unwrap(),
        previous_baseline: Digests::new(),
        artifacts: files::tree(&review).unwrap(),
    };
    files::write_json(&review.join("candidate.json"), &candidate).unwrap();
    files::write_json(
        &review_root.join("latest-software.json"),
        &json!({"run":"software-1-1","ready":true}),
    )
    .unwrap();
    (temp, directory, review, acceptance, candidate)
}

#[test]
fn candidate_guards_reject_changed_artifacts_inputs_baselines_and_failed_measurements() {
    for changed in ["artifact", "inputs", "baseline", "failedReport"] {
        let (temp, directory, review, acceptance, mut candidate) = candidate_fixture();
        assert!(verify_candidate(&temp.0, &directory, &review, &candidate, &acceptance).is_ok());
        match changed {
            "artifact" => fs::write(review.join("default/baseColor.png"), b"tampered").unwrap(),
            "inputs" => fs::write(directory.join("material.mix"), b"changed source").unwrap(),
            "baseline" => {
                fs::create_dir(directory.join("expected")).unwrap();
                fs::write(directory.join("expected/new.png"), b"new baseline").unwrap();
            }
            _ => {
                let mut report: Value = files::json(&review.join("report.json")).unwrap();
                report["machineChecksPassed"] = json!(false);
                files::write_json(&review.join("report.json"), &report).unwrap();
                candidate.artifacts.insert(
                    "report.json".into(),
                    files::digest(&review.join("report.json")).unwrap(),
                );
            }
        }
        assert!(
            verify_candidate(&temp.0, &directory, &review, &candidate, &acceptance).is_err(),
            "{changed}"
        );
    }
}

#[test]
fn golden_acceptance_copies_only_reviewed_outputs_retains_evidence_and_cannot_replay() {
    let (temp, directory, review, acceptance, candidate) = candidate_fixture();
    update(&temp.0, "glazed-ceramic").unwrap();
    assert!(baseline(&directory, &acceptance).unwrap().is_some());
    assert!(review.join("update.json").is_file());
    assert!(!directory.join("expected/report.json").exists());
    assert_eq!(
        files::digest(&directory.join("expected/default/baseColor.png")).unwrap(),
        candidate.artifacts["default/baseColor.png"]
    );
    assert!(update(&temp.0, "glazed-ceramic").is_err());
    fs::write(directory.join("expected/default/baseColor.png"), b"damaged").unwrap();
    assert!(baseline(&directory, &acceptance).is_err());
}

#[test]
fn candidate_file_paths_cannot_escape_the_review_directory() {
    let temp = Temp::new();
    for path in ["../expected/x", "/absolute", "a/../../b", "a\\b", ""] {
        assert!(files::relative(&temp.0, path).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(env::temp_dir(), temp.0.join("link")).unwrap();
        assert!(files::relative(&temp.0, "link/file").is_err());
    }
}
