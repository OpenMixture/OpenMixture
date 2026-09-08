use serde_json::Value;
use std::process::Command;

#[test]
fn cpu_probe_runs_from_an_unrelated_directory_without_an_adapter() {
    let output = Command::new(env!("CARGO_BIN_EXE_mixture-native-consumer"))
        .arg("check")
        .current_dir(std::env::temp_dir())
        .env("MIXTURE_GPU_BACKEND", "none")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["gpuExecuted"], false);
    assert_eq!(
        report["invalidOverride"]["diagnostics"][0]["parameterId"],
        "cellsX"
    );
}

#[test]
fn disabled_backend_preserves_context_diagnostic_without_initializing_wgpu() {
    let output = Command::new(env!("CARGO_BIN_EXE_mixture-native-consumer"))
        .args(["gpu", "none", "hardware"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(
        report["diagnostics"][0]["code"],
        "MIX_GPU_ADAPTER_UNAVAILABLE"
    );
    assert_eq!(report["context"]["requested"]["backend"], "none");
    assert_eq!(
        report["context"]["requested"]["effectiveBackends"],
        serde_json::json!([])
    );
    assert!(report["context"]["adapter"].is_null());
    assert_eq!(report["context"]["computeProbe"], "notRun");
}

#[test]
fn missing_or_invalid_gpu_policy_is_a_usage_failure() {
    for args in [
        vec!["gpu"],
        vec!["gpu", "bogus", "hardware"],
        vec!["check", "extra"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture-native-consumer"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
}
