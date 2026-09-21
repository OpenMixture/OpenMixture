//! Real-process plan inspection with no GPU, stable reports, overrides, and failures.
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(path)
}
fn run(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mixture"))
        .arg("inspect")
        .arg(path)
        .args(args)
        .env("MIXTURE_GPU_BACKEND", "none")
        .env("MIXTURE_GPU_SOFTWARE", "invalid")
        .output()
        .unwrap()
}
fn report(output: &Output, status: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn inspection_snapshot_is_path_independent_and_never_changes_source() {
    let path = fixture("examples/checker.mix");
    let before = std::fs::read(&path).unwrap();
    let a = report(&run(&path, &["--plan", "--json"]), 0);
    assert_eq!(a["schemaVersion"], 2);
    assert_eq!(a["ok"], true);
    assert_eq!(a["diagnostics"], serde_json::json!([]));
    assert_eq!(
        a["plan"],
        serde_json::from_str::<Value>(include_str!(
            "../../mixture-core/tests/snapshots/plan-v2-checker.json"
        ))
        .unwrap()
    );
    let copied = std::env::temp_dir().join(format!("mixture-inspect-{}.mix", std::process::id()));
    std::fs::write(&copied, &before).unwrap();
    let b = report(&run(&copied, &["--json", "--plan"]), 0);
    assert_eq!(a, b);
    assert_eq!(before, std::fs::read(&path).unwrap());
    let human = run(&path, &["--plan"]);
    assert!(human.status.success());
    let text = String::from_utf8(human.stdout).unwrap();
    for expected in [
        "Compiled RenderPlan v2: 64x64, 1 passes",
        "Hash: sha256:",
        "checker",
        "baseColor",
        "peak 65584",
    ] {
        assert!(text.contains(expected), "{text}");
    }
    std::fs::remove_file(copied).unwrap();
}
#[test]
fn request_options_slice_override_and_normalize_output_order() {
    let path = fixture("fixtures/format/valid/all-m2.mix");
    let args = [
        "--plan",
        "--json",
        "--size",
        "65x3",
        "--output",
        "roughness,baseColor",
        "--set",
        "frequency=23",
        "--set",
        "strength=0.5",
    ];
    let a = report(&run(&path, &args), 0);
    assert_eq!(a["plan"]["size"], serde_json::json!([65, 3]));
    assert_eq!(
        a["plan"]["passes"][0]["kernel"]["cells"],
        serde_json::json!([23, 8])
    );
    assert_eq!(a["plan"]["outputs"][0]["channel"], "baseColor");
    let b = report(
        &run(&path, &["--plan", "--json", "--output", "roughness"]),
        0,
    );
    assert_eq!(b["plan"]["passes"].as_array().unwrap().len(), 1);
    assert_eq!(b["plan"]["passes"][0]["origin"]["node"]["id"], "mask");
}
#[test]
fn semantic_failures_have_structured_codes_paths_and_null_plans() {
    let path = fixture("examples/checker.mix");
    for (args, code) in [
        (vec!["--output", "unknown"], "MIX_COMPILE_INVALID_REQUEST"),
        (
            vec!["--output", "baseColor,baseColor"],
            "MIX_COMPILE_INVALID_REQUEST",
        ),
        (vec!["--size", "0"], "MIX_COMPILE_INVALID_REQUEST"),
        (
            vec!["--size", "2049"],
            "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED",
        ),
        (vec!["--set", "cellsX=16"], "MIX_COMPILE_INVALID_REQUEST"),
        (
            vec!["--set", "frequency=8.0"],
            "MIX_PARAMETER_INVALID_VALUE",
        ),
    ] {
        let mut args = args;
        args.extend(["--plan", "--json"]);
        let value = report(&run(&path, &args), 2);
        assert_eq!(value["ok"], false);
        assert!(value["plan"].is_null());
        assert!(
            value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == code && d["stage"] == "compile")
        );
        assert_eq!(
            value["diagnostics"][0]["documentPath"],
            path.to_string_lossy().as_ref()
        );
    }
    let human = run(&path, &["--plan", "--set", "frequency=0"]);
    assert_eq!(human.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&human.stdout).contains("parameter: cellsX"));
}
#[test]
fn file_and_document_failures_keep_existing_stage_and_exit_contracts() {
    for (path, exit, code) in [
        (fixture("missing.mix"), 1, "MIX_IO_READ_FAILED"),
        (
            fixture("fixtures/format/invalid/cycle.mix"),
            2,
            "MIX_GRAPH_CYCLE",
        ),
        (
            fixture("fixtures/format/invalid/duplicate-key.mix"),
            2,
            "MIX_FORMAT_INVALID_DOCUMENT",
        ),
    ] {
        let value = report(&run(&path, &["--plan", "--json"]), exit);
        assert!(value["plan"].is_null());
        assert!(
            value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == code)
        );
    }
    let path =
        std::env::temp_dir().join(format!("mixture-inspect-large-{}.mix", std::process::id()));
    std::fs::File::create(&path)
        .unwrap()
        .set_len(3 * 1024 * 1024)
        .unwrap();
    let value = report(&run(&path, &["--plan", "--json"]), 2);
    assert_eq!(
        value["diagnostics"][0]["code"],
        "MIX_LIMIT_DECODED_BYTES_EXCEEDED"
    );
    assert_eq!(
        value["diagnostics"][0]["evidence"]["observed"],
        2 * 1024 * 1024 + 1
    );
    std::fs::remove_file(path).unwrap();
}
#[test]
fn usage_is_explicit_and_dash_paths_are_supported() {
    for args in [
        vec![],
        vec!["a.mix"],
        vec!["--plan"],
        vec!["a.mix", "--plan", "--plan"],
        vec!["a.mix", "--plan", "--json", "--json"],
        vec!["a.mix", "--plan", "--size"],
        vec!["a.mix", "--plan", "--size", "1x2x3"],
        vec!["a.mix", "--plan", "--size", "2", "--size", "3"],
        vec![
            "a.mix", "--plan", "--output", "normal", "--output", "height",
        ],
        vec!["a.mix", "--plan", "--set", "frequency=NaN"],
        vec![
            "a.mix",
            "--plan",
            "--set",
            "frequency=2",
            "--set",
            "frequency=3",
        ],
        vec!["a.mix", "--plan", "--unknown"],
        vec!["a.mix", "b.mix", "--plan"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .arg("inspect")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_mixture"))
        .args(["inspect", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--set"));
    let dir = std::env::temp_dir().join(format!("mixture-inspect-dash-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(fixture("examples/checker.mix"), dir.join("-material.mix")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mixture"))
        .current_dir(&dir)
        .args(["inspect", "--plan", "--json", "--", "-material.mix"])
        .output()
        .unwrap();
    report(&out, 0);
    std::fs::remove_dir_all(dir).unwrap();
}
