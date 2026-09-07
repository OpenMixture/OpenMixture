//! Render command validation and unavailable-GPU behavior require no GPU.

use serde_json::Value;
use std::{path::PathBuf, process::Command};

#[test]
fn checker_cli_invalid_invocation_never_creates_output() {
    for args in [
        vec![],
        vec!["unknown"],
        vec!["checker"],
        vec!["checker", "--size", "x", "--out", "unused.png"],
        vec!["checker", "--out"],
        vec!["checker", "--out", "unused.png", "--json", "--json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .arg("render-builtin")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn checker_cli_validates_before_gpu_and_reports_operational_failure_separately() {
    let path: PathBuf = std::env::temp_dir().join(format!(
        "mixture-checker-must-not-exist-{}.png",
        std::process::id()
    ));
    assert!(!path.exists());
    for (size, exit, code, stage) in [
        ("0", 2, "MIX_PARAMETER_INVALID_VALUE", "validation"),
        (
            "2049",
            2,
            "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED",
            "validation",
        ),
        ("64", 1, "MIX_GPU_ADAPTER_UNAVAILABLE", "gpuAdapter"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .args([
                "render-builtin",
                "checker",
                "--size",
                size,
                "--backend",
                "none",
                "--json",
                "--out",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(exit));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["ok"], false);
        assert_eq!(report["diagnostics"][0]["code"], code);
        assert_eq!(report["diagnostics"][0]["stage"], stage);
        assert!(report["execution"].is_null());
        assert!(report["writtenBytes"].is_null());
        if exit == 2 {
            assert!(report["context"].is_null());
        }
        assert!(!path.exists());
    }
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn checker_gpu_cli_png_round_trip_skip_mode_and_output_failure() {
    fn command() -> Command {
        Command::new(env!("CARGO_BIN_EXE_mixture"))
    }
    fn gpu_args(command: &mut Command) {
        command.args([
            "--backend",
            &std::env::var("MIXTURE_GPU_BACKEND").unwrap_or_else(|_| "auto".into()),
        ]);
        if std::env::var("MIXTURE_GPU_SOFTWARE").is_ok_and(|value| value == "1") {
            command.arg("--software");
        }
    }
    let path =
        std::env::temp_dir().join(format!("mixture-checker-output-{}.png", std::process::id()));
    let mut render = command();
    render
        .args(["render-builtin", "checker", "--size", "64", "--out"])
        .arg(&path)
        .arg("--json");
    gpu_args(&mut render);
    let output = render.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["execution"]["passCount"], 1);
    let png = std::fs::read(&path).unwrap();
    assert_eq!(report["writtenBytes"].as_u64(), Some(png.len() as u64));
    let mut reader = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!((info.width, info.height), (64, 64));
    assert_eq!(
        pixels,
        include_bytes!("../../../fixtures/nodes/checker/checker-64.rgba")
    );
    std::fs::remove_file(&path).unwrap();

    let mut skipped = command();
    skipped.args(["doctor", "--skip-probe", "--json"]);
    gpu_args(&mut skipped);
    let output = skipped.output().unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["verdict"], "unverified");
    assert_eq!(report["computeProbe"], "notRun");
    assert!(report["execution"].is_null());

    let mut failed = command();
    failed
        .args(["render-builtin", "checker", "--out"])
        .arg(std::env::temp_dir())
        .arg("--json");
    gpu_args(&mut failed);
    let output = failed.output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(report["diagnostics"][0]["code"], "MIX_ENCODING_FAILED");
    assert_eq!(report["diagnostics"][0]["stage"], "encoding");
    assert_eq!(report["execution"]["readbackBytes"], 32768);
    assert!(report["writtenBytes"].is_null());
}
