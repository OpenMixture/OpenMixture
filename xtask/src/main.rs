//! Private repository automation. No material or rendering semantics belong here.

mod dependencies;
mod links;

use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

type TaskResult<T = ()> = Result<T, Box<dyn Error>>;

const HELP: &str = "Usage: cargo xtask <command>

Available repository commands:
  check       Format, dependency policy, Clippy, tests, rustdoc, and local doc links
  fmt         Check Rust formatting
  clippy      Check all workspace targets and features, denying warnings
  test        Run workspace tests, including doctests
  test-core   Run only mixture-core tests (no GPU)
  doc         Build workspace rustdoc, denying warnings
  deps        Check the current dependency and publication policy
  links       Check Markdown links to local files and directories

Shader, graph, material, golden, and GPU commands arrive in later milestones.";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> TaskResult {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let command = match args.as_slice() {
        [] => "--help",
        [argument] => argument.to_str().ok_or("command must be valid UTF-8")?,
        _ => return Err(format!("expected one command\n\n{HELP}").into()),
    };
    let root = workspace_root()?;
    match command {
        "--help" | "-h" | "help" => println!("{HELP}"),
        "check" => {
            for task in ["fmt", "deps", "clippy", "test", "doc", "links"] {
                run_task(&root, task)?;
            }
            println!("All repository checks passed.");
        }
        "fmt" | "deps" | "clippy" | "test" | "test-core" | "doc" | "links" => {
            run_task(&root, command)?;
        }
        _ => return Err(format!("unknown or unimplemented command: {command}\n\n{HELP}").into()),
    }
    Ok(())
}

fn workspace_root() -> TaskResult<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask must reside in the workspace's xtask directory".into())
}

fn cargo(root: &Path) -> Command {
    let mut command = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(root);
    command
}

fn run_cargo(root: &Path, arguments: &[&str], rustdoc: bool) -> TaskResult {
    println!("cargo {}", arguments.join(" "));
    let mut command = cargo(root);
    command.args(arguments);
    if rustdoc {
        // Cargo gives encoded flags precedence; preserve caller flags and append our policy.
        let mut flags = match env::var("CARGO_ENCODED_RUSTDOCFLAGS") {
            Ok(flags) => flags,
            Err(_) => env::var("RUSTDOCFLAGS")
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("\x1f"),
        };
        if !flags.is_empty() {
            flags.push('\x1f');
        }
        flags.push_str("-Dwarnings");
        command.env("CARGO_ENCODED_RUSTDOCFLAGS", flags);
    }
    let status = command.status()?;
    if !status.success() {
        return Err(format!("cargo {} failed ({status})", arguments.join(" ")).into());
    }
    Ok(())
}

fn run_task(root: &Path, task: &str) -> TaskResult {
    match task {
        "fmt" => run_cargo(root, &["fmt", "--all", "--", "--check"], false),
        "clippy" => run_cargo(
            root,
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
            false,
        ),
        "test" => run_cargo(
            root,
            &["test", "--workspace", "--all-features", "--locked"],
            false,
        ),
        "test-core" => run_cargo(root, &["test", "-p", "mixture-core", "--locked"], false),
        "doc" => run_cargo(
            root,
            &[
                "doc",
                "--workspace",
                "--all-features",
                "--no-deps",
                "--locked",
            ],
            true,
        ),
        "deps" => dependencies::check(root),
        "links" => links::check(root),
        _ => Err(format!("unknown task: {task}").into()),
    }
}
