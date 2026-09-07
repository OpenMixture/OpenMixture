//! Verify command dispatch and offline checks through the actual xtask executable.

use std::process::Command;

#[test]
fn supported_repository_checks_work_outside_the_workspace_directory() {
    for command in ["--help", "deps", "links"] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .current_dir(std::env::temp_dir())
            .arg(command)
            .output()
            .expect("xtask should start");
        assert!(
            output.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn unknown_and_not_yet_implemented_commands_fail() {
    for command in [
        "unknown",
        "shader-check",
        "gpu-smoke",
        "golden",
        "test-plan",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .arg(command)
            .output()
            .expect("xtask should start");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("unimplemented command"));
    }
}
