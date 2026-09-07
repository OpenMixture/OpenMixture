//! Process-level checks for the CLI's implemented command surface.

use std::process::Command;

#[test]
fn help_and_version_report_the_current_command_surface() {
    for argument in ["--help", "-h", "--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .arg(argument)
            .output()
            .expect("CLI should start");
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("GPU context diagnostics"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn missing_unknown_and_future_commands_never_claim_success() {
    for arguments in [
        vec![],
        vec!["validate", "missing.mix"],
        vec!["inspect", "missing.mix", "--plan"],
        vec!["render", "missing.mix"],
        vec!["render-builtin", "checker"],
        vec!["unknown"],
        vec!["--help", "unexpected"],
        vec!["--version", "unexpected"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
            .args(arguments)
            .output()
            .expect("CLI should start");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("not implemented yet"));
    }
}
