//! Verify command dispatch and offline checks through the actual xtask executable.

use std::process::Command;

#[test]
fn supported_repository_checks_work_outside_the_workspace_directory() {
    for command in ["--help", "deps", "evidence", "links"] {
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
    for command in ["unknown", "test-unknown"] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .arg(command)
            .output()
            .expect("xtask should start");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("unimplemented command"));
    }
}

#[test]
fn consumer_check_is_advertised_and_rejects_an_implicit_gpu_flag() {
    let help = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&help.stdout).contains("test-consumer"));
    let invalid = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["test-consumer", "--gpu"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("expected one command"));
}

#[test]
fn golden_updates_require_explicit_acceptance_and_refuse_ci_before_any_work() {
    for (args, ci, message) in [
        (
            vec!["golden", "update", "glazed-ceramic"],
            false,
            "--accept",
        ),
        (
            vec!["golden", "update", "glazed-ceramic", "--accept"],
            true,
            "refuses CI",
        ),
        (vec!["test-material", "../escape"], false, "material ID"),
        (vec!["golden", "check", "--accept"], false, "Usage:"),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command
            .args(args)
            .env_remove("CI")
            .env_remove("GITHUB_ACTIONS");
        if ci {
            command.env("CI", "false");
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(message),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn gpu_smoke_rejects_invalid_policy_before_starting_cargo_or_gpu() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("gpu-smoke")
        .env("MIXTURE_GPU_BACKEND", "none")
        .output()
        .expect("xtask should start");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid smoke policy"));
}

#[test]
fn node_test_rejects_unknown_ids_and_invalid_adapter_policy() {
    for (node, expected) in [
        ("unknown", "unknown node test"),
        ("checker", "Invalid smoke policy"),
        ("transform-2d", "Invalid smoke policy"),
        ("warp", "Invalid smoke policy"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["test-node", node])
            .env("MIXTURE_GPU_BACKEND", "none")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn package_check_is_advertised_and_keeps_gpu_execution_explicit() {
    let help = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&help.stdout).contains("package-check"));
    let invalid = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["package-check", "--gpu"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("expected one command"));
}
