//! A measured naive 2K workload selected from every M3 material case.
//! Selection/verification belongs to tooling; allocation and pixels stay in wgpu.

use super::{
    Acceptance, CHANNELS, Variant, files, gpu_smoke, material, software_source, validate_render,
};
use crate::{TaskResult, cargo};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

// This is the documented M3 workload gate, independently checked against actual
// descriptor accounting. The compiler also applies its normal SafetyLimits.
const BUDGET: u64 = 512 * 1024 * 1024;
const SIZE: u32 = 2048;
const MATERIALS: [&str; 3] = ["glazed-ceramic", "leather", "wood"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RankedCase {
    material: String,
    case: String,
    overrides: BTreeMap<String, Value>,
    plan_hash: String,
    pass_count: usize,
    estimates: Value,
}

pub(crate) fn run(root: &Path) -> TaskResult {
    let (backend, software) = gpu_smoke::policy()?;
    run_with_policy(root, &backend, &software)
}

fn run_with_policy(root: &Path, backend: &str, software: &str) -> TaskResult {
    let policy = if software == "1" {
        "software"
    } else {
        "hardware"
    };
    let output_root = files::relative(root, "tmp/trace-2k")?;
    fs::create_dir_all(&output_root)?;
    let run = format!(
        "{policy}-{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id()
    );
    let output = files::relative(&output_root, &run)?;
    fs::create_dir(&output)?;
    let latest = output_root.join(format!("latest-{policy}.json"));
    files::write_json(&latest, &json!({"run":run,"ok":false}))?;
    let result = (|| {
        if software == "1" && backend != "vulkan" {
            return Err("trace-2k software execution requires explicit vulkan".into());
        }
        let fixtures = MATERIALS
            .into_iter()
            .map(|id| material(root, id))
            .collect::<TaskResult<Vec<_>>>()?;
        trace(root, &output, backend, software, &fixtures)
    })();
    if let Err(error) = &result {
        // The first CLI error and source chain remain in inspect/render/doctor JSON.
        files::write_json(
            &output.join("failure.json"),
            &json!({"schemaVersion":1,"ok":false,"message":error.to_string(),"evidence":"Retain the first failing inspect/doctor/render JSON and stderr alongside this record; no fallback or golden update occurred."}),
        )?;
    }
    result?;
    files::write_json(&latest, &json!({"run":run,"ok":true}))?;
    println!(
        "2K naive lifetime trace passed; evidence: {}",
        output.join("trace.json").display()
    );
    Ok(())
}

fn trace(
    root: &Path,
    output: &Path,
    backend: &str,
    software: &str,
    fixtures: &[(std::path::PathBuf, Acceptance)],
) -> TaskResult {
    let inputs = snapshot_inputs(root, fixtures)?;
    let mut baselines = BTreeMap::new();
    for (directory, acceptance) in fixtures {
        baselines.insert(
            acceptance.material.clone(),
            files::tree(&directory.join("expected"))?,
        );
    }
    let mut ranking = Vec::new();
    for (directory, acceptance) in fixtures {
        for case in &acceptance.cases {
            let overrides = match &case.variant {
                Some(id) => {
                    files::json::<Variant>(&files::relative(
                        directory,
                        &format!("variants/{id}.json"),
                    )?)?
                    .overrides
                }
                None => BTreeMap::new(),
            };
            let mut args = arguments("inspect", &acceptance.material, &overrides);
            args.push("--plan".to_owned());
            let name = format!("inspect-{}-{}", acceptance.material, case.id);
            let command = cargo(root)
                .args([
                    "run",
                    "--locked",
                    "--all-features",
                    "-p",
                    "mixture-cli",
                    "--",
                ])
                .args(&args)
                .output()?;
            fs::write(output.join(format!("{name}.json")), &command.stdout)?;
            fs::write(output.join(format!("{name}.stderr.log")), &command.stderr)?;
            if !command.status.success() {
                return Err(format!("{name} failed; inspect its structured diagnostics before any GPU work or lifetime change").into());
            }
            let report: Value = serde_json::from_slice(&command.stdout)?;
            let plan = &report["plan"];
            if report["ok"] != true
                || report["diagnostics"] != json!([])
                || plan["size"] != json!([SIZE, SIZE])
            {
                return Err("incomplete 2K inspection evidence".into());
            }
            let entry = RankedCase {
                material: acceptance.material.clone(),
                case: case.id.clone(),
                overrides,
                plan_hash: plan["hash"].as_str().ok_or("missing plan hash")?.to_owned(),
                pass_count: plan["passes"].as_array().ok_or("missing passes")?.len(),
                estimates: plan["estimates"].clone(),
            };
            number(&entry.estimates, "peakBytes")?;
            number(&entry.estimates, "cumulativeBytes")?;
            ranking.push(entry);
        }
    }
    rank(&mut ranking);
    let selected = ranking.first().ok_or("no M3 cases to rank")?;
    files::write_json(
        &output.join("selection.json"),
        &json!({"schemaVersion":1,"size":[SIZE,SIZE],"ranking":ranking,"selection":"highest peakBytes, then passCount, lexical material, default case first, lexical case","selected":{"material":selected.material,"case":selected.case}}),
    )?;
    let mut acceptance = fixtures
        .iter()
        .find(|(_, a)| a.material == selected.material)
        .ok_or("selected fixture is missing")?
        .1
        .clone();
    let source = if software == "1" {
        Some(software_source(root, &acceptance)?)
    } else {
        None
    };
    let expected = if software == "1" {
        Some("SwiftShader".to_owned())
    } else {
        env::var("MIXTURE_GPU_EXPECT_ADAPTER").ok()
    };
    let doctor = gpu_smoke::command_report(
        root,
        &["doctor", "--json"],
        backend,
        software,
        output,
        "doctor",
    )?;
    gpu_smoke::validate_report(&doctor, backend, software == "1", expected.as_deref())?;
    if software == "0" && doctor["adapter"]["deviceType"] == "Cpu" {
        return Err("hardware trace selected a CPU adapter; request software explicitly".into());
    }
    println!(
        "2K trace selects {}/{}: {} passes, {} peak estimated bytes",
        selected.material, selected.case, selected.pass_count, selected.estimates["peakBytes"]
    );
    let mut args = arguments("render", &selected.material, &selected.overrides);
    args.extend([
        "--out".to_owned(),
        output.join("png").to_string_lossy().into_owned(),
    ]);
    let render = gpu_smoke::command_report(
        root,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        backend,
        software,
        output,
        "render",
    )?;
    acceptance.size = SIZE;
    validate_render(&render, &doctor, &acceptance)?;
    if render["planHash"] != selected.plan_hash
        || render["execution"]["estimates"] != selected.estimates
    {
        return Err("rendered plan differs from the ranked 2K inspection".into());
    }
    validate_allocations(&render["execution"], selected.pass_count)?;
    let mut artifacts = BTreeMap::new();
    for channel in CHANNELS {
        let path = output.join("png").join(format!("{channel}.png"));
        super::pixels::Image::read(&path, SIZE, &acceptance.channels[channel].encoding)?;
        artifacts.insert(format!("png/{channel}.png"), files::digest(&path)?);
    }
    if snapshot_inputs(root, fixtures)? != inputs {
        return Err("source or fixture inputs changed during 2K trace".into());
    }
    for (directory, acceptance) in fixtures {
        if files::tree(&directory.join("expected"))? != baselines[&acceptance.material] {
            return Err("1K baseline changed during trace; discard the trace".into());
        }
    }
    let mut exact_render_arguments = args.clone();
    exact_render_arguments.extend(["--backend".to_owned(), backend.to_owned()]);
    if software == "1" {
        exact_render_arguments.push("--software".to_owned());
    }
    files::write_json(
        &output.join("trace.json"),
        &json!({
            "schemaVersion":1,"kind":"m3-naive-2k-allocation-trace","ok":true,"size":[SIZE,SIZE],"transientBudgetBytes":BUDGET,
            "policy":{"backend":backend,"software":software=="1","expectedAdapter":expected},"softwareSource":source,
            "ranking":ranking,"selected":{"material":selected.material,"case":selected.case,"overrides":selected.overrides},
            "requestedOutputs":CHANNELS,"planHash":selected.plan_hash,"execution":render["execution"],
            "inputs":inputs,"baselinesUnchanged":true,"artifacts":artifacts,
            "reproduce":{"command":"cargo xtask trace-2k","renderArguments":exact_render_arguments,"environment":{"MIXTURE_GPU_BACKEND":backend,"MIXTURE_GPU_SOFTWARE":software,"MIXTURE_GPU_EXPECT_ADAPTER":expected}},
            "allocationScope":"Successful texture/uniform/staging descriptors; driver/pipeline overhead and CPU PNG/RGBA buffers are excluded.",
            "lifetimeDecision":"Measured naive peak fits the documented budget. No last-consumer release, pooling or compatible texture reuse is justified by this workload.",
            "limits":"This fixed workload measurement is not a 2K golden or proof about every possible graph; GPU timings are CPU wall measurements around the recorded stages.",
            "remoteCi":"deferred; this local trace does not close remote acceptance"
        }),
    )?;
    Ok(())
}

fn snapshot_inputs(
    root: &Path,
    fixtures: &[(std::path::PathBuf, Acceptance)],
) -> TaskResult<files::Digests> {
    let mut inputs = BTreeMap::new();
    for (directory, _) in fixtures {
        merge_inputs(&mut inputs, files::inputs(root, directory)?)?;
    }
    Ok(inputs)
}

fn merge_inputs(inputs: &mut files::Digests, next: files::Digests) -> TaskResult {
    for (path, hash) in next {
        if inputs.get(&path).is_some_and(|before| before != &hash) {
            return Err(format!("source changed while taking input snapshot: {path}").into());
        }
        inputs.insert(path, hash);
    }
    Ok(())
}

fn arguments(command: &str, material: &str, overrides: &BTreeMap<String, Value>) -> Vec<String> {
    let mut args = vec![
        command.to_owned(),
        format!("fixtures/materials/{material}/material.mix"),
        "--size".to_owned(),
        SIZE.to_string(),
        "--output".to_owned(),
        CHANNELS.join(","),
        "--json".to_owned(),
    ];
    for (id, value) in overrides {
        args.extend(["--set".to_owned(), format!("{id}={value}")]);
    }
    args
}

fn rank(cases: &mut [RankedCase]) {
    cases.sort_by(|a, b| {
        b.estimates["peakBytes"]
            .as_u64()
            .cmp(&a.estimates["peakBytes"].as_u64())
            .then_with(|| b.pass_count.cmp(&a.pass_count))
            .then_with(|| a.material.cmp(&b.material))
            .then_with(|| (a.case != "default").cmp(&(b.case != "default")))
            .then_with(|| a.case.cmp(&b.case))
    });
}

fn number(value: &Value, field: &str) -> TaskResult<u64> {
    value[field]
        .as_u64()
        .ok_or_else(|| format!("missing integer allocation field: {field}").into())
}

fn validate_allocations(execution: &Value, passes: usize) -> TaskResult {
    let actual = &execution["allocations"];
    let estimate = &execution["estimates"];
    for field in [
        "textureBytes",
        "uniformBytes",
        "cumulativeBytes",
        "peakBytes",
    ] {
        if number(actual, field)? != number(estimate, field)? {
            return Err(
                format!("naive allocation count differs from plan estimate: {field}").into(),
            );
        }
    }
    if number(execution, "passCount")? != passes as u64
        || number(actual, "peakBytes")? > BUDGET
        || number(actual, "liveBytes")? != 0
        || number(actual, "reusedBytes")? != 0
        || number(actual, "releasedBytes")? != number(actual, "cumulativeBytes")?
        || number(actual, "textureCount")? != passes as u64
        || number(actual, "uniformCount")? != passes as u64
        || number(actual, "stagingCount")? != CHANNELS.len() as u64
        || number(actual, "stagingBytes")? != number(estimate, "cumulativeReadbackBytes")?
        || number(actual, "peakStagingBytes")? != number(estimate, "readbackBufferBytes")?
    {
        return Err("2K trace did not prove a released, bounded naive allocation schedule".into());
    }
    for field in ["pipelineMs", "executionMs", "readbackMs", "totalMs"] {
        if execution["timings"][field]
            .as_f64()
            .is_none_or(|n| !n.is_finite() || n < 0.0)
        {
            return Err("trace lacks finite stage timing evidence".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trace_ranks_actual_peak_before_material_name_or_pass_count() {
        let case = |id: &str, peak, passes| RankedCase {
            material: id.into(),
            case: "default".into(),
            overrides: BTreeMap::new(),
            plan_hash: "hash".into(),
            pass_count: passes,
            estimates: json!({"peakBytes":peak}),
        };
        let mut cases = vec![
            case("wood", 50, 9),
            case("ceramic", 70, 8),
            case("leather", 50, 5),
        ];
        rank(&mut cases);
        assert_eq!(
            cases
                .iter()
                .map(|c| c.material.as_str())
                .collect::<Vec<_>>(),
            ["ceramic", "wood", "leather"]
        );
    }
    #[test]
    fn trace_rejects_missing_counters_leaks_reuse_and_over_budget_measurements() {
        let valid = json!({"passCount":2,"allocations":{"textureCount":2,"textureBytes":16,"uniformCount":2,"uniformBytes":32,"stagingCount":4,"stagingBytes":32,"peakStagingBytes":8,"cumulativeBytes":80,"peakBytes":56,"liveBytes":0,"releasedBytes":80,"reusedBytes":0},"estimates":{"textureBytes":16,"uniformBytes":32,"cumulativeReadbackBytes":32,"readbackBufferBytes":8,"cumulativeBytes":80,"peakBytes":56},"timings":{"pipelineMs":0,"executionMs":1,"readbackMs":2,"totalMs":3}});
        assert!(validate_allocations(&valid, 2).is_ok());
        for (field, value) in [
            ("liveBytes", json!(1)),
            ("releasedBytes", json!(79)),
            ("reusedBytes", json!(8)),
            ("stagingCount", json!(3)),
            ("textureCount", json!(1)),
            ("peakBytes", Value::Null),
        ] {
            let mut bad = valid.clone();
            bad["allocations"][field] = value;
            assert!(validate_allocations(&bad, 2).is_err(), "{field}");
        }
        for count in [Value::Null, json!(0), json!(3)] {
            let mut bad = valid.clone();
            bad["passCount"] = count;
            assert!(validate_allocations(&bad, 2).is_err());
        }
        let mut bad = valid;
        bad["allocations"]["peakBytes"] = json!(BUDGET + 1);
        bad["estimates"]["peakBytes"] = json!(BUDGET + 1);
        assert!(validate_allocations(&bad, 2).is_err());
    }
    #[test]
    fn trace_rejects_conflicting_source_snapshots_and_invalidates_early_failure() {
        let mut inputs = BTreeMap::from([("shared.rs".to_owned(), "old".to_owned())]);
        assert!(
            merge_inputs(
                &mut inputs,
                BTreeMap::from([("shared.rs".to_owned(), "new".to_owned())])
            )
            .is_err()
        );
        assert_eq!(inputs["shared.rs"], "old");
        let temp = std::env::temp_dir().join(format!(
            "mixture-trace-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let output = temp.join("tmp/trace-2k");
        fs::create_dir_all(&output).unwrap();
        let latest = output.join("latest-hardware.json");
        files::write_json(&latest, &json!({"run":"previous","ok":true})).unwrap();
        assert!(run_with_policy(&temp, "metal", "0").is_err());
        let latest: Value = files::json(&latest).unwrap();
        assert_eq!(latest["ok"], false);
        assert_ne!(latest["run"], "previous");
        assert!(
            output
                .join(latest["run"].as_str().unwrap())
                .join("failure.json")
                .is_file()
        );
        let software_latest = output.join("latest-software.json");
        files::write_json(&software_latest, &json!({"run":"previous","ok":true})).unwrap();
        let error = run_with_policy(&temp, "auto", "1").unwrap_err();
        assert!(error.to_string().contains("requires explicit vulkan"));
        let software_latest: Value = files::json(&software_latest).unwrap();
        assert_eq!(software_latest["ok"], false);
        assert_ne!(software_latest["run"], "previous");
        let failure: Value = files::json(
            &output
                .join(software_latest["run"].as_str().unwrap())
                .join("failure.json"),
        )
        .unwrap();
        assert!(
            failure["message"]
                .as_str()
                .unwrap()
                .contains("requires explicit vulkan")
        );
        fs::remove_dir_all(temp).unwrap();
    }
}
