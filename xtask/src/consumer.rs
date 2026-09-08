//! Build and exercise the separate application through Cargo and its executable.
use crate::{TaskResult, cargo, gpu_smoke};
use serde_json::{Value, json};
use std::{env, fs, path::Path, process::Command};

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
    fs::write(
        directory.join("status.json"),
        serde_json::to_vec_pretty(&json!({
            "ok": true, "completed": true, "gpuExecuted": false,
            "report": "cpu.stdout.log", "packagedCratesValidated": false,
        }))?,
    )?;
    println!(
        "Independent consumer CPU checks passed; evidence: {}",
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
    fs::write(
        directory.join("status.json"),
        br#"{"ok":true,"completed":true}"#,
    )?;
    println!(
        "Independent consumer GPU checks passed; evidence: {}",
        directory.display()
    );
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

fn validate_cpu(report: &Value) -> TaskResult {
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

fn validate_gpu(report: &Value) -> TaskResult {
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
