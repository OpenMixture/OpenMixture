//! Verify Cargo archives through an isolated, version-resolved local consumer.
//! No publication, registry writes, production bindings or pixel logic live here.
use crate::{TaskResult, cargo, consumer, gpu_smoke};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Component, Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};
const PACKAGES: [&str; 3] = ["mixture-core", "mixture-wgpu", "mixture-cli"];
const APP: &str = "mixture-native-consumer";
type Policy<'a> = (&'a str, bool, Option<&'a str>);

pub(super) fn check(root: &Path) -> TaskResult {
    run(root, None).map(|_| ())
}
pub(super) fn gpu(
    root: &Path,
    destination: &Path,
    backend: &str,
    software: bool,
    expected: Option<&str>,
) -> TaskResult {
    let status = run(root, Some((backend, software, expected)))?;
    write_json(
        &destination.join("package-status.json"),
        &json!({"ok":true,"completed":true,"packageEvidence":status}),
    )
}

fn run(root: &Path, policy: Option<Policy<'_>>) -> TaskResult<PathBuf> {
    let name = format!(
        "{}-{}-{}",
        if policy.is_some() { "gpu" } else { "cpu" },
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let base = root.join("tmp/package-check");
    fs::create_dir_all(&base)?;
    let directory = base.join(&name);
    fs::create_dir(&directory)?;
    // Invalidate discovery before any fallible staging or policy checks.
    write_json(
        &directory.join("status.json"),
        &json!({"schemaVersion":1,"ok":false,"completed":false}),
    )?;
    write_json(
        &base.join("latest.json"),
        &json!({"ok":false,"completed":false,"status":directory.join("status.json")}),
    )?;
    let lab = env::temp_dir().join(format!("mixture-package-{name}"));
    fs::create_dir(&lab)?;
    let lab = fs::canonicalize(lab)?;
    if lab.starts_with(fs::canonicalize(root)?) {
        return Err(
            "package staging must be outside the producer repository; choose an external TMPDIR"
                .into(),
        );
    }
    let mut status = json!({"schemaVersion":1,"ok":false,"completed":false,"gpuExecuted":false,
        "packagedCratesValidated":false,"mode":if policy.is_some(){"gpu"}else{"cpu"},
        "stagingDirectory":lab,"stagingRemoved":false});
    let status_path = directory.join("status.json");
    write_json(&status_path, &status)?;
    write_json(
        &base.join("latest.json"),
        &json!({"ok":false,"completed":false,"status":status_path}),
    )?;
    println!(
        "Verifying local Cargo packages outside the repository; evidence: {}",
        directory.display()
    );
    let result = verify(root, &lab, &directory, policy);
    match result {
        Ok(evidence) => {
            // Keep archives, locks and all reports; transient extracted source is owned here.
            fs::remove_dir_all(&lab)?;
            status["ok"] = json!(true);
            status["completed"] = json!(true);
            status["packagedCratesValidated"] = json!(true);
            status["gpuExecuted"] = json!(policy.is_some());
            status["stagingRemoved"] = json!(true);
            status["verification"] = evidence;
            write_json(&status_path, &status)?;
            write_json(
                &base.join("latest.json"),
                &json!({"ok":true,"completed":true,"status":status_path}),
            )?;
            println!(
                "Packaged Rust and CLI consumption passed ({})",
                if policy.is_some() { "GPU" } else { "CPU only" }
            );
            Ok(status_path)
        }
        Err(error) => {
            status["message"] = json!(error.to_string());
            write_json(&status_path, &status)?;
            Err(format!(
                "package verification failed: {error}; inspect {}",
                directory.display()
            )
            .into())
        }
    }
}
fn write_json(path: &Path, value: &Value) -> TaskResult {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn hash(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn tree(path: &Path) -> TaskResult<BTreeMap<String, String>> {
    fn visit(base: &Path, path: &Path, files: &mut BTreeMap<String, String>) -> TaskResult {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_symlink() {
                return Err(
                    format!("package input may not be a symlink: {}", path.display()).into(),
                );
            }
            if kind.is_dir() {
                visit(base, &path, files)?;
            } else if kind.is_file() {
                files.insert(
                    path.strip_prefix(base)?
                        .to_str()
                        .ok_or("non-UTF8 package path")?
                        .replace('\\', "/"),
                    hash(&fs::read(path)?),
                );
            } else {
                return Err("unsupported package input file kind".into());
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    visit(path, path, &mut files)?;
    Ok(files)
}
fn source_snapshot(root: &Path) -> TaskResult<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "LICENSE-MIT",
        "LICENSE-APACHE",
    ] {
        files.insert(name.into(), hash(&fs::read(root.join(name))?));
    }
    for directory in [
        "crates",
        "xtask/src",
        "xtask/tests",
        "examples/native-consumer/src",
        "examples/native-consumer/tests",
    ] {
        for (path, digest) in tree(&root.join(directory))? {
            files.insert(format!("{directory}/{path}"), digest);
        }
    }
    for name in ["constant-scalar", "transform-2d", "warp"] {
        let path = format!("fixtures/nodes/{name}/input.mix");
        files.insert(path.clone(), hash(&fs::read(root.join(path))?));
    }
    for name in ["Cargo.toml", "Cargo.lock", "input.mix"] {
        let path = format!("examples/native-consumer/{name}");
        files.insert(path.clone(), hash(&fs::read(root.join(path))?));
    }
    Ok(files)
}
fn raw(command: &mut Command, directory: &Path, name: &str) -> TaskResult<Output> {
    write_json(
        &directory.join("phase.json"),
        &json!({"phase":name,"program":command.get_program().to_string_lossy(),"args":command.get_args().map(|a|a.to_string_lossy().into_owned()).collect::<Vec<_>>(),"cwd":command.get_current_dir()}),
    )?;
    let output = command.output()?;
    fs::write(directory.join(format!("{name}.stdout.log")), &output.stdout)?;
    fs::write(directory.join(format!("{name}.stderr.log")), &output.stderr)?;
    write_json(
        &directory.join(format!("{name}.command.json")),
        &json!({"program":command.get_program().to_string_lossy(),"args":command.get_args().map(|a|a.to_string_lossy().into_owned()).collect::<Vec<_>>(),"cwd":command.get_current_dir(),"exitCode":output.status.code(),"success":output.status.success()}),
    )?;
    Ok(output)
}
fn capture(command: &mut Command, directory: &Path, name: &str) -> TaskResult<Vec<u8>> {
    let output = raw(command, directory, name)?;
    if !output.status.success() {
        return Err(format!("{name} failed ({})", output.status).into());
    }
    Ok(output.stdout)
}
fn replace_once(text: &str, from: &str, to: &str) -> TaskResult<String> {
    if text.matches(from).count() != 1 {
        return Err(format!("expected exactly one consumer manifest entry: {from}").into());
    }
    Ok(text.replacen(from, to, 1))
}
fn copy_tree(from: &Path, to: &Path) -> TaskResult {
    fs::create_dir(to)?;
    for path in tree(from)?.keys() {
        let destination = to.join(path);
        fs::create_dir_all(destination.parent().ok_or("missing parent")?)?;
        fs::copy(from.join(path), destination)?;
    }
    Ok(())
}
fn command(lab: &Path, target: &Path, action: &str) -> Command {
    let mut command = cargo(lab);
    command
        .arg(action)
        .args([
            "--workspace",
            "--all-features",
            "--offline",
            "--locked",
            "--target-dir",
        ])
        .arg(target);
    command
}
fn names(metadata: &Value) -> TaskResult<BTreeMap<String, &Value>> {
    let mut result = BTreeMap::new();
    for package in metadata["packages"]
        .as_array()
        .ok_or("metadata omitted packages")?
    {
        let name = package["name"].as_str().ok_or("package omitted name")?;
        if (PACKAGES.contains(&name) || name == APP)
            && result.insert(name.to_owned(), package).is_some()
        {
            return Err("multiple resolutions of one Mixture package".into());
        }
    }
    Ok(result)
}
fn validate_resolution(metadata: &Value, lab: &Path, version: &str) -> TaskResult {
    let packages = names(metadata)?;
    if packages.len() != 4
        || metadata["workspace_members"]
            .as_array()
            .is_none_or(|m| m.len() != 4)
    {
        return Err(
            "isolated workspace must contain three archived packages and the independent consumer"
                .into(),
        );
    }
    for (name, package) in packages {
        let expected = if name == APP {
            lab.join("consumer/Cargo.toml")
        } else {
            lab.join(format!("packages/{name}-{version}/Cargo.toml"))
        };
        if package["manifest_path"] != json!(expected)
            || !package["source"].is_null()
            || !package["publish"].as_array().is_some_and(Vec::is_empty)
            || (name != APP && package["version"] != version)
        {
            return Err(format!(
                "{name} did not resolve exclusively to its private extracted local package"
            )
            .into());
        }
        for dependency in package["dependencies"]
            .as_array()
            .ok_or("missing dependencies")?
        {
            if dependency["name"]
                .as_str()
                .is_some_and(|n| PACKAGES.contains(&n))
                && (!dependency["path"].is_null() || dependency["req"] != format!("={version}"))
            {
                return Err(
                    "packaged/consumer dependency retained a producer source path or wrong version"
                        .into(),
                );
            }
        }
        for target in package["targets"].as_array().ok_or("missing targets")? {
            let source = Path::new(target["src_path"].as_str().ok_or("target omitted source")?);
            if !source.starts_with(expected.parent().ok_or("missing manifest parent")?) {
                return Err("package target escapes extracted source".into());
            }
        }
    }
    Ok(())
}
// Cargo-generated lockfiles use basic quoted strings for these identity fields.
// Compare identities/checksums, not dependency lists pruned by the isolated workspace.
fn registry_pins(lock: &str) -> TaskResult<BTreeMap<(String, String, String), String>> {
    let mut pins = BTreeMap::new();
    for block in lock.split("[[package]]").skip(1) {
        let field = |name: &str| -> TaskResult<Option<String>> {
            block
                .lines()
                .find_map(|line| line.strip_prefix(&format!("{name} = ")))
                .map(|value| serde_json::from_str(value).map_err(Into::into))
                .transpose()
        };
        if let Some(source) = field("source")? {
            if !source.starts_with("registry+") {
                return Err("unreviewed non-registry external package source".into());
            }
            let key = (
                field("name")?.ok_or("lock package name missing")?,
                field("version")?.ok_or("lock package version missing")?,
                source,
            );
            if pins
                .insert(key, field("checksum")?.ok_or("registry checksum missing")?)
                .is_some()
            {
                return Err("duplicate lock package identity".into());
            }
        }
    }
    if pins.is_empty() {
        return Err("lockfile omitted external dependency pins".into());
    }
    Ok(pins)
}
fn validate_pins(original: &str, isolated: &str) -> TaskResult<usize> {
    let expected = registry_pins(original)?;
    let actual = registry_pins(isolated)?;
    if actual
        .iter()
        .any(|(key, value)| expected.get(key) != Some(value))
    {
        return Err("isolated resolution changed an external version/source/checksum".into());
    }
    Ok(actual.len())
}
fn validate_archive_entries(list: &str, prefix: &str) -> TaskResult<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for entry in list.lines() {
        if entry.contains('\\')
            || Path::new(entry)
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("unsafe archive path".into());
        }
        let relative = entry
            .strip_prefix(&format!("{prefix}/"))
            .ok_or("archive member outside expected package")?;
        if relative.is_empty() || !result.insert(relative.to_owned()) {
            return Err("empty or duplicate archive member".into());
        }
    }
    for required in [
        "Cargo.toml",
        "Cargo.toml.orig",
        "README.md",
        "README.zh-CN.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
    ] {
        if !result.contains(required) {
            return Err(format!("package omitted required file {required}").into());
        }
    }
    Ok(result)
}
fn verify(
    root: &Path,
    lab: &Path,
    directory: &Path,
    policy: Option<Policy<'_>>,
) -> TaskResult<Value> {
    let before = source_snapshot(root)?;
    write_json(&directory.join("source-hashes.json"), &json!(before))?;
    for name in PACKAGES {
        for license in ["LICENSE-MIT", "LICENSE-APACHE"] {
            if fs::read(root.join(license))?
                != fs::read(root.join(format!("crates/{name}/{license}")))?
            {
                return Err(format!("{name} license differs from repository license").into());
            }
        }
    }
    for name in ["constant-scalar", "transform-2d", "warp"] {
        if fs::read(root.join(format!("fixtures/nodes/{name}/input.mix")))?
            != fs::read(root.join(format!("crates/mixture-wgpu/src/testdata/{name}.mix")))?
        {
            return Err(
                format!("packaged unit fixture {name} differs from its canonical input").into(),
            );
        }
    }
    let metadata: Value = serde_json::from_slice(&capture(
        cargo(root).args(["metadata", "--locked", "--format-version", "1", "--no-deps"]),
        directory,
        "producer-metadata",
    )?)?;
    let packages = names(&metadata)?;
    let version = packages
        .get("mixture-core")
        .and_then(|p| p["version"].as_str())
        .ok_or("core version missing")?;
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
    {
        return Err("invalid package version path".into());
    }
    for name in PACKAGES {
        if packages.get(name).is_none_or(|p| p["version"] != version) {
            return Err("local package versions must match".into());
        }
    }
    // Fetch only locked public dependencies before enforcing offline staged builds.
    capture(cargo(root).args(["fetch", "--locked"]), directory, "fetch")?;
    capture(
        cargo(root)
            .args([
                "package",
                "--locked",
                "--offline",
                "--no-verify",
                "--exclude-lockfile",
                "--allow-dirty",
                "-p",
                "mixture-core",
                "-p",
                "mixture-wgpu",
                "-p",
                "mixture-cli",
                "--target-dir",
            ])
            .arg(directory.join("cargo")),
        directory,
        "package",
    )?;
    let extracted = lab.join("packages");
    fs::create_dir(&extracted)?;
    let archives = directory.join("archives");
    fs::create_dir(&archives)?;
    let mut archive_hashes = BTreeMap::new();
    let mut inventories = BTreeMap::new();
    for name in PACKAGES {
        let prefix = format!("{name}-{version}");
        let filename = format!("{prefix}.crate");
        let archive = archives.join(&filename);
        fs::copy(directory.join("cargo/package").join(&filename), &archive)?;
        archive_hashes.insert(filename, hash(&fs::read(&archive)?));
        let list = String::from_utf8(capture(
            Command::new("tar").args(["-tzf"]).arg(&archive),
            directory,
            &format!("list-{name}"),
        )?)?;
        let entries = validate_archive_entries(&list, &prefix)?;
        capture(
            Command::new("tar")
                .arg("-xzf")
                .arg(&archive)
                .arg("-C")
                .arg(&extracted),
            directory,
            &format!("extract-{name}"),
        )?;
        let files = tree(&extracted.join(&prefix))?;
        if entries != files.keys().cloned().collect() {
            return Err("archive listing differs from extracted regular files".into());
        }
        // Source includes/metadata can only select bytes from this repository package.
        for (path, digest) in &files {
            if !matches!(
                path.as_str(),
                "Cargo.toml" | "Cargo.toml.orig" | ".cargo_vcs_info.json"
            ) && before.get(&format!("crates/{name}/{path}")) != Some(digest)
            {
                return Err(format!("unexpected archive bytes: {name}/{path}").into());
            }
        }
        if files.get("Cargo.toml.orig") != before.get(&format!("crates/{name}/Cargo.toml")) {
            return Err("archive original manifest differs from input".into());
        }
        fs::write(
            directory.join(format!("{name}-Cargo.toml")),
            fs::read(extracted.join(&prefix).join("Cargo.toml"))?,
        )?;
        inventories.insert(name.to_owned(), files);
    }
    fs::remove_dir_all(directory.join("cargo"))?;
    write_json(
        &directory.join("archive-hashes.json"),
        &json!(archive_hashes),
    )?;
    write_json(&directory.join("package-files.json"), &json!(inventories))?;
    let app = lab.join("consumer");
    fs::create_dir(&app)?;
    for name in ["src", "tests"] {
        copy_tree(
            &root.join("examples/native-consumer").join(name),
            &app.join(name),
        )?;
    }
    fs::copy(
        root.join("examples/native-consumer/input.mix"),
        app.join("input.mix"),
    )?;
    let mut manifest = fs::read_to_string(root.join("examples/native-consumer/Cargo.toml"))?;
    manifest = replace_once(&manifest, "[workspace]\n", "")?;
    for name in ["mixture-core", "mixture-wgpu"] {
        manifest = replace_once(
            &manifest,
            &format!("{name} = {{ path = \"../../crates/{name}\" }}"),
            &format!("{name} = \"={version}\""),
        )?;
    }
    fs::write(app.join("Cargo.toml"), &manifest)?;
    fs::write(directory.join("consumer-Cargo.toml"), &manifest)?;
    let members: Vec<_> = std::iter::once("consumer".to_owned())
        .chain(PACKAGES.map(|name| format!("packages/{name}-{version}")))
        .collect();
    let workspace = format!(
        "[workspace]\nmembers = {}\nresolver = \"3\"\n\n[patch.crates-io]\nmixture-core = {{ path = \"packages/mixture-core-{version}\" }}\nmixture-wgpu = {{ path = \"packages/mixture-wgpu-{version}\" }}\n",
        serde_json::to_string(&members)?
    );
    fs::write(lab.join("Cargo.toml"), &workspace)?;
    fs::write(directory.join("workspace-Cargo.toml"), &workspace)?;
    fs::copy(
        root.join("rust-toolchain.toml"),
        lab.join("rust-toolchain.toml"),
    )?;
    let lock = fs::read_to_string(root.join("Cargo.lock"))?;
    fs::write(lab.join("Cargo.lock"), &lock)?;
    // Only this disposable lock may change. Normalize local membership offline,
    // then reject any new external identity/checksum before every locked build.
    capture(
        cargo(lab).args([
            "metadata",
            "--offline",
            "--all-features",
            "--format-version",
            "1",
        ]),
        directory,
        "resolve",
    )?;
    let pins = validate_pins(&lock, &fs::read_to_string(lab.join("Cargo.lock"))?)?;
    fs::copy(
        lab.join("Cargo.lock"),
        directory.join("resolved-Cargo.lock"),
    )?;
    let metadata: Value = serde_json::from_slice(&capture(
        cargo(lab).args([
            "metadata",
            "--offline",
            "--locked",
            "--all-features",
            "--format-version",
            "1",
        ]),
        directory,
        "resolved-metadata",
    )?)?;
    validate_resolution(&metadata, lab, version)?;
    let target = root.join("target/package-consumer");
    capture(&mut command(lab, &target, "build"), directory, "build")?;
    capture(&mut command(lab, &target, "test"), directory, "tests")?;
    let mut doc = command(lab, &target, "doc");
    doc.arg("--no-deps");
    let mut flags = env::var("CARGO_ENCODED_RUSTDOCFLAGS").unwrap_or_else(|_| {
        env::var("RUSTDOCFLAGS")
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("\x1f")
    });
    if !flags.is_empty() {
        flags.push('\x1f');
    }
    flags.push_str("-Dwarnings");
    doc.env("CARGO_ENCODED_RUSTDOCFLAGS", flags);
    capture(&mut doc, directory, "docs")?;
    let asset = extracted.join(format!(
        "mixture-wgpu-{version}/shaders/nodes/constant.wgsl"
    ));
    let shader = fs::read(&asset)?;
    fs::remove_file(&asset)?;
    let negative = raw(
        cargo(lab)
            .args([
                "check",
                "--offline",
                "--locked",
                "--all-features",
                "-p",
                "mixture-wgpu",
                "--target-dir",
            ])
            .arg(&target),
        directory,
        "missing-shader",
    );
    // Restore even when spawning/checking failed; archives themselves never mutate.
    fs::write(&asset, shader)?;
    let negative = negative?;
    if negative.status.success()
        || !String::from_utf8_lossy(&negative.stderr).contains("constant.wgsl")
    {
        return Err(
            "missing packaged runtime shader did not produce the expected build failure".into(),
        );
    }
    for name in PACKAGES {
        if tree(&extracted.join(format!("{name}-{version}")))? != inventories[name] {
            return Err("verification modified extracted package source".into());
        }
    }
    let run = lab.join("run");
    fs::create_dir(&run)?;
    let executable = target.join(format!("debug/{APP}{}", env::consts::EXE_SUFFIX));
    let cli = target.join(format!("debug/mixture{}", env::consts::EXE_SUFFIX));
    let cpu: Value = serde_json::from_slice(&capture(
        Command::new(&executable).arg("check").current_dir(&run),
        directory,
        "consumer-cpu",
    )?)?;
    consumer::validate_cpu(&cpu)?;
    cli_contract(lab, &target, &cli, directory, false, None)?;
    if let Some((backend, software, expected)) = policy {
        let mut app = Command::new(&executable);
        app.current_dir(&run).args([
            "gpu",
            backend,
            if software { "software" } else { "hardware" },
        ]);
        if let Some(expected) = expected {
            app.arg(expected);
        }
        let gpu: Value = serde_json::from_slice(&capture(&mut app, directory, "consumer-gpu")?)?;
        consumer::validate_gpu(&gpu)?;
        gpu_smoke::validate_adapter(&gpu["context"], backend, software, expected)?;
        cli_contract(lab, &target, &cli, directory, true, policy)?;
    }
    if source_snapshot(root)? != before {
        return Err("producer source changed during package verification".into());
    }
    Ok(
        json!({"packageVersion":version,"archives":archive_hashes,"externalPinnedPackages":pins,
            "resolution":"Cargo-normalized archives extracted outside the producer; version-only consumer and package dependencies resolved through local [patch.crates-io] entries to those extracted archives",
            "lockPolicy":"archives omit lockfiles while publication is disabled; disposable verification lock is seeded from the committed workspace lock, local membership normalized offline, external name/version/source/checksum pins checked, then all builds/tests use --offline --locked",
            "buildCache":target,"cpuConsumer":"consumer-cpu.stdout.log","cliCpu":"cli-cpu/status.json",
            "gpuConsumer":policy.map(|_|"consumer-gpu.stdout.log"),"cliGpu":policy.map(|_|"cli-gpu/status.json"),
            "missingShaderRejected":true,"extractedSourcesUnchanged":true,"producerSourcesUnchanged":true,
            "binaryHashes":{"consumer":hash(&fs::read(executable)?),"cli":hash(&fs::read(cli)?)}
        }),
    )
}
fn cli_contract(
    lab: &Path,
    target: &Path,
    cli: &Path,
    directory: &Path,
    gpu: bool,
    policy: Option<Policy<'_>>,
) -> TaskResult {
    let name = if gpu { "cli-gpu" } else { "cli-cpu" };
    let evidence = lab.join(name);
    let mut command = cargo(lab);
    command
        .args([
            "test",
            "--offline",
            "--locked",
            "--all-features",
            "-p",
            APP,
            "--target-dir",
        ])
        .arg(target)
        .args([
            "--test",
            "cli_contract",
            if gpu {
                "cli_contract_gpu"
            } else {
                "cli_contract_cpu"
            },
            "--",
            "--ignored",
            "--exact",
            "--nocapture",
        ])
        .env("MIXTURE_CONSUMER_CLI", cli)
        .env("MIXTURE_CONSUMER_EVIDENCE_DIR", &evidence);
    if let Some((backend, software, expected)) = policy {
        command
            .env("MIXTURE_GPU_BACKEND", backend)
            .env("MIXTURE_GPU_SOFTWARE", if software { "1" } else { "0" });
        if let Some(expected) = expected {
            command.env("MIXTURE_GPU_EXPECT_ADAPTER", expected);
        } else {
            command.env_remove("MIXTURE_GPU_EXPECT_ADAPTER");
        }
    }
    let result = capture(&mut command, directory, &format!("{name}-tests"));
    // Preserve incomplete receipts/streams as well as successes.
    if evidence.is_dir() {
        copy_tree(&evidence, &directory.join(name))?;
    }
    result?;
    let report: Value = serde_json::from_slice(&fs::read(evidence.join("status.json"))?)?;
    consumer::validate_cli_status(&report, gpu)?;
    if let Some((backend, software, expected)) = policy {
        let doctor: Value = serde_json::from_slice(&fs::read(evidence.join("doctor.stdout"))?)?;
        gpu_smoke::validate_report(&doctor, backend, software, expected)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_paths_and_required_distribution_files_are_checked() {
        let valid = [
            "Cargo.toml",
            "Cargo.toml.orig",
            "README.md",
            "README.zh-CN.md",
            "LICENSE-MIT",
            "LICENSE-APACHE",
            "src/lib.rs",
        ]
        .map(|p| format!("crate-0.1.0/{p}"))
        .join("\n");
        assert!(validate_archive_entries(&valid, "crate-0.1.0").is_ok());
        for invalid in [
            format!("{valid}\ncrate-0.1.0/../outside"),
            format!("{valid}\n/absolute"),
            format!("{valid}\nother/src/lib.rs"),
            format!("{valid}\ncrate-0.1.0/src/lib.rs"),
            valid.replace("crate-0.1.0/LICENSE-MIT", ""),
        ] {
            assert!(validate_archive_entries(&invalid, "crate-0.1.0").is_err());
        }
    }
    #[test]
    fn disposable_lock_may_prune_but_must_not_upgrade_or_change_checksums() {
        let package = |name: &str, version: &str, checksum: &str| {
            format!(
                "[[package]]\nname = {name:?}\nversion = {version:?}\nsource = \"registry+https://example.test/index\"\nchecksum = {checksum:?}\n"
            )
        };
        let kept = package("a", "1.0.0", "pinned");
        let original = format!("{kept}{}", package("unused", "2.0.0", "original"));
        assert_eq!(validate_pins(&original, &kept).unwrap(), 1);
        for bad in [
            package("a", "1.0.1", "new"),
            package("a", "1.0.0", "changed"),
            package("unreviewed", "1.0.0", "new"),
            String::new(),
        ] {
            assert!(validate_pins(&original, &bad).is_err());
        }
    }
    fn metadata(lab: &Path) -> Value {
        let packages:Vec<_>=PACKAGES.into_iter().chain([APP]).map(|name|json!({
            "name":name,"version":if name==APP{"0.0.0"}else{"0.1.0"},"publish":[],"source":null,
            "manifest_path":if name==APP{lab.join("consumer/Cargo.toml")}else{lab.join(format!("packages/{name}-0.1.0/Cargo.toml"))},
            "dependencies":[{"name":"mixture-core","req":"=0.1.0","source":"registry+https://github.com/rust-lang/crates.io-index"}],
            "targets":[{"src_path":if name==APP{lab.join("consumer/src/main.rs")}else{lab.join(format!("packages/{name}-0.1.0/src/lib.rs"))}}]
        })).collect();
        json!({"packages":packages,"workspace_members":["a","b","c","d"]})
    }
    #[test]
    fn resolution_rejects_registry_fallback_producer_paths_and_wrong_versions() {
        let lab = env::temp_dir().join("isolated-package-test");
        let good = metadata(&lab);
        assert!(validate_resolution(&good, &lab, "0.1.0").is_ok());
        for (pointer, value) in [
            ("/packages/0/source", json!("registry")),
            (
                "/packages/1/manifest_path",
                json!("producer/crates/mixture-wgpu/Cargo.toml"),
            ),
            ("/packages/0/version", json!("0.2.0")),
            ("/packages/0/publish", Value::Null),
            ("/packages/3/dependencies/0/req", json!("*")),
            ("/packages/1/targets/0/src_path", json!("/outside/lib.rs")),
        ] {
            let mut bad = good.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(
                validate_resolution(&bad, &lab, "0.1.0").is_err(),
                "accepted {pointer}"
            );
        }
        let mut bad = good;
        bad["packages"][3]["dependencies"][0]["path"] = json!("../producer");
        assert!(validate_resolution(&bad, &lab, "0.1.0").is_err());
    }
    #[test]
    fn consumer_manifest_rewrite_refuses_missing_or_ambiguous_entries() {
        assert_eq!(
            replace_once("a path b", "path", "version").unwrap(),
            "a version b"
        );
        assert!(replace_once("no entry", "path", "version").is_err());
        assert!(replace_once("path path", "path", "version").is_err());
    }
}
