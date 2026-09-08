//! Private repository automation. No material or rendering semantics belong here.

mod consumer;
mod dependencies;
mod golden;
mod gpu_smoke;
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
  check       Format, dependency policy, Clippy, tests, consumer, rustdoc, and doc links
  fmt         Check Rust formatting
  clippy      Check all workspace targets and features, denying warnings
  test        Run workspace tests, including doctests
  test-core   Run only mixture-core tests (no GPU)
  test-format Run strict .mix decoding, graph, node-contract, and validate CLI tests
  test-plan   Run deterministic compilation, plan/hash snapshots, and inspect CLI tests
  test-consumer Check independent public Rust and CLI consumption without a GPU
  test-node <id> Validate focused fixtures and run that node on an explicit GPU
  test-material <id> Render material cases and check pixels, structure, and causality
  golden check Render and compare all material goldens (never updates baselines)
  golden update <id> --accept Accept a previously rendered software candidate; refuses CI
  trace-2k    Rank all M3 material cases and measure the largest at 2048 on a GPU
  gpu-smoke   Run checker golden, graph examples, native consumer, and GPU regressions
  shader-check Validate every built-in WGSL kernel and its uniform ABI without a GPU
  doc         Build workspace rustdoc, denying warnings
  deps        Check the current dependency and publication policy
  links       Check Markdown links to local files and directories

GPU and material checks are explicit; ordinary check/test do not acquire a GPU.";

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
    if args
        .first()
        .is_some_and(|arg| arg == "golden" || arg == "test-material")
    {
        return golden::dispatch(&workspace_root()?, &args);
    }
    if let [command, node] = args.as_slice()
        && command == "test-node"
    {
        return gpu_smoke::run_node(
            &workspace_root()?,
            node.to_str().ok_or("node ID must be UTF-8")?,
        );
    }
    let command = match args.as_slice() {
        [] => "--help",
        [argument] => argument.to_str().ok_or("command must be valid UTF-8")?,
        _ => return Err(format!("expected one command\n\n{HELP}").into()),
    };
    let root = workspace_root()?;
    match command {
        "--help" | "-h" | "help" => println!("{HELP}"),
        "trace-2k" => golden::trace::run(&root)?,
        "check" => {
            for task in [
                "fmt",
                "deps",
                "clippy",
                "test",
                "test-consumer",
                "doc",
                "links",
            ] {
                run_task(&root, task)?;
            }
            println!("All repository checks passed.");
        }
        "fmt" | "deps" | "clippy" | "test" | "test-core" | "doc" | "links" | "gpu-smoke"
        | "shader-check" | "test-format" | "test-plan" | "test-consumer" => {
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
        "test-format" => {
            run_cargo(
                root,
                &[
                    "test",
                    "--locked",
                    "-p",
                    "mixture-core",
                    "--test",
                    "format",
                    "--test",
                    "validation",
                    "--test",
                    "registry",
                ],
                false,
            )?;
            run_cargo(
                root,
                &[
                    "test",
                    "--locked",
                    "-p",
                    "mixture-cli",
                    "--test",
                    "validate",
                ],
                false,
            )
        }
        "test-plan" => {
            run_cargo(
                root,
                &["test", "--locked", "-p", "mixture-core", "--test", "plan"],
                false,
            )?;
            run_cargo(
                root,
                &["test", "--locked", "-p", "mixture-cli", "--test", "inspect"],
                false,
            )
        }
        "test-core" => run_cargo(root, &["test", "-p", "mixture-core", "--locked"], false),
        "test-consumer" => consumer::check(root),
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
        "gpu-smoke" => gpu_smoke::run(root),
        "shader-check" => run_cargo(
            root,
            &[
                "test",
                "--locked",
                "-p",
                "mixture-wgpu",
                "shader_validates_without_a_gpu",
            ],
            false,
        ),
        _ => Err(format!("unknown task: {task}").into()),
    }
}
