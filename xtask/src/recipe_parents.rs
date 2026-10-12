//! MAT-04d pixel identity of the recipe parents through the independent public consumer.
use crate::{TaskResult, cargo, gpu_smoke};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Accepted-graph rows: brick 5 cases x 4 sizes; woven 12 cases x 4 sizes + 3 stress rows.
const BRICK_ROWS: usize = 20;
const WOVEN_ROWS: usize = 51;

pub(super) fn run(root: &Path) -> TaskResult {
    let (backend, software) = gpu_smoke::policy()?;
    let base = root.join("tmp/materials/recipe-parents");
    fs::create_dir_all(&base)?;
    let directory = base.join(format!(
        "{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id()
    ));
    fs::create_dir(&directory)?;
    let status = directory.join("report.json");
    fs::write(&status, br#"{"ok":false,"completed":false}"#)?;
    let receipt_path = directory.join("identity.json");
    let output = cargo(root)
        .args([
            "test",
            "--release",
            "--locked",
            "--all-features",
            "--manifest-path",
            "examples/native-consumer/Cargo.toml",
            "--target-dir",
            "target/native-consumer",
            "--test",
            "recipe_parents",
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("MIXTURE_GPU_BACKEND", backend)
        .env("MIXTURE_GPU_SOFTWARE", software)
        .env("MIXTURE_RECIPE_EVIDENCE", &receipt_path)
        .output()?;
    fs::write(directory.join("stdout.log"), &output.stdout)?;
    fs::write(directory.join("stderr.log"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!(
            "recipe parent identity failed; inspect {}: {}",
            directory.display(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let receipt: Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    validate(&receipt)?;
    fs::write(
        status,
        serde_json::to_vec_pretty(&json!({
            "ok":true,"completed":true,"identity":"identity.json",
            "workingTreeClean":receipt["workingTreeClean"],
            "scope":"Byte identity of both recipe parents against the accepted v1 graphs on this adapter; descriptor and matched-adapter timing budgets. Acceptance is recorded separately."
        }))?,
    )?;
    println!("Recipe parent identity passed: {}", directory.display());
    Ok(())
}

/// Every frozen row must be present and identical; software runs may only skip woven 2048².
fn validate(receipt: &Value) -> TaskResult {
    if receipt["completed"] != true
        || receipt["gates"]["pixelIdentity"] != true
        || receipt["gates"]["budgets"] != true
    {
        return Err("recipe parent identity receipt is incomplete or failed".into());
    }
    let rows = receipt["rows"].as_array().ok_or("missing rows")?;
    let count = |material: &str| rows.iter().filter(|r| r["material"] == material).count();
    if count("brick-paving") != BRICK_ROWS || count("woven-fabric") != WOVEN_ROWS {
        return Err("recipe parent identity receipt is missing frozen rows".into());
    }
    for row in rows {
        match row["status"].as_str() {
            Some("rendered") => {
                let channels = row["channels"].as_array().ok_or("missing channels")?;
                if channels.is_empty() || channels.iter().any(|c| c["identical"] != true) {
                    return Err("a rendered row is not byte-identical".into());
                }
            }
            Some("notRunOnSoftware")
                if receipt["softwareAdapter"] == true
                    && row["material"] == "woven-fabric"
                    && row["size"] == json!([2048, 2048]) => {}
            _ => return Err("unexpected recipe parent row status".into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn receipt(software: bool) -> Value {
        let row = |material: &str, size: [u32; 2]| {
            if software && material == "woven-fabric" && size == [2048, 2048] {
                json!({"material":material,"size":size,"status":"notRunOnSoftware"})
            } else {
                json!({"material":material,"size":size,"status":"rendered","channels":[{"identical":true}]})
            }
        };
        let mut rows: Vec<Value> = (0..BRICK_ROWS)
            .map(|_| row("brick-paving", [256, 256]))
            .collect();
        rows.extend((0..WOVEN_ROWS - 1).map(|_| row("woven-fabric", [256, 256])));
        rows.push(row("woven-fabric", [2048, 2048]));
        json!({"completed":true,"softwareAdapter":software,"gates":{"pixelIdentity":true,"budgets":true},"rows":rows})
    }
    #[test]
    fn failed_truncated_or_unidentical_identity_runs_are_rejected() {
        validate(&receipt(false)).unwrap();
        validate(&receipt(true)).unwrap();
        for (path, value) in [
            (vec!["completed"], json!(false)),
            (vec!["gates", "pixelIdentity"], json!(false)),
            (vec!["gates", "budgets"], json!(false)),
        ] {
            let mut invalid = receipt(false);
            let mut target = &mut invalid;
            for key in path {
                target = &mut target[key];
            }
            *target = value;
            assert!(validate(&invalid).is_err());
        }
        let mut truncated = receipt(false);
        truncated["rows"].as_array_mut().unwrap().pop();
        assert!(validate(&truncated).is_err());
        let mut differing = receipt(false);
        differing["rows"][0]["channels"][0]["identical"] = json!(false);
        assert!(validate(&differing).is_err());
        // Hardware runs may not skip any row.
        let mut skipped = receipt(true);
        skipped["softwareAdapter"] = json!(false);
        assert!(validate(&skipped).is_err());
    }
}
