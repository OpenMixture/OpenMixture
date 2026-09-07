//! Explicit GPU checks; never included in the ordinary `check` or `test` path.

use crate::{TaskResult, cargo, run_cargo};
use serde_json::Value;
use std::{env, fs, path::Path};

pub(super) fn run(root: &Path) -> TaskResult {
    let backend = env::var("MIXTURE_GPU_BACKEND").unwrap_or_else(|_| "auto".into());
    let software = env::var("MIXTURE_GPU_SOFTWARE").unwrap_or_else(|_| "0".into());
    if !matches!(backend.as_str(), "auto" | "vulkan" | "metal" | "dx12")
        || !matches!(software.as_str(), "0" | "1")
    {
        return Err("Invalid smoke policy: MIXTURE_GPU_BACKEND=auto|vulkan|metal|dx12, MIXTURE_GPU_SOFTWARE=0|1".into());
    }
    let directory = root.join("tmp/gpu-smoke");
    fs::create_dir_all(&directory)?;
    let mut command = cargo(root);
    command.args([
        "run",
        "--locked",
        "-p",
        "mixture-cli",
        "--",
        "doctor",
        "--json",
        "--backend",
        &backend,
    ]);
    if software == "1" {
        command.arg("--software");
    }
    println!("Acquiring doctor context (backend={backend}, software={software})");
    let output = command.output()?;
    fs::write(directory.join("doctor.json"), &output.stdout)?;
    fs::write(directory.join("doctor.stderr.log"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!(
            "doctor failed ({}); inspect {}",
            output.status,
            directory.display()
        )
        .into());
    }
    let report: Value = serde_json::from_slice(&output.stdout)?;
    validate_report(
        &report,
        &backend,
        software == "1",
        env::var("MIXTURE_GPU_EXPECT_ADAPTER").ok().as_deref(),
    )?;
    run_cargo(
        root,
        &[
            "test",
            "--locked",
            "-p",
            "mixture-wgpu",
            "--test",
            "context",
            "context_gpu_smoke_owns_independent_contexts",
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ],
        false,
    )?;
    println!(
        "GPU acquisition smoke passed: {} ({}, {}); verdict unverified. Evidence: {}",
        report["adapter"]["name"],
        report["adapter"]["deviceType"],
        report["adapter"]["backend"],
        directory.display()
    );
    Ok(())
}

fn validate_report(
    report: &Value,
    backend: &str,
    software: bool,
    expected_adapter: Option<&str>,
) -> TaskResult {
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["verdict"] != "unverified"
        || report["computeProbe"] != "notRun"
        || report["readbackProbe"] != "notRun"
        || report["diagnostics"] != serde_json::json!([])
        || !report["device"].is_object()
        || !report["adapter"]["supportedLimits"].is_object()
        || report["requested"]["backend"] != backend
        || report["requested"]["softwareAdapter"] != software
    {
        return Err("doctor returned an invalid acquisition report".into());
    }
    let actual_backend = report["adapter"]["backend"]
        .as_str()
        .ok_or("missing adapter backend")?;
    if !matches!(actual_backend, "Vulkan" | "Metal" | "Dx12")
        || (backend != "auto" && !actual_backend.eq_ignore_ascii_case(backend))
    {
        return Err(format!("unexpected adapter backend: {actual_backend}").into());
    }
    if software && report["adapter"]["deviceType"] != "Cpu" {
        return Err("software policy did not select a CPU adapter".into());
    }
    let name = report["adapter"]["name"]
        .as_str()
        .filter(|name| !name.is_empty())
        .ok_or("missing adapter name")?;
    if let Some(expected) = expected_adapter
        && !name.contains(expected)
    {
        return Err(format!("expected adapter containing {expected:?}, selected {name:?}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_smoke_rejects_false_health_and_wrong_adapter_evidence() {
        let valid = serde_json::json!({
            "schemaVersion": 1, "ok": true, "verdict": "unverified", "diagnostics": [],
            "computeProbe": "notRun", "readbackProbe": "notRun",
            "requested": { "backend": "vulkan", "softwareAdapter": true },
            "adapter": { "name": "SwiftShader Device", "backend": "Vulkan", "deviceType": "Cpu", "supportedLimits": {} }, "device": {}
        });
        assert!(validate_report(&valid, "vulkan", true, Some("SwiftShader")).is_ok());
        for (field, value) in [
            ("verdict", serde_json::json!("healthy")),
            ("ok", serde_json::json!(false)),
            ("device", Value::Null),
        ] {
            let mut invalid = valid.clone();
            invalid[field] = value;
            assert!(validate_report(&invalid, "vulkan", true, None).is_err());
        }
        assert!(validate_report(&valid, "metal", true, None).is_err());
        assert!(validate_report(&valid, "vulkan", true, Some("different adapter")).is_err());
        let mut invalid = valid.clone();
        invalid["adapter"]["backend"] = serde_json::json!("Noop");
        assert!(validate_report(&invalid, "vulkan", true, None).is_err());
    }
}
