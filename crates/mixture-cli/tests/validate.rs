//! Process-level validation, exit-code, bounded-read, and no-mutation regressions.
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/format")
        .join(name)
}
fn validate(path: &Path, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mixture"));
    command.arg("validate").arg(path);
    if json {
        command.arg("--json");
    }
    // The CPU-only command does not consult smoke policy or acquire a GPU.
    command
        .env("MIXTURE_GPU_BACKEND", "none")
        .env("MIXTURE_GPU_SOFTWARE", "invalid")
        .output()
        .unwrap()
}
#[test]
fn validate_cli_success_is_structured_and_source_is_never_changed() {
    for name in ["valid/checker.mix", "valid/all-m2.mix"] {
        let path = fixture(name);
        let before = std::fs::read(&path).unwrap();
        let output = validate(&path, true);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            json!({"ok":true,"diagnostics":[]})
        );
        assert!(output.stderr.is_empty());
        let output = validate(&path, false);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Valid .mix v1 material"));
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
#[test]
fn validate_cli_fixture_failures_keep_codes_paths_and_input_exit_status() {
    for (name, code, stage) in [
        ("unknown-field", "MIX_FORMAT_INVALID_DOCUMENT", "parse"),
        ("duplicate-key", "MIX_FORMAT_INVALID_DOCUMENT", "parse"),
        (
            "unsupported-version",
            "MIX_FORMAT_UNSUPPORTED_VERSION",
            "validation",
        ),
        (
            "invalid-parameters",
            "MIX_PARAMETER_INVALID_VALUE",
            "validation",
        ),
        ("cycle", "MIX_GRAPH_CYCLE", "validation"),
        ("type-mismatch", "MIX_PORT_TYPE_MISMATCH", "validation"),
        ("duplicate-ids", "MIX_NODE_DUPLICATE_ID", "validation"),
        ("duplicate-edges", "MIX_GRAPH_DUPLICATE_EDGE", "validation"),
        (
            "missing-base-color",
            "MIX_PORT_REQUIRED_CONNECTION",
            "validation",
        ),
        (
            "invalid-exposed",
            "MIX_EXPOSED_PARAMETER_INVALID",
            "validation",
        ),
        ("unknown-port", "MIX_PORT_UNKNOWN", "validation"),
    ] {
        let path = fixture(&format!("invalid/{name}.mix"));
        let before = std::fs::read(&path).unwrap();
        let output = validate(&path, true);
        assert_eq!(output.status.code(), Some(2), "{name}");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["ok"], false);
        assert!(
            report["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == code && d["stage"] == stage)
        );
        for diagnostic in report["diagnostics"].as_array().unwrap() {
            assert_eq!(diagnostic["documentPath"], path.to_string_lossy().as_ref());
        }
        assert!(output.stderr.is_empty());
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
    let output = validate(&fixture("invalid/invalid-parameters.mix"), false);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("node: checker"));
    assert!(text.contains("parameter: cellsX"));
    assert!(text.contains("Suggestion:"));
}
#[test]
fn validate_cli_distinguishes_file_io_from_invalid_bytes_and_oversized_input() {
    let missing = fixture("does-not-exist.mix");
    for path in [&missing, &std::env::temp_dir()] {
        let output = validate(path, true);
        assert_eq!(output.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["diagnostics"][0]["code"], "MIX_IO_READ_FAILED");
        assert_eq!(report["ok"], false);
        assert!(report["diagnostics"][0]["evidence"]["sourceMessage"].is_string());
    }
    let path = std::env::temp_dir().join(format!(
        "mixture-validate-boundary-{}.mix",
        std::process::id()
    ));
    std::fs::write(&path, [0xff]).unwrap();
    let output = validate(&path, true);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["diagnostics"][0]["code"],
        "MIX_PARSE_INVALID_UTF8"
    );
    std::fs::File::create(&path)
        .unwrap()
        .set_len(8 * 1024 * 1024)
        .unwrap();
    let output = validate(&path, true);
    assert_eq!(output.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["diagnostics"][0]["code"],
        "MIX_LIMIT_DECODED_BYTES_EXCEEDED"
    );
    assert_eq!(
        report["diagnostics"][0]["evidence"]["observed"],
        2 * 1024 * 1024 + 1
    );
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 8 * 1024 * 1024);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn validate_cli_rejects_bad_usage_and_supports_options_and_dash_paths() {
    for args in [
        vec![],
        vec!["--json"],
        vec!["x.mix", "--json", "--json"],
        vec!["x.mix", "y.mix"],
        vec!["--unknown"],
        vec!["--"],
        vec![""],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .arg("validate")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    for help in ["--help", "-h"] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .args(["validate", help])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("No GPU is initialized"));
    }
    let directory =
        std::env::temp_dir().join(format!("mixture-validate-dash-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    std::fs::copy(
        fixture("valid/checker.mix"),
        directory.join("--checker.mix"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
        .current_dir(&directory)
        .args(["validate", "--json", "--", "--checker.mix"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["ok"],
        true
    );
    std::fs::remove_file(directory.join("--checker.mix")).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
