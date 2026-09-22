//! Compare independently supplied browser PNGs using existing material quality rules.
use super::{
    browser_quality, files, material,
    model::{CHANNELS, Variant},
    pixels::{self, Image},
};
use crate::TaskResult;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

fn material_source(manifest: &Value) -> TaskResult<&'static str> {
    match manifest.get("noiseVersion") {
        None => Ok("material.mix"),
        Some(value) if value.as_u64() == Some(2) => Ok("material-noise-v2.mix"),
        _ => Err("unsupported explicit noise migration identity".into()),
    }
}

#[test]
fn noise_migration_selects_only_the_reviewed_fixture() {
    assert_eq!(material_source(&json!({})).unwrap(), "material.mix");
    assert_eq!(
        material_source(&json!({"noiseVersion":2})).unwrap(),
        "material-noise-v2.mix"
    );
    for value in [json!(null), json!(1), json!(3), json!("2"), json!(2.5)] {
        assert!(material_source(&json!({"noiseVersion":value})).is_err());
    }
}

pub(crate) fn run(root: &Path, native: &Path, browser: &Path, measure: bool) -> TaskResult {
    files::write_json(
        &browser.join("comparison.json"),
        &json!({"schemaVersion":3,"ok":false,"status":"incomplete"}),
    )?;
    fs::write(browser.join("mode.txt"), "Comparison incomplete\n")?;
    let manifest: Value = files::json(&native.join("manifest.json"))?;
    let source_file = material_source(&manifest)?;
    let receipt: Value = files::json(&browser.join("receipt.json"))?;
    if manifest["schemaVersion"] != 1
        || receipt["schemaVersion"] != 1
        || receipt["manifestSha256"]
            != files::digest(&native.join("manifest.json"))?.trim_start_matches("sha256:")
        || receipt["build"]["engineRevision"] != manifest["runtimeRevision"]
    {
        return Err("browser/native provenance mismatch".into());
    }
    let policy = browser_quality::Policy::load(root)?;
    let mut records = Vec::new();
    let mut numerical_ok = true;
    let mut semantics_ok = true;
    let mut quality_ok = true;
    let mut all_ok = true;
    for id in ["glazed-ceramic", "leather", "wood"] {
        let (directory, acceptance) = material(root, id)?;
        let mut defaults = BTreeMap::new();
        for case in &acceptance.cases {
            let inputs = manifest["cases"].as_array().ok_or("missing native cases")?;
            let matching: Vec<_> = inputs
                .iter()
                .filter(|v| v["material"] == id && v["id"] == case.id)
                .collect();
            if matching.len() != 1 {
                return Err("native case missing or duplicated".into());
            }
            let input = matching[0];
            let overrides = if let Some(variant) = &case.variant {
                files::json::<Variant>(&directory.join(format!("variants/{variant}.json")))?
                    .overrides
            } else {
                BTreeMap::new()
            };
            if input["sourceSha256"]
                != files::digest(&directory.join(source_file))?.trim_start_matches("sha256:")
                || input["acceptanceSha256"]
                    != files::digest(&directory.join("acceptance.json"))?
                        .trim_start_matches("sha256:")
                || input["overrides"] != json!(overrides)
            {
                return Err("native input differs from current fixture contract".into());
            }
            let native_dir = native.join(id).join(&case.id);
            let browser_dir = browser.join(id).join(&case.id);
            let result: Value = files::json(&browser_dir.join("result.json"))?;
            let native_result: Value = files::json(&native_dir.join("native.json"))?;
            let plan_ok = plan_equivalent(&native_result["inspection"]["plan"], &result["plan"])
                && result["plan"]["hash"] == input["plan"]["hash"]
                && native_result["inspection"]["plan"] == input["plan"]
                && native_result["render"]["planHash"] == input["plan"]["hash"];
            let mut images = BTreeMap::new();
            let mut channels = BTreeMap::new();
            let mut rows = Vec::new();
            for channel in CHANNELS {
                let name = format!("{channel}.png");
                if input["nativePixels"][channel]
                    != files::digest(&native_dir.join(&name))?.trim_start_matches("sha256:")
                {
                    return Err("native pixels changed since preparation".into());
                }
                let encoding = &acceptance.channels[channel].encoding;
                let before = Image::read(&native_dir.join(&name), 1024, encoding)?;
                let image = Image::read(&browser_dir.join(&name), 1024, encoding)?;
                let quality_comparison =
                    browser_quality::compare(&before, &image, channel, &policy);
                let structure = pixels::structure(&image, &case.checks[channel]);
                let cause = if case.id == "default" {
                    None
                } else {
                    Some(pixels::causality(
                        &defaults[channel],
                        &image,
                        &case.changes[channel],
                    ))
                };
                numerical_ok &= quality_comparison["ok"] == true;
                semantics_ok &= plan_ok;
                quality_ok &=
                    structure["ok"] == true && cause.as_ref().is_none_or(|c| c["ok"] == true);
                all_ok &= plan_ok
                    && structure["ok"] == true
                    && cause.as_ref().is_none_or(|c| c["ok"] == true)
                    && (measure || quality_comparison["ok"] == true);
                channels.insert(channel, json!({"comparison":quality_comparison,"structure":structure,"causality":cause,
                    "nativePngSha256":files::digest(&native_dir.join(&name))?,"browserPngSha256":files::digest(&browser_dir.join(&name))?}));
                rows.push((
                    channel.to_owned(),
                    vec![
                        ("NATIVE".into(), Some(before.clone())),
                        ("BROWSER".into(), Some(image.clone())),
                        ("DIFF X4".into(), Some(pixels::difference(&before, &image))),
                    ],
                ));
                if case.id == "default" {
                    defaults.insert(channel.to_owned(), image.clone());
                }
                images.insert(channel.to_owned(), image);
            }
            let relationships: Vec<_> = case
                .relationships
                .iter()
                .map(|rule| pixels::relationship(&images, rule))
                .collect();
            quality_ok &= relationships.iter().all(|r| r["ok"] == true);
            all_ok &= relationships.iter().all(|r| r["ok"] == true);
            pixels::sheet(&browser_dir.join("comparison.png"), &rows)?;
            records.push(json!({"material":id,"case":case.id,"planMatches":plan_ok,"channels":channels,"relationships":relationships}));
        }
    }
    if manifest["cases"].as_array().map(Vec::len) != Some(records.len())
        || receipt["cases"].as_array().map(Vec::len) != Some(records.len())
    {
        return Err("unexpected material matrix cardinality".into());
    }
    let report = json!({"schemaVersion":3,"mode":if measure {"measurement only"}else{"acceptance"},"ok":all_ok,
        "nativeManifestSha256":files::digest(&native.join("manifest.json"))?,"browserReceiptSha256":files::digest(&browser.join("receipt.json"))?,
        "profile":policy,"profileSha256":files::digest(&root.join("docs/browser-quality-v2.json"))?,
        "gates":{"semantics":semantics_ok,"materialStructure":quality_ok,"numericalAgreement":numerical_ok},
        "cases":records});
    files::write_json(&browser.join("comparison.json"), &report)?;
    if !all_ok {
        return Err(
            "browser material comparison failed; inspect comparison.json; no baseline changed"
                .into(),
        );
    }
    fs::write(
        browser.join("mode.txt"),
        if measure {
            "Measured; not accepted\n"
        } else {
            "Passed frozen comparison gates\n"
        },
    )?;
    println!(
        "Browser material {} complete: {}",
        if measure { "measurement" } else { "comparison" },
        browser.display()
    );
    Ok(())
}

// The JS bridge widens f32 values and transports u64 as bigint strings in receipts.
// Preserve every integer exactly; only actual floating-point fields use f32 semantics.
pub(super) fn plan_equivalent(native: &Value, browser: &Value) -> bool {
    match (native, browser) {
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|other| plan_equivalent(v, other)))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(v, other)| plan_equivalent(v, other))
        }
        (Value::Number(a), other) if a.is_u64() => {
            a.as_u64()
                == other
                    .as_u64()
                    .or_else(|| other.as_str().and_then(|s| s.parse::<u64>().ok()))
        }
        (Value::Number(a), Value::Number(b)) => {
            a.as_f64().map(|v| v as f32) == b.as_f64().map(|v| v as f32)
        }
        _ => native == browser,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plan_projection_preserves_integer_identity_and_normalizes_only_float_precision() {
        assert!(plan_equivalent(
            &json!({"value":0.45,"bytes":18446744073709551615u64}),
            &json!({"value":0.44999998807907104,"bytes":"18446744073709551615"})
        ));
        assert!(!plan_equivalent(&json!(16777217u64), &json!(16777216u64)));
        assert!(!plan_equivalent(&json!(0.45), &json!(0.46)));
        assert!(!plan_equivalent(&json!({"id":"1"}), &json!({"id":1})));
        assert!(!plan_equivalent(
            &json!({"size":[1024,1024]}),
            &json!({"size":[1024]})
        ));
    }
    #[test]
    fn failed_recheck_invalidates_previous_acceptance() {
        let directory = std::env::temp_dir().join(format!(
            "mixture-browser-recheck-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        files::write_json(&directory.join("comparison.json"), &json!({"ok":true})).unwrap();
        let result = run(&directory, &directory, &directory, false);
        assert!(result.is_err());
        let report: Value = files::json(&directory.join("comparison.json")).unwrap();
        assert_eq!(report["ok"], false);
        assert_eq!(
            fs::read_to_string(directory.join("mode.txt")).unwrap(),
            "Comparison incomplete\n"
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
