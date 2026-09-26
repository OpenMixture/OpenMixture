//! Thin CPU-only plan compilation and presentation using the public core API.
use mixture_core::{DiagnosticReport, RenderPlan, compile, plan::PassOrigin};
use serde::Serialize;
use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

const HELP: &str = "Usage: mixture inspect <file.mix> --plan [--json]
                       [--size <pixels|widthxheight>] [--output <channel,...>]
                       [--set <publicId=JSON>]...

Compile a deterministic RenderPlan without initializing a GPU or changing the file.
Defaults: 64x64, baseColor only, no overrides, standard core safety limits.
Channels: baseColor, normal, roughness, metallic, height, ambientOcclusion, opacity, emissive.
--output accepts a comma-separated list of unique channels; its order does not matter.
--set accepts a declared exposed ID and an exact JSON value, e.g. --set 'frequency=16'.
--json writes {schemaVersion, plan, ok, diagnostics}; plan is null on failure.
Exit 0: compiled; 1: file/report I/O failure; 2: invalid invocation, source, or request.
Use -- before a path starting with '-'. --plan is required.";

struct Options {
    path: PathBuf,
    json: bool,
    compile: super::compile_options::CompileOptions,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema_version: u32,
    plan: Option<RenderPlan>,
    #[serde(flatten)]
    diagnostics: DiagnosticReport,
}
pub(crate) fn run(arguments: &[OsString]) -> ExitCode {
    if matches!(arguments, [arg] if arg == "--help" || arg == "-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let options = match parse(arguments) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let result = options
        .compile
        .request()
        .map_err(|error| (error.report().diagnostics().to_vec(), 2))
        .and_then(|request| {
            super::document_io::load(&options.path, &request.limits).and_then(|document| {
                compile(&document, &request)
                    .map_err(|error| (error.report().diagnostics().to_vec(), 2))
            })
        });
    let (plan, diagnostics, exit) = match result {
        Ok(plan) => (Some(plan), Vec::new(), 0),
        Err((diagnostics, exit)) => (None, diagnostics, exit),
    };
    let report = Report {
        schema_version: 3,
        plan,
        diagnostics: DiagnosticReport::new(diagnostics.into_iter().map(|mut d| {
            d.document_path = Some(options.path.to_string_lossy().into_owned());
            d
        })),
    };
    let mut stdout = io::stdout().lock();
    let written = if options.json {
        serde_json::to_writer_pretty(&mut stdout, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(stdout))
    } else {
        human(&mut stdout, &report)
    };
    if let Err(error) = written.and_then(|()| stdout.flush()) {
        eprintln!("Could not write plan report: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}
fn parse(arguments: &[OsString]) -> Result<Options, String> {
    let (mut path, mut plan, mut json) = (None, false, false);
    let mut compile = super::compile_options::CompileOptions::default();
    let mut options = true;
    let mut args = arguments.iter();
    while let Some(arg) = args.next() {
        match arg.to_str().filter(|_| options) {
            Some("--") => options = false,
            Some("--plan") if !plan => plan = true,
            Some("--json") if !json => json = true,
            Some(option) if compile.parse_option(option, &mut args)? => {}
            Some(option) if option.starts_with('-') => {
                return Err(format!("Unknown or duplicate inspect option: {option}"));
            }
            _ if path.is_none() && !arg.is_empty() => path = Some(PathBuf::from(arg)),
            _ => return Err("Expected exactly one nonempty input path.".into()),
        }
    }
    if !plan {
        return Err("Missing required --plan option.".into());
    }
    Ok(Options {
        path: path.ok_or("Missing input path.")?,
        json,
        compile,
    })
}
fn human(out: &mut impl Write, report: &Report) -> io::Result<()> {
    if let Some(plan) = &report.plan {
        writeln!(
            out,
            "Compiled RenderPlan v{}: {}x{}, {} passes",
            plan.version(),
            plan.size()[0],
            plan.size()[1],
            plan.passes().len()
        )?;
        writeln!(out, "Hash: {}", plan.hash())?;
        for pass in plan.passes() {
            let origin = match &pass.origin {
                PassOrigin::Node { node } => node.id.clone(),
                PassOrigin::InputDefault { node, port } => format!("{}.{} default", node.id, port),
            };
            writeln!(
                out,
                "  pass {}: {origin}, {:?} -> resource {}",
                pass.id.index(),
                pass.kernel.id(),
                pass.output.index()
            )?;
        }
        for output in plan.outputs() {
            writeln!(
                out,
                "  {} -> resource {} ({:?})",
                output.channel.as_str(),
                output.resource.index(),
                output.input
            )?;
        }
        writeln!(
            out,
            "Estimated GPU bytes: peak {}, cumulative {}",
            plan.estimates().peak_bytes,
            plan.estimates().cumulative_bytes
        )?;
    } else {
        writeln!(out, "Could not compile material plan.")?;
        super::human_diagnostics::write(out, &report.diagnostics)?;
    }
    Ok(())
}
