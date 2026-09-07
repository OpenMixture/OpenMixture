//! CLI presentation only. GPU policy and acquisition belong to mixture-wgpu.

use std::{
    ffi::OsString,
    io::{self, Write},
    process::ExitCode,
};

use mixture_wgpu::{ContextReport, DoctorVerdict, GpuContext, GpuContextOptions};

const HELP: &str = "Usage: mixture doctor [--json] [--backend auto|vulkan|metal|dx12|none]
                      [--power-preference high-performance|low-power] [--software] [--skip-probe]

  --json               Write one structured report to stdout
  --skip-probe         Acquire context only; report unverified
  --backend            Permit one native backend (default: auto); none disables GPU
  --power-preference   Adapter preference (default: high-performance)
  --software           Require a software adapter through the same wgpu path
  --help, -h           Show this help without initializing a GPU

Exit 0: verified compute/readback (healthy), or explicit --skip-probe (unverified).
Exit 1: acquisition/probe failed (unhealthy), or output failed.
Exit 2: invalid arguments; usage errors go to stderr, including with --json.
WGPU_* environment variables do not override these options.";

pub(crate) fn run(arguments: &[OsString]) -> ExitCode {
    if matches!(arguments, [arg] if arg == "--help" || arg == "-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let (options, json, skip_probe) = match parse(arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let result = pollster::block_on(GpuContext::request(options));
    let report = match result {
        Ok(mut context) if !skip_probe => pollster::block_on(context.probe_checker()),
        Ok(context) => context.report().clone(),
        Err(error) => error.report().clone(),
    };
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let written = if json {
        serde_json::to_writer_pretty(&mut output, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(output))
    } else {
        write_human(&mut output, &report)
    };
    if let Err(error) = written.and_then(|()| output.flush()) {
        eprintln!("Could not write doctor report: {error}");
        return ExitCode::FAILURE;
    }
    if report.diagnostics().is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse(arguments: &[OsString]) -> Result<(GpuContextOptions, bool, bool), String> {
    let mut options = GpuContextOptions::default();
    let mut json = false;
    let mut skip_probe = false;
    let mut seen = std::collections::BTreeSet::new();
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        let flag = argument.to_str().ok_or("Arguments must be valid UTF-8.")?;
        if !seen.insert(flag) {
            return Err(format!("Duplicate option: {flag}"));
        }
        match flag {
            "--json" => json = true,
            "--skip-probe" => skip_probe = true,
            _ if super::gpu_options::parse_option(flag, &mut args, &mut options)? => {}
            _ => return Err(format!("Unknown doctor option: {flag}")),
        }
    }
    Ok((options, json, skip_probe))
}

fn write_human(out: &mut impl Write, report: &ContextReport) -> io::Result<()> {
    let verdict = match report.verdict() {
        DoctorVerdict::Healthy => "healthy",
        DoctorVerdict::Unverified => "unverified",
        DoctorVerdict::Unhealthy => "unhealthy",
    };
    writeln!(out, "Mixture doctor: {verdict}")?;
    writeln!(
        out,
        "Compute probe: {}\nReadback probe: {}",
        report.compute_probe(),
        report.readback_probe()
    )?;
    writeln!(out, "Requested policy and device requirements:")?;
    serde_json::to_writer_pretty(&mut *out, report.requested()).map_err(io::Error::other)?;
    writeln!(out)?;
    if let Some(adapter) = report.adapter() {
        writeln!(
            out,
            "Selected adapter: {} ({}, {})",
            adapter.name, adapter.device_type, adapter.backend
        )?;
        writeln!(out, "Adapter capabilities:")?;
        serde_json::to_writer_pretty(&mut *out, adapter).map_err(io::Error::other)?;
        writeln!(out)?;
    }
    if let Some(device) = report.device() {
        writeln!(out, "Selected device features and limits:")?;
        serde_json::to_writer_pretty(&mut *out, device).map_err(io::Error::other)?;
        writeln!(out)?;
    }
    if let Some(execution) = report.execution() {
        writeln!(out, "Checker execution and readback:")?;
        serde_json::to_writer_pretty(&mut *out, execution).map_err(io::Error::other)?;
        writeln!(out)?;
    }
    for diagnostic in report.diagnostics().diagnostics() {
        writeln!(out, "{diagnostic}")?;
        for (key, value) in &diagnostic.evidence {
            writeln!(out, "  {key}: {value:?}")?;
        }
        if let Some(suggestion) = &diagnostic.suggestion {
            writeln!(out, "  Suggestion: {suggestion}")?;
        }
    }
    Ok(())
}
