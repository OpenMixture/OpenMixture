//! A separate native application using only public Mixture APIs.
mod cpu;
mod gpu;

use mixture_core::{CompileError, DocumentError};
use mixture_wgpu::{BackendPreference, GpuContextError, GpuContextOptions};
use serde_json::{Value, json};
use std::{error::Error, ffi::OsString, path::PathBuf, process::ExitCode};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const HELP: &str = "Usage: mixture-native-consumer check
       mixture-native-consumer gpu <auto|metal|vulkan|dx12|none> <hardware|software> [expected-adapter]
       mixture-native-consumer latest <auto|metal|vulkan|dx12|none> <hardware|software> [expected-adapter]
       mixture-native-consumer measure <file.mix> <new-output-directory> <backend> <hardware|software> [expected-adapter]

latest runs a bounded freshness sequence on consumer-owned input.
check is CPU-only. gpu checks the consumer-owned input at 65x3.
measure returns four 1K RGBA8 channels for a first and reused-renderer call.
GPU modes require an explicit policy. No environment adapter overrides are read.";

#[derive(Debug)]
enum Mode {
    Check,
    Gpu(Selection),
    Latest(Selection),
    Measure(PathBuf, PathBuf, Selection),
}

#[derive(Debug)]
struct Selection {
    options: GpuContextOptions,
    expected_adapter: Option<String>,
}

fn selection(args: &[OsString]) -> Result<Selection> {
    if !(2..=3).contains(&args.len()) {
        return Err(HELP.into());
    }
    let backend = match args[0].to_str() {
        Some("auto") => BackendPreference::Auto,
        Some("metal") => BackendPreference::Metal,
        Some("vulkan") => BackendPreference::Vulkan,
        Some("dx12") => BackendPreference::Dx12,
        Some("none") => BackendPreference::None,
        _ => return Err(HELP.into()),
    };
    let software_adapter = match args[1].to_str() {
        Some("hardware") => false,
        Some("software") => true,
        _ => return Err(HELP.into()),
    };
    let expected_adapter = args
        .get(2)
        .map(|arg| -> Result<String> {
            arg.to_str()
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| "expected adapter must be nonempty UTF-8".into())
        })
        .transpose()?;
    Ok(Selection {
        options: GpuContextOptions {
            backend,
            software_adapter,
            ..Default::default()
        },
        expected_adapter,
    })
}

fn parse(args: &[OsString]) -> Result<Mode> {
    match args {
        [] => Ok(Mode::Check),
        [command] if command == "check" => Ok(Mode::Check),
        [command, rest @ ..] if command == "latest" => Ok(Mode::Latest(selection(rest)?)),
        [command, rest @ ..] if command == "gpu" => Ok(Mode::Gpu(selection(rest)?)),
        [command, input, out, rest @ ..] if command == "measure" => {
            Ok(Mode::Measure(input.into(), out.into(), selection(rest)?))
        }
        _ => Err(HELP.into()),
    }
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn failure(error: &(dyn Error + 'static)) -> Value {
    let mut report = json!({"schemaVersion": 1, "ok": false, "message": error.to_string()});
    if let Some(error) = error.downcast_ref::<DocumentError>() {
        report["diagnostics"] = json!(error.report().diagnostics());
    } else if let Some(error) = error.downcast_ref::<CompileError>() {
        report["diagnostics"] = json!(error.report().diagnostics());
    } else if let Some(error) = error.downcast_ref::<GpuContextError>() {
        report["context"] = json!(error.report());
        report["diagnostics"] = json!([error.diagnostic()]);
    } else if let Some(error) = error.downcast_ref::<gpu::RenderFailure>() {
        report["context"] = json!(error.context);
        report["diagnostics"] = json!([error.error.diagnostic()]);
    }
    let mut sources = Vec::new();
    let mut source = error.source();
    while let Some(error) = source {
        sources.push(error.to_string());
        source = error.source();
    }
    report["sourceChain"] = json!(sources);
    report
}

fn main() -> ExitCode {
    let mode = match parse(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(mode) => mode,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let result = match mode {
        Mode::Check => cpu::check(),
        Mode::Latest(selection) => pollster::block_on(gpu::latest(&selection)),
        Mode::Gpu(selection) => pollster::block_on(gpu::check(&selection)),
        Mode::Measure(input, out, selection) => {
            pollster::block_on(gpu::measure(&input, &out, &selection))
        }
    };
    let (report, code) = match result {
        Ok(report) => (report, ExitCode::SUCCESS),
        Err(error) => (failure(error.as_ref()), ExitCode::FAILURE),
    };
    match serde_json::to_writer_pretty(std::io::stdout().lock(), &report) {
        Ok(()) => code,
        Err(error) => {
            eprintln!("could not write consumer report: {error}");
            ExitCode::FAILURE
        }
    }
}
