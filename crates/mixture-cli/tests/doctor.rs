//! Process-level doctor contract tests; all paths avoid GPU initialization.

use serde_json::{Value, json};
use std::process::{Command, Output};

fn doctor(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mixture")).arg("doctor").args(args)
        // These must not override explicit CLI policy.
        .env("WGPU_BACKEND", "metal").env("WGPU_POWER_PREF", "low")
        .output().expect("doctor starts")
}

#[test]
fn doctor_json_failure_has_stable_envelope_and_exit_code() {
    let output = doctor(&[
        "--json",
        "--backend",
        "none",
        "--software",
        "--power-preference",
        "low-power",
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).expect("one JSON report");
    assert_eq!(report["schemaVersion"], 1);
    assert_eq!(report["ok"], false);
    assert_eq!(report["verdict"], "unhealthy");
    assert_eq!(report["requested"]["backend"], "none");
    assert_eq!(report["requested"]["softwareAdapter"], true);
    assert_eq!(report["requested"]["powerPreference"], "low-power");
    assert_eq!(report["requested"]["effectiveBackends"], json!([]));
    assert_eq!(report["requested"]["requiredFeatures"], json!([]));
    assert!(report["requested"]["requiredLimits"].is_object());
    assert_eq!(report["adapter"], Value::Null);
    assert_eq!(report["device"], Value::Null);
    assert_eq!(report["computeProbe"], "notRun");
    assert_eq!(report["readbackProbe"], "notRun");
    assert_eq!(
        report["diagnostics"],
        json!([{
            "code": "MIX_GPU_ADAPTER_UNAVAILABLE", "stage": "gpuAdapter", "severity": "error",
            "message": "No compiled GPU backend is permitted by the requested policy.",
            "suggestion": "Select auto or a native backend available on this platform."
        }])
    );
}

#[test]
fn doctor_human_failure_explains_policy_and_unverified_probes() {
    let output = doctor(&["--backend", "none"]);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Mixture doctor: unhealthy",
        "Compute probe: not run",
        "Readback probe: not run",
        "MIX_GPU_ADAPTER_UNAVAILABLE",
        "Suggestion:",
    ] {
        assert!(text.contains(expected), "missing {expected}");
    }
}

#[test]
fn doctor_help_is_available_without_gpu_access() {
    for option in ["--help", "-h"] {
        let output = doctor(&[option]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("Exit 0: context acquired, verdict unverified")
        );
    }
}

#[test]
fn doctor_rejects_invalid_options_before_acquisition() {
    for args in [
        vec!["--backend"],
        vec!["--backend", "bogus"],
        vec!["--power-preference"],
        vec!["--power-preference", "bogus"],
        vec!["--json", "--json"],
        vec!["--software", "--software"],
        vec!["--backend", "none", "--backend", "auto"],
        vec!["--unknown"],
        vec!["--json", "--unknown"],
        vec!["--help", "--json"],
        vec!["material.mix"],
    ] {
        let output = doctor(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: mixture doctor"));
    }
}
