//! GPU test tiers: tests ignored with a `full-qualification:` reason run only in the
//! full tier (pushes to main, manual runs and labeled pull requests).
use crate::TaskResult;
use std::{env, fs, path::Path};

const MARKER: &str = "#[ignore = \"full-qualification:";

/// Read `MIXTURE_GPU_TIER` (`standard` by default, or `full`).
pub(super) fn full_tier() -> TaskResult<bool> {
    match env::var("MIXTURE_GPU_TIER").as_deref() {
        Err(_) | Ok("standard") => Ok(false),
        Ok("full") => Ok(true),
        Ok(other) => {
            Err(format!("Invalid MIXTURE_GPU_TIER={other}: expected standard|full").into())
        }
    }
}

/// Names of full-tier tests under `crates/`, sorted, after checking that no other
/// ignored test name contains one of them (libtest `--skip` matches substrings).
pub(super) fn full_tier_tests(root: &Path) -> TaskResult<Vec<String>> {
    let mut full = Vec::new();
    let mut ignored = Vec::new();
    let mut files = Vec::new();
    collect(&root.join("crates"), &mut files)?;
    files.sort();
    for file in files {
        scan(&fs::read_to_string(&file)?, &mut full, &mut ignored);
    }
    full.sort();
    full.dedup();
    for name in &full {
        if let Some(other) = ignored
            .iter()
            .find(|o| *o != name && o.contains(name.as_str()))
        {
            return Err(format!(
                "full-tier test `{name}` would also skip ignored test `{other}`; rename one"
            )
            .into());
        }
    }
    Ok(full)
}

fn collect(directory: &Path, files: &mut Vec<std::path::PathBuf>) -> TaskResult {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collect(&path, files)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Attach each `#[ignore ...]` attribute to the next `fn` declaration.
fn scan(source: &str, full: &mut Vec<String>, ignored: &mut Vec<String>) {
    let mut pending: Option<bool> = None;
    for line in source.lines().map(str::trim) {
        if line.starts_with("#[ignore") {
            pending = Some(line.starts_with(MARKER));
        } else if let Some(is_full) = pending {
            let declaration = line.trim_start_matches("pub ").trim_start_matches("async ");
            if let Some(rest) = declaration.strip_prefix("fn ") {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if is_full {
                    full.push(name.clone());
                }
                ignored.push(name);
                pending = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_attaches_markers_to_the_next_function_only() {
        let source = r#"
#[test]
#[ignore = "full-qualification: slow probe"]
fn graph_gpu_slow() {}
#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
#[allow(clippy::x)]
fn graph_gpu_fast() {}
#[test]
fn plain() {}
"#;
        let (mut full, mut ignored) = (Vec::new(), Vec::new());
        scan(source, &mut full, &mut ignored);
        assert_eq!(full, ["graph_gpu_slow"]);
        assert_eq!(ignored, ["graph_gpu_slow", "graph_gpu_fast"]);
    }

    #[test]
    fn repository_full_tier_tests_are_found_without_collisions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let full = full_tier_tests(&root).unwrap();
        assert!(
            full.contains(&"graph_gpu_woven_flat".to_owned()),
            "{full:?}"
        );
    }
}
