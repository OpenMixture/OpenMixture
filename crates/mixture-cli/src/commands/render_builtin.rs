//! Built-in checker orchestration, PNG encoding, file I/O, and presentation.

use mixture_core::{Diagnostic, DiagnosticCode, DiagnosticReport, SafetyLimits, Stage};
use mixture_wgpu::{CheckerRequest, ContextReport, ExecutionReport, GpuContext, GpuContextOptions};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

const HELP: &str =
    "Usage: mixture render-builtin checker [--size <pixels>] --out <file.png> [--json]
          [--backend auto|vulkan|metal|dx12|none]
          [--power-preference high-performance|low-power] [--software]

Render one opaque 8x8 black/white checker using wgpu compute and rgba16float readback.
Default size: 64. Allowed size: 1..=2048. PNG encoding happens in the CLI.
The output parent directory must exist. An existing output file is replaced.
Exit 0: PNG written; 1: GPU/readback/encoding/I/O failure; 2: invalid request.
--json writes one structured report to stdout; usage errors remain on stderr.";

struct Arguments {
    request: CheckerRequest,
    output: PathBuf,
    gpu: GpuContextOptions,
    json: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema_version: u32,
    request: CheckerRequest,
    output: PathBuf,
    written_bytes: Option<u64>,
    context: Option<ContextReport>,
    execution: Option<ExecutionReport>,
    #[serde(flatten)]
    diagnostics: DiagnosticReport,
}

pub(crate) fn run(arguments: &[OsString]) -> ExitCode {
    if matches!(arguments, [arg] if arg == "--help" || arg == "-h")
        || matches!(arguments, [name, arg] if name == "checker" && (arg == "--help" || arg == "-h"))
    {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let args = match parse(arguments) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let mut report = Report {
        schema_version: 1,
        request: args.request,
        output: args.output.clone(),
        written_bytes: None,
        context: None,
        execution: None,
        diagnostics: DiagnosticReport::new([]),
    };
    let failure = execute(&args, &mut report).err();
    let exit = match &failure {
        None => 0,
        Some(error) if error.stage == Stage::Validation => 2,
        Some(_) => 1,
    };
    report.diagnostics = DiagnosticReport::new(failure.map(|error| *error));
    let mut stdout = io::stdout().lock();
    let written = if args.json {
        serde_json::to_writer_pretty(&mut stdout, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(stdout))
    } else {
        write_human(&mut stdout, &report)
    };
    if let Err(error) = written.and_then(|()| stdout.flush()) {
        eprintln!("Could not write render report: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}

fn execute(args: &Arguments, report: &mut Report) -> Result<(), Box<Diagnostic>> {
    let limits = SafetyLimits::default();
    args.request
        .validate(&limits)
        .map_err(|error| error.diagnostic().clone())?;
    let mut context = match pollster::block_on(GpuContext::request(args.gpu)) {
        Ok(context) => context,
        Err(error) => {
            report.context = Some(error.report().clone());
            return Err(Box::new(error.diagnostic().clone()));
        }
    };
    report.context = Some(context.report().clone());
    let output = pollster::block_on(context.render_checker(args.request, &limits))
        .map_err(|error| error.diagnostic().clone())?;
    report.execution = Some(output.report().clone());
    let png = encode_png(args.request.width, args.request.height, output.pixels())?;
    std::fs::write(&args.output, &png)
        .map_err(|error| encoding_error("Could not write the PNG output file.", error))?;
    report.written_bytes = Some(png.len() as u64);
    Ok(())
}

fn parse(arguments: &[OsString]) -> Result<Arguments, String> {
    let Some((name, rest)) = arguments.split_first() else {
        return Err("Missing built-in name.".into());
    };
    if name != "checker" {
        return Err("Only the checker built-in is implemented.".into());
    }
    let mut request = CheckerRequest::default();
    let mut output = None;
    let mut gpu = GpuContextOptions::default();
    let mut json = false;
    let mut seen = BTreeSet::new();
    let mut args = rest.iter();
    while let Some(argument) = args.next() {
        let flag = argument.to_str().ok_or("Option names must be UTF-8.")?;
        if !seen.insert(flag) {
            return Err(format!("Duplicate option: {flag}"));
        }
        match flag {
            "--json" => json = true,
            "--size" => {
                let value = args
                    .next()
                    .and_then(|value| value.to_str())
                    .ok_or("Missing size.")?;
                let size = value
                    .parse::<u32>()
                    .map_err(|_| "Size must be an unsigned integer.")?;
                request = CheckerRequest {
                    width: size,
                    height: size,
                };
            }
            "--out" => {
                let value = args
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or("Missing output path.")?;
                output = Some(PathBuf::from(value));
            }
            _ if super::gpu_options::parse_option(flag, &mut args, &mut gpu)? => {}
            _ => return Err(format!("Unknown render option: {flag}")),
        }
    }
    Ok(Arguments {
        request,
        output: output.ok_or("--out is required.")?,
        gpu,
        json,
    })
}

fn encoding_error(
    message: &str,
    error: impl std::error::Error + Send + Sync + 'static,
) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::EncodingFailed, Stage::Encoding, message)
        .with_evidence("sourceMessage", error.to_string())
        .with_source(error)
        .with_suggestion(
            "Check the output path, parent directory, permissions, and available disk space.",
        )
}

fn encode_png(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>, Box<Diagnostic>> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder
        .write_header()
        .map_err(|error| encoding_error("Could not encode PNG header.", error))?;
    writer
        .write_image_data(pixels)
        .map_err(|error| encoding_error("Could not encode PNG pixels.", error))?;
    writer
        .finish()
        .map_err(|error| encoding_error("Could not finish PNG encoding.", error))?;
    Ok(bytes)
}

fn write_human(out: &mut impl Write, report: &Report) -> io::Result<()> {
    if let Some(bytes) = report.written_bytes {
        writeln!(
            out,
            "Wrote {} ({}x{}, {bytes} PNG bytes)",
            report.output.display(),
            report.request.width,
            report.request.height
        )?;
    }
    if let Some(execution) = &report.execution {
        writeln!(
            out,
            "Adapter: {} ({})\nPasses: {}\nReadback: {} bytes ({} mapped, row stride {})\nTotal: {:.3} ms",
            execution.adapter.name,
            execution.adapter.backend,
            execution.pass_count,
            execution.readback_bytes,
            execution.mapped_bytes,
            execution.padded_bytes_per_row,
            execution.timings.total_ms
        )?;
    }
    if report.execution.is_none()
        && let Some(adapter) = report.context.as_ref().and_then(ContextReport::adapter)
    {
        writeln!(
            out,
            "Selected adapter: {} ({})",
            adapter.name, adapter.backend
        )?;
    }
    for diagnostic in report.diagnostics.diagnostics() {
        writeln!(out, "{diagnostic}")?;
        for (key, value) in &diagnostic.evidence {
            writeln!(out, "  {key}: {value:?}")?;
        }
        if let Some(suggestion) = &diagnostic.suggestion {
            writeln!(out, "Suggestion: {suggestion}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checker_png_encoding_preserves_dimensions_channels_and_srgb() {
        let pixels = [0, 0, 0, 255, 255, 255, 255, 255];
        let encoded = encode_png(2, 1, &pixels).unwrap();
        let mut decoder = png::Decoder::new(std::io::Cursor::new(encoded))
            .read_info()
            .unwrap();
        assert!(decoder.info().srgb.is_some());
        let mut decoded = vec![0; decoder.output_buffer_size().unwrap()];
        let info = decoder.next_frame(&mut decoded).unwrap();
        assert_eq!((info.width, info.height), (2, 1));
        assert_eq!(info.color_type, png::ColorType::Rgba);
        assert_eq!(decoded, pixels);
        let failure = encode_png(2, 1, &pixels[..4]).unwrap_err();
        assert_eq!(failure.code, DiagnosticCode::EncodingFailed);
        assert!(std::error::Error::source(&failure).is_some());
    }
}
