//! Check Cargo's resolved manifests against the current foundation boundaries.

use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

use crate::{TaskResult, cargo};

const POLICY: &[(&str, &[&str])] = &[
    ("mixture-core", &["serde", "serde_json"]),
    (
        "mixture-wgpu",
        &[
            "mixture-core",
            "wgpu",
            "serde",
            "half",
            "naga",
            "pollster",
            "serde_json",
        ],
    ),
    (
        "mixture-cli",
        &[
            "mixture-core",
            "mixture-wgpu",
            "pollster",
            "serde_json",
            "serde",
            "png",
        ],
    ),
    ("xtask", &["pulldown-cmark", "serde_json", "png"]),
];

pub(super) fn check(root: &Path) -> TaskResult {
    let output = cargo(root)
        .args(["metadata", "--format-version", "1", "--no-deps", "--locked"])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    validate(&serde_json::from_slice(&output.stdout)?)?;
    println!("Dependency policy passed (four private crates, wgpu confined to mixture-wgpu).");
    Ok(())
}

fn validate(metadata: &Value) -> TaskResult {
    let packages = metadata["packages"]
        .as_array()
        .ok_or("metadata has no packages")?;
    let members = metadata["workspace_members"]
        .as_array()
        .ok_or("metadata has no workspace members")?;
    let mut actual = BTreeMap::new();
    for package in packages {
        if !members.contains(&package["id"]) {
            continue;
        }
        let name = package["name"].as_str().ok_or("package has no name")?;
        if actual.insert(name, package).is_some() {
            return Err(format!("duplicate workspace package: {name}").into());
        }
    }
    if actual.len() != POLICY.len() || members.len() != POLICY.len() {
        return Err(
            "The initial workspace requires exactly mixture-core, mixture-wgpu, mixture-cli, and xtask".into(),
        );
    }
    for (name, allowed) in POLICY {
        let package = actual
            .get(name)
            .ok_or_else(|| format!("missing workspace crate: {name}"))?;
        if !package["publish"].as_array().is_some_and(Vec::is_empty) {
            return Err(format!(
                "{name}: publication must remain disabled during the initial implementation train"
            )
            .into());
        }
        if package["license"].as_str() != Some("MIT OR Apache-2.0") {
            return Err(format!("{name}: expected MIT OR Apache-2.0 license").into());
        }
        let dependencies = package["dependencies"]
            .as_array()
            .ok_or("missing dependency list")?;
        for dependency in dependencies {
            // Cargo supplies the real package name even for renamed dependencies.
            // Check normal, build, dev, optional, and target-specific dependencies alike.
            let dependency_name = dependency["name"]
                .as_str()
                .ok_or("dependency has no name")?;
            if !allowed.contains(&dependency_name) {
                return Err(format!("{name} -> {dependency_name} violates the dependency policy; see docs/development.md").into());
            }
            if (*name == "mixture-wgpu"
                && matches!(dependency_name, "pollster" | "serde_json" | "naga"))
                && dependency["kind"].as_str() != Some("dev")
            {
                return Err(format!(
                    "{name} -> {dependency_name} is restricted to tests and examples"
                )
                .into());
            }
            if dependency_name.starts_with("mixture-")
                && (dependency["path"].as_str().is_none() || !dependency["source"].is_null())
            {
                return Err(format!(
                    "{name} -> {dependency_name} must use the local workspace crate"
                )
                .into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn baseline() -> Value {
        let packages: Vec<_> = POLICY.iter().map(|(name, allowed)| json!({
            "id": name,
            "name": name,
            "publish": [],
            "license": "MIT OR Apache-2.0",
            "dependencies": allowed.iter().map(|dependency| json!({
                "name": dependency,
                "kind": if *name == "mixture-wgpu" && matches!(*dependency, "pollster" | "serde_json" | "naga") { Some("dev") } else { None },
                "path": if dependency.starts_with("mixture-") { Some("../local") } else { None },
                "source": if dependency.starts_with("mixture-") { None } else { Some("registry") },
            })).collect::<Vec<_>>()
        })).collect();
        json!({ "packages": packages, "workspace_members": POLICY.iter().map(|(name, _)| name).collect::<Vec<_>>() })
    }

    #[test]
    fn accepts_the_foundation_boundaries() {
        validate(&baseline()).expect("foundation should pass");
    }

    #[test]
    fn rejects_platform_cli_and_renamed_dependencies_in_core() {
        for dependency in ["wgpu", "clap", "web-sys", "mixture-cli", "mixture-wgpu"] {
            let mut metadata = baseline();
            metadata["packages"][0]["dependencies"] = json!([{
                "name": dependency, "rename": "innocent-alias", "kind": "dev", "target": "cfg(windows)"
            }]);
            assert!(
                validate(&metadata).is_err(),
                "{dependency} must be rejected"
            );
        }
    }

    #[test]
    fn rejects_reversed_library_edges_and_extra_crates() {
        let mut metadata = baseline();
        metadata["packages"][1]["dependencies"] = json!([{"name": "mixture-cli"}]);
        assert!(validate(&metadata).is_err());
        let mut metadata = baseline();
        metadata["workspace_members"] = json!([
            "mixture-core",
            "mixture-wgpu",
            "mixture-cli",
            "xtask",
            "mixture-wasm"
        ]);
        assert!(validate(&metadata).is_err());
    }

    #[test]
    fn rejects_publication_and_registry_substitutes() {
        let mut metadata = baseline();
        metadata["packages"][3]["publish"] = Value::Null;
        assert!(validate(&metadata).is_err());
        let mut metadata = baseline();
        metadata["packages"][1]["dependencies"][0]["source"] = json!("registry");
        assert!(validate(&metadata).is_err());
    }

    #[test]
    fn accepts_runtime_json_dependency_for_strict_document_decoding() {
        let mut metadata = baseline();
        metadata["packages"][0]["dependencies"] = json!([{"name": "serde_json", "kind": null}]);
        assert!(validate(&metadata).is_ok());
    }

    #[test]
    fn rejects_gpu_dependency_outside_executor_and_blocking_executor_dependencies() {
        for index in [0, 2, 3] {
            let mut metadata = baseline();
            metadata["packages"][index]["dependencies"] = json!([{"name": "wgpu", "kind": "dev"}]);
            assert!(validate(&metadata).is_err());
        }
        for dependency in ["pollster", "serde_json", "naga"] {
            let mut metadata = baseline();
            metadata["packages"][1]["dependencies"] = json!([{"name": dependency, "kind": null}]);
            assert!(validate(&metadata).is_err());
        }
    }
}
