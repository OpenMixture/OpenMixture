//! Graph CLI source/compile ordering, actual PNGs, provenance, and exit contracts.
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
}
fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mixture"))
}
fn report(output: Output, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn render_validates_source_and_request_before_acquiring_gpu_or_creating_outputs() {
    let out = std::env::temp_dir().join(format!("mixture-render-invalid-{}", std::process::id()));
    assert!(!out.exists());
    for (source, args, exit, code) in [
        (
            "fixtures/format/invalid/cycle.mix",
            vec![],
            2,
            "MIX_GRAPH_CYCLE",
        ),
        ("missing.mix", vec![], 1, "MIX_IO_READ_FAILED"),
        (
            "examples/checker.mix",
            vec!["--size", "0"],
            2,
            "MIX_COMPILE_INVALID_REQUEST",
        ),
        (
            "examples/checker.mix",
            vec!["--size", "2049"],
            2,
            "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED",
        ),
        (
            "examples/checker.mix",
            vec!["--set", "frequency=0"],
            2,
            "MIX_PARAMETER_INVALID_VALUE",
        ),
        (
            "examples/checker.mix",
            vec!["--output", "height,height"],
            2,
            "MIX_COMPILE_INVALID_REQUEST",
        ),
        (
            "examples/checker.mix",
            vec!["--output", "unknown"],
            2,
            "MIX_COMPILE_INVALID_REQUEST",
        ),
        (
            "examples/checker.mix",
            vec![],
            1,
            "MIX_GPU_ADAPTER_UNAVAILABLE",
        ),
    ] {
        let value = report(
            command()
                .arg("render")
                .arg(root().join(source))
                .args(args)
                .args(["--backend", "none", "--json", "--out"])
                .arg(&out)
                .output()
                .unwrap(),
            exit,
        );
        assert_eq!(value["ok"], false);
        assert!(value["outputs"].as_array().unwrap().is_empty());
        assert!(value["execution"].is_null());
        assert!(
            value["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == code)
        );
        if code != "MIX_GPU_ADAPTER_UNAVAILABLE" {
            assert!(value["context"].is_null());
        } else {
            assert!(value["context"].is_object());
            assert!(value["planHash"].as_str().unwrap().starts_with("sha256:"));
        }
        assert!(!out.exists());
    }
}
#[test]
fn render_usage_rejects_ambiguous_flags_and_accepts_dash_paths() {
    for args in [
        vec![],
        vec!["x.mix"],
        vec!["--out", "out"],
        vec!["x.mix", "--out", "out", "--plan"],
        vec![
            "x.mix",
            "--out",
            "out",
            "--backend",
            "none",
            "--backend",
            "auto",
        ],
        vec![
            "x.mix",
            "--out",
            "out",
            "--set",
            "frequency=3",
            "--set",
            "frequency=4",
        ],
        vec!["x.mix", "--out", "out", "--size", "2x"],
        vec!["x.mix", "--out", "out", "--set", "frequency=NaN"],
        vec!["x.mix", "y.mix", "--out", "out"],
        vec!["x.mix", "--out", "out", "--json", "--json"],
    ] {
        let output = command().arg("render").args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let help = command().args(["render", "--help"]).output().unwrap();
    assert!(help.status.success());
    let dir = std::env::temp_dir().join(format!("mixture-render-dash-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(root().join("examples/checker.mix"), dir.join("-input.mix")).unwrap();
    let value = report(
        command()
            .current_dir(&dir)
            .args([
                "render",
                "--out",
                "out",
                "--backend",
                "none",
                "--json",
                "--",
                "-input.mix",
            ])
            .output()
            .unwrap(),
        1,
    );
    assert_eq!(
        value["diagnostics"][0]["code"],
        "MIX_GPU_ADAPTER_UNAVAILABLE"
    );
    std::fs::remove_dir_all(dir).unwrap();
}
fn gpu_args(command: &mut Command) {
    command.args([
        "--backend",
        &std::env::var("MIXTURE_GPU_BACKEND").unwrap_or_else(|_| "auto".into()),
    ]);
    if std::env::var("MIXTURE_GPU_SOFTWARE").is_ok_and(|s| s == "1") {
        command.arg("--software");
    }
}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_cli_examples_png_metadata_hash_slicing_and_write_failure() {
    let directory = std::env::temp_dir().join(format!("mixture-graph-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for (example, count) in [("checker", 3), ("levels", 4), ("blend", 6)] {
        let source = root().join(format!("examples/{example}.mix"));
        let before = std::fs::read(&source).unwrap();
        let out = directory.join(example);
        let args = [
            "--size",
            "65x3",
            "--output",
            "roughness,normal,baseColor",
            "--json",
        ];
        let mut cmd = command();
        cmd.arg("render")
            .arg(&source)
            .args(args)
            .arg("--out")
            .arg(&out);
        gpu_args(&mut cmd);
        let value = report(cmd.output().unwrap(), 0);
        assert_eq!(value["execution"]["passCount"], count);
        let inspected = report(
            command()
                .arg("inspect")
                .arg(&source)
                .arg("--plan")
                .args(args)
                .output()
                .unwrap(),
            0,
        );
        assert_eq!(value["planHash"], inspected["plan"]["hash"]);
        assert_eq!(value["execution"]["planHash"], value["planHash"]);
        assert!(value["execution"]["adapter"]["name"].is_string());
        for file in value["outputs"].as_array().unwrap() {
            let name = file["channel"].as_str().unwrap();
            let path = out.join(format!("{name}.png"));
            assert_eq!(file["path"], path.to_string_lossy().as_ref());
            let bytes = std::fs::read(path).unwrap();
            assert_eq!(file["writtenBytes"].as_u64(), Some(bytes.len() as u64));
            let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
                .read_info()
                .unwrap();
            if name == "baseColor" {
                assert!(reader.info().srgb.is_some());
                assert_eq!(file["encoding"], "rgba8-srgb");
            } else {
                assert!(reader.info().srgb.is_none());
                assert_eq!(reader.info().gama_chunk.unwrap().into_scaled(), 100000);
                assert_eq!(file["encoding"], "rgba8-linear");
            }
            let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
            let info = reader.next_frame(&mut pixels).unwrap();
            assert_eq!((info.width, info.height), (65, 3));
            if name == "normal" {
                assert_eq!(&pixels[..4], &[128, 128, 255, 255]);
                assert_eq!(file["source"]["source"], "default");
            }
        }
        assert_eq!(before, std::fs::read(source).unwrap());
    }
    let out = directory.join("sliced");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("keep.txt"), "keep").unwrap();
    let mut cmd = command();
    cmd.arg("render")
        .arg(root().join("examples/blend.mix"))
        .args(["--output", "roughness", "--set", "frequency=32", "--out"])
        .arg(&out)
        .arg("--json");
    gpu_args(&mut cmd);
    let value = report(cmd.output().unwrap(), 0);
    assert_eq!(value["execution"]["passCount"], 1);
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 2);
    // A directory blocks the second canonical output: completed files remain reported.
    let out = directory.join("partial");
    std::fs::create_dir_all(out.join("roughness.png")).unwrap();
    let mut cmd = command();
    cmd.arg("render")
        .arg(root().join("examples/checker.mix"))
        .args(["--output", "baseColor,roughness", "--out"])
        .arg(&out)
        .arg("--json");
    gpu_args(&mut cmd);
    let value = report(cmd.output().unwrap(), 1);
    assert_eq!(value["outputs"].as_array().unwrap().len(), 1);
    assert_eq!(value["diagnostics"][0]["code"], "MIX_ENCODING_FAILED");
    assert_eq!(value["ok"], false);
    std::fs::remove_dir_all(directory).unwrap();
}
