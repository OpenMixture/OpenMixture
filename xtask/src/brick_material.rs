//! MAT-01 property/matrix checks through the independent public consumer.
use crate::{TaskResult, cargo, gpu_smoke};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) fn run(root: &Path) -> TaskResult {
    let (backend, software) = gpu_smoke::policy()?;
    let base = root.join("tmp/materials/brick-paving");
    fs::create_dir_all(&base)?;
    let directory = base.join(format!(
        "{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id()
    ));
    fs::create_dir(&directory)?;
    let status = directory.join("report.json");
    fs::write(&status, br#"{"ok":false,"completed":false}"#)?;
    let native = directory.join("native");
    let output = cargo(root)
        .args([
            "test",
            "--release",
            "--locked",
            "--all-features",
            "--manifest-path",
            "examples/native-consumer/Cargo.toml",
            "--test",
            "brick_material",
            "brick_material_public_gpu_matrix",
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("MIXTURE_GPU_BACKEND", backend)
        .env("MIXTURE_GPU_SOFTWARE", software)
        .env(
            "MIXTURE_BRICK_FIXTURE_DIR",
            root.join("fixtures/materials/brick-paving"),
        )
        .env("MIXTURE_BRICK_EVIDENCE_DIR", &native)
        .env_remove("MIXTURE_BRICK_BROWSER_DIR")
        .output()?;
    fs::write(directory.join("stdout.log"), &output.stdout)?;
    fs::write(directory.join("stderr.log"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!(
            "brick material failed; inspect {}: {}",
            directory.display(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let receipt: Value = serde_json::from_slice(&fs::read(native.join("native-matrix.json"))?)?;
    validate(&receipt)?;
    fs::write(
        status,
        serde_json::to_vec_pretty(
            &json!({"ok":true,"completed":true,"materialAccepted":false,"native":"native/native-matrix.json","scope":"Native property, package, multiresolution and matched-adapter performance checks; browser and human acceptance separate"}),
        )?,
    )?;
    println!(
        "Brick material machine checks passed: {}",
        directory.display()
    );
    Ok(())
}

fn validate(receipt: &Value) -> TaskResult {
    if receipt["ok"] != true
        || receipt["completed"] != true
        || receipt["debugAssertions"] != false
        || receipt["cases"].as_array().map(Vec::len) != Some(20)
        || receipt["stress"].as_array().map(Vec::len) != Some(2)
    {
        return Err("incomplete brick Native matrix receipt".into());
    }
    for row in receipt["cases"].as_array().ok_or("missing cases")? {
        if row["repeatExact"] != true || row["packageExact"] != true {
            return Err("missing brick repeat/package verification".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_failed_debug_or_truncated_material_runs_are_rejected() {
        let valid = json!({"ok":true,"completed":true,"debugAssertions":false,"cases":vec![json!({"repeatExact":true,"packageExact":true});20],"stress":[{},{}]});
        validate(&valid).unwrap();
        for (field, value) in [
            ("ok", json!(false)),
            ("completed", json!(false)),
            ("debugAssertions", json!(true)),
            ("cases", json!([])),
            ("stress", json!([])),
        ] {
            let mut invalid = valid.clone();
            invalid[field] = value;
            assert!(validate(&invalid).is_err());
        }
        let mut invalid = valid;
        invalid["cases"][0]["packageExact"] = json!(false);
        assert!(validate(&invalid).is_err());
    }
}
