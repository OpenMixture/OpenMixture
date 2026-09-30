//! Regression coverage for OS paths and asset diagnostic contracts.
use serde_json::Value;
use std::{ffi::OsString, path::PathBuf, process::Command};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[cfg(windows)]
fn non_unicode() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0xd800, 0x78])
}
#[cfg(unix)]
fn non_unicode() -> OsString {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(b"\xffx").to_owned()
}

fn report(args: &[OsString], exit: i32) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
        .current_dir(root())
        .args(args)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(exit), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
fn args(parts: &[&str]) -> Vec<OsString> {
    parts.iter().map(OsString::from).collect()
}

#[test]
fn non_unicode_input_paths_produce_complete_json() {
    let path = root()
        .join("tmp/nonexistent-adversarial")
        .join(non_unicode());
    for command in [
        vec!["asset", "inspect"],
        vec!["asset", "render"],
        vec!["asset", "pack"],
        vec!["render"],
    ] {
        let mut arguments = args(&command);
        arguments.push(path.clone().into());
        if command.last() != Some(&"inspect") {
            arguments.extend(args(&["--out", "tmp/unused-adversarial"]));
        }
        let result = report(&arguments, 1);
        assert_eq!(result["input"], path.to_string_lossy().as_ref());
        assert_eq!(result["diagnostics"][0]["code"], "MIX_IO_READ_FAILED");
        assert_eq!(result["diagnostics"][0]["stage"], "parse");
    }
}

#[test]
fn non_unicode_output_paths_produce_complete_json() {
    let path = root()
        .join("tmp/nonexistent-adversarial")
        .join(non_unicode());
    let mut arguments = args(&[
        "render",
        "examples/checker.mix",
        "--backend",
        "none",
        "--out",
    ]);
    arguments.push(path.clone().into());
    let result = report(&arguments, 1);
    assert_eq!(result["outputDirectory"], path.to_string_lossy().as_ref());
    let mut arguments = args(&["asset", "pack", "examples/checker.mix", "--out"]);
    arguments.push(path.clone().into());
    let result = report(&arguments, 1);
    assert_eq!(result["diagnostics"][0]["code"], "MIX_IO_WRITE_FAILED");
    assert_eq!(result["diagnostics"][0]["stage"], "encoding");
    assert_eq!(
        result["diagnostics"][0]["evidence"]["path"],
        path.to_string_lossy().as_ref()
    );
}

#[test]
fn non_unicode_asset_paths_work_for_success_and_read_failures() {
    let directory = root()
        .join("tmp")
        .join(format!("asset-os-path-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join(non_unicode());
    let mut arguments = args(&["asset", "pack", "examples/checker.mix", "--out"]);
    arguments.push(output.clone().into());
    // Some filesystems (APFS) reject non-UTF-8 names; the report must still be complete.
    if std::fs::File::create(&output).is_err() {
        let failure = report(&arguments, 1);
        assert_eq!(failure["diagnostics"][0]["code"], "MIX_IO_WRITE_FAILED");
        assert_eq!(
            failure["diagnostics"][0]["evidence"]["path"],
            output.to_string_lossy().as_ref()
        );
        std::fs::remove_dir(&directory).unwrap();
        return;
    }
    std::fs::remove_file(&output).unwrap();
    let value = report(&arguments, 0);
    assert_eq!(value["output"], output.to_string_lossy().as_ref());
    let mut arguments = args(&["asset", "inspect"]);
    arguments.push(output.clone().into());
    assert_eq!(report(&arguments, 0)["ok"], true);
    std::fs::remove_file(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    let failure = report(&arguments, 1);
    assert_eq!(failure["diagnostics"][0]["stage"], "parse");
    assert_eq!(
        failure["diagnostics"][0]["evidence"]["path"],
        output.to_string_lossy().as_ref()
    );
    std::fs::remove_dir(&output).unwrap();
    std::fs::remove_dir(&directory).unwrap();
}

#[test]
fn asset_pack_zero_size_uses_compile_diagnostics_before_reading() {
    let result = report(
        &args(&[
            "asset",
            "pack",
            "missing.mix",
            "--out",
            "tmp/unused-adversarial",
            "--size",
            "0",
        ]),
        2,
    );
    assert_eq!(
        result["diagnostics"][0]["code"],
        "MIX_COMPILE_INVALID_REQUEST"
    );
    assert_eq!(result["diagnostics"][0]["stage"], "compile");
}
