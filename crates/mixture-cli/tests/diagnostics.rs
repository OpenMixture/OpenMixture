//! The same public diagnostic must retain its context in every human command.
use std::{path::Path, process::Command};

fn missing_warp(command: &str, json: bool) -> std::process::Output {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/nodes/warp/missing-input.mix");
    let mut process = Command::new(env!("CARGO_BIN_EXE_mixture"));
    process.arg(command).arg(source);
    match command {
        "inspect" => {
            process.arg("--plan");
        }
        "render" => {
            process.args(["--out", "unused-output", "--backend", "none"]);
        }
        _ => {}
    }
    if json {
        process.arg("--json");
    }
    process.output().unwrap()
}

fn human_context(command: &str) {
    let output = missing_warp(command, false);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "MIX_PORT_REQUIRED_CONNECTION",
        "node: sample",
        "port: displacement",
        "document:",
        "missing-input.mix",
        "stage: Validation",
        "Suggestion:",
    ] {
        assert!(
            text.contains(expected),
            "{command} omitted {expected}: {text}"
        );
    }
}

#[test]
fn validate_human_retains_complete_context() {
    human_context("validate");
}

#[test]
fn inspect_human_retains_complete_context() {
    human_context("inspect");
}

#[test]
fn render_human_retains_complete_context() {
    human_context("render");
}

#[test]
fn missing_warp_json_context_and_exit_are_identical_across_commands() {
    let mut previous = None;
    for command in ["validate", "inspect", "render"] {
        let output = missing_warp(command, true);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stderr.is_empty());
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let diagnostics = &report["diagnostics"];
        assert_eq!(report["ok"], false);
        assert_eq!(diagnostics[0]["nodeId"], "sample");
        assert_eq!(diagnostics[0]["portId"], "displacement");
        assert_eq!(diagnostics[0]["stage"], "validation");
        if let Some(previous) = previous {
            assert_eq!(diagnostics, &previous);
        }
        previous = Some(diagnostics.clone());
        if command == "render" {
            assert!(report["context"].is_null());
            assert!(report["execution"].is_null());
        }
    }
}

#[test]
fn render_human_keeps_invalid_override_parameter_and_measured_evidence() {
    let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
        .arg("render")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/checker.mix"))
        .args([
            "--set",
            "frequency=0",
            "--out",
            "unused-output",
            "--backend",
            "none",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "parameter: cellsX",
        "node: checker",
        "stage: Compile",
        "publicId:",
    ] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
}
