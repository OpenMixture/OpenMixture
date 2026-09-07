//! Minimal, honest CLI entry point for the M0 foundation.

use std::{env, ffi::OsStr, process::ExitCode};

const HELP: &str = "Mixture — M0 foundation

Usage: mixture --help
       mixture --version

Material parsing and rendering are not implemented yet.
See ROADMAP.md and INITIAL_PRS.md for the implementation sequence.
Run `cargo xtask check` from the repository to verify the foundation.";

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let first = args.next();
    let single_argument = args.next().is_none();

    match first.as_deref() {
        Some(arg)
            if single_argument && (arg == OsStr::new("--help") || arg == OsStr::new("-h")) =>
        {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Some(arg)
            if single_argument && (arg == OsStr::new("--version") || arg == OsStr::new("-V")) =>
        {
            println!("mixture {} (M0 foundation)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("No runtime commands are available in the M0 foundation.\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}
