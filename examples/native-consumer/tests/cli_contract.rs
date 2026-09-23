//! Independent process consumer: no Mixture Rust imports or producer-owned assets.
//! The two explicit tests require a prebuilt CLI and a new evidence directory.
use serde_json::{Value, json};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const INPUT: &[u8] = include_bytes!("../input.mix");
const MISSING_WARP: &[u8] = include_bytes!("inputs/missing-warp.mix");
const CHANNELS: [&str; 4] = ["baseColor", "normal", "roughness", "height"];
const REQUEST: [&str; 6] = [
    "--size",
    "65x3",
    "--output",
    "height,roughness,normal,baseColor",
    "--set",
    "repeat=16",
];

struct Harness {
    binary: PathBuf,
    directory: PathBuf,
    gpu: bool,
    cases: Vec<Value>,
}

impl Harness {
    fn new(gpu: bool) -> Self {
        let binary = fs::canonicalize(
            std::env::var_os("MIXTURE_CONSUMER_CLI")
                .expect("set MIXTURE_CONSUMER_CLI to an already-built mixture executable"),
        )
        .expect("CLI must exist before running the independent consumer");
        let directory = PathBuf::from(
            std::env::var_os("MIXTURE_CONSUMER_EVIDENCE_DIR")
                .expect("set MIXTURE_CONSUMER_EVIDENCE_DIR to a new absolute directory"),
        );
        assert!(directory.is_absolute(), "evidence path must be absolute");
        fs::create_dir(&directory).expect("consumer evidence directory must be new");
        let harness = Self {
            binary,
            directory,
            gpu,
            cases: Vec::new(),
        };
        harness.status(false);
        fs::write(harness.directory.join("input.mix"), INPUT).unwrap();
        fs::write(harness.directory.join("missing-warp.mix"), MISSING_WARP).unwrap();
        fs::write(harness.directory.join("malformed.mix"), b"{").unwrap();
        let mut unknown: Value = serde_json::from_slice(INPUT).unwrap();
        unknown["edges"][0]["from"]["portId"] = json!("missing");
        fs::write(
            harness.directory.join("unknown-port.mix"),
            serde_json::to_vec(&unknown).unwrap(),
        )
        .unwrap();
        harness
    }

    fn status(&self, completed: bool) {
        let report = json!({
            "schemaVersion": 1, "ok": completed, "completed": completed,
            "mode": if self.gpu { "cli-contract-gpu" } else { "cli-contract-cpu" },
            "gpuExecuted": self.gpu && completed,
            "cli": self.binary, "workingDirectory": self.directory, "cases": self.cases,
            "packagedCratesValidated": false,
        });
        fs::write(
            self.directory.join("status.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }

    fn run(&mut self, name: &str, args: &[&str], exit: i32) -> Output {
        assert!(!self.cases.iter().any(|case| case["id"] == name));
        let output = Command::new(&self.binary)
            .args(args)
            .current_dir(&self.directory)
            .output()
            .expect("built CLI should start");
        fs::write(
            self.directory.join(format!("{name}.stdout")),
            &output.stdout,
        )
        .unwrap();
        fs::write(
            self.directory.join(format!("{name}.stderr")),
            &output.stderr,
        )
        .unwrap();
        self.cases.push(json!({
            "id": name, "args": args, "exitCode": output.status.code(), "expectedExitCode": exit,
            "stdout": format!("{name}.stdout"), "stderr": format!("{name}.stderr"),
        }));
        self.status(false);
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{name}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn json(&mut self, name: &str, args: &[&str], exit: i32) -> Value {
        let output = self.run(name, args, exit);
        if !self.gpu {
            assert!(
                output.stderr.is_empty(),
                "parsed CPU commands keep stderr empty"
            );
        }
        // from_slice rejects extra JSON documents and non-JSON stdout prefixes/suffixes.
        let value: Value =
            serde_json::from_slice(&output.stdout).expect("one complete stdout JSON report");
        diagnostic_envelope(&value);
        assert_eq!(value["ok"], exit == 0);
        match args[0] {
            "validate" => assert!(
                value.get("schemaVersion").is_none(),
                "existing validate envelope is unversioned"
            ),
            "inspect" => {
                assert_eq!(value["schemaVersion"], 3);
                assert!(
                    value.get("plan").is_some(),
                    "plan is present, including null on failure"
                );
            }
            "doctor" => context_envelope(&value),
            "render" => render_envelope(&value),
            "asset" => assert_eq!(value["schemaVersion"], 1),
            _ => panic!("unexpected test command"),
        }
        value
    }

    fn finish(self) {
        assert_eq!(fs::read(self.directory.join("input.mix")).unwrap(), INPUT);
        assert_eq!(
            fs::read(self.directory.join("missing-warp.mix")).unwrap(),
            MISSING_WARP
        );
        self.status(true);
        println!(
            "{} independent CLI invocations passed; evidence: {}",
            self.cases.len(),
            self.directory.display()
        );
    }
}

fn diagnostic_envelope(value: &Value) {
    assert!(value["ok"].is_boolean());
    for diagnostic in value["diagnostics"].as_array().expect("diagnostics array") {
        for field in ["code", "stage", "severity", "message"] {
            assert!(diagnostic[field].is_string());
        }
        for field in [
            "documentPath",
            "nodeId",
            "portId",
            "parameterId",
            "suggestion",
        ] {
            if let Some(field) = diagnostic.get(field) {
                assert!(field.is_string());
            }
        }
        if let Some(evidence) = diagnostic.get("evidence") {
            assert!(
                evidence
                    .as_object()
                    .unwrap()
                    .values()
                    .all(|v| v.is_boolean()
                        || v.as_u64().is_some()
                        || v.is_string()
                        || (diagnostic["stage"] == "package"
                            && v.as_array().is_some_and(|a| a.iter().all(Value::is_string))))
            );
        }
        assert!(
            diagnostic.get("source").is_none(),
            "native source objects are not wire data"
        );
    }
}

fn diagnostic<'a>(value: &'a Value, code: &str, stage: &str, source: &str) -> &'a Value {
    let d = value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == code)
        .unwrap();
    assert_eq!(d["stage"], stage);
    assert_eq!(d["severity"], "error");
    assert_eq!(d["documentPath"], source);
    d
}

fn context_envelope(value: &Value) {
    assert_eq!(value["schemaVersion"], 1);
    for field in ["verdict", "computeProbe", "readbackProbe"] {
        assert!(value[field].is_string());
    }
    for field in ["adapter", "device"] {
        assert!(
            value.get(field).is_some(),
            "{field} is present even when null"
        );
        assert!(value[field].is_null() || value[field].is_object());
    }
    let requested = &value["requested"];
    for field in ["backend", "powerPreference"] {
        assert!(requested[field].is_string());
    }
    assert!(requested["softwareAdapter"].is_boolean());
    for field in ["effectiveBackends", "requiredFeatures"] {
        assert!(requested[field].is_array());
    }
    assert!(requested["requiredLimits"].is_object());
    diagnostic_envelope(value);
}

fn render_envelope(value: &Value) {
    assert_eq!(value["schemaVersion"], 3);
    for field in ["input", "outputDirectory"] {
        assert!(value[field].is_string());
    }
    for field in ["planHash", "context", "execution"] {
        assert!(value.get(field).is_some());
    }
    assert!(value["outputs"].is_array());
    if value["context"].is_object() {
        context_envelope(&value["context"]);
    }
}

fn no_render_work(value: &Value, directory: &Path) {
    assert!(value["planHash"].is_null());
    assert!(value["context"].is_null());
    assert!(value["execution"].is_null());
    assert_eq!(value["outputs"], json!([]));
    assert!(!directory.exists());
}

#[test]
#[ignore = "requires built CLI; cargo xtask test-consumer invokes this CPU-only test explicitly"]
fn cli_contract_cpu() {
    let mut h = Harness::new(false);
    let validated = h.json("validate-valid", &["validate", "input.mix", "--json"], 0);
    assert_eq!(validated, json!({"ok": true, "diagnostics": []}));
    let mut inspect_args = vec!["inspect", "input.mix", "--plan", "--json"];
    inspect_args.extend(REQUEST);
    let original = h.json("inspect-valid", &inspect_args, 0);
    let plan = &original["plan"];
    assert_eq!(plan["version"], 3);
    assert_eq!(plan["documentVersion"], 1);
    assert_eq!(plan["size"], json!([65, 3]));
    assert_eq!(plan["passes"].as_array().unwrap().len(), 4);
    assert_eq!(
        plan["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["channel"].as_str().unwrap())
            .collect::<Vec<_>>(),
        CHANNELS
    );
    let hash = plan["hash"].as_str().unwrap();
    assert_eq!(hash.len(), 71);
    assert!(hash.starts_with("sha256:"));
    assert!(
        hash[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    );
    assert!(
        plan["estimates"]
            .as_object()
            .unwrap()
            .values()
            .all(|v| v.as_u64().is_some())
    );
    let reordered = h.json(
        "inspect-reordered",
        &[
            "inspect",
            "input.mix",
            "--plan",
            "--json",
            "--size",
            "65x3",
            "--output",
            "baseColor,normal,roughness,height",
            "--set",
            "repeat=16",
        ],
        0,
    );
    assert_eq!(original, reordered);
    let changed = h.json(
        "inspect-override",
        &[
            "inspect",
            "input.mix",
            "--plan",
            "--json",
            "--size",
            "65x3",
            "--output",
            "baseColor,normal,roughness,height",
            "--set",
            "repeat=4",
        ],
        0,
    );
    assert_ne!(plan["hash"], changed["plan"]["hash"]);
    let sliced = h.json(
        "inspect-sliced",
        &[
            "inspect",
            "input.mix",
            "--plan",
            "--json",
            "--size",
            "65x3",
            "--output",
            "roughness",
        ],
        0,
    );
    assert_eq!(sliced["plan"]["passes"].as_array().unwrap().len(), 1);

    let mut missing_warp = None;
    for command in ["validate", "inspect", "render"] {
        let extra = match command {
            "inspect" => vec!["--plan"],
            "render" => vec!["--out", "unused", "--backend", "none"],
            _ => vec![],
        };
        for (source, exit, code, stage) in [
            (
                "missing-warp.mix",
                2,
                "MIX_PORT_REQUIRED_CONNECTION",
                "validation",
            ),
            ("malformed.mix", 2, "MIX_PARSE_INVALID_JSON", "parse"),
            ("missing.mix", 1, "MIX_IO_READ_FAILED", "parse"),
        ] {
            let mut args = vec![command, source, "--json"];
            args.extend(extra.iter().copied());
            let value = h.json(&format!("{command}-{source}-json"), &args, exit);
            let d = diagnostic(&value, code, stage, source);
            if source == "missing-warp.mix" {
                assert_eq!(d["nodeId"], "sample");
                assert_eq!(d["portId"], "displacement");
                assert!(d.get("parameterId").is_none());
                if let Some(previous) = &missing_warp {
                    assert_eq!(previous, &value["diagnostics"]);
                }
                missing_warp = Some(value["diagnostics"].clone());
                let human_args: Vec<_> = args.iter().copied().filter(|v| *v != "--json").collect();
                let human = h.run(&format!("{command}-missing-warp-human"), &human_args, exit);
                assert!(human.stderr.is_empty());
                let human = String::from_utf8(human.stdout).unwrap();
                for expected in [
                    code,
                    "document: missing-warp.mix",
                    "stage: Validation",
                    "node: sample",
                    "port: displacement",
                ] {
                    assert!(human.contains(expected), "{command}: {human}");
                }
            } else {
                assert!(d["evidence"]["sourceMessage"].is_string());
                if source == "malformed.mix" {
                    assert!(d["evidence"]["line"].as_u64().is_some());
                    assert!(d["evidence"]["column"].as_u64().is_some());
                }
            }
            if command == "inspect" {
                assert!(value["plan"].is_null());
            }
            if command == "render" {
                no_render_work(&value, &h.directory.join("unused"));
            }
        }
    }
    let unknown = h.json(
        "unknown-port",
        &["inspect", "unknown-port.mix", "--plan", "--json"],
        2,
    );
    let d = diagnostic(
        &unknown,
        "MIX_PORT_UNKNOWN",
        "validation",
        "unknown-port.mix",
    );
    assert_eq!(d["nodeId"], "pattern");
    assert_eq!(d["portId"], "missing");
    let human = h.run(
        "unknown-port-human",
        &["inspect", "unknown-port.mix", "--plan"],
        2,
    );
    assert!(human.stderr.is_empty());
    let human = String::from_utf8(human.stdout).unwrap();
    for expected in [
        "MIX_PORT_UNKNOWN",
        "document: unknown-port.mix",
        "node: pattern",
        "port: missing",
    ] {
        assert!(human.contains(expected), "{human}");
    }
    let replay = h.json(
        "unknown-port-replay",
        &["inspect", "unknown-port.mix", "--plan", "--json"],
        2,
    );
    assert_eq!(
        unknown, replay,
        "CPU diagnostic ordering must be deterministic"
    );
    assert_eq!(
        fs::read(h.directory.join("unknown-port.stdout")).unwrap(),
        fs::read(h.directory.join("unknown-port-replay.stdout")).unwrap()
    );

    for command in ["inspect", "render"] {
        let extra = if command == "inspect" {
            vec!["--plan"]
        } else {
            vec!["--out", "unused", "--backend", "none"]
        };
        let mut args = vec![command, "input.mix", "--json", "--set", "repeat=0"];
        args.extend(extra.iter().copied());
        let value = h.json(&format!("{command}-invalid-override"), &args, 2);
        let d = diagnostic(
            &value,
            "MIX_PARAMETER_INVALID_VALUE",
            "compile",
            "input.mix",
        );
        assert_eq!(d["nodeId"], "pattern");
        assert_eq!(d["parameterId"], "cellsX");
        assert_eq!(d["evidence"]["publicId"], "repeat");
        assert_eq!(d["evidence"]["observed"], "0");
        if command == "render" {
            no_render_work(&value, &h.directory.join("unused"));
        }
        let human_args: Vec<_> = args.iter().copied().filter(|v| *v != "--json").collect();
        let human = h.run(&format!("{command}-invalid-override-human"), &human_args, 2);
        assert!(
            String::from_utf8(human.stdout)
                .unwrap()
                .contains("parameter: cellsX")
        );
        let mut args = vec![command, "input.mix", "--json", "--size", "2049x3"];
        args.extend(extra);
        let value = h.json(&format!("{command}-budget"), &args, 2);
        let d = diagnostic(
            &value,
            "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED",
            "compile",
            "input.mix",
        );
        assert_eq!(d["evidence"]["configured"].as_u64(), Some(2048));
        assert_eq!(d["evidence"]["observed"].as_u64(), Some(2049));
        if command == "render" {
            no_render_work(&value, &h.directory.join("unused"));
        }
    }
    let doctor = h.json(
        "doctor-none",
        &["doctor", "--backend", "none", "--software", "--json"],
        1,
    );
    assert_eq!(doctor["verdict"], "unhealthy");
    assert_eq!(doctor["requested"]["backend"], "none");
    assert_eq!(doctor["requested"]["softwareAdapter"], true);
    assert_eq!(doctor["requested"]["effectiveBackends"], json!([]));
    assert!(doctor["adapter"].is_null() && doctor["device"].is_null());
    assert_eq!(doctor["computeProbe"], "notRun");
    assert_eq!(doctor["readbackProbe"], "notRun");
    assert!(doctor.get("execution").is_none());
    assert_eq!(
        doctor["diagnostics"][0]["code"],
        "MIX_GPU_ADAPTER_UNAVAILABLE"
    );
    assert!(doctor["diagnostics"][0].get("documentPath").is_none());
    let mut args = vec![
        "render",
        "input.mix",
        "--json",
        "--out",
        "unused",
        "--backend",
        "none",
    ];
    args.extend(REQUEST);
    let none = h.json("render-none", &args, 1);
    assert_eq!(none["planHash"], plan["hash"]);
    assert_eq!(none["context"]["verdict"], "unhealthy");
    assert_eq!(none["context"]["requested"]["effectiveBackends"], json!([]));
    assert!(none["execution"].is_null());
    assert_eq!(none["outputs"], json!([]));
    assert!(!h.directory.join("unused").exists());

    for (name, args) in [
        (
            "validate-usage",
            vec!["validate", "input.mix", "--json", "--unknown"],
        ),
        ("inspect-usage", vec!["inspect", "input.mix", "--json"]),
        (
            "doctor-usage",
            vec!["doctor", "--json", "--backend", "bogus"],
        ),
        ("render-usage", vec!["render", "input.mix", "--json"]),
    ] {
        let output = h.run(name, &args, 2);
        assert!(
            output.stdout.is_empty(),
            "usage with --json still has no stdout JSON"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
    asset_workflow(&mut h, None);
    h.finish();
}

fn assert_adapter(context: &Value, backend: &str, software: bool) {
    context_envelope(context);
    assert_eq!(context["requested"]["backend"], backend);
    assert_eq!(context["requested"]["softwareAdapter"], software);
    let adapter = &context["adapter"];
    assert!(!adapter["name"].as_str().unwrap().is_empty());
    if backend != "auto" {
        assert_eq!(adapter["backend"].as_str().unwrap().to_lowercase(), backend);
    }
    if software {
        assert_eq!(adapter["deviceType"], "Cpu");
    }
    if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
        assert!(adapter["name"].as_str().unwrap().contains(&expected));
    }
    assert!(adapter["vendor"].as_u64().is_some() && adapter["device"].as_u64().is_some());
    for field in ["driver", "driverInfo"] {
        assert!(adapter[field].is_string());
    }
    assert!(adapter["supportedLimits"].is_object() && adapter["supportedFeatures"].is_array());
    assert!(context["device"]["limits"].is_object() && context["device"]["features"].is_array());
}

fn timings(execution: &Value) {
    for field in ["pipelineMs", "executionMs", "readbackMs", "totalMs"] {
        let value = execution["timings"][field].as_f64().unwrap();
        assert!(value.is_finite() && value >= 0.0);
    }
}

fn rendered(value: &Value, plan: &Value, backend: &str, software: bool) {
    assert_adapter(&value["context"], backend, software);
    assert_eq!(
        value["context"]["verdict"], "unverified",
        "render retains its acquisition snapshot"
    );
    assert!(value["context"].get("execution").is_none());
    assert_eq!(value["planHash"], plan["hash"]);
    let execution = &value["execution"];
    assert_eq!(execution["planHash"], plan["hash"]);
    assert_eq!(execution["adapter"], value["context"]["adapter"]);
    assert_eq!(execution["size"], plan["size"]);
    assert_eq!(
        execution["passCount"].as_u64().unwrap() as usize,
        plan["passes"].as_array().unwrap().len()
    );
    for field in ["readbackBytes", "mappedBytes", "rgbaBytes"] {
        assert!(execution[field].as_u64().is_some());
    }
    for field in ["pipelineCache", "estimates", "allocations"] {
        assert!(
            execution[field]
                .as_object()
                .unwrap()
                .values()
                .all(|v| v.as_u64().is_some())
        );
    }
    assert_eq!(execution["allocations"]["liveBytes"], 0);
    assert_eq!(
        execution["allocations"]["cumulativeBytes"],
        execution["allocations"]["releasedBytes"]
    );
    timings(execution);
}

fn pixels(directory: &Path, file: &Value, planned: &Value) -> Vec<u8> {
    for field in ["channel", "kind", "size"] {
        let expected = if field == "size" {
            json!([65, 3])
        } else {
            planned[field].clone()
        };
        assert_eq!(file[field], expected);
    }
    assert_eq!(file["source"], planned["input"]);
    let path = Path::new(file["path"].as_str().unwrap());
    let bytes = fs::read(directory.join(path)).expect("reported output must already be complete");
    assert_eq!(file["writtenBytes"].as_u64(), Some(bytes.len() as u64));
    let mut reader = png::Decoder::new(Cursor::new(bytes)).read_info().unwrap();
    if file["channel"] == "baseColor" {
        assert_eq!(file["encoding"], "rgba8-srgb");
        assert!(reader.info().srgb.is_some());
    } else {
        assert_eq!(file["encoding"], "rgba8-linear");
        assert!(reader.info().srgb.is_none());
        assert_eq!(reader.info().gama_chunk.unwrap().into_scaled(), 100000);
    }
    let mut decoded = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut decoded).unwrap();
    assert_eq!((info.width, info.height), (65, 3));
    assert_eq!(info.color_type, png::ColorType::Rgba);
    assert_eq!(info.bit_depth, png::BitDepth::Eight);
    assert_eq!(info.buffer_size(), 65 * 3 * 4);
    decoded
}

#[test]
#[ignore = "requires built CLI and explicit GPU; cargo xtask gpu-smoke invokes this test"]
fn cli_contract_gpu() {
    let mut h = Harness::new(true);
    let backend = std::env::var("MIXTURE_GPU_BACKEND").expect("explicit GPU backend");
    let software = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Ok("1") => true,
        Ok("0") => false,
        _ => panic!("explicit software policy must be 0 or 1"),
    };
    let mut gpu_args = vec!["--backend", &backend];
    if software {
        gpu_args.push("--software");
    }
    for (name, skipped) in [("doctor", false), ("doctor-skipped", true)] {
        let mut args = vec!["doctor", "--json"];
        args.extend(&gpu_args);
        if skipped {
            args.push("--skip-probe");
        }
        let value = h.json(name, &args, 0);
        assert_adapter(&value, &backend, software);
        assert_eq!(
            value["verdict"],
            if skipped { "unverified" } else { "healthy" }
        );
        for field in ["computeProbe", "readbackProbe"] {
            assert_eq!(value[field], if skipped { "notRun" } else { "passed" });
        }
        if skipped {
            assert!(value.get("execution").is_none());
        } else {
            assert_eq!(value["execution"]["passCount"], 1);
            assert_eq!(value["execution"]["adapter"], value["adapter"]);
            timings(&value["execution"]);
        }
    }
    let mut first_plan = Value::Null;
    let mut first_pixels = Vec::new();
    for (label, outputs, repeat) in [
        ("full", "height,roughness,normal,baseColor", "repeat=16"),
        ("override", "height,roughness,normal,baseColor", "repeat=4"),
        ("sliced", "roughness", "repeat=16"),
    ] {
        let request = ["--size", "65x3", "--output", outputs, "--set", repeat];
        let mut args = vec!["inspect", "input.mix", "--plan", "--json"];
        args.extend(request);
        let inspected = h.json(&format!("inspect-{label}"), &args, 0);
        let plan = &inspected["plan"];
        let mut args = vec!["render", "input.mix", "--out", label, "--json"];
        args.extend(request);
        args.extend(&gpu_args);
        let value = h.json(&format!("render-{label}"), &args, 0);
        rendered(&value, plan, &backend, software);
        let files = value["outputs"].as_array().unwrap();
        assert_eq!(files.len(), if label == "sliced" { 1 } else { 4 });
        assert_eq!(
            fs::read_dir(h.directory.join(label)).unwrap().count(),
            files.len()
        );
        let mut base_color = Vec::new();
        for (file, planned) in files.iter().zip(plan["outputs"].as_array().unwrap()) {
            let channel = file["channel"].as_str().unwrap();
            assert_eq!(
                Path::new(file["path"].as_str().unwrap()),
                Path::new(label).join(format!("{channel}.png"))
            );
            let data = pixels(&h.directory, file, planned);
            let values = data.as_chunks::<4>().0;
            if channel == "baseColor" {
                let a = [137, 188, 225, 255];
                let b = [255, 0, 0, 128];
                assert_eq!(values[0], a);
                assert!(values.contains(&b) && values.iter().all(|p| *p == a || *p == b));
                base_color = data;
            } else {
                let literal = match channel {
                    "normal" => [128, 128, 255, 255],
                    "roughness" => [64, 64, 64, 255],
                    "height" => [0, 0, 0, 255],
                    _ => panic!("unexpected fixture channel"),
                };
                assert!(values.iter().all(|p| *p == literal));
            }
        }
        if label == "full" {
            first_plan = plan.clone();
            first_pixels = base_color;
        } else if label == "override" {
            assert_ne!(first_plan["hash"], plan["hash"]);
            assert_ne!(first_pixels, base_color);
        } else {
            assert_eq!(value["execution"]["passCount"], 1);
        }
    }
    for (name, json_mode) in [("partial", true), ("partial-human", false)] {
        let directory = h.directory.join(name);
        fs::create_dir_all(directory.join("normal.png")).unwrap();
        fs::write(directory.join("keep.txt"), b"consumer-owned marker").unwrap();
        let mut args = vec!["render", "input.mix", "--out", name];
        args.extend(REQUEST);
        args.extend(&gpu_args);
        if json_mode {
            args.push("--json");
            let value = h.json("render-partial", &args, 1);
            rendered(&value, &first_plan, &backend, software);
            let files = value["outputs"].as_array().unwrap();
            assert_eq!(files.len(), 1, "completed canonical prefix only");
            assert_eq!(
                pixels(&h.directory, &files[0], &first_plan["outputs"][0]),
                first_pixels
            );
            let d = diagnostic(&value, "MIX_ENCODING_FAILED", "encoding", "input.mix");
            assert_eq!(
                Path::new(d["evidence"]["outputPath"].as_str().unwrap()),
                Path::new(name).join("normal.png")
            );
            assert!(d["evidence"]["sourceMessage"].is_string());
        } else {
            let output = h.run("render-partial-human", &args, 1);
            let text = String::from_utf8(output.stdout).unwrap();
            for expected in [
                "Wrote",
                "baseColor.png",
                "MIX_ENCODING_FAILED",
                "stage: Encoding",
                "document: input.mix",
                "outputPath:",
            ] {
                assert!(text.contains(expected), "{text}");
            }
        }
        assert!(directory.join("baseColor.png").is_file());
        assert!(directory.join("normal.png").is_dir());
        assert!(
            !directory.join("roughness.png").exists() && !directory.join("height.png").exists()
        );
        assert_eq!(
            fs::read(directory.join("keep.txt")).unwrap(),
            b"consumer-owned marker"
        );
    }
    asset_workflow(&mut h, Some(&gpu_args));
    h.finish();
}

fn asset_workflow(h: &mut Harness, gpu: Option<&[&str]>) {
    let source=br#"{"version":1,"nodes":[{"id":"base","type":"constant-color","version":1},{"id":"image","type":"image-input","version":1,"parameters":{"resourceId":"Input"}},{"id":"out","type":"material-output","version":1}],"edges":[{"from":{"nodeId":"base","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},{"from":{"nodeId":"image","portId":"value"},"to":{"nodeId":"out","portId":"height"}}],"exposedParameters":[{"id":"source","nodeId":"image","parameterId":"resourceId"}]}"#;
    fs::write(h.directory.join("asset.mix"), source).unwrap();
    fs::write(
        h.directory.join("pixels.rgba"),
        [128, 37, 91, 255].repeat(65 * 3),
    )
    .unwrap();
    for (name, out) in [
        ("asset-pack", "asset.mixpack"),
        ("asset-pack-repeat", "repeat.mixpack"),
    ] {
        h.json(
            name,
            &[
                "asset",
                "pack",
                "asset.mix",
                "--size",
                "65x3",
                "--image",
                "Input",
                "pixels.rgba",
                "--out",
                out,
                "--json",
            ],
            0,
        );
    }
    let bytes = fs::read(h.directory.join("asset.mixpack")).unwrap();
    assert_eq!(bytes, fs::read(h.directory.join("repeat.mixpack")).unwrap());
    fs::create_dir(h.directory.join("moved")).unwrap();
    fs::rename(
        h.directory.join("asset.mixpack"),
        h.directory.join("moved/renamed.data"),
    )
    .unwrap();
    fs::remove_file(h.directory.join("asset.mix")).unwrap();
    fs::remove_file(h.directory.join("pixels.rgba")).unwrap();
    let metadata = h.json(
        "asset-inspect",
        &["asset", "inspect", "moved/renamed.data", "--json"],
        0,
    );
    assert_eq!(metadata["asset"]["resources"][0]["id"], "Input");
    assert!(metadata["plan"].is_null());
    assert!(metadata["render"].is_null());
    let plan = h.json(
        "asset-plan",
        &[
            "asset",
            "inspect",
            "moved/renamed.data",
            "--plan",
            "--size",
            "65x3",
            "--output",
            "height",
            "--json",
        ],
        0,
    )["plan"]
        .clone();
    if let Some(gpu) = gpu {
        let mut args = vec![
            "asset",
            "render",
            "moved/renamed.data",
            "--size",
            "65x3",
            "--output",
            "height",
            "--out",
            "asset-png",
            "--json",
        ];
        args.extend_from_slice(gpu);
        let rendered = h.json("asset-render", &args, 0);
        assert_eq!(rendered["render"]["planHash"], plan["hash"]);
        let decoded = pixels(
            &h.directory,
            &rendered["render"]["outputs"][0],
            &plan["outputs"][0],
        );
        assert!(
            decoded
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [128, 128, 128, 255])
        );
    } else {
        let rejected = h.json(
            "asset-override",
            &[
                "asset",
                "inspect",
                "moved/renamed.data",
                "--plan",
                "--size",
                "65x3",
                "--set",
                r#"source="Input""#,
                "--json",
            ],
            2,
        );
        assert_eq!(
            rejected["diagnostics"][0]["code"],
            "MIX_PACKAGE_RESOURCE_OVERRIDE"
        );
        let rejected = h.json(
            "asset-budget",
            &[
                "asset",
                "inspect",
                "moved/renamed.data",
                "--package-buffer-bytes",
                "1",
                "--json",
            ],
            2,
        );
        assert_eq!(
            rejected["diagnostics"][0]["code"],
            "MIX_PACKAGE_LIMIT_EXCEEDED"
        );
        h.json(
            "asset-no-gpu",
            &[
                "asset",
                "render",
                "moved/renamed.data",
                "--size",
                "65x3",
                "--output",
                "height",
                "--out",
                "no-asset-png",
                "--backend",
                "none",
                "--json",
            ],
            1,
        );
        assert!(!h.directory.join("no-asset-png").exists());
        fs::write(h.directory.join("bad.mixpack"), &bytes[..bytes.len() - 1]).unwrap();
        let rejected = h.json(
            "asset-bad-before-gpu",
            &[
                "asset",
                "render",
                "bad.mixpack",
                "--out",
                "bad-output",
                "--backend",
                "none",
                "--json",
            ],
            2,
        );
        assert_eq!(rejected["diagnostics"][0]["code"], "MIX_PACKAGE_INVALID");
        assert!(rejected["render"].is_null());
        assert!(!h.directory.join("bad-output").exists());
    }
}
