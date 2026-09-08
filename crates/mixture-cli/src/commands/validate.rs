//! Bounded file I/O and presentation over the public core validation API.

use mixture_core::{DiagnosticReport, SafetyLimits};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const HELP: &str = "Usage: mixture validate <file.mix> [--json]

Validate strict .mix v1 JSON, node contracts, ports, graph structure, and budgets.
No GPU is initialized. Input is read up to the 2 MiB limit plus one detection byte.
--json writes one {ok, diagnostics} report to stdout.
Exit 0: valid document; 1: file/report I/O failure; 2: invalid invocation or input.
Use -- before a path starting with '-'. Files are never changed.";

pub(crate) fn run(arguments: &[OsString]) -> ExitCode {
    if matches!(arguments, [arg] if arg == "--help" || arg == "-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let (path, json) = match parse(arguments) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let limits = SafetyLimits::default();
    let (diagnostics, exit) = match super::document_io::load(&path, &limits) {
        Ok(_) => (Vec::new(), 0),
        Err(failure) => failure,
    };
    let report = DiagnosticReport::new(diagnostics.into_iter().map(|mut diagnostic| {
        diagnostic.document_path = Some(path.to_string_lossy().into_owned());
        diagnostic
    }));
    let mut stdout = io::stdout().lock();
    let written = if json {
        serde_json::to_writer_pretty(&mut stdout, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(stdout))
    } else {
        write_human(&mut stdout, &path, &report)
    };
    if let Err(error) = written.and_then(|()| stdout.flush()) {
        eprintln!("Could not write validation report: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}
fn parse(arguments: &[OsString]) -> Result<(PathBuf, bool), String> {
    let (mut path, mut json, mut options) = (None, false, true);
    for argument in arguments {
        if options && argument == "--json" {
            if json {
                return Err("Duplicate --json option.".into());
            }
            json = true;
        } else if options && argument == "--" {
            options = false;
        } else if options && argument.to_str().is_some_and(|arg| arg.starts_with('-')) {
            return Err(format!(
                "Unknown validate option: {}",
                argument.to_string_lossy()
            ));
        } else if path.is_none() && !argument.is_empty() {
            path = Some(PathBuf::from(argument));
        } else {
            return Err("Expected exactly one nonempty input path.".into());
        }
    }
    Ok((path.ok_or("Missing input path.")?, json))
}
fn write_human(out: &mut impl Write, path: &Path, report: &DiagnosticReport) -> io::Result<()> {
    if report.is_ok() {
        return writeln!(out, "Valid .mix v1 material: {}", path.display());
    }
    writeln!(out, "Invalid material: {}", path.display())?;
    super::human_diagnostics::write(out, report)?;
    Ok(())
}
