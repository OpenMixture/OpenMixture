//! Source I/O -> public compiler -> explicit renderer -> per-channel PNG reports.
use super::{
    compile_options::CompileOptions,
    png_output::{encode_png, encoding_error},
};
use mixture_core::{
    Diagnostic, DiagnosticReport, InputSource, OutputChannel, compile, plan::PlanHash,
    registry::PortKind,
};
use mixture_wgpu::{
    ContextReport, GpuContext, GpuContextOptions, OutputEncoding, RenderReport, Renderer,
};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};
const HELP:&str="Usage: mixture render <file.mix> --out <directory> [--json]
                      [--size <pixels|widthxheight>] [--output <channel,...>]
                      [--set <publicId=JSON>]...
                      [--backend auto|vulkan|metal|dx12|none]
                      [--power-preference high-performance|low-power] [--software]

Compile the source before acquiring a GPU, then execute its RenderPlan through wgpu.
Default: 64x64, baseColor only. Compile options match inspect --plan.
Write <channel>.png for each requested channel in canonical channel order.
Create the output directory if needed; replace same-named output files.
Color RGB is sRGB, alpha linear; scalar/normal PNGs are linear (gamma 1.0).
No other files are removed. A failed write reports any files already completed.
Use -- before a path starting with '-'. --out is required.
Exit 0: all PNGs written; 1: GPU/readback/encoding/I/O failure; 2: invalid invocation/source/request.
--json writes one report; usage errors remain on stderr.";
struct Options {
    path: PathBuf,
    out: PathBuf,
    compile: CompileOptions,
    gpu: GpuContextOptions,
    json: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputFile {
    channel: OutputChannel,
    kind: PortKind,
    source: InputSource,
    size: [u32; 2],
    encoding: OutputEncoding,
    path: PathBuf,
    written_bytes: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema_version: u32,
    input: PathBuf,
    output_directory: PathBuf,
    plan_hash: Option<PlanHash>,
    context: Option<ContextReport>,
    execution: Option<RenderReport>,
    outputs: Vec<OutputFile>,
    #[serde(flatten)]
    diagnostics: DiagnosticReport,
}
pub(crate) fn run(arguments: &[OsString]) -> ExitCode {
    if matches!(arguments,[arg] if arg=="--help"||arg=="-h") {
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
    let mut report = Report {
        schema_version: 1,
        input: options.path.clone(),
        output_directory: options.out.clone(),
        plan_hash: None,
        context: None,
        execution: None,
        outputs: Vec::new(),
        diagnostics: DiagnosticReport::new([]),
    };
    let json = options.json;
    let exit = match execute(options, &mut report) {
        Ok(()) => 0,
        Err((diagnostics, exit)) => {
            report.diagnostics = DiagnosticReport::new(diagnostics.into_iter().map(|mut d| {
                d.document_path = Some(report.input.to_string_lossy().into_owned());
                d
            }));
            exit
        }
    };
    let mut stdout = io::stdout().lock();
    let written = if json {
        serde_json::to_writer_pretty(&mut stdout, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(stdout))
    } else {
        human(&mut stdout, &report)
    };
    if let Err(error) = written.and_then(|()| stdout.flush()) {
        eprintln!("Could not write render report: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}
fn execute(options: Options, report: &mut Report) -> Result<(), (Vec<Diagnostic>, u8)> {
    let request = options
        .compile
        .request()
        .map_err(|e| (e.report().diagnostics().to_vec(), 2))?;
    let document = super::document_io::load(&options.path, &request.limits)?;
    let plan = compile(&document, &request).map_err(|e| (e.report().diagnostics().to_vec(), 2))?;
    report.plan_hash = Some(plan.hash().clone());
    let context = match pollster::block_on(GpuContext::request(options.gpu)) {
        Ok(context) => context,
        Err(error) => {
            report.context = Some(error.report().clone());
            return Err((vec![error.diagnostic().clone()], 1));
        }
    };
    report.context = Some(context.report().clone());
    let mut renderer = Renderer::new(context);
    let output = pollster::block_on(renderer.render(&plan))
        .map_err(|e| (vec![e.diagnostic().clone()], 1))?;
    report.execution = Some(output.report().clone());
    std::fs::create_dir_all(&options.out).map_err(|e| {
        (
            vec![encoding_error("Could not create the output directory.", e)],
            1,
        )
    })?;
    for channel in output.channels() {
        let path = options
            .out
            .join(format!("{}.png", channel.channel.as_str()));
        let png = encode_png(
            channel.size[0],
            channel.size[1],
            channel.pixels(),
            channel.encoding,
        )
        .map_err(|e| (vec![*e], 1))?;
        std::fs::write(&path, &png).map_err(|e| {
            (
                vec![
                    encoding_error("Could not write a channel PNG.", e)
                        .with_evidence("outputPath", path.to_string_lossy().as_ref()),
                ],
                1,
            )
        })?;
        report.outputs.push(OutputFile {
            channel: channel.channel,
            kind: channel.kind,
            source: channel.source.clone(),
            size: channel.size,
            encoding: channel.encoding,
            path,
            written_bytes: png.len() as u64,
        });
    }
    Ok(())
}
fn parse(arguments: &[OsString]) -> Result<Options, String> {
    let (mut path, mut out, mut json, mut options) = (None, None, false, true);
    let mut compile = CompileOptions::default();
    let mut gpu = GpuContextOptions::default();
    let mut seen = BTreeSet::new();
    let mut args = arguments.iter();
    while let Some(arg) = args.next() {
        if options && arg == "--" {
            options = false;
            continue;
        }
        if options && arg.to_str().is_some_and(|s| s.starts_with('-')) {
            let flag = arg.to_str().ok_or("Option names must be UTF-8.")?;
            if flag != "--set" && !seen.insert(flag) {
                return Err(format!("Duplicate {flag} option."));
            }
            match flag {
                "--json" => json = true,
                "--out" => {
                    out = Some(PathBuf::from(
                        args.next()
                            .filter(|p| !p.is_empty())
                            .ok_or("Missing output directory.")?,
                    ))
                }
                _ if compile.parse_option(flag, &mut args)? => {}
                _ if super::gpu_options::parse_option(flag, &mut args, &mut gpu)? => {}
                _ => return Err(format!("Unknown render option: {flag}")),
            }
        } else if path.is_none() && !arg.is_empty() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err("Expected exactly one nonempty input path.".into());
        }
    }
    Ok(Options {
        path: path.ok_or("Missing input path.")?,
        out: out.ok_or("--out is required.")?,
        compile,
        gpu,
        json,
    })
}
fn human(out: &mut impl Write, report: &Report) -> io::Result<()> {
    if let Some(execution) = &report.execution {
        writeln!(
            out,
            "Plan: {}\nAdapter: {} ({})\nPasses: {}\nGPU bytes: peak {}, cumulative {}\nTotal: {:.3} ms",
            execution.plan_hash,
            execution.adapter.name,
            execution.adapter.backend,
            execution.pass_count,
            execution.estimates.peak_bytes,
            execution.estimates.cumulative_bytes,
            execution.timings.total_ms
        )?;
    } else if let Some(adapter) = report.context.as_ref().and_then(ContextReport::adapter) {
        writeln!(
            out,
            "Selected adapter: {} ({})",
            adapter.name, adapter.backend
        )?;
    }
    for file in &report.outputs {
        writeln!(
            out,
            "Wrote {} ({}x{}, {:?}, {} PNG bytes)",
            file.path.display(),
            file.size[0],
            file.size[1],
            file.encoding,
            file.written_bytes
        )?;
    }
    super::human_diagnostics::write(out, &report.diagnostics)?;
    Ok(())
}
