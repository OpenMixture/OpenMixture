//! Thin CPU-only plan compilation and presentation using the public core API.
use mixture_core::{CompileRequest, DiagnosticReport, RenderPlan, compile, plan::PassOrigin};
use serde::Serialize;
use std::{
    collections::BTreeMap,
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
    size: [u32; 2],
    outputs: Option<String>,
    overrides: BTreeMap<String, serde_json::Value>,
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
    let mut request = CompileRequest {
        size: options.size,
        overrides: options.overrides,
        ..Default::default()
    };
    let channels = options.outputs.map(|s| {
        s.split(',')
            .map(str::parse::<mixture_core::OutputChannel>)
            .collect::<Result<Vec<_>, _>>()
    });
    let result = match channels.transpose() {
        Err(error) => Err((error.report().diagnostics().to_vec(), 2)),
        Ok(channels) => {
            if let Some(channels) = channels {
                request.outputs = channels;
            }
            super::document_io::load(&options.path, &request.limits).and_then(|document| {
                compile(&document, &request)
                    .map_err(|error| (error.report().diagnostics().to_vec(), 2))
            })
        }
    };
    let (plan, diagnostics, exit) = match result {
        Ok(plan) => (Some(plan), Vec::new(), 0),
        Err((diagnostics, exit)) => (None, diagnostics, exit),
    };
    let report = Report {
        schema_version: 1,
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
    let (mut path, mut plan, mut json, mut size, mut outputs) = (None, false, false, None, None);
    let mut overrides = BTreeMap::new();
    let mut options = true;
    let mut args = arguments.iter();
    while let Some(arg) = args.next() {
        match arg.to_str().filter(|_| options) {
            Some("--") => options = false,
            Some("--plan") if !plan => plan = true,
            Some("--json") if !json => json = true,
            Some("--size" | "--output" | "--set") => {
                let name = arg.to_string_lossy();
                let value = args
                    .next()
                    .and_then(|v| v.to_str())
                    .ok_or_else(|| format!("Missing UTF-8 value for {name}."))?;
                match name.as_ref() {
                    "--size" if size.is_none() => size = Some(parse_size(value)?),
                    "--output" if outputs.is_none() => outputs = Some(value.to_owned()),
                    "--set" => {
                        let (id, json) = value
                            .split_once('=')
                            .filter(|(id, _)| !id.is_empty())
                            .ok_or("Expected --set publicId=JSON.")?;
                        let parsed = serde_json::from_str(json)
                            .map_err(|e| format!("Invalid JSON override for {id}: {e}"))?;
                        if overrides.insert(id.to_owned(), parsed).is_some() {
                            return Err(format!("Duplicate override ID: {id}."));
                        }
                    }
                    _ => return Err(format!("Duplicate {name} option.")),
                }
            }
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
        size: size.unwrap_or([64, 64]),
        outputs,
        overrides,
    })
}
fn parse_size(value: &str) -> Result<[u32; 2], String> {
    let (w, h) = value.split_once('x').unwrap_or((value, value));
    let number = |s: &str| {
        s.parse::<u32>().map_err(|_| {
            "Expected --size pixels or widthxheight using unsigned 32-bit integers.".to_owned()
        })
    };
    Ok([number(w)?, number(h)?])
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
        for diagnostic in report.diagnostics.diagnostics() {
            writeln!(out, "{diagnostic}")?;
            if let Some(node) = &diagnostic.node_id {
                writeln!(out, "  node: {node}")?;
            }
            if let Some(parameter) = &diagnostic.parameter_id {
                writeln!(out, "  parameter: {parameter}")?;
            }
            for (key, value) in &diagnostic.evidence {
                writeln!(out, "  {key}: {value:?}")?;
            }
            if let Some(suggestion) = &diagnostic.suggestion {
                writeln!(out, "  Suggestion: {suggestion}")?;
            }
        }
    }
    Ok(())
}
