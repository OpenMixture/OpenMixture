//! Bounded fixture reads and content-bound review candidates.

use crate::TaskResult;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(super) type Digests = BTreeMap<String, String>;

pub(super) fn json<T: DeserializeOwned>(path: &Path) -> TaskResult<T> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(format!("{}: JSON exceeds 4 MiB", path.display()).into());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()).into())
}

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> TaskResult {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

pub(super) fn digest(path: &Path) -> TaskResult<String> {
    let mut input = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let hex = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("sha256:{hex}"))
}

pub(super) fn relative(root: &Path, name: &str) -> TaskResult<PathBuf> {
    let path = Path::new(name);
    if name.is_empty()
        || name.contains('\\')
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("unsafe artifact path".into());
    }
    let mut resolved = root.to_path_buf();
    for component in path.components() {
        resolved.push(component);
        reject_symlink(&resolved)?;
    }
    Ok(resolved)
}

pub(super) fn reject_symlink(path: &Path) -> TaskResult {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("refusing symlink: {}", path.display()).into())
        }
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

pub(super) fn tree(root: &Path) -> TaskResult<Digests> {
    let mut result = BTreeMap::new();
    if root.exists() {
        walk(root, root, &mut result)?;
    }
    Ok(result)
}

fn walk(root: &Path, path: &Path, result: &mut Digests) -> TaskResult {
    reject_symlink(path)?;
    if path.is_file() {
        let name = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        result.insert(name, digest(path)?);
    } else {
        for entry in fs::read_dir(path)? {
            walk(root, &entry?.path(), result)?;
        }
    }
    Ok(())
}

pub(super) fn inputs(root: &Path, material: &Path) -> TaskResult<Digests> {
    let mut result = BTreeMap::new();
    for directory in [
        "crates/mixture-core/src",
        "crates/mixture-wgpu/src",
        "crates/mixture-wgpu/shaders",
        "crates/mixture-cli/src",
        "xtask/src",
    ] {
        for (name, hash) in tree(&relative(root, directory)?)? {
            result.insert(format!("{directory}/{name}"), hash);
        }
    }
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/mixture-core/Cargo.toml",
        "crates/mixture-wgpu/Cargo.toml",
        "crates/mixture-cli/Cargo.toml",
        "xtask/Cargo.toml",
        ".github/scripts/setup-swiftshader.sh",
    ] {
        result.insert(name.to_owned(), digest(&relative(root, name)?)?);
    }
    for name in ["material.mix", "acceptance.json"] {
        let path = relative(material, name)?;
        result.insert(
            path.strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/"),
            digest(&path)?,
        );
    }
    for (name, hash) in tree(&relative(material, "variants")?)? {
        result.insert(
            format!(
                "{}/variants/{name}",
                material
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/")
            ),
            hash,
        );
    }
    Ok(result)
}

pub(super) fn verify(root: &Path, expected: &Digests) -> TaskResult {
    for (name, hash) in expected {
        if digest(&relative(root, name)?)? != *hash {
            return Err(
                format!("review artifact changed: {name}; run the check and review again").into(),
            );
        }
    }
    Ok(())
}
