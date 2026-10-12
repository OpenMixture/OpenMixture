//! MAT-04d pixel identity: recipe parents against the accepted brick and woven graphs.
//!
//! For every frozen case and size, render the accepted v1 graph and its `.mix` v2
//! recipe parent through the public API on one explicit adapter and require
//! byte-identical channels. Also gate descriptor peaks and hardware timing budgets.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, ParentDocument, RecipeLimits, RecipeSet,
    RenderPlan, SafetyLimits, ValidatedDocument, compile,
};
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions, RenderOutput, Renderer};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Command, time::Instant};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(path: &str) -> Vec<u8> {
    std::fs::read(root().join(path)).unwrap()
}
fn json_file(path: &str) -> Value {
    serde_json::from_slice(&read(path)).unwrap()
}
fn renderer() -> Renderer {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        Ok("metal") => BackendPreference::Metal,
        other => panic!("explicit GPU backend required: {other:?}"),
    };
    let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        other => panic!("explicit MIXTURE_GPU_SOFTWARE=0/1 required: {other:?}"),
    };
    Renderer::new(
        pollster::block_on(GpuContext::request(GpuContextOptions {
            backend,
            software_adapter,
            ..Default::default()
        }))
        .unwrap(),
    )
}

fn accepted(material: &str) -> ValidatedDocument {
    let limits = SafetyLimits::default();
    MaterialDocument::decode(
        &read(&format!("fixtures/materials/{material}/material.mix")),
        &limits,
    )
    .unwrap()
    .into_validated(&limits)
    .unwrap()
}
fn parent(material: &str, plan: &Value) -> ValidatedDocument {
    let (limits, recipe_limits) = (SafetyLimits::default(), RecipeLimits::default());
    let sources: Vec<Vec<u8>> = plan["instances"]
        .as_object()
        .unwrap()
        .keys()
        .map(|id| read(&format!("fixtures/recipes/{id}.mixrecipe")))
        .collect();
    let slices: Vec<&[u8]> = sources.iter().map(Vec::as_slice).collect();
    let recipes = RecipeSet::decode(&slices, &limits, &recipe_limits).unwrap();
    ParentDocument::decode(
        &read(&format!(
            "fixtures/materials/{material}/material-recipes.mix"
        )),
        &limits,
        &recipe_limits,
    )
    .unwrap()
    .into_validated(&recipes, &limits, &recipe_limits)
    .unwrap()
}

/// Conceptual case controls mapped to accepted v1 and v2 public IDs. Brick uses
/// controls.json; woven ports the frozen request builder's derived values.
fn overrides(
    material: &str,
    controls: &Value,
    parent: &Value,
) -> (Map<String, Value>, Map<String, Value>) {
    if material == "brick-paving" {
        let mapping = json_file("fixtures/materials/brick-paving/controls.json");
        let (mut old, mut new) = (Map::new(), Map::new());
        for (control, value) in controls.as_object().unwrap() {
            for id in mapping[control].as_array().unwrap() {
                old.insert(id.as_str().unwrap().into(), value.clone());
            }
            match parent["conceptualControls"].get(control) {
                Some(ids) => ids.as_array().unwrap().iter().for_each(|id| {
                    new.insert(id.as_str().unwrap().into(), value.clone());
                }),
                None => {
                    new.insert(control.clone(), value.clone());
                }
            }
        }
        return (old, new);
    }
    let plan = json_file("fixtures/materials/woven-fabric/qualification-plan.json");
    let mut values = plan["defaults"].as_object().unwrap().clone();
    values.extend(controls.as_object().unwrap().clone());
    let detail = values["detailAmount"].as_f64().unwrap();
    let rough = values["yarnRoughness"].as_f64().unwrap() * (1.0 - 2.0 * detail);
    values.insert("roughnessMin".into(), json!(rough));
    for axis in ["warp", "weft"] {
        let dark: Vec<Value> = values[&format!("{axis}Color")]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if i == 3 {
                    json!(1)
                } else {
                    json!(v.as_f64().unwrap() * (1.0 - 4.0 * detail))
                }
            })
            .collect();
        values.insert(format!("{axis}Dark"), Value::Array(dark));
    }
    let old = plan["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["id"].as_str().unwrap().into(),
                values[b["control"].as_str().unwrap()].clone(),
            )
        })
        .collect();
    let new = parent["exposed"]["v2"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| {
            (
                id.as_str().unwrap().into(),
                values[id.as_str().unwrap()].clone(),
            )
        })
        .collect();
    (old, new)
}

/// (case, controls, sizes) for every frozen case and stress row of one material.
fn rows(material: &str) -> Vec<(String, Value, Vec<[u32; 2]>)> {
    let plan = json_file(&format!(
        "fixtures/materials/{material}/qualification-plan.json"
    ));
    let key = if material == "brick-paving" {
        "overrides"
    } else {
        "controls"
    };
    let size = |s: &Value| [s[0].as_u64().unwrap() as u32, s[1].as_u64().unwrap() as u32];
    let sizes: Vec<[u32; 2]> = plan["sizes"].as_array().unwrap().iter().map(size).collect();
    let mut rows: Vec<_> = plan["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["id"].as_str().unwrap().to_string(),
                c[key].clone(),
                sizes.clone(),
            )
        })
        .collect();
    for stress in plan["stress"].as_array().into_iter().flatten() {
        let sizes = stress["sizes"]
            .as_array()
            .unwrap()
            .iter()
            .map(size)
            .collect();
        rows.push((
            format!("stress-{}", stress["id"].as_str().unwrap()),
            stress[key].clone(),
            sizes,
        ));
    }
    rows
}

fn request(
    size: [u32; 2],
    outputs: &[OutputChannel],
    overrides: Map<String, Value>,
) -> CompileRequest {
    CompileRequest {
        size,
        outputs: outputs.to_vec(),
        overrides: overrides.into_iter().collect(),
        ..CompileRequest::default()
    }
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn render(gpu: &mut Renderer, plan: &RenderPlan) -> RenderOutput {
    let output = pollster::block_on(gpu.render(plan)).unwrap();
    assert_eq!(output.report().allocations.live_bytes, 0);
    output
}

#[test]
#[ignore = "requires release GPU and a fresh MAT-04d evidence path"]
fn recipe_parents_match_accepted_graphs_on_this_adapter() {
    if cfg!(debug_assertions) {
        panic!("timings require release mode");
    }
    let receipt = PathBuf::from(std::env::var("MIXTURE_RECIPE_EVIDENCE").unwrap());
    assert!(!receipt.exists(), "use a fresh receipt path");
    std::fs::write(&receipt, br#"{"ok":false,"completed":false}"#).unwrap();
    let software = std::env::var("MIXTURE_GPU_SOFTWARE").unwrap() == "1";
    let backend = std::env::var("MIXTURE_GPU_BACKEND").unwrap();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(root())
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    let revision = git(&["rev-parse", "HEAD"]).trim().to_string();
    let clean = git(&["status", "--porcelain"]).is_empty();
    let acceptance = json_file("fixtures/recipes/acceptance-plan.json");
    let mut table = Vec::new();
    let mut timing = Vec::new();
    let mut adapter = Value::Null;
    let mut identical = true;
    let mut budgets = true;
    for material in ["brick-paving", "woven-fabric"] {
        let parent_plan = acceptance["parents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["material"] == material)
            .unwrap()
            .clone();
        let qualification = json_file(&format!(
            "fixtures/materials/{material}/qualification-plan.json"
        ));
        let outputs: Vec<OutputChannel> = qualification["channels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().parse().unwrap())
            .collect();
        let peak_budget = parent_plan["maxDescriptorPeakBytes"].as_u64().unwrap();
        let (old_doc, new_doc) = (accepted(material), parent(material, &parent_plan));
        for (case, controls, sizes) in rows(material) {
            let (v1, v2) = overrides(material, &controls, &parent_plan);
            for size in sizes {
                if software && material == "woven-fabric" && size == [2048, 2048] {
                    // MAT-03 amendment 4: actual Cpu software adapters do not run woven 2048².
                    table.push(json!({"material":material,"case":case,"size":size,"status":"notRunOnSoftware"}));
                    continue;
                }
                let old_plan = compile(&old_doc, &request(size, &outputs, v1.clone())).unwrap();
                let new_plan = compile(&new_doc, &request(size, &outputs, v2.clone())).unwrap();
                assert_eq!(new_plan.passes().len(), old_plan.passes().len());
                let mut gpu = renderer();
                let (old, new) = (render(&mut gpu, &old_plan), render(&mut gpu, &new_plan));
                adapter = serde_json::to_value(&new.report().adapter).unwrap();
                let mut channels = Vec::new();
                for channel in new.channels() {
                    let other = old
                        .channels()
                        .iter()
                        .find(|c| c.channel == channel.channel)
                        .unwrap();
                    let same = other.pixels() == channel.pixels();
                    identical &= same;
                    channels.push(json!({"channel":channel.channel.as_str(),"sha256":digest(channel.pixels()),"identical":same}));
                }
                let peak = new_plan.estimates().peak_bytes;
                budgets &= peak <= peak_budget;
                table.push(json!({"material":material,"case":case,"size":size,"status":"rendered",
                    "parentPlanHash":new_plan.hash().as_str(),"acceptedPlanHash":old_plan.hash().as_str(),
                    "passes":new_plan.passes().len(),"parentPeakBytes":peak,"acceptedPeakBytes":old_plan.estimates().peak_bytes,
                    "channels":channels}));
            }
        }
        // Frozen timing budgets for the parent at the material's timed default sizes.
        for size in [1024_u32, 2048] {
            if software && material == "woven-fabric" && size == 2048 {
                continue;
            }
            let plan = compile(
                &new_doc,
                &request(
                    [size, size],
                    &outputs,
                    overrides(material, &json!({}), &parent_plan).1,
                ),
            )
            .unwrap();
            let mut gpu = renderer();
            let mut times = Vec::new();
            let mut name = String::new();
            for _ in 0..=qualification["timing"]["warmSamples"].as_u64().unwrap() {
                let start = Instant::now();
                let output = render(&mut gpu, &plan);
                times.push(start.elapsed().as_secs_f64() * 1000.);
                name = output.report().adapter.name.clone();
            }
            let mut warm = times[1..].to_vec();
            warm.sort_by(f64::total_cmp);
            let median = warm[warm.len() / 2];
            let policy = if software { "software" } else { "hardware" };
            let budget = &qualification["timing"][policy];
            let matched = if software {
                name.contains("SwiftShader")
            } else {
                name.contains(budget["adapter"].as_str().unwrap())
            };
            let key = if size == 1024 {
                "warmMedian1024Ms"
            } else {
                "warmMedian2048Ms"
            };
            let within = median <= budget[key].as_f64().unwrap()
                && (size != 1024 || times[0] <= budget["cold1024Ms"].as_f64().unwrap());
            // Software budgets are recorded for trend review but never gate a run.
            budgets &= software || (matched && within);
            timing.push(json!({"material":material,"size":[size,size],"adapter":name,"coldMs":times[0],"warmMs":&times[1..],
                "warmMedianMs":median,"budgetPolicy":policy,"matchedAdapter":matched,"budgetPassed":matched && within}));
        }
    }
    if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
        assert!(
            adapter["name"].as_str().unwrap().contains(&expected),
            "unexpected adapter"
        );
    }
    let rendered = table
        .iter()
        .filter(|row| row["status"] == "rendered")
        .count();
    let ok = identical && budgets && clean;
    std::fs::write(&receipt, serde_json::to_vec_pretty(&json!({
        "kind":"mat04d-recipe-parent-identity","ok":ok,"completed":true,
        "sourceRevision":revision,"workingTreeClean":clean,"backend":backend,"softwareAdapter":software,
        "adapter":adapter,"gates":{"pixelIdentity":identical,"budgets":budgets},
        "renderedRows":rendered,"rows":table,"timing":timing,
    })).unwrap()).unwrap();
    assert!(
        identical,
        "every channel must be byte-identical; see {}",
        receipt.display()
    );
    assert!(
        budgets,
        "descriptor and timing budgets; see {}",
        receipt.display()
    );
}
