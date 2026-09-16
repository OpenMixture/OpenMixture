//! Guarded material goldens, orchestrated through the existing public CLI.

pub(crate) mod browser;
mod files;
mod model;
mod pixels;
pub(crate) mod studio;
#[cfg(test)]
mod tests;
pub(super) mod trace;

use crate::{TaskResult, gpu_smoke};
use files::Digests;
use model::{Acceptance, CHANNELS, Tolerance, Variant};
use pixels::Image;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const USAGE: &str = "Usage: cargo xtask golden check\n       cargo xtask test-material <id>\n       cargo xtask golden update <id> --accept\nChecks render review candidates, never write expected/. Updates consume a prior software candidate without rendering and refuse CI.";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Baseline {
    schema_version: u32,
    material: String,
    software_revision: String,
    plan_hashes: BTreeMap<String, String>,
    files: Digests,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Candidate {
    schema_version: u32,
    material: String,
    inputs: Digests,
    previous_baseline: Digests,
    artifacts: Digests,
}

pub(super) fn dispatch(root: &Path, args: &[OsString]) -> TaskResult {
    let args = args
        .iter()
        .map(|s| s.to_str().ok_or("golden arguments must be UTF-8"))
        .collect::<Result<Vec<_>, _>>()?;
    match args.as_slice() {
        ["golden", "check"] => {
            let mut ids = Vec::new();
            for entry in fs::read_dir(root.join("fixtures/materials"))? {
                let entry = entry?;
                if entry.path().join("acceptance.json").is_file() {
                    ids.push(
                        entry
                            .file_name()
                            .into_string()
                            .map_err(|_| "material ID must be UTF-8")?,
                    );
                }
            }
            ids.sort();
            if ids.is_empty() {
                return Err("no material acceptance fixtures found".into());
            }
            let mut failed = Vec::new();
            for id in ids {
                if let Err(error) = check(root, &id) {
                    eprintln!("{id}: {error}");
                    failed.push(id);
                }
            }
            if !failed.is_empty() {
                return Err(format!("material checks failed: {}", failed.join(", ")).into());
            }
            Ok(())
        }
        ["test-material", id] => check(root, id),
        ["golden", "update", id, "--accept"] => {
            // Presence, even CI=false, is deliberately conservative. No rendering or I/O first.
            if env::var_os("CI").is_some() || env::var_os("GITHUB_ACTIONS").is_some() {
                return Err("golden update refuses CI; review and accept locally".into());
            }
            update(root, id)
        }
        _ => Err(USAGE.into()),
    }
}

fn material(root: &Path, id: &str) -> TaskResult<(PathBuf, Acceptance)> {
    if !model::valid_id(id) {
        return Err("invalid material ID".into());
    }
    let directory = files::relative(root, &format!("fixtures/materials/{id}"))?;
    let acceptance: Acceptance = files::json(&directory.join("acceptance.json"))?;
    acceptance.validate(id)?;
    Ok((directory, acceptance))
}

fn baseline(directory: &Path, acceptance: &Acceptance) -> TaskResult<Option<Baseline>> {
    let expected = files::relative(directory, "expected")?;
    let actual = files::tree(&expected)?;
    if actual.is_empty() {
        return Ok(None);
    }
    let manifest: Baseline = files::json(&expected.join("manifest.json"))?;
    if manifest.schema_version != 1
        || manifest.material != acceptance.material
        || manifest.software_revision != acceptance.software_revision
        || manifest.files.is_empty()
        || actual.len() != manifest.files.len() + 1
        || !actual.contains_key("manifest.json")
        || manifest.plan_hashes.is_empty()
    {
        return Err("incomplete or incompatible golden manifest; baseline was not modified".into());
    }
    for case in manifest.plan_hashes.keys() {
        if !model::valid_id(case)
            || CHANNELS.iter().any(|channel| {
                !manifest
                    .files
                    .contains_key(&format!("{case}/{channel}.png"))
            })
        {
            return Err("incomplete golden case coverage".into());
        }
    }
    if manifest.files.len() != manifest.plan_hashes.len() * CHANNELS.len() {
        return Err("unexpected files in golden manifest".into());
    }
    files::verify(&expected, &manifest.files)?;
    Ok(Some(manifest))
}

fn output_names(acceptance: &Acceptance) -> Vec<String> {
    let mut names = acceptance
        .cases
        .iter()
        .flat_map(|case| CHANNELS.map(|channel| format!("{}/{channel}.png", case.id)))
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn review_root(root: &Path, id: &str) -> TaskResult<PathBuf> {
    let directory = files::relative(root, &format!("tmp/golden/{id}"))?;
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn check(root: &Path, id: &str) -> TaskResult {
    let (directory, acceptance) = material(root, id)?;
    let (backend, software) = gpu_smoke::policy()?;
    let software = software == "1";
    if software && backend != "vulkan" {
        return Err("material software goldens require explicit vulkan backend".into());
    }
    let policy = if software { "software" } else { "hardware" };
    let review_root = review_root(root, id)?;
    let run = format!(
        "{policy}-{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id()
    );
    let review = files::relative(&review_root, &run)?;
    fs::create_dir(&review)?;
    // Invalidate an earlier successful candidate before any operation in this run can fail.
    files::write_json(
        &review_root.join(format!("latest-{policy}.json")),
        &json!({"run":run,"ready":false}),
    )?;
    let initial_inputs = files::inputs(root, &directory)?;
    let previous_baseline = files::tree(&directory.join("expected"))?;
    let baseline = baseline(&directory, &acceptance)?;
    let driver = if software {
        Some(software_source(root, &acceptance)?)
    } else {
        None
    };
    let expected_adapter = if software {
        Some("SwiftShader".to_owned())
    } else {
        env::var("MIXTURE_GPU_EXPECT_ADAPTER").ok()
    };
    let software_arg = if software { "1" } else { "0" };
    println!(
        "Rendering {id} on {backend} ({policy}), {}x{}, {} cases",
        acceptance.size,
        acceptance.size,
        acceptance.cases.len()
    );
    let doctor = gpu_smoke::command_report(
        root,
        &["doctor", "--json"],
        &backend,
        software_arg,
        &review,
        "doctor",
    )?;
    gpu_smoke::validate_report(&doctor, &backend, software, expected_adapter.as_deref())?;
    if !software && doctor["adapter"]["deviceType"] == "Cpu" {
        return Err(
            "hardware comparison selected a CPU adapter; request the software policy explicitly"
                .into(),
        );
    }
    let tolerance = if software {
        Tolerance::EXACT
    } else {
        acceptance.hardware_tolerance
    };
    let mut cases = Vec::new();
    let mut default_images = BTreeMap::new();
    let mut overview = Vec::new();
    let mut tiles = Vec::new();
    let (mut machine_passed, mut comparisons_passed) = (true, baseline.is_some());
    for case in &acceptance.cases {
        let overrides = if let Some(variant) = &case.variant {
            let variant: Variant = files::json(&files::relative(
                &directory,
                &format!("variants/{variant}.json"),
            )?)?;
            if variant.overrides.is_empty() {
                return Err("variant has no parameter overrides".into());
            }
            variant.overrides
        } else {
            BTreeMap::new()
        };
        let input = directory
            .join("material.mix")
            .strip_prefix(root)?
            .to_string_lossy()
            .into_owned();
        let output = review.join(&case.id);
        let mut arguments = vec![
            "render".into(),
            input,
            "--size".into(),
            acceptance.size.to_string(),
            "--output".into(),
            CHANNELS.join(","),
            "--out".into(),
            output.to_string_lossy().into_owned(),
            "--json".into(),
        ];
        for (name, value) in &overrides {
            arguments.extend(["--set".into(), format!("{name}={value}")]);
        }
        let report = gpu_smoke::command_report(
            root,
            &arguments.iter().map(String::as_str).collect::<Vec<_>>(),
            &backend,
            software_arg,
            &review,
            &format!("render-{}", case.id),
        )?;
        validate_render(&report, &doctor, &acceptance)?;
        let plan_matches = baseline.as_ref().is_some_and(|b| {
            report["planHash"].as_str() == b.plan_hashes.get(&case.id).map(String::as_str)
        });
        comparisons_passed &= plan_matches;
        let mut channels = BTreeMap::new();
        let mut case_images = BTreeMap::new();
        let mut rows = Vec::new();
        let mut previews = Vec::new();
        for channel in CHANNELS {
            let filename = format!("{}/{channel}.png", case.id);
            let contract = &acceptance.channels[channel];
            let image = Image::read(&review.join(&filename), acceptance.size, &contract.encoding)?;
            case_images.insert(channel.to_owned(), image.clone());
            let shape = pixels::structure(&image, &case.checks[channel]);
            machine_passed &= shape["ok"] == true;
            let cause = if case.id == "default" {
                None
            } else {
                let result =
                    pixels::causality(&default_images[channel], &image, &case.changes[channel]);
                machine_passed &= result["ok"] == true;
                Some(result)
            };
            let before = if baseline
                .as_ref()
                .is_some_and(|b| b.files.contains_key(&filename))
            {
                Some(Image::read(
                    &directory.join("expected").join(&filename),
                    acceptance.size,
                    &contract.encoding,
                )?)
            } else {
                None
            };
            let comparison = before
                .as_ref()
                .map(|b| pixels::compare(b, &image, tolerance))
                .unwrap_or_else(|| json!({"status":"missing","ok":false}));
            comparisons_passed &= comparison["ok"] == true;
            let diff = before.as_ref().map(|b| pixels::difference(b, &image));
            rows.push((
                channel.to_owned(),
                vec![
                    (
                        if before.is_some() {
                            "BEFORE"
                        } else {
                            "BEFORE MISSING"
                        }
                        .into(),
                        before,
                    ),
                    ("AFTER".into(), Some(image.clone())),
                    ("DIFF X4".into(), diff),
                ],
            ));
            previews.push((channel.to_owned(), Some(image.clone())));
            channels.insert(channel.to_owned(),json!({"statistics":image.statistics(),"structure":shape,"causality":cause,"comparison":comparison,
                "floatReadback":{"ok":true,"nonFiniteComponents":0,"outOfRangeComponents":0,"range":[0,1],"evidence":"mixture-wgpu readback validates every f16 component before RGBA8 encoding; invalid values fail render"}}));
            if channel == "baseColor" {
                tiles.push((
                    case.id.clone(),
                    vec![("2X2 REPEAT".into(), Some(pixels::tiled(&image)))],
                ));
            }
            if case.id == "default" {
                default_images.insert(channel.to_owned(), image);
            }
        }
        pixels::sheet(&review.join(format!("contact-{}.png", case.id)), &rows)?;
        let relationships = case
            .relationships
            .iter()
            .map(|rule| pixels::relationship(&case_images, rule))
            .collect::<Vec<_>>();
        machine_passed &= relationships.iter().all(|r| r["ok"] == true);
        overview.push((case.id.clone(), previews));
        cases.push(json!({"id":case.id,"overrides":overrides,"planHash":report["planHash"],"planMatchesGolden":plan_matches,"execution":report["execution"],"outputs":report["outputs"],"channels":channels,"relationships":relationships}));
    }
    pixels::sheet(&review.join("overview.png"), &overview)?;
    pixels::sheet(&review.join("tiling.png"), &tiles)?;
    // Rendering can take time: never publish an accept-ready candidate for mixed input states.
    if files::inputs(root, &directory)? != initial_inputs
        || files::tree(&directory.join("expected"))? != previous_baseline
    {
        return Err("inputs or baseline changed during rendering; rerun before reviewing".into());
    }
    let ok = machine_passed && comparisons_passed;
    let report = json!({"schemaVersion":1,"material":id,"size":acceptance.size,"policy":policy,"softwareSource":driver,
        "adapter":doctor["adapter"],"requested":doctor["requested"],"ok":ok,"machineChecksPassed":machine_passed,"goldenComparisonsPassed":comparisons_passed,
        "baselinePresent":baseline.is_some(),"humanReview":"not decided by this command; see fixture reports/human-review.json",
        "cases":cases,"contactSheets":acceptance.cases.iter().map(|c|format!("contact-{}.png",c.id)).chain(["overview.png".into(),"tiling.png".into()]).collect::<Vec<_>>(),"remoteCi":"not inferred by this report; verify the CI run and revision separately"});
    files::write_json(&review.join("report.json"), &report)?;
    let candidate = Candidate {
        schema_version: 1,
        material: id.to_owned(),
        inputs: initial_inputs,
        previous_baseline,
        artifacts: files::tree(&review)?,
    };
    files::write_json(&review.join("candidate.json"), &candidate)?;
    files::write_json(
        &review_root.join(format!("latest-{policy}.json")),
        &json!({"run":run,"ready":machine_passed}),
    )?;
    println!(
        "Material machine checks: {machine_passed}; golden comparisons: {comparisons_passed}. Review: {}",
        review.join("report.json").display()
    );
    if !ok {
        return Err("material acceptance failed or baseline is missing; inspect report and contact sheets; no baseline was changed".into());
    }
    Ok(())
}

fn validate_render(report: &Value, doctor: &Value, acceptance: &Acceptance) -> TaskResult {
    let outputs = report["outputs"]
        .as_array()
        .ok_or("render report has no outputs")?;
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["diagnostics"] != json!([])
        || report["execution"]["size"] != json!([acceptance.size, acceptance.size])
        || report["planHash"] != report["execution"]["planHash"]
        || !report["planHash"]
            .as_str()
            .is_some_and(|s| s.starts_with("sha256:") && s.len() == 71)
        || report["context"]["adapter"] != doctor["adapter"]
        || report["execution"]["adapter"] != doctor["adapter"]
        || outputs.len() != CHANNELS.len()
        || report["execution"]["readbackBytes"] != json!(u64::from(acceptance.size).pow(2) * 8 * 4)
    {
        return Err("incomplete or inconsistent material execution evidence".into());
    }
    for (output, channel) in outputs.iter().zip(CHANNELS) {
        if output["channel"] != channel
            || output["encoding"] != acceptance.channels[channel].encoding
            || output["source"]["source"] != acceptance.channels[channel].source
            || output["size"] != json!([acceptance.size, acceptance.size])
            || output["writtenBytes"].as_u64().is_none_or(|n| n == 0)
        {
            return Err(format!("missing or wrong {channel} output provenance/encoding").into());
        }
    }
    Ok(())
}

// The source pin is checked, not merely copied from an environment assertion. Adapter
// identity and loader configuration are separate evidence; this is not a hermetic-build claim.
fn software_source(root: &Path, acceptance: &Acceptance) -> TaskResult<Value> {
    let source = env::var_os("MIXTURE_SWIFTSHADER_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("tmp/swiftshader/source"));
    let output = Command::new("git")
        .arg("-C")
        .arg(&source)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let revision = String::from_utf8(output.stdout)?.trim().to_owned();
    if !output.status.success() || revision != acceptance.software_revision {
        return Err("pinned SwiftShader source not found; run setup-swiftshader.sh or set MIXTURE_SWIFTSHADER_SOURCE to its existing checkout".into());
    }
    let clean = Command::new("git")
        .arg("-C")
        .arg(&source)
        .args(["diff", "--quiet", "HEAD", "--"])
        .status()?;
    if !clean.success() {
        return Err("SwiftShader tracked source differs from the pinned revision".into());
    }
    Ok(
        json!({"revision":revision,"source":source,"platform":env::consts::OS,"architecture":env::consts::ARCH,
        "loader":{"DYLD_LIBRARY_PATH":env::var_os("DYLD_LIBRARY_PATH"),"VK_DRIVER_FILES":env::var_os("VK_DRIVER_FILES"),"VK_ICD_FILENAMES":env::var_os("VK_ICD_FILENAMES")}}),
    )
}

fn update(root: &Path, id: &str) -> TaskResult {
    let (directory, acceptance) = material(root, id)?;
    let review_root = review_root(root, id)?;
    let latest: Value = files::json(&review_root.join("latest-software.json"))?;
    if latest["ready"] != true {
        return Err("no complete software candidate; run a material check first".into());
    }
    let run = latest["run"]
        .as_str()
        .filter(|s| {
            s.starts_with("software-")
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
        .ok_or("invalid candidate run ID")?;
    let review = files::relative(&review_root, run)?;
    if review.join("update.json").exists() {
        return Err("candidate already accepted; render a new candidate for another update".into());
    }
    let candidate: Candidate = files::json(&review.join("candidate.json"))?;
    verify_candidate(root, &directory, &review, &candidate, &acceptance)?;
    let report: Value = files::json(&review.join("report.json"))?;
    let names = output_names(&acceptance);
    let mut plan_hashes = BTreeMap::new();
    for case in report["cases"].as_array().ok_or("candidate has no cases")? {
        plan_hashes.insert(
            case["id"].as_str().ok_or("case has no ID")?.to_owned(),
            case["planHash"]
                .as_str()
                .ok_or("case has no plan hash")?
                .to_owned(),
        );
    }
    let baseline = Baseline {
        schema_version: 1,
        material: id.to_owned(),
        software_revision: acceptance.software_revision,
        plan_hashes,
        files: names
            .iter()
            .map(|name| {
                Ok((
                    name.clone(),
                    files::digest(&files::relative(&review, name)?)?,
                ))
            })
            .collect::<TaskResult<_>>()?,
    };
    let staged = files::relative(&directory, &format!("expected-{run}"))?;
    fs::create_dir(&staged)?;
    for name in &names {
        let target = files::relative(&staged, name)?;
        fs::create_dir_all(target.parent().ok_or("output has no parent")?)?;
        fs::copy(files::relative(&review, name)?, target)?;
    }
    files::write_json(&staged.join("manifest.json"), &baseline)?;
    files::verify(&staged, &baseline.files)?;
    let mut accepted_sheets = Vec::new();
    for case in &acceptance.cases {
        let mut rows = Vec::new();
        for channel in CHANNELS {
            let name = format!("{}/{channel}.png", case.id);
            let encoding = &acceptance.channels[channel].encoding;
            let after = Image::read(&review.join(&name), acceptance.size, encoding)?;
            let before = if candidate.previous_baseline.contains_key(&name) {
                Some(Image::read(
                    &directory.join("expected").join(&name),
                    acceptance.size,
                    encoding,
                )?)
            } else {
                None
            };
            let difference = before
                .as_ref()
                .map(|image| pixels::difference(image, &after));
            rows.push((
                channel.to_owned(),
                vec![
                    (
                        if before.is_some() {
                            "BEFORE"
                        } else {
                            "BEFORE MISSING"
                        }
                        .into(),
                        before,
                    ),
                    ("AFTER".into(), Some(after)),
                    ("DIFF X4".into(), difference),
                ],
            ));
        }
        let filename = format!("accepted-contact-{}.png", case.id);
        pixels::sheet(&review.join(&filename), &rows)?;
        accepted_sheets.push(filename);
    }
    let expected = files::relative(&directory, "expected")?;
    let previous = files::relative(&review, "previous-expected")?;
    // Verify immediately before replacement, and retain the old complete directory for review.
    if files::tree(&expected)? != candidate.previous_baseline
        || files::inputs(root, &directory)? != candidate.inputs
    {
        return Err("inputs or baseline changed before acceptance; nothing replaced".into());
    }
    let had_expected = expected.exists();
    if had_expected {
        fs::rename(&expected, &previous)?;
    }
    if let Err(error) = fs::rename(&staged, &expected) {
        if had_expected {
            fs::rename(&previous, &expected)?;
        }
        return Err(error.into());
    }
    files::write_json(
        &review.join("update.json"),
        &json!({"schemaVersion":1,"material":id,"acceptedFlag":true,"renderedDuringUpdate":false,
        "previousBaseline":candidate.previous_baseline,"newBaseline":files::tree(&expected)?,"candidateReportSha256":candidate.artifacts["report.json"],
        "cases":report["cases"],"contactSheets":accepted_sheets,
        "humanReview":"separate record; --accept does not assert human visual approval","gitStagedOrCommitted":false}),
    )?;
    println!(
        "Accepted existing software candidate for {id}. No rendering, staging, or commit. Acceptance report: {}",
        review.join("update.json").display()
    );
    Ok(())
}

fn verify_candidate(
    root: &Path,
    directory: &Path,
    review: &Path,
    candidate: &Candidate,
    acceptance: &Acceptance,
) -> TaskResult {
    if candidate.schema_version != 1
        || candidate.material != acceptance.material
        || files::inputs(root, directory)? != candidate.inputs
    {
        return Err("candidate inputs are stale; rerun the material check and review".into());
    }
    if files::tree(&directory.join("expected"))? != candidate.previous_baseline {
        return Err("candidate baseline is stale; rerun and review".into());
    }
    if !candidate.artifacts.contains_key("report.json") {
        return Err("candidate is missing its report digest".into());
    }
    files::verify(review, &candidate.artifacts)?;
    let report: Value = files::json(&review.join("report.json"))?;
    if report["schemaVersion"] != 1
        || report["material"] != acceptance.material
        || report["policy"] != "software"
        || report["machineChecksPassed"] != true
        || report["softwareSource"]["revision"] != acceptance.software_revision
        || report["adapter"]["backend"] != "Vulkan"
        || report["adapter"]["deviceType"] != "Cpu"
        || !report["adapter"]["name"]
            .as_str()
            .is_some_and(|name| name.contains("SwiftShader"))
        || report["cases"]
            .as_array()
            .is_none_or(|cases| cases.len() != acceptance.cases.len())
    {
        return Err("candidate lacks passing pinned software evidence".into());
    }
    for name in output_names(acceptance) {
        if !candidate.artifacts.contains_key(&name) {
            return Err(format!("missing candidate channel: {name}").into());
        }
    }
    // Re-measure the content we are about to install; a report flag alone is insufficient.
    let mut default_images = BTreeMap::new();
    for (case, evidence) in acceptance.cases.iter().zip(
        report["cases"]
            .as_array()
            .ok_or("missing candidate cases")?,
    ) {
        if evidence["id"] != case.id
            || !evidence["planHash"].as_str().is_some_and(|s| {
                s.strip_prefix("sha256:")
                    .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
            })
        {
            return Err("invalid candidate case identity/plan hash".into());
        }
        for channel in CHANNELS {
            let image = Image::read(
                &files::relative(review, &format!("{}/{channel}.png", case.id))?,
                acceptance.size,
                &acceptance.channels[channel].encoding,
            )?;
            if pixels::structure(&image, &case.checks[channel])["ok"] != true
                || (case.id != "default"
                    && pixels::causality(&default_images[channel], &image, &case.changes[channel])
                        ["ok"]
                        != true)
            {
                return Err("candidate fails rechecked structure/causality gates".into());
            }
            if case.id == "default" {
                default_images.insert(channel.to_owned(), image);
            }
        }
    }
    Ok(())
}
