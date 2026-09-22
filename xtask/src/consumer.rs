//! Build and exercise the separate application through Cargo and its executable.
use crate::{TaskResult, cargo, gpu_smoke};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const MANIFEST: &str = "examples/native-consumer/Cargo.toml";

fn consumer_cargo(root: &Path, action: &str) -> Command {
    let mut command = cargo(root);
    command
        .arg(action)
        .arg("--manifest-path")
        .arg(root.join(MANIFEST));
    command
}

fn compile_command(root: &Path, action: &str) -> Command {
    let mut command = consumer_cargo(root, action);
    command
        .args(["--locked", "--all-features"])
        .arg("--target-dir")
        .arg(root.join("target/native-consumer"));
    command
}

fn captured(command: &mut Command, directory: &Path, name: &str) -> TaskResult<Vec<u8>> {
    let output = command.output()?;
    fs::write(directory.join(format!("{name}.stdout.log")), &output.stdout)?;
    fs::write(directory.join(format!("{name}.stderr.log")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!(
            "native consumer {name} failed ({}); inspect {}",
            output.status,
            directory.display()
        )
        .into());
    }
    Ok(output.stdout)
}

fn binary(root: &Path) -> Command {
    let mut command = Command::new(root.join(format!(
        "target/native-consumer/debug/mixture-native-consumer{}",
        env::consts::EXE_SUFFIX
    )));
    // The application must not depend on running from the producing repository.
    command.current_dir(env::temp_dir());
    command
}

pub(super) fn check(root: &Path) -> TaskResult {
    let directory = root.join("tmp/consumer-check");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("status.json"),
        br#"{"ok":false,"completed":false}"#,
    )?;
    let metadata = captured(
        consumer_cargo(root, "metadata").args(["--locked", "--format-version", "1", "--no-deps"]),
        &directory,
        "metadata",
    )?;
    validate_workspace(&serde_json::from_slice(&metadata)?)?;
    captured(
        consumer_cargo(root, "fmt").args(["--", "--check"]),
        &directory,
        "fmt",
    )?;
    captured(
        compile_command(root, "clippy").args(["--all-targets", "--", "-D", "warnings"]),
        &directory,
        "clippy",
    )?;
    captured(&mut compile_command(root, "test"), &directory, "tests")?;
    captured(&mut compile_command(root, "build"), &directory, "build")?;
    let report = captured(binary(root).arg("check"), &directory, "cpu")?;
    let report: Value = serde_json::from_slice(&report)?;
    validate_cpu(&report)?;
    let cli_evidence = cli_contract(root, &directory, None)?;
    fs::write(
        directory.join("status.json"),
        serde_json::to_vec_pretty(&json!({
            "ok": true, "completed": true, "gpuExecuted": false,
            "report": "cpu.stdout.log", "packagedCratesValidated": false,
            "cliEvidence": cli_evidence,
        }))?,
    )?;
    println!(
        "Independent Rust and CLI consumer CPU checks passed; evidence: {}",
        directory.display()
    );
    Ok(())
}

pub(super) fn gpu(
    root: &Path,
    directory: &Path,
    backend: &str,
    software: bool,
    expected_adapter: Option<&str>,
) -> TaskResult {
    let directory = directory.join("native-consumer");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("status.json"),
        br#"{"ok":false,"completed":false}"#,
    )?;
    captured(
        compile_command(root, "test")
            .args(["--test", "brick_pattern", "--", "--ignored", "--nocapture"])
            .env("MIXTURE_GPU_BACKEND", backend)
            .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" })
            .env(
                "MIXTURE_BRICK_EVIDENCE",
                directory.join("brick-pattern.json"),
            ),
        &directory,
        "brick-pattern",
    )?;
    captured(
        compile_command(root, "test")
            .args(["--test", "scalar_blend", "--", "--ignored", "--nocapture"])
            .env("MIXTURE_GPU_BACKEND", backend)
            .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" })
            .env(
                "MIXTURE_SCALAR_EVIDENCE",
                directory.join("scalar-blend.json"),
            ),
        &directory,
        "scalar-blend",
    )?;
    captured(
        compile_command(root, "test")
            .args([
                "--test",
                "resources",
                "--test",
                "asset_qualification",
                "--",
                "--ignored",
                "--nocapture",
            ])
            .env(
                "MIXTURE_ASSET_EVIDENCE",
                directory.join("asset-qualification.json"),
            )
            .env("MIXTURE_GPU_BACKEND", backend)
            .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" }),
        &directory,
        "resources",
    )?;
    captured(&mut compile_command(root, "build"), &directory, "build")?;
    let mut command = binary(root);
    command.args([
        "gpu",
        backend,
        if software { "software" } else { "hardware" },
    ]);
    if let Some(expected) = expected_adapter {
        command.arg(expected);
    }
    let report = captured(&mut command, &directory, "gpu")?;
    let report: Value = serde_json::from_slice(&report)?;
    validate_gpu(&report)?;
    gpu_smoke::validate_adapter(&report["context"], backend, software, expected_adapter)?;
    let cli_evidence = cli_contract(
        root,
        &directory,
        Some((backend, software, expected_adapter)),
    )?;
    let loss_evidence =
        device_loss_contract(root, &directory, backend, software, expected_adapter)?;
    let latest_evidence = latest_contract(root, &directory, backend, software, expected_adapter)?;
    fs::write(
        directory.join("status.json"),
        serde_json::to_vec_pretty(
            &json!({"ok":true,"completed":true,"cliEvidence":cli_evidence,"deviceLossEvidence":loss_evidence,"latestEvidence":latest_evidence}),
        )?,
    )?;
    println!(
        "Independent Rust and CLI consumer GPU checks passed; evidence: {}",
        directory.display()
    );
    Ok(())
}

fn latest_contract(
    root: &Path,
    directory: &Path,
    backend: &str,
    software: bool,
    expected: Option<&str>,
) -> TaskResult<String> {
    let mut command = binary(root);
    command.args([
        "latest",
        backend,
        if software { "software" } else { "hardware" },
    ]);
    if let Some(expected) = expected {
        command.arg(expected);
    }
    let report: Value = serde_json::from_slice(&captured(&mut command, directory, "latest")?)?;
    validate_latest(&report)?;
    gpu_smoke::validate_adapter(&report["context"], backend, software, expected)?;
    let name = format!(
        "latest-cli-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let mut command = compile_command(root, "test");
    command
        .args([
            "--test",
            "latest_cli",
            "latest_cli_contract",
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ])
        .env("MIXTURE_CONSUMER_LATEST_DIR", directory.join(&name))
        .env(
            "MIXTURE_CONSUMER_CLI",
            root.join(format!("target/debug/mixture{}", env::consts::EXE_SUFFIX)),
        )
        .env("MIXTURE_GPU_BACKEND", backend)
        .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" });
    if let Some(expected) = expected {
        command.env("MIXTURE_GPU_EXPECT_ADAPTER", expected);
    } else {
        command.env_remove("MIXTURE_GPU_EXPECT_ADAPTER");
    }
    captured(&mut command, directory, "latest-cli-tests")?;
    let receipt: Value =
        serde_json::from_slice(&fs::read(directory.join(&name).join("status.json"))?)?;
    validate_latest_cli(&receipt)?;
    for index in [0, 1, 3, 4] {
        gpu_smoke::validate_adapter(
            &receipt["cases"][index]["report"]["context"],
            backend,
            software,
            expected,
        )?;
    }
    Ok(format!("{name}/status.json"))
}
fn validate_latest(report: &Value) -> TaskResult {
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["completed"] != true
        || report["gpuExecuted"] != true
        || report["mode"] != "latest"
        || report["startedGenerations"] != json!([1, 2, 4, 5])
        || report["publishedGenerations"] != json!([1, 5])
        || report["displayedGeneration"] != 5
        || report["displayedCurrent"] != true
        || report["oldDisplayWasStaleAfterFailure"] != true
        || report["maximumCoexistingOutputs"] != 2
        || report["retainedOutputBytes"] != 3120
        || report["failedRequest"]["diagnostics"][0]["code"] != "MIX_PARAMETER_INVALID_VALUE"
        || report["pixelsCheckedAfterRendererDrop"] != true
        || report["executions"].as_array().is_none_or(|rows| {
            rows.len() != 3
                || rows.iter().any(|r| {
                    r["execution"]["allocations"]["liveBytes"] != 0
                        || r["execution"]["passCount"] != 4
                })
        })
    {
        return Err("latest consumer omitted bounded freshness/ownership evidence".into());
    }
    Ok(())
}
fn validate_latest_cli(report: &Value) -> TaskResult {
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["completed"] != true
        || report["gpuExecuted"] != true
        || report["publishedGenerations"] != json!([1, 5])
        || report["displayedGeneration"] != 5
        || report["obsoleteDirectoriesRemoved"] != json!([1, 2, 3, 4])
        || report["retainedDirectories"] != json!(["generation-5"])
        || report["oldDisplayWasStaleAfterFailure"] != true
        || report["unrelatedPreserved"] != true
        || report["cases"].as_array().is_none_or(|rows| {
            rows.len() != 5
                || rows
                    .iter()
                    .zip([0, 0, 2, 1, 0])
                    .any(|(r, exit)| r["exitCode"] != exit)
        })
    {
        return Err("latest CLI consumer omitted fresh completion or cleanup evidence".into());
    }
    Ok(())
}

fn device_loss_contract(
    root: &Path,
    directory: &Path,
    backend: &str,
    software: bool,
    expected_adapter: Option<&str>,
) -> TaskResult<String> {
    let name = format!(
        "device-loss-{}-{}.json",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let mut command = compile_command(root, "test");
    command
        .args([
            "--test",
            "device_loss",
            "device_loss_contract",
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ])
        .env(
            "MIXTURE_CONSUMER_DEVICE_LOSS_EVIDENCE",
            directory.join(&name),
        )
        .env("MIXTURE_GPU_BACKEND", backend)
        .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" });
    if let Some(expected) = expected_adapter {
        command.env("MIXTURE_GPU_EXPECT_ADAPTER", expected);
    } else {
        command.env_remove("MIXTURE_GPU_EXPECT_ADAPTER");
    }
    captured(&mut command, directory, "device-loss-tests")?;
    let report: Value = serde_json::from_slice(&fs::read(directory.join(&name))?)?;
    validate_device_loss(&report)?;
    gpu_smoke::validate_adapter(&report["context"], backend, software, expected_adapter)?;
    Ok(name)
}

fn validate_device_loss(report: &Value) -> TaskResult {
    let cases = report["cases"]
        .as_array()
        .ok_or("device-loss consumer omitted its cases")?;
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["completed"] != true
        || report["gpuExecuted"] != true
        || cases.len() != 2
        || cases.iter().enumerate().any(|(index, case)| {
            case["warmCache"] != (index == 1)
                || case["cacheBeforeLoss"]
                    .as_u64()
                    .is_none_or(|count| (count > 0) != (index == 1))
                || case["cacheAfterLoss"] != 0
                || case["repeatedFailures"] != 2
                || case["reason"] != "deviceLost"
                || case["diagnostic"]["code"] != "MIX_GPU_DEVICE_LOST"
                || case["diagnostic"]["stage"] != "gpuExecution"
                || case["deviceLoss"]["reason"] != "destroyed"
                || case["allocations"]["cumulativeBytes"] != 0
                || case["allocations"]["liveBytes"] != 0
                || case["allocations"]["releasedBytes"] != 0
                || case["independentExecution"]["passCount"]
                    .as_u64()
                    .is_none_or(|count| count == 0)
                || case["independentExecution"]["planHash"].as_str().is_none()
                || case["independentExecution"]["planHash"]
                    != case["diagnostic"]["evidence"]["planHash"]
                || case["independentExecution"]["allocations"]["liveBytes"] != 0
        })
    {
        return Err(
            "device-loss consumer did not verify cold/warm loss, cleanup and independent execution"
                .into(),
        );
    }
    Ok(())
}

fn cli_contract(
    root: &Path,
    directory: &Path,
    gpu: Option<(&str, bool, Option<&str>)>,
) -> TaskResult<String> {
    captured(
        cargo(root)
            .args(["build", "--locked", "--all-features", "-p", "mixture-cli"])
            .arg("--target-dir")
            .arg(root.join("target")),
        directory,
        "cli-build",
    )?;
    let name = format!(
        "cli-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let evidence = directory.join(&name);
    let mut command = compile_command(root, "test");
    command
        .args([
            "--test",
            "cli_contract",
            if gpu.is_some() {
                "cli_contract_gpu"
            } else {
                "cli_contract_cpu"
            },
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ])
        .env(
            "MIXTURE_CONSUMER_CLI",
            root.join(format!("target/debug/mixture{}", env::consts::EXE_SUFFIX)),
        )
        .env("MIXTURE_CONSUMER_EVIDENCE_DIR", &evidence);
    if let Some((backend, software, expected)) = gpu {
        command
            .env("MIXTURE_GPU_BACKEND", backend)
            .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" });
        if let Some(expected) = expected {
            command.env("MIXTURE_GPU_EXPECT_ADAPTER", expected);
        } else {
            command.env_remove("MIXTURE_GPU_EXPECT_ADAPTER");
        }
    }
    captured(&mut command, directory, "cli-tests")?;
    let status: Value = serde_json::from_slice(&fs::read(evidence.join("status.json"))?)?;
    validate_cli_status(&status, gpu.is_some())?;
    if let Some((backend, software, expected)) = gpu {
        let doctor: Value = serde_json::from_slice(&fs::read(evidence.join("doctor.stdout"))?)?;
        gpu_smoke::validate_report(&doctor, backend, software, expected)?;
    }
    Ok(format!("{name}/status.json"))
}

pub(super) fn validate_cli_status(status: &Value, gpu: bool) -> TaskResult {
    let required: &[&str] = if gpu {
        &[
            "asset-render",
            "doctor",
            "doctor-skipped",
            "render-full",
            "render-override",
            "render-sliced",
            "render-partial",
            "render-partial-human",
        ]
    } else {
        &[
            "asset-pack",
            "asset-plan",
            "asset-bad-before-gpu",
            "validate-valid",
            "inspect-valid",
            "validate-missing-warp.mix-json",
            "inspect-missing-warp-human",
            "render-missing-warp-human",
            "render-invalid-override",
            "render-budget",
            "doctor-none",
            "render-none",
            "inspect-usage",
            "render-usage",
        ]
    };
    let cases = status["cases"]
        .as_array()
        .ok_or("CLI consumer omitted its invocations")?;
    if status["schemaVersion"] != 1
        || status["ok"] != true
        || status["completed"] != true
        || status["mode"]
            != if gpu {
                "cli-contract-gpu"
            } else {
                "cli-contract-cpu"
            }
        || status["gpuExecuted"] != gpu
        || required
            .iter()
            .any(|id| !cases.iter().any(|case| case["id"] == *id))
        || cases.iter().any(|case| {
            case["exitCode"].as_i64().is_none() || case["exitCode"] != case["expectedExitCode"]
        })
    {
        return Err(
            "CLI consumer tests did not complete the required report/exit/output cases".into(),
        );
    }
    Ok(())
}

fn validate_workspace(metadata: &Value) -> TaskResult {
    let members = metadata["workspace_members"]
        .as_array()
        .ok_or("consumer metadata omitted members")?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("consumer metadata omitted packages")?;
    let Some(package) = packages
        .iter()
        .find(|package| members.contains(&package["id"]))
    else {
        return Err("consumer is not its own workspace member".into());
    };
    if members.len() != 1 || package["name"] != "mixture-native-consumer" {
        return Err("consumer must remain a separate one-package workspace".into());
    }
    if !package["publish"].as_array().is_some_and(Vec::is_empty) {
        return Err("consumer fixture must not be publishable".into());
    }
    Ok(())
}

pub(super) fn validate_cpu(report: &Value) -> TaskResult {
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["mode"] != "check"
        || report["gpuExecuted"] != false
        || report["invalidOverride"]["ok"] != false
        || report["invalidOverride"]["diagnostics"][0]["parameterId"] != "cellsX"
        || report["outputs"]
            .as_array()
            .is_none_or(|outputs| outputs.len() != 4)
    {
        return Err("consumer returned no complete CPU/diagnostic evidence".into());
    }
    Ok(())
}

pub(super) fn validate_gpu(report: &Value) -> TaskResult {
    validate_cpu(&report["cpu"])?;
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["mode"] != "gpu"
        || report["gpuExecuted"] != true
        || report["pixelsCheckedAfterRendererDrop"] != true
        || report["literalPixelsPassed"] != true
        || report["overrideChangedPixels"] != true
        || report["original"]["execution"]["planHash"] != report["cpu"]["planHash"]
        || report["changedOverride"]["execution"]["planHash"] == report["cpu"]["planHash"]
    {
        return Err("consumer returned no completed public GPU/ownership evidence".into());
    }
    for label in ["original", "changedOverride"] {
        let execution = &report[label]["execution"];
        if execution["passCount"] != 4
            || execution["size"] != json!([65, 3])
            || execution["allocations"]["liveBytes"] != 0
            || report[label]["channels"]
                .as_array()
                .is_none_or(|channels| channels.len() != 4)
        {
            return Err("consumer GPU result is incomplete or retains per-call resources".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_receipts_reject_skipped_wrong_generation_or_leaked_outputs() {
        let good = json!({"schemaVersion":1,"ok":true,"completed":true,"gpuExecuted":true,"mode":"latest",
            "startedGenerations":[1,2,4,5],"publishedGenerations":[1,5],"displayedGeneration":5,"displayedCurrent":true,
            "oldDisplayWasStaleAfterFailure":true,"maximumCoexistingOutputs":2,"retainedOutputBytes":3120,
            "failedRequest":{"diagnostics":[{"code":"MIX_PARAMETER_INVALID_VALUE"}]},"pixelsCheckedAfterRendererDrop":true,
            "executions":vec![json!({"execution":{"allocations":{"liveBytes":0},"passCount":4}});3]});
        assert!(validate_latest(&good).is_ok());
        for (pointer, value) in [
            ("/completed", json!(false)),
            ("/publishedGenerations", json!([1, 2, 5])),
            ("/displayedGeneration", json!(1)),
            ("/oldDisplayWasStaleAfterFailure", json!(false)),
            ("/maximumCoexistingOutputs", json!(3)),
            ("/executions/1/execution/allocations/liveBytes", json!(16)),
        ] {
            let mut bad = good.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(validate_latest(&bad).is_err(), "accepted {pointer}");
        }
        let good = json!({"schemaVersion":1,"ok":true,"completed":true,"gpuExecuted":true,
            "publishedGenerations":[1,5],"displayedGeneration":5,"obsoleteDirectoriesRemoved":[1,2,3,4],
            "retainedDirectories":["generation-5"],"oldDisplayWasStaleAfterFailure":true,"unrelatedPreserved":true,
            "cases":([0,0,2,1,0].map(|exit|json!({"exitCode":exit})))});
        assert!(validate_latest_cli(&good).is_ok());
        for (pointer, value) in [
            ("/completed", json!(false)),
            ("/cases", json!([])),
            (
                "/retainedDirectories",
                json!(["generation-1", "generation-5"]),
            ),
            ("/cases/3/exitCode", json!(0)),
        ] {
            let mut bad = good.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(validate_latest_cli(&bad).is_err(), "accepted {pointer}");
        }
    }

    #[test]
    fn device_loss_receipt_rejects_skipped_or_incomplete_lifecycle_checks() {
        let case = |warm: bool| {
            json!({
                "warmCache":warm,"cacheBeforeLoss":u64::from(warm),"cacheAfterLoss":0,
                "repeatedFailures":2,"reason":"deviceLost","deviceLoss":{"reason":"destroyed"},
                "diagnostic":{"code":"MIX_GPU_DEVICE_LOST","stage":"gpuExecution","evidence":{"planHash":"test-plan"}},
                "allocations":{"cumulativeBytes":0,"releasedBytes":0,"liveBytes":0},
                "independentExecution":{"planHash":"test-plan","passCount":1,"allocations":{"liveBytes":0}},
            })
        };
        let good = json!({"schemaVersion":1,"ok":true,"completed":true,"gpuExecuted":true,"cases":[case(false),case(true)]});
        assert!(validate_device_loss(&good).is_ok());
        for (field, value) in [
            ("ok", json!(false)),
            ("completed", json!(false)),
            ("gpuExecuted", json!(false)),
            ("cases", json!([])),
        ] {
            let mut bad = good.clone();
            bad[field] = value;
            assert!(validate_device_loss(&bad).is_err(), "accepted {field}");
        }
        for pointer in [
            "/cases/0/repeatedFailures",
            "/cases/1/warmCache",
            "/cases/1/cacheBeforeLoss",
            "/cases/1/reason",
            "/cases/1/diagnostic/code",
            "/cases/1/deviceLoss/reason",
            "/cases/1/allocations/liveBytes",
            "/cases/1/independentExecution/planHash",
            "/cases/1/independentExecution/passCount",
        ] {
            let mut bad = good.clone();
            *bad.pointer_mut(pointer).unwrap() = Value::Null;
            assert!(validate_device_loss(&bad).is_err(), "accepted {pointer}");
        }
    }

    #[test]
    fn cli_contract_receipt_rejects_skipped_tests_partial_cases_and_wrong_exits() {
        let cases: Vec<_> = [
            ("asset-render", 0),
            ("doctor", 0),
            ("doctor-skipped", 0),
            ("render-full", 0),
            ("render-override", 0),
            ("render-sliced", 0),
            ("render-partial", 1),
            ("render-partial-human", 1),
        ]
        .into_iter()
        .map(|(id, exit)| json!({"id":id,"exitCode":exit,"expectedExitCode":exit}))
        .collect();
        let valid = json!({"schemaVersion":1,"ok":true,"completed":true,
            "mode":"cli-contract-gpu","gpuExecuted":true,"cases":cases});
        assert!(validate_cli_status(&valid, true).is_ok());
        let mut skipped = valid.clone();
        skipped["cases"] = json!([]);
        assert!(validate_cli_status(&skipped, true).is_err());
        let mut incomplete = valid.clone();
        incomplete["cases"].as_array_mut().unwrap().pop();
        assert!(validate_cli_status(&incomplete, true).is_err());
        let mut wrong_exit = valid.clone();
        wrong_exit["cases"][6]["exitCode"] = json!(0);
        assert!(validate_cli_status(&wrong_exit, true).is_err());
        let mut unexecuted = valid;
        unexecuted["gpuExecuted"] = json!(false);
        assert!(validate_cli_status(&unexecuted, true).is_err());
        assert!(validate_cli_status(&unexecuted, false).is_err());
    }

    #[test]
    fn consumer_build_commands_lock_the_detached_manifest_and_target() {
        let root = Path::new("repository");
        for action in ["build", "test", "clippy"] {
            let command = compile_command(root, action);
            let args: Vec<_> = command
                .get_args()
                .map(|arg| arg.to_string_lossy())
                .collect();
            assert_eq!(args[0], action);
            assert!(args.iter().any(|arg| arg == "--locked"));
            assert!(
                args.iter()
                    .any(|arg| arg.ends_with("native-consumer/Cargo.toml")
                        || arg.ends_with("native-consumer\\Cargo.toml"))
            );
            assert!(args.iter().any(|arg| arg == "--all-features"));
            assert!(!args.iter().any(|arg| arg == "--workspace"));
            assert_eq!(command.get_current_dir(), Some(root));
        }
    }

    #[test]
    fn consumer_evidence_rejects_skipped_gpu_and_missing_ownership() {
        let cpu = json!({
            "schemaVersion": 1, "ok": true, "mode": "check", "gpuExecuted": false,
            "planHash": "first", "outputs": [{},{},{},{}],
            "invalidOverride": {"ok": false, "diagnostics": [{"parameterId": "cellsX"}]},
        });
        let output = json!({
            "execution": {"planHash": "first", "passCount": 4, "size": [65,3], "allocations": {"liveBytes": 0}},
            "channels": [{},{},{},{}],
        });
        let mut valid = json!({
            "schemaVersion": 1, "ok": true, "mode": "gpu", "gpuExecuted": true,
            "pixelsCheckedAfterRendererDrop": true, "literalPixelsPassed": true,
            "overrideChangedPixels": true, "cpu": cpu, "original": output, "changedOverride": output,
        });
        valid["changedOverride"]["execution"]["planHash"] = json!("changed");
        assert!(validate_gpu(&valid).is_ok());
        for field in [
            "gpuExecuted",
            "pixelsCheckedAfterRendererDrop",
            "literalPixelsPassed",
            "overrideChangedPixels",
        ] {
            let mut invalid = valid.clone();
            invalid[field] = json!(false);
            assert!(validate_gpu(&invalid).is_err());
        }
        let mut invalid = valid.clone();
        invalid["original"]["execution"]["allocations"]["liveBytes"] = json!(8);
        assert!(validate_gpu(&invalid).is_err());
        invalid = valid;
        invalid["cpu"]["gpuExecuted"] = json!(true);
        assert!(validate_cpu(&invalid["cpu"]).is_err());
    }
}
