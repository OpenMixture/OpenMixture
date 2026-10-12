//! Bounded file I/O and presentation over the public core validation API.

use mixture_core::{DiagnosticReport, SafetyLimits};
use std::{
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const HELP: &str = "Usage: mixture validate <file.mix> [--json] [--recipe <file.mixrecipe>]...

Validate strict .mix v1 JSON, node contracts, ports, graph structure, and budgets.
--recipe supplies one recipe file; repeat it for the exact closure of a .mix v2 parent,
which is then validated and expanded. Recipe files are never discovered.
No GPU is initialized. Input is read up to the 2 MiB limit plus one detection byte.
--json writes one {ok, diagnostics} report to stdout.
Exit 0: valid document; 1: file/report I/O failure; 2: invalid invocation or input.
Use -- before a path starting with '-'. Files are never changed.";

pub(crate) fn run(arguments: &[OsString]) -> ExitCode {
    if matches!(arguments, [arg] if arg == "--help" || arg == "-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let (path, json, recipes) = match parse(arguments) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let limits = SafetyLimits::default();
    let mut version = 1;
    let (diagnostics, exit) = match super::document_io::load(&path, &limits, &recipes) {
        Ok(document) => {
            version = document.document().version;
            (Vec::new(), 0)
        }
        Err(failure) => failure,
    };
    let report = DiagnosticReport::new(diagnostics.into_iter().map(|mut diagnostic| {
        diagnostic
            .document_path
            .get_or_insert_with(|| path.to_string_lossy().into_owned());
        diagnostic
    }));
    let mut stdout = io::stdout().lock();
    let written = if json {
        serde_json::to_writer_pretty(&mut stdout, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(stdout))
    } else {
        write_human(&mut stdout, &path, version, &report)
    };
    if let Err(error) = written.and_then(|()| stdout.flush()) {
        eprintln!("Could not write validation report: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}
fn parse(arguments: &[OsString]) -> Result<(PathBuf, bool, Vec<PathBuf>), String> {
    let (mut path, mut json, mut options) = (None, false, true);
    let mut recipes = Vec::new();
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        if options && argument == "--recipe" {
            let recipe = arguments
                .next()
                .filter(|v| !v.is_empty())
                .ok_or("Missing --recipe file path.")?;
            recipes.push(PathBuf::from(recipe));
        } else if options && argument == "--json" {
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
    Ok((path.ok_or("Missing input path.")?, json, recipes))
}
fn write_human(
    out: &mut impl Write,
    path: &Path,
    version: u32,
    report: &DiagnosticReport,
) -> io::Result<()> {
    if report.is_ok() {
        return writeln!(out, "Valid .mix v{version} material: {}", path.display());
    }
    writeln!(out, "Invalid material: {}", path.display())?;
    super::human_diagnostics::write(out, report)?;
    Ok(())
}
