//! Thin command dispatch and presentation for Mixture.

mod commands {
    pub mod doctor;
    mod gpu_options;
    pub mod render_builtin;
    pub mod validate;
}

use std::{env, ffi::OsStr, process::ExitCode};

const HELP: &str = "Mixture — material validation and GPU diagnostics

Usage: mixture doctor [--json] [--backend auto|vulkan|metal|dx12|none]
                      [--power-preference high-performance|low-power] [--software]
       mixture render-builtin checker [--size <pixels>] --out <file.png> [--json]
       mixture validate <file.mix> [--json]
       mixture --help
       mixture --version

Doctor verifies checker compute/readback; --skip-probe reports unverified.
Use `mixture doctor --help` for options and exit codes.
Strict .mix v1 validation is implemented; graph compilation and rendering are future work.";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [argument] if argument == "--help" || argument == "-h" => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        [argument] if argument == "--version" || argument == "-V" => {
            println!(
                "mixture {} (material validation and GPU diagnostics)",
                env!("CARGO_PKG_VERSION")
            );
            ExitCode::SUCCESS
        }
        [command, rest @ ..] if command == OsStr::new("doctor") => commands::doctor::run(rest),
        [command, rest @ ..] if command == OsStr::new("render-builtin") => {
            commands::render_builtin::run(rest)
        }
        [command, rest @ ..] if command == OsStr::new("validate") => commands::validate::run(rest),
        _ => {
            eprintln!("Invalid or unimplemented command.\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}
