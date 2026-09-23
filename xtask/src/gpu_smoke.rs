//! Explicit real-GPU compute/readback checks; ordinary check/test never run them.

use crate::{TaskResult, cargo, run_cargo};
use serde_json::{Value, json};
use std::{env, fs, io::Cursor, path::Path};

pub(super) fn run(root: &Path) -> TaskResult {
    let directory = root.join("tmp/gpu-smoke");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("package-status.json"),
        br#"{"ok":false,"completed":false}"#,
    )?;
    let (backend, software) = policy()?;
    let expected = env::var("MIXTURE_GPU_EXPECT_ADAPTER").ok();
    println!("Running checker GPU smoke (backend={backend}, software={software})");
    let doctor = command_report(
        root,
        &["doctor", "--json"],
        &backend,
        &software,
        &directory,
        "doctor",
    )?;
    validate_report(&doctor, &backend, software == "1", expected.as_deref())?;
    let skipped = command_report(
        root,
        &["doctor", "--skip-probe", "--json"],
        &backend,
        &software,
        &directory,
        "doctor-skipped",
    )?;
    validate_adapter(&skipped, &backend, software == "1", expected.as_deref())?;
    if skipped["ok"] != true
        || skipped["verdict"] != "unverified"
        || skipped["computeProbe"] != "notRun"
        || skipped["readbackProbe"] != "notRun"
        || !skipped["execution"].is_null()
    {
        return Err("skipped doctor probe claimed execution or health".into());
    }
    let rendered = command_report(
        root,
        &[
            "render-builtin",
            "checker",
            "--size",
            "64",
            "--out",
            "tmp/gpu-smoke/checker.png",
            "--json",
        ],
        &backend,
        &software,
        &directory,
        "checker",
    )?;
    validate_adapter(
        &rendered["context"],
        &backend,
        software == "1",
        expected.as_deref(),
    )?;
    let png = fs::read(directory.join("checker.png"))?;
    if rendered["ok"] != true
        || rendered["writtenBytes"].as_u64() != Some(png.len() as u64)
        || !valid_execution(&rendered["execution"])
    {
        return Err("render-builtin did not report a completed checker PNG".into());
    }
    let mut reader = png::Decoder::new(Cursor::new(png)).read_info()?;
    let mut decoded = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or("invalid PNG output size")?
    ];
    let info = reader.next_frame(&mut decoded)?;
    if info.width != 64
        || info.height != 64
        || info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
    {
        return Err("checker PNG dimensions or format differ from the golden contract".into());
    }
    let golden = fs::read(root.join("fixtures/nodes/checker/checker-64.rgba"))?;
    let mismatches = decoded
        .iter()
        .zip(&golden)
        .filter(|(actual, expected)| actual != expected)
        .count();
    let matches = decoded.len() == golden.len() && mismatches == 0;
    fs::write(
        directory.join("checker-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "ok": matches, "golden": "fixtures/nodes/checker/checker-64.rgba", "decodedBytes": decoded.len(), "goldenBytes": golden.len(), "differingBytes": mismatches,
        }))?,
    )?;
    if !matches {
        return Err(
            "GPU checker differs from the reviewed SwiftShader golden; baseline was not modified"
                .into(),
        );
    }
    // Exercise the public .mix -> compile -> wgpu -> PNG path for each M2 example.
    for (name, passes) in [("checker", 3), ("levels", 4), ("blend", 6)] {
        let input = format!("examples/{name}.mix");
        let output = format!("tmp/gpu-smoke/graph-{name}");
        let report = command_report(
            root,
            &[
                "render",
                &input,
                "--size",
                "64",
                "--output",
                "baseColor,normal,roughness",
                "--out",
                &output,
                "--json",
            ],
            &backend,
            &software,
            &directory,
            &format!("graph-{name}"),
        )?;
        validate_adapter(
            &report["context"],
            &backend,
            software == "1",
            expected.as_deref(),
        )?;
        if report["ok"] != true
            || report["execution"]["passCount"] != passes
            || report["planHash"] != report["execution"]["planHash"]
            || !report["planHash"]
                .as_str()
                .is_some_and(|s| s.starts_with("sha256:") && s.len() == 71)
            || report["outputs"]
                .as_array()
                .is_none_or(|outputs| outputs.len() != 3)
        {
            return Err(format!("graph {name} did not report complete plan execution").into());
        }
        if name == "checker" {
            let mut reader = png::Decoder::new(Cursor::new(fs::read(
                root.join(&output).join("baseColor.png"),
            )?))
            .read_info()?;
            let mut pixels = vec![0; reader.output_buffer_size().ok_or("invalid graph PNG")?];
            reader.next_frame(&mut pixels)?;
            if pixels != golden {
                return Err("graph checker differs from the existing golden".into());
            }
        }
    }
    // Keep unrelated node workloads and deliberate device destruction separate.
    // Individual tests still exercise their explicit independent contexts.
    let tests = cargo(root)
        .args([
            "test",
            "--locked",
            "--all-features",
            "-p",
            "mixture-wgpu",
            "-p",
            "mixture-cli",
            "--",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("MIXTURE_NODE_EVIDENCE_DIR", directory.join("nodes"))
        .output()?;
    fs::write(directory.join("gpu-tests.stdout.log"), &tests.stdout)?;
    fs::write(directory.join("gpu-tests.stderr.log"), &tests.stderr)?;
    if !tests.status.success() {
        return Err(format!("GPU tests failed; inspect {}", directory.display()).into());
    }
    crate::consumer::gpu(
        root,
        &directory,
        &backend,
        software == "1",
        expected.as_deref(),
    )?;
    crate::package::gpu(
        root,
        &directory,
        &backend,
        software == "1",
        expected.as_deref(),
    )?;
    println!(
        "GPU checker and graph smoke passed: {} ({}, {}); doctor healthy; golden exact. Evidence: {}",
        doctor["adapter"]["name"],
        doctor["adapter"]["deviceType"],
        doctor["adapter"]["backend"],
        directory.display()
    );
    Ok(())
}

pub(super) fn policy() -> TaskResult<(String, String)> {
    let backend = env::var("MIXTURE_GPU_BACKEND").unwrap_or_else(|_| "auto".into());
    let software = env::var("MIXTURE_GPU_SOFTWARE").unwrap_or_else(|_| "0".into());
    if !matches!(backend.as_str(), "auto" | "vulkan" | "metal" | "dx12")
        || !matches!(software.as_str(), "0" | "1")
    {
        return Err("Invalid smoke policy: MIXTURE_GPU_BACKEND=auto|vulkan|metal|dx12, MIXTURE_GPU_SOFTWARE=0|1".into());
    }
    Ok((backend, software))
}

pub(super) fn run_node(root: &Path, node: &str) -> TaskResult {
    if !matches!(
        node,
        "image-input"
            | "constant-scalar"
            | "constant-color"
            | "checker"
            | "levels"
            | "blend"
            | "scalar-blend"
            | "scalar-mask-blend"
            | "scalar-morphology"
            | "scalar-subtract"
            | "brick-pattern"
            | "material-output"
            | "fractal-noise"
            | "gradient-map"
            | "height-to-normal"
            | "transform-2d"
            | "warp"
    ) {
        return Err(format!("unknown node test: {node}").into());
    }
    let (backend, software) = policy()?;
    run_cargo(
        root,
        &[
            "test",
            "--locked",
            "--all-features",
            "-p",
            "mixture-wgpu",
            "--test",
            "nodes",
            "node_fixtures_validate_defaults_boundaries_invalid_values_without_gpu",
        ],
        false,
    )?;
    let filter = format!("node_{}_gpu", node.replace('-', "_"));
    let directory = root.join("tmp/node-tests").join(&backend);
    let status = cargo(root)
        .args([
            "test",
            "--locked",
            "--all-features",
            "-p",
            "mixture-wgpu",
            "--test",
            "nodes",
            &filter,
            "--",
            "--exact",
            "--ignored",
            "--nocapture",
        ])
        .env("MIXTURE_GPU_BACKEND", &backend)
        .env("MIXTURE_GPU_SOFTWARE", &software)
        .env("MIXTURE_NODE_EVIDENCE_DIR", &directory)
        .status()?;
    if !status.success() {
        return Err(format!("{node} GPU test failed ({status})").into());
    }
    println!(
        "Node {node} passed; evidence: {}",
        directory.join(format!("{node}.json")).display()
    );
    Ok(())
}

pub(super) fn command_report(
    root: &Path,
    arguments: &[&str],
    backend: &str,
    software: &str,
    directory: &Path,
    name: &str,
) -> TaskResult<Value> {
    let mut command = cargo(root);
    command
        .args([
            "run",
            "--locked",
            "--all-features",
            "-p",
            "mixture-cli",
            "--",
        ])
        .args(arguments)
        .args(["--backend", backend]);
    if software == "1" {
        command.arg("--software");
    }
    let output = command.output()?;
    fs::write(directory.join(format!("{name}.json")), &output.stdout)?;
    fs::write(directory.join(format!("{name}.stderr.log")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!(
            "{name} failed ({}); inspect {}",
            output.status,
            directory.display()
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn valid_execution(report: &Value) -> bool {
    report["passCount"] == 1
        && report["width"] == 64
        && report["height"] == 64
        && report["readbackBytes"] == 32768
        && report["rgbaBytes"] == 16384
        && report["mappedBytes"] == 32768
        && report["paddedBytesPerRow"] == 512
        && report["textureFormat"] == "rgba16float"
        && report["dispatch"] == json!([8, 8, 1])
}

pub(super) fn validate_report(
    report: &Value,
    backend: &str,
    software: bool,
    expected_adapter: Option<&str>,
) -> TaskResult {
    if report["schemaVersion"] != 1
        || report["ok"] != true
        || report["verdict"] != "healthy"
        || report["computeProbe"] != "passed"
        || report["readbackProbe"] != "passed"
        || report["diagnostics"] != json!([])
        || !valid_execution(&report["execution"])
    {
        return Err("doctor returned no verified compute/readback evidence".into());
    }
    validate_adapter(report, backend, software, expected_adapter)
}

pub(super) fn validate_adapter(
    report: &Value,
    backend: &str,
    software: bool,
    expected_adapter: Option<&str>,
) -> TaskResult {
    if !report["device"].is_object()
        || !report["adapter"]["supportedLimits"].is_object()
        || report["requested"]["backend"] != backend
        || report["requested"]["softwareAdapter"] != software
    {
        return Err("requested and actual adapter evidence is incomplete".into());
    }
    let actual = report["adapter"]["backend"]
        .as_str()
        .ok_or("missing backend")?;
    if !matches!(actual, "Metal" | "Vulkan" | "Dx12")
        || (backend != "auto" && !actual.eq_ignore_ascii_case(backend))
    {
        return Err(format!("unexpected adapter backend: {actual}").into());
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
        return Err(format!("expected {expected:?}, selected {name:?}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_smoke_requires_actual_probe_evidence_and_the_requested_adapter() {
        let valid = json!({
            "schemaVersion": 1, "ok": true, "verdict": "healthy", "computeProbe": "passed", "readbackProbe": "passed", "diagnostics": [],
            "requested": {"backend": "vulkan", "softwareAdapter": true},
            "adapter": {"name": "SwiftShader Device", "backend": "Vulkan", "deviceType": "Cpu", "supportedLimits": {}}, "device": {},
            "execution": {"passCount": 1, "width": 64, "height": 64, "readbackBytes": 32768, "rgbaBytes": 16384, "mappedBytes": 32768, "paddedBytesPerRow": 512, "textureFormat": "rgba16float", "dispatch": [8,8,1]}
        });
        assert!(validate_report(&valid, "vulkan", true, Some("SwiftShader")).is_ok());
        for (field, value) in [
            ("verdict", json!("unverified")),
            ("computeProbe", json!("notRun")),
            ("execution", Value::Null),
        ] {
            let mut bad = valid.clone();
            bad[field] = value;
            assert!(validate_report(&bad, "vulkan", true, None).is_err());
        }
        assert!(validate_report(&valid, "metal", true, None).is_err());
        assert!(validate_report(&valid, "vulkan", true, Some("another driver")).is_err());
        let mut bad = valid.clone();
        bad["adapter"]["deviceType"] = json!("IntegratedGpu");
        assert!(validate_report(&bad, "vulkan", true, None).is_err());
    }
}
