//! Built-CLI consumer with generation-owned output paths and explicit cleanup.
use mixture_native_consumer::latest::{Latest, OutputDirectory};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn save(parent: &Path, report: &Value) {
    fs::write(
        parent.join("status.json"),
        serde_json::to_vec_pretty(report).unwrap(),
    )
    .unwrap();
}
fn render(
    parent: &Path,
    directory: &OutputDirectory,
    repeat: u32,
    report: &mut Value,
) -> Result<(), Value> {
    let backend = std::env::var("MIXTURE_GPU_BACKEND").expect("explicit backend");
    let software = std::env::var("MIXTURE_GPU_SOFTWARE").expect("explicit software policy");
    assert!(matches!(software.as_str(), "0" | "1"));
    let mut command = Command::new(std::env::var_os("MIXTURE_CONSUMER_CLI").expect("prebuilt CLI"));
    command
        .current_dir(parent)
        .args([
            "render",
            "input.mix",
            "--size",
            "65x3",
            "--output",
            "baseColor,normal,roughness,height",
            "--set",
            &format!("repeat={repeat}"),
            "--backend",
            &backend,
            "--out",
        ])
        .arg(directory.path())
        .arg("--json");
    if software == "1" {
        command.arg("--software");
    }
    let output = command.output().unwrap();
    let name = directory.path().file_name().unwrap().to_str().unwrap();
    fs::write(parent.join(format!("{name}.stdout")), &output.stdout).unwrap();
    fs::write(parent.join(format!("{name}.stderr")), &output.stderr).unwrap();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        output.status.code(),
        Some(if value["ok"] == true {
            0
        } else if repeat == 0 {
            2
        } else {
            1
        })
    );
    if let Some(adapter) = value["execution"]["adapter"].as_object() {
        if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
            assert!(adapter["name"].as_str().unwrap().contains(&expected));
        }
        if software == "1" {
            assert_eq!(adapter["deviceType"], "Cpu");
        }
    }
    report["cases"].as_array_mut().unwrap().push(json!({"generationDirectory":name,"repeat":repeat,"exitCode":output.status.code(),"report":value}));
    save(parent, report);
    if value["ok"] == true {
        Ok(())
    } else {
        Err(value)
    }
}
fn normal_bytes(path: &Path) -> Vec<u8> {
    let decoder = png::Decoder::new(std::io::BufReader::new(fs::File::open(path).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut bytes = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut bytes).unwrap();
    assert_eq!((info.width, info.height), (65, 3));
    bytes.truncate(info.buffer_size());
    bytes
}
#[test]
#[ignore = "requires prebuilt CLI, explicit GPU policy and a fresh evidence directory"]
fn latest_cli_contract() {
    let parent = PathBuf::from(
        std::env::var_os("MIXTURE_CONSUMER_LATEST_DIR").expect("new absolute directory"),
    );
    assert!(parent.is_absolute());
    fs::create_dir(&parent).unwrap();
    fs::write(parent.join("input.mix"), include_bytes!("../input.mix")).unwrap();
    fs::write(parent.join("unrelated"), b"keep").unwrap();
    let mut report =
        json!({"schemaVersion":1,"ok":false,"completed":false,"gpuExecuted":false,"cases":[]});
    save(&parent, &report);
    let mut state = Latest::<u32, OutputDirectory, Value>::default();
    // Display 1; finish 2 after 3 was requested. No renderer writes into display 1.
    let first = state.request(16).unwrap();
    state.start();
    let directory = OutputDirectory::create(&parent, first).unwrap();
    render(&parent, &directory, 16, &mut report).unwrap();
    let first_path = directory.path().to_owned();
    let original = fs::read(first_path.join("baseColor.png")).unwrap();
    assert!(state.complete(first, Ok(directory)).is_none());
    let old = state.request(4).unwrap();
    state.start();
    let failed = state.request(0).unwrap();
    let directory = OutputDirectory::create(&parent, old).unwrap();
    let old_path = directory.path().to_owned();
    render(&parent, &directory, 4, &mut report).unwrap();
    state
        .complete(old, Ok(directory))
        .unwrap()
        .remove()
        .unwrap();
    assert!(!old_path.exists());
    let (generation, repeat) = state.start().unwrap();
    assert_eq!(generation, failed);
    let directory = OutputDirectory::create(&parent, generation).unwrap();
    let failed_path = directory.path().to_owned();
    let error = render(&parent, &directory, repeat, &mut report).unwrap_err();
    directory.remove().unwrap();
    state.complete(generation, Err(error));
    assert!(!failed_path.exists());
    assert_eq!(state.displayed().unwrap().0, first);
    assert!(!state.displayed().unwrap().2);
    assert!(state.failure().is_some());
    assert_eq!(
        fs::read(first_path.join("baseColor.png")).unwrap(),
        original
    );
    // A partial write also remains generation-local and is explicitly cleaned.
    let partial = state.request(4).unwrap();
    state.start();
    let directory = OutputDirectory::create(&parent, partial).unwrap();
    let partial_path = directory.path().to_owned();
    fs::create_dir(directory.path().join("normal.png")).unwrap();
    let error = render(&parent, &directory, 4, &mut report).unwrap_err();
    assert!(directory.path().join("baseColor.png").is_file());
    assert_eq!(error["outputs"].as_array().unwrap().len(), 1);
    directory.remove().unwrap();
    state.complete(partial, Err(error));
    assert!(!partial_path.exists());
    assert!(!state.displayed().unwrap().2);
    let current = state.request(4).unwrap();
    state.start();
    let directory = OutputDirectory::create(&parent, current).unwrap();
    render(&parent, &directory, 4, &mut report).unwrap();
    assert!(
        normal_bytes(&directory.path().join("normal.png"))
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [128, 128, 255, 255])
    );
    assert_ne!(
        fs::read(directory.path().join("baseColor.png")).unwrap(),
        original
    );
    state
        .complete(current, Ok(directory))
        .unwrap()
        .remove()
        .unwrap();
    assert!(!first_path.exists());
    assert!(state.displayed().unwrap().2);
    assert_eq!(state.displayed().unwrap().0, current);
    assert_eq!(fs::read(parent.join("unrelated")).unwrap(), b"keep");
    let retained: Vec<_> = fs::read_dir(&parent)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    assert_eq!(
        retained,
        vec![state.displayed().unwrap().1.path().to_owned()]
    );
    report["ok"] = json!(true);
    report["completed"] = json!(true);
    report["gpuExecuted"] = json!(true);
    report["publishedGenerations"] = json!([1, 5]);
    report["displayedGeneration"] = json!(5);
    report["obsoleteDirectoriesRemoved"] = json!([1, 2, 3, 4]);
    report["retainedDirectories"] = json!(["generation-5"]);
    report["oldDisplayWasStaleAfterFailure"] = json!(true);
    report["unrelatedPreserved"] = json!(true);
    save(&parent, &report);
}
