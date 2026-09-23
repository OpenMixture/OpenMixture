//! Explicit local file adapters; the shared asset codec owns archive semantics.
use super::compile_options::CompileOptions;
use mixture_asset::{AssetError, AssetLimits, OwnedAsset};
use mixture_core::ImageBinding;
use mixture_wgpu::GpuContextOptions;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const HELP:&str="Usage: mixture asset pack <file.mix> --out <file.mixpack> [--size <pixels|widthxheight>] [--image <id> <raw.rgba>]... [--json]
       mixture asset inspect <file.mixpack> [--plan] [--size <pixels|widthxheight>] [--output <channels>] [--set <id=JSON>] [--json]
       mixture asset render <file.mixpack> --out <directory> [--size <pixels|widthxheight>] [--output <channels>] [--set <id=JSON>] [--json]
                      [--backend auto|vulkan|metal|dx12|none] [--software] [--power-preference high-performance|low-power]
All commands accept --package-bytes, --manifest-bytes, --package-buffer-bytes (unsigned byte ceilings).
Defaults: 64x64, baseColor; package v1 limits. Resource images are exact same-size packed rgba8-linear bytes.
pack requires every default resource ID, never discovers adjacent files, and refuses an existing output file.
inspect is CPU-only. --plan applies compile options; otherwise compile options are rejected.
render prepares before GPU acquisition and shares existing PNG output behavior. No archive extraction or network lookup.
Use -- before an input path starting with '-'. Exit 0 success, 1 I/O/GPU, 2 invalid invocation/package/request.";
struct Options {
    operation: String,
    input: PathBuf,
    out: Option<PathBuf>,
    images: Vec<(String, PathBuf)>,
    compile: CompileOptions,
    plan: bool,
    gpu: GpuContextOptions,
    json: bool,
    limits: AssetLimits,
}
struct Failure {
    diagnostics: Vec<Value>,
    exit: u8,
}
impl From<AssetError> for Failure {
    fn from(error: AssetError) -> Self {
        let diagnostics = match error {
            AssetError::Package(d) => vec![json!(d)],
            AssetError::Document(e) => e.report().diagnostics().iter().map(|d| json!(d)).collect(),
            AssetError::Compile(e) => e.report().diagnostics().iter().map(|d| json!(d)).collect(),
        };
        Self {
            diagnostics,
            exit: 2,
        }
    }
}
fn issue(code: &str, message: &str, evidence: Value, exit: u8) -> Failure {
    Failure {
        diagnostics: vec![
            json!({"code":code,"stage":"package","severity":"error","message":message,"evidence":evidence,"suggestion":"Check explicit file paths, metadata and package policy."}),
        ],
        exit,
    }
}
fn io(error: std::io::Error) -> Failure {
    issue(
        "MIX_IO_READ_FAILED",
        "Asset file operation failed",
        json!({"sourceMessage":error.to_string()}),
        1,
    )
}
fn bound(name: &str, observed: u64, configured: u64) -> Result<(), Failure> {
    if observed > configured {
        Err(issue(
            "MIX_PACKAGE_LIMIT_EXCEEDED",
            "Asset policy exceeded",
            json!({"limit":name,"observed":observed,"configured":configured}),
            2,
        ))
    } else {
        Ok(())
    }
}
fn read(path: &Path, max: u64) -> Result<Vec<u8>, Failure> {
    if !std::fs::symlink_metadata(path)
        .map_err(io)?
        .file_type()
        .is_file()
    {
        return Err(issue(
            "MIX_IO_READ_FAILED",
            "Expected an explicit regular input file",
            json!({"path":path}),
            1,
        ));
    }
    let mut file = File::open(path).map_err(io)?;
    let size = file.metadata().map_err(io)?.len();
    bound("fileBytes", size, max)?;
    let length = usize::try_from(size).map_err(|_| {
        issue(
            "MIX_PACKAGE_LIMIT_EXCEEDED",
            "File exceeds host representation",
            json!({"observed":size}),
            2,
        )
    })?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|e| {
        issue(
            "MIX_PACKAGE_ALLOCATION_FAILED",
            "Cannot reserve file bytes",
            json!({"sourceMessage":e.to_string()}),
            2,
        )
    })?;
    bytes.resize(length, 0);
    file.read_exact(&mut bytes).map_err(io)?;
    if file.read(&mut [0u8; 1]).map_err(io)? != 0 {
        return Err(issue(
            "MIX_PACKAGE_INVALID",
            "Input changed while reading",
            json!({"path":path}),
            2,
        ));
    }
    Ok(bytes)
}
pub(crate) fn run(args: &[OsString]) -> ExitCode {
    if args.is_empty()
        || matches!(args,[help] if help=="--help"||help=="-h")
        || matches!(args,[_,help] if help=="--help"||help=="-h")
    {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let options = match parse(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let mut report = json!({"schemaVersion":1,"operation":options.operation,"input":options.input,"ok":false,"diagnostics":[],"asset":null,"plan":null,"render":null});
    let exit = match execute(&options, &mut report) {
        Ok(exit) => {
            report["ok"] = json!(exit == 0);
            exit
        }
        Err(e) => {
            report["diagnostics"] = json!(e.diagnostics);
            e.exit
        }
    };
    let mut stdout = std::io::stdout().lock();
    let result = if options.json {
        serde_json::to_writer_pretty(&mut stdout, &report)
            .map_err(std::io::Error::other)
            .and_then(|_| writeln!(stdout))
    } else {
        writeln!(
            stdout,
            "Asset {}: {}\n{}",
            options.operation,
            if exit == 0 { "ok" } else { "failed" },
            report
        )
    };
    if let Err(e) = result.and_then(|_| stdout.flush()) {
        eprintln!("Could not write asset report: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(exit)
}
fn execute(o: &Options, report: &mut Value) -> Result<u8, Failure> {
    let limits = o.limits.with_retained_bytes(0)?;
    if o.operation == "pack" {
        return pack(o, limits, report);
    }
    // File length is known before reservation; owned codec additionally charges D+M.
    let read_limit = limits
        .package
        .package_bytes
        .min(limits.package.package_buffer_bytes);
    let asset = OwnedAsset::from_vec(read(&o.input, read_limit)?, &limits)?;
    report["asset"] = json!(asset.view().inspect());
    if o.operation == "inspect" && !o.plan {
        return Ok(0);
    }
    let request = o
        .compile
        .request_ref()
        .map_err(|e| Failure::from(AssetError::Compile(e)))?;
    let prepared = asset.prepare(&request)?;
    report["plan"] = json!(prepared.plan());
    drop(asset);
    if o.operation == "inspect" {
        return Ok(0);
    }
    let out = o
        .out
        .clone()
        .ok_or_else(|| issue("MIX_PACKAGE_INVALID", "Missing output path", json!({}), 2))?;
    let (render, exit) = super::render::prepared(o.input.clone(), out, o.gpu, prepared);
    report["render"] = serde_json::to_value(render).map_err(|e| io(std::io::Error::other(e)))?;
    report["diagnostics"] = report["render"]["diagnostics"].clone();
    Ok(exit)
}
fn pack(o: &Options, limits: AssetLimits, report: &mut Value) -> Result<u8, Failure> {
    let request = o
        .compile
        .request_ref()
        .map_err(|e| Failure::from(AssetError::Compile(e)))?;
    let [width, height] = request.size;
    bound(
        "outputDimension",
        u64::from(width.max(height)),
        limits.safety.output_dimension,
    )?;
    if width == 0 || height == 0 {
        return Err(issue(
            "MIX_PACKAGE_INVALID",
            "Positive image dimensions required",
            json!({}),
            2,
        ));
    }
    bound(
        "resourceCount",
        o.images.len() as u64,
        limits.resources.resource_count,
    )?;
    let image_bytes = u64::from(width) * u64::from(height) * 4;
    let total = image_bytes * o.images.len() as u64;
    bound("resourceBytes", total, limits.resources.resource_bytes)?;
    let source = read(
        &o.input,
        limits
            .safety
            .decoded_bytes
            .min(limits.package.package_buffer_bytes),
    )?;
    let doc = mixture_core::MaterialDocument::decode(&source, &limits.safety)
        .and_then(|d| d.into_validated(&limits.safety))
        .map_err(|e| Failure::from(AssetError::Document(e)))?;
    let ids = mixture_core::resources::document_image_ids(&doc)
        .map_err(|e| Failure::from(AssetError::Compile(e)))?;
    if ids != o.images.iter().map(|(id, _)| id.clone()).collect() {
        return Err(issue(
            "MIX_PACKAGE_INVALID",
            "Explicit images must match the full default resource set",
            json!({}),
            2,
        ));
    }
    let remaining = limits.with_retained_bytes(total)?;
    remaining.with_retained_bytes(source.len() as u64)?;
    let data = o
        .images
        .iter()
        .map(|(_, path)| read(path, image_bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let retained = data.iter().map(|b| b.capacity() as u64).sum();
    let remaining = limits.with_retained_bytes(retained)?;
    let bindings: Vec<_> = o
        .images
        .iter()
        .zip(&data)
        .map(|((id, _), data)| ImageBinding {
            id,
            width,
            height,
            format: "rgba8-linear",
            bytes_per_row: u64::from(width) * 4,
            data,
        })
        .collect();
    let bytes = mixture_asset::write(&source, &bindings, &remaining)?;
    let out = o
        .out
        .as_ref()
        .ok_or_else(|| issue("MIX_PACKAGE_INVALID", "Missing output path", json!({}), 2))?;
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(out)
        .map_err(|e| {
            issue(
                "MIX_IO_WRITE_FAILED",
                "Cannot create asset output",
                json!({"path":out,"sourceMessage":e.to_string()}),
                1,
            )
        })?;
    file.write_all(&bytes)
        .and_then(|_| file.flush())
        .map_err(|e| {
            issue(
                "MIX_IO_WRITE_FAILED",
                "Cannot complete asset output",
                json!({"path":out,"sourceMessage":e.to_string()}),
                1,
            )
        })?;
    report["output"] = json!(out);
    report["writtenBytes"] = json!(bytes.len());
    Ok(0)
}
fn parse(arguments: &[OsString]) -> Result<Options, String> {
    let operation = arguments
        .first()
        .and_then(|a| a.to_str())
        .filter(|a| matches!(*a, "pack" | "inspect" | "render"))
        .ok_or("Expected pack, inspect or render")?
        .to_owned();
    let mut o = Options {
        operation,
        input: PathBuf::new(),
        out: None,
        images: Vec::new(),
        compile: CompileOptions::default(),
        plan: false,
        gpu: GpuContextOptions::default(),
        json: false,
        limits: AssetLimits::default(),
    };
    let mut options = true;
    let mut seen = BTreeSet::new();
    let mut ids = BTreeSet::new();
    // Compile/GPU adapters accept a slice iterator; preserve OS strings for paths.
    let rest = &arguments[1..];
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        if options && arg == "--" {
            options = false;
            continue;
        }
        if options && arg.to_str().is_some_and(|a| a.starts_with('-')) {
            let flag = arg.to_str().ok_or("Flags must be UTF-8")?;
            if !matches!(flag, "--set" | "--image") && !seen.insert(flag.to_owned()) {
                return Err(format!("Duplicate {flag}"));
            }
            if flag == "--set" {
                seen.insert(flag.to_owned());
            }
            match flag {
                "--json" => o.json = true,
                "--plan" if o.operation == "inspect" => o.plan = true,
                "--out" if o.operation != "inspect" => {
                    o.out = Some(PathBuf::from(
                        args.next()
                            .filter(|a| !a.is_empty())
                            .ok_or("Missing --out path")?,
                    ))
                }
                "--image" if o.operation == "pack" => {
                    let id = args
                        .next()
                        .and_then(|v| v.to_str())
                        .filter(|s| mixture_core::resources::valid_image_id(s))
                        .ok_or("Expected logical image ID")?;
                    if !ids.insert(id.to_owned()) {
                        return Err("Duplicate image ID".into());
                    }
                    let path = args
                        .next()
                        .filter(|p| !p.is_empty())
                        .ok_or("Missing raw image path")?;
                    o.images.push((id.to_owned(), PathBuf::from(path)));
                }
                "--package-bytes" | "--manifest-bytes" | "--package-buffer-bytes" => {
                    let text = args
                        .next()
                        .and_then(|a| a.to_str())
                        .ok_or("Missing byte limit")?;
                    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
                        return Err("Byte limits must be unsigned decimal u64 integers".into());
                    }
                    let value = text.parse::<u64>().map_err(|_| "Byte limit overflow")?;
                    match flag {
                        "--package-bytes" => o.limits.package.package_bytes = value,
                        "--manifest-bytes" => o.limits.package.manifest_bytes = value,
                        _ => o.limits.package.package_buffer_bytes = value,
                    }
                }
                _ if (o.operation != "pack" || flag == "--size")
                    && o.compile.parse_option(flag, &mut args)? => {}
                _ if o.operation == "render"
                    && super::gpu_options::parse_option(flag, &mut args, &mut o.gpu)? => {}
                _ => return Err(format!("Unknown asset option: {flag}")),
            }
        } else if o.input.as_os_str().is_empty() && !arg.is_empty() {
            o.input = PathBuf::from(arg);
        } else {
            return Err("Expected one input path".into());
        }
    }
    if o.input.as_os_str().is_empty() || (o.operation != "inspect" && o.out.is_none()) {
        return Err("Input and required output paths must be explicit".into());
    }
    if o.operation == "inspect"
        && !o.plan
        && seen
            .iter()
            .any(|s| matches!(s.as_str(), "--size" | "--output"))
    {
        return Err("Compile options require --plan".into());
    }
    if o.operation == "inspect" && !o.plan && seen.contains("--set") {
        return Err("Overrides require --plan".into());
    }
    Ok(o)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn arguments(parts: &[&str]) -> Vec<OsString> {
        parts.iter().map(OsString::from).collect()
    }
    #[test]
    fn explicit_asset_invocations_reject_ambiguous_or_inapplicable_options() {
        for args in [
            vec!["inspect", "a", "--set", "x=1"],
            vec!["inspect", "a", "--size", "2"],
            vec!["inspect", "a", "--backend", "none"],
            vec!["pack", "a", "--out", "b", "--output", "height"],
            vec![
                "pack", "a", "--out", "b", "--image", "X", "x", "--image", "X", "y",
            ],
            vec!["render", "a"],
            vec!["inspect", "a", "--package-bytes", "-1"],
            vec!["inspect", "a", "--package-bytes", "18446744073709551616"],
            vec!["inspect", "a", "--json", "--json"],
        ] {
            assert!(parse(&arguments(&args)).is_err(), "{args:?}");
        }
        let escaped = parse(&arguments(&["inspect", "--", "--set"])).unwrap();
        assert_eq!(escaped.input, PathBuf::from("--set"));
        assert!(parse(&arguments(&["inspect", "a", "--plan", "--set", "x=1"])).is_ok());
    }
}
