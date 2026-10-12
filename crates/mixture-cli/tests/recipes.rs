//! MAT-04c CLI recipe adapter: explicit --recipe files, closure diagnostics and reports.
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
}
fn recipe(id: &str) -> PathBuf {
    root().join(format!("fixtures/recipes/{id}.mixrecipe"))
}
fn parent(material: &str) -> PathBuf {
    root().join(format!(
        "fixtures/materials/{material}/material-recipes.mix"
    ))
}
fn accepted(material: &str) -> PathBuf {
    root().join(format!("fixtures/materials/{material}/material.mix"))
}
fn brick_recipes() -> Vec<PathBuf> {
    vec![recipe("brick-structure"), recipe("masked-layer-surface")]
}
fn woven_recipes() -> Vec<PathBuf> {
    [
        "weave-structure",
        "directional-grain",
        "masked-layer-surface",
    ]
    .into_iter()
    .map(recipe)
    .collect()
}
fn run(args: &[&str], source: &Path, recipes: &[PathBuf]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mixture"));
    command.args(args).arg(source);
    for path in recipes {
        command.arg("--recipe").arg(path);
    }
    command
        .env("MIXTURE_GPU_BACKEND", "none")
        .env("MIXTURE_GPU_SOFTWARE", "invalid")
        .output()
        .unwrap()
}
fn report(output: Output, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn first(report: &Value) -> &Value {
    &report["diagnostics"][0]
}

#[test]
fn validate_accepts_parents_only_with_their_exact_closure() {
    for (material, recipes) in [
        ("brick-paving", brick_recipes()),
        ("woven-fabric", woven_recipes()),
    ] {
        let source = parent(material);
        let before = std::fs::read(&source).unwrap();
        let ok = report(run(&["validate", "--json"], &source, &recipes), 0);
        assert_eq!(ok, json!({"ok": true, "diagnostics": []}));
        let human = run(&["validate"], &source, &recipes);
        assert!(String::from_utf8_lossy(&human.stdout).contains("Valid .mix v2 material"));
        assert_eq!(
            std::fs::read(&source).unwrap(),
            before,
            "sources are never changed"
        );
    }
    let source = parent("brick-paving");
    // Without --recipe the v1-only path reports the version, exactly as before.
    let failed = report(run(&["validate", "--json"], &source, &[]), 2);
    assert_eq!(first(&failed)["code"], "MIX_FORMAT_UNSUPPORTED_VERSION");

    let missing = report(
        run(
            &["validate", "--json"],
            &source,
            &[recipe("masked-layer-surface")],
        ),
        2,
    );
    assert_eq!(first(&missing)["code"], "MIX_RECIPE_MISSING");
    assert_eq!(first(&missing)["evidence"]["instanceId"], "structure");
    assert_eq!(
        first(&missing)["documentPath"],
        source.to_string_lossy().as_ref()
    );

    let mut extra = brick_recipes();
    extra.push(recipe("weave-structure"));
    let unused = report(run(&["validate", "--json"], &source, &extra), 2);
    assert_eq!(first(&unused)["code"], "MIX_RECIPE_UNUSED");

    let v1 = report(
        run(
            &["validate", "--json"],
            &accepted("brick-paving"),
            &brick_recipes(),
        ),
        2,
    );
    let codes: Vec<&Value> = v1["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| &d["code"])
        .collect();
    assert_eq!(
        codes,
        [&json!("MIX_RECIPE_UNUSED"), &json!("MIX_RECIPE_UNUSED")]
    );
}

#[test]
fn recipe_file_failures_name_the_recipe_file() {
    let directory = std::env::temp_dir().join(format!("mixture-recipes-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let mut broken: Value =
        serde_json::from_slice(&std::fs::read(recipe("masked-layer-surface")).unwrap()).unwrap();
    broken["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"out","type":"material-output","version":1}));
    let broken_path = directory.join("broken.mixrecipe");
    std::fs::write(&broken_path, serde_json::to_vec(&broken).unwrap()).unwrap();
    let source = parent("brick-paving");
    let failed = report(
        run(
            &["validate", "--json"],
            &source,
            &[recipe("brick-structure"), broken_path.clone()],
        ),
        2,
    );
    let forbidden = failed["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "MIX_RECIPE_FORBIDDEN_NODE")
        .expect("forbidden node diagnostic");
    assert_eq!(
        forbidden["documentPath"],
        broken_path.to_string_lossy().as_ref()
    );
    assert_eq!(forbidden["evidence"]["recipeIndex"], 1);

    let absent = directory.join("absent.mixrecipe");
    let unread = report(
        run(
            &["validate", "--json"],
            &source,
            std::slice::from_ref(&absent),
        ),
        1,
    );
    assert_eq!(first(&unread)["code"], "MIX_IO_READ_FAILED");
    assert_eq!(
        first(&unread)["documentPath"],
        absent.to_string_lossy().as_ref()
    );

    // The count ceiling is reported before any recipe file is opened.
    let many: Vec<PathBuf> = (0..17)
        .map(|i| directory.join(format!("never-{i}.mixrecipe")))
        .collect();
    let limited = report(run(&["validate", "--json"], &source, &many), 2);
    assert_eq!(first(&limited)["code"], "MIX_LIMIT_RECIPE_COUNT_EXCEEDED");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn inspect_reports_the_closure_only_for_parents() {
    let args = [
        "inspect",
        "--plan",
        "--json",
        "--size",
        "256",
        "--output",
        "baseColor,roughness,height,normal",
    ];
    let v2 = report(run(&args, &parent("brick-paving"), &brick_recipes()), 0);
    assert_eq!(v2["schemaVersion"], 3);
    assert_eq!(v2["plan"]["documentVersion"], 2);
    let recipes: Vec<&str> = v2["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(recipes, ["brick-structure", "masked-layer-surface"]);
    for recipe in v2["recipes"].as_array().unwrap() {
        assert_eq!(recipe["recipeVersion"], 1);
        assert_eq!(recipe["sha256"].as_str().unwrap().len(), 64);
    }
    let v1 = report(run(&args, &accepted("brick-paving"), &[]), 0);
    assert!(
        v1.get("recipes").is_none(),
        "v1 reports keep their exact shape"
    );
    assert_eq!(v1["plan"]["documentVersion"], 1);
    assert_eq!(
        v2["plan"]["passes"].as_array().unwrap().len(),
        v1["plan"]["passes"].as_array().unwrap().len()
    );

    let mut overridden = args.to_vec();
    overridden.extend(["--set", "seed=7"]);
    assert_eq!(
        report(
            run(&overridden, &parent("brick-paving"), &brick_recipes()),
            0
        )["plan"]["documentVersion"],
        2
    );
    overridden.extend(["--set", "heightPattern=7"]);
    let rejected = report(
        run(&overridden, &parent("brick-paving"), &brick_recipes()),
        2,
    );
    assert_eq!(first(&rejected)["code"], "MIX_COMPILE_INVALID_REQUEST");

    let human = run(
        &["inspect", "--plan"],
        &parent("brick-paving"),
        &brick_recipes(),
    );
    assert!(String::from_utf8_lossy(&human.stdout).contains("Recipe: brick-structure@1 sha256:"));
}

#[test]
fn asset_pack_rejects_v2_parents() {
    let out = std::env::temp_dir().join(format!(
        "mixture-recipe-pack-{}.mixpack",
        std::process::id()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_mixture"))
        .args(["asset", "pack"])
        .arg(parent("brick-paving"))
        .arg("--out")
        .arg(&out)
        .arg("--json")
        .output()
        .unwrap();
    let value = report(output, 2);
    assert_eq!(first(&value)["code"], "MIX_FORMAT_UNSUPPORTED_VERSION");
    assert!(!out.exists());
}

fn gpu_args(command: &mut Command) {
    command.args([
        "--backend",
        &std::env::var("MIXTURE_GPU_BACKEND").unwrap_or_else(|_| "auto".into()),
    ]);
    if std::env::var("MIXTURE_GPU_SOFTWARE").is_ok_and(|s| s == "1") {
        command.arg("--software");
    }
}
fn render(source: &Path, recipes: &[PathBuf], out: &Path, extra: &[&str]) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mixture"));
    command
        .arg("render")
        .arg(source)
        .arg("--out")
        .arg(out)
        .arg("--json");
    command.args(extra);
    for path in recipes {
        command.arg("--recipe").arg(path);
    }
    gpu_args(&mut command);
    let output = command.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

/// Material, recipe files, channels, v2 overrides and the equivalent v1 overrides.
type RenderCase = (
    &'static str,
    Vec<PathBuf>,
    &'static str,
    Vec<&'static str>,
    Vec<&'static str>,
);

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn rendered_parents_match_accepted_graphs_byte_for_byte() {
    let base = std::env::temp_dir().join(format!("mixture-recipe-render-{}", std::process::id()));
    let cases: [RenderCase; 2] = [
        (
            "brick-paving",
            brick_recipes(),
            "baseColor,normal,roughness,height",
            vec!["--set", "columns=6", "--set", "rows=12"],
            vec![
                "--set",
                "heightColumns=6",
                "--set",
                "colorColumns=6",
                "--set",
                "maskColumns=6",
                "--set",
                "heightRows=12",
                "--set",
                "colorRows=12",
                "--set",
                "maskRows=12",
            ],
        ),
        (
            "woven-fabric",
            woven_recipes(),
            "baseColor,metallic,roughness,height,normal",
            vec!["--set", "warpCount=12", "--set", "warpSeed=7"],
            vec![
                "--set",
                "warpshare_warpCount=12",
                "--set",
                "coverage_warpCount=12",
                "--set",
                "height_warpCount=12",
                "--set",
                "warpSeed=7",
            ],
        ),
    ];
    for (material, recipes, channels, v2_set, v1_set) in cases {
        for size in ["64", "257x129"] {
            for (label, v2_extra, v1_extra) in [
                ("default", vec![], vec![]),
                ("varied", v2_set.clone(), v1_set.clone()),
            ] {
                let dir = base.join(format!("{material}-{size}-{label}"));
                let common = ["--size", size, "--output", channels];
                let (new, old) = (dir.join("v2"), dir.join("v1"));
                render(
                    &parent(material),
                    &recipes,
                    &new,
                    &[&common[..], &v2_extra[..]].concat(),
                );
                render(
                    &accepted(material),
                    &[],
                    &old,
                    &[&common[..], &v1_extra[..]].concat(),
                );
                for channel in channels.split(',') {
                    let file = format!("{channel}.png");
                    assert_eq!(
                        std::fs::read(new.join(&file)).unwrap(),
                        std::fs::read(old.join(&file)).unwrap(),
                        "{material} {size} {label}: {channel} must be byte-identical"
                    );
                }
            }
        }
    }
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn render_accepts_repeated_recipes_and_fails_before_gpu_acquisition() {
    let out =
        std::env::temp_dir().join(format!("mixture-recipe-render-cpu-{}", std::process::id()));
    let output = run(
        &[
            "render",
            "--json",
            "--out",
            out.to_str().unwrap(),
            "--set",
            "seed=-1",
        ],
        &parent("brick-paving"),
        &brick_recipes(),
    );
    let value = report(output, 2);
    assert_eq!(first(&value)["code"], "MIX_PARAMETER_INVALID_VALUE");
    assert_eq!(first(&value)["evidence"]["instanceId"], "structure");
    assert!(
        value["context"].is_null(),
        "no GPU is acquired for an invalid request"
    );
    assert!(!out.exists());
}
