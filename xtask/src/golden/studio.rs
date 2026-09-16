//! Detached Studio downloads checked with the existing engine-owned pixel rules.
use super::{
    browser::plan_equivalent,
    files,
    model::{CHANNELS, Case, Tolerance},
    pixels::{self, Image},
};
use crate::TaskResult;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) fn run(root: &Path, native: &Path, browser: &Path) -> TaskResult {
    files::write_json(
        &browser.join("comparison.json"),
        &json!({"ok":false,"status":"incomplete"}),
    )?;
    fs::write(browser.join("mode.txt"), "Comparison incomplete\n")?;
    let manifest: Value = files::json(&native.join("manifest.json"))?;
    let receipt: Value = files::json(&browser.join("receipt.json"))?;
    let criteria_path = root.join("scripts/browser-runtime/studio-criteria.json");
    let criteria: Value = files::json(&criteria_path)?;
    if manifest["schemaVersion"] != 1
        || receipt["schemaVersion"] != 1
        || manifest["criteriaSha256"]
            != files::digest(&criteria_path)?.trim_start_matches("sha256:")
        || receipt["manifestSha256"]
            != files::digest(&native.join("manifest.json"))?.trim_start_matches("sha256:")
        || receipt["build"]["engineRevision"] != manifest["runtimeRevision"]
        || receipt["archiveSha256"] != manifest["archiveSha256"]
    {
        return Err("Studio provenance mismatch".into());
    }
    let inputs = manifest["cases"].as_array().ok_or("missing native cases")?;
    let runs = receipt["cases"].as_array().ok_or("missing Player cases")?;
    let specs = criteria["cases"].as_array().ok_or("missing criteria")?;
    if inputs.len() != 7 || runs.len() != 7 || specs.len() != 7 {
        return Err("wrong Studio matrix cardinality".into());
    }
    let tolerances: BTreeMap<String, Tolerance> =
        files::json(&root.join("docs/browser-tolerances.json"))?;
    let mut defaults = BTreeMap::new();
    let mut records = Vec::new();
    let mut ok = true;
    for spec in specs {
        let material = spec["material"].as_str().ok_or("material")?;
        let id = spec["id"].as_str().ok_or("case")?;
        let matching: Vec<_> = inputs
            .iter()
            .filter(|i| i["material"] == material && i["id"] == id)
            .collect();
        let completed: Vec<_> = runs
            .iter()
            .filter(|i| i["material"] == material && i["id"] == id)
            .collect();
        if matching.len() != 1 || completed.len() != 1 {
            return Err("missing/duplicate case".into());
        }
        let input = matching[0];
        let rules: Case = serde_json::from_value(spec["rules"].clone())?;
        let before_dir = native.join(material).join(id);
        let after_dir = browser.join(material).join(id);
        let native_result: Value = files::json(&before_dir.join("native.json"))?;
        let result: Value = files::json(&after_dir.join("result.json"))?;
        let plan_ok = plan_equivalent(&input["plan"], &result["plan"])
            && input["plan"] == native_result["inspection"]["plan"]
            && input["plan"]["hash"] == native_result["render"]["planHash"]
            && result["plan"]["hash"] == input["plan"]["hash"]
            && result["sourceSha256"] == input["sourceSha256"]
            && completed[0]["sourceSha256"] == input["sourceSha256"];
        ok &= plan_ok;
        let mut images = BTreeMap::new();
        let mut native_images = BTreeMap::new();
        let mut channels = BTreeMap::new();
        let mut rows = Vec::new();
        for channel in CHANNELS {
            let encoding = if channel == "baseColor" {
                "rgba8-srgb"
            } else {
                "rgba8-linear"
            };
            let before_path = before_dir.join(format!("{channel}.png"));
            let after_path = after_dir.join(format!("{channel}.png"));
            let channel_result: Vec<_> = result["channels"]
                .as_array()
                .ok_or("channels")?
                .iter()
                .filter(|c| c["channel"] == channel)
                .collect();
            if channel_result.len() != 1
                || input["nativePixels"][channel]
                    != files::digest(&before_path)?.trim_start_matches("sha256:")
                || channel_result[0]["pngSha256"]
                    != files::digest(&after_path)?.trim_start_matches("sha256:")
            {
                return Err("pixel provenance mismatch".into());
            }
            let channel_plan_ok =
                plan_equivalent(&input["channelPlans"][channel], &channel_result[0]["plan"])
                    && input["channelPlans"][channel]["hash"] == channel_result[0]["plan"]["hash"];
            let before = Image::read(&before_path, 1024, encoding)?;
            let image = Image::read(&after_path, 1024, encoding)?;
            let comparison = pixels::compare(&before, &image, tolerances[channel]);
            let structure = pixels::structure(&image, &rules.checks[channel]);
            let native_structure = pixels::structure(&before, &rules.checks[channel]);
            let cause = if rules.changes.is_empty() {
                None
            } else {
                Some(pixels::causality(
                    &defaults[&(material.to_owned(), channel.to_owned())],
                    &image,
                    &rules.changes[channel],
                ))
            };
            ok &= channel_plan_ok
                && comparison["ok"] == true
                && structure["ok"] == true
                && native_structure["ok"] == true
                && cause.as_ref().is_none_or(|c| c["ok"] == true);
            channels.insert(channel,json!({"comparison":comparison,"structure":structure,"nativeStructure":native_structure,"causality":cause,"planMatches":channel_plan_ok}));
            rows.push((
                channel.to_owned(),
                vec![
                    ("NATIVE".into(), Some(before.clone())),
                    ("PLAYER".into(), Some(image.clone())),
                    ("DIFF X4".into(), Some(pixels::difference(&before, &image))),
                ],
            ));
            if id == "default" {
                defaults.insert((material.to_owned(), channel.to_owned()), image.clone());
            }
            images.insert(channel.to_owned(), image);
            native_images.insert(channel.to_owned(), before);
        }
        let relationships: Vec<_> = rules
            .relationships
            .iter()
            .map(|r| pixels::relationship(&images, r))
            .collect();
        let native_relationships: Vec<_> = rules
            .relationships
            .iter()
            .map(|r| pixels::relationship(&native_images, r))
            .collect();
        ok &= relationships
            .iter()
            .chain(&native_relationships)
            .all(|r| r["ok"] == true);
        pixels::sheet(&after_dir.join("comparison.png"), &rows)?;
        records.push(json!({"material":material,"id":id,"planMatches":plan_ok,"channels":channels,"relationships":relationships,"nativeRelationships":native_relationships}));
    }
    files::write_json(
        &browser.join("comparison.json"),
        &json!({"schemaVersion":1,"ok":ok,"criteriaSha256":files::digest(&criteria_path)?,"toleranceSha256":files::digest(&root.join("docs/browser-tolerances.json"))?,"cases":records}),
    )?;
    if !ok {
        return Err("Studio comparison failed; no criteria or golden changed".into());
    }
    fs::write(browser.join("mode.txt"), "Passed frozen Studio gates\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_recheck_invalidates_previous_acceptance() {
        let path = std::env::temp_dir().join(format!("studio-recheck-{}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        files::write_json(&path.join("comparison.json"), &json!({"ok":true})).unwrap();
        fs::write(path.join("mode.txt"), "Passed frozen Studio gates\n").unwrap();
        assert!(run(&path, &path, &path).is_err());
        let value: Value = files::json(&path.join("comparison.json")).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(
            fs::read_to_string(path.join("mode.txt")).unwrap(),
            "Comparison incomplete\n"
        );
        fs::remove_dir_all(path).unwrap();
    }
}
