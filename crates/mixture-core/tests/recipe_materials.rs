//! MAT-04c CPU gates: the brick and woven recipe parents against the accepted v1 graphs.
//!
//! Structural equivalence, control mapping, instance isolation, presets and
//! descriptor budgets for every frozen case. Pixel identity is MAT-04d GPU work.
use mixture_core::{
    CompileRequest, MaterialDocument, OutputChannel, ParentDocument, RecipeLimits, RecipeSet,
    SafetyLimits, ValidatedDocument, compile, normalize,
};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn read(path: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap()
}
fn json_file(path: &str) -> Value {
    serde_json::from_slice(&read(path)).unwrap()
}
fn plan() -> Value {
    json_file("fixtures/recipes/acceptance-plan.json")
}
fn parent_plan(material: &str) -> Value {
    plan()["parents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["material"] == material)
        .unwrap()
        .clone()
}
fn v1(material: &str) -> ValidatedDocument {
    let limits = SafetyLimits::default();
    MaterialDocument::decode(
        &read(&format!("fixtures/materials/{material}/material.mix")),
        &limits,
    )
    .unwrap()
    .into_validated(&limits)
    .unwrap()
}
fn v2(material: &str) -> ValidatedDocument {
    let (limits, recipe_limits) = (SafetyLimits::default(), RecipeLimits::default());
    let parent = parent_plan(material);
    let sources: Vec<Vec<u8>> = parent["instances"]
        .as_object()
        .unwrap()
        .keys()
        .map(|id| read(&format!("fixtures/recipes/{id}.mixrecipe")))
        .collect();
    let slices: Vec<&[u8]> = sources.iter().map(Vec::as_slice).collect();
    let recipes = RecipeSet::decode(&slices, &limits, &recipe_limits).unwrap();
    ParentDocument::decode(
        &read(parent["path"].as_str().unwrap()),
        &limits,
        &recipe_limits,
    )
    .unwrap()
    .into_validated(&recipes, &limits, &recipe_limits)
    .unwrap()
}

/// Accepted v1 node IDs and the expanded IDs that replace them; others keep their ID.
fn renames(material: &str) -> BTreeMap<&'static str, &'static str> {
    let pairs: &[(&str, &str)] = match material {
        "brick-paving" => &[
            ("heightPattern", "structure--height"),
            ("colorPattern", "structure--color"),
            ("maskPattern", "structure--mask"),
            ("mortarColor", "surface--underColor"),
            ("mortarRoughness", "surface--underRoughness"),
            ("baseColor", "surface--color"),
            ("roughness", "surface--roughness"),
        ],
        _ => &[
            ("n00-weftNoise", "weft--noise"),
            ("n01-weftDirection", "weft--direction"),
            ("n02-weftGrain", "weft--grain"),
            ("n10-weftColor", "weft--color"),
            ("n03-warpNoise", "warp--noise"),
            ("n04-warpDirection", "warp--direction"),
            ("n05-warpGrain", "warp--grain"),
            ("n07-warpColor", "warp--color"),
            ("n06-weave-warp-share", "weave--warpShare"),
            ("n09-weave-coverage", "weave--coverage"),
            ("n17-weave-height", "weave--height"),
            ("n12-backingColor", "surface--underColor"),
            ("n15-backingRoughness", "surface--underRoughness"),
            ("n13-baseColor", "surface--color"),
            ("n16-roughness", "surface--roughness"),
        ],
    };
    pairs.iter().copied().collect()
}

/// Normalized nodes (type, version, every effective parameter) and edges, keyed by
/// expanded ID, so two graphs compare equal exactly when they are isomorphic.
type Graph = (
    BTreeMap<String, Value>,
    BTreeSet<(String, String, String, String)>,
);
fn graph(
    document: &ValidatedDocument,
    overrides: &Map<String, Value>,
    renames: &BTreeMap<&str, &str>,
) -> Graph {
    let request = CompileRequest {
        overrides: overrides.clone().into_iter().collect(),
        ..CompileRequest::default()
    };
    let normalized = normalize(document, &request).unwrap();
    let name = |id: &str| renames.get(id).map_or(id.to_string(), |r| r.to_string());
    let nodes = normalized
        .document()
        .nodes
        .iter()
        .map(|n| {
            (
                name(&n.id),
                json!({"type": n.type_id, "version": n.version, "parameters": n.parameters}),
            )
        })
        .collect();
    let edges = normalized
        .document()
        .edges
        .iter()
        .map(|e| {
            (
                name(&e.from.node_id),
                e.from.port_id.clone(),
                name(&e.to.node_id),
                e.to.port_id.clone(),
            )
        })
        .collect();
    (nodes, edges)
}

/// Brick conceptual controls: v1 bindings come from controls.json, v2 IDs from the plan.
fn brick_overrides(controls: &Value) -> (Map<String, Value>, Map<String, Value>) {
    let mapping = json_file("fixtures/materials/brick-paving/controls.json");
    let conceptual = &parent_plan("brick-paving")["conceptualControls"];
    let (mut old, mut new) = (Map::new(), Map::new());
    for (control, value) in controls.as_object().unwrap() {
        for id in mapping[control].as_array().unwrap() {
            old.insert(id.as_str().unwrap().into(), value.clone());
        }
        match conceptual.get(control) {
            Some(ids) => ids.as_array().unwrap().iter().for_each(|id| {
                new.insert(id.as_str().unwrap().into(), value.clone());
            }),
            None => {
                new.insert(control.clone(), value.clone());
            }
        }
    }
    (old, new)
}

/// Woven caller mapping, ported from scripts/woven-fabric-requests.mjs: derived
/// roughnessMin and dark colors, then v1 bindings and v2 public IDs.
fn woven_overrides(changes: &Value) -> (Map<String, Value>, Map<String, Value>) {
    let plan = json_file("fixtures/materials/woven-fabric/qualification-plan.json");
    let mut values = plan["defaults"].as_object().unwrap().clone();
    values.extend(changes.as_object().unwrap().clone());
    let detail = values["detailAmount"].as_f64().unwrap();
    let rough = values["yarnRoughness"].as_f64().unwrap() * (1.0 - 2.0 * detail);
    values.insert("roughnessMin".into(), json!(rough));
    for axis in ["warp", "weft"] {
        let color: Vec<Value> = values[&format!("{axis}Color")]
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
        values.insert(format!("{axis}Dark"), Value::Array(color));
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
    let new = parent_plan("woven-fabric")["exposed"]["v2"]
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

fn cases(material: &str) -> Vec<(String, Value)> {
    let plan = json_file(&format!(
        "fixtures/materials/{material}/qualification-plan.json"
    ));
    let key = if material == "brick-paving" {
        "overrides"
    } else {
        "controls"
    };
    let mut rows = vec![("defaults".to_string(), json!({}))];
    for section in ["cases", "stress"] {
        for case in plan[section].as_array().into_iter().flatten() {
            rows.push((case["id"].as_str().unwrap().into(), case[key].clone()));
        }
    }
    rows
}
fn overrides(material: &str, controls: &Value) -> (Map<String, Value>, Map<String, Value>) {
    if material == "brick-paving" {
        brick_overrides(controls)
    } else {
        woven_overrides(controls)
    }
}

#[test]
fn parents_are_isomorphic_to_the_accepted_graphs_for_every_frozen_case() {
    for material in ["brick-paving", "woven-fabric"] {
        let (old, new, renames) = (v1(material), v2(material), renames(material));
        assert_eq!(
            new.recipe_closure().len(),
            parent_plan(material)["instances"]
                .as_object()
                .unwrap()
                .len()
        );
        let cases = cases(material);
        assert!(cases.len() > 4, "{material}: frozen cases are present");
        for (case, controls) in cases {
            let (v1_overrides, v2_overrides) = overrides(material, &controls);
            assert_eq!(
                graph(&old, &v1_overrides, &renames),
                graph(&new, &v2_overrides, &BTreeMap::new()),
                "{material}/{case}: expanded parent must equal the accepted graph"
            );
        }
    }
}

#[test]
fn parent_public_ids_and_budgets_match_the_frozen_plan() {
    for material in ["brick-paving", "woven-fabric"] {
        let parent = parent_plan(material);
        let exposed: BTreeSet<String> = v2(material)
            .document()
            .exposed_parameters
            .iter()
            .map(|b| b.id.clone())
            .collect();
        let frozen: BTreeSet<String> = parent["exposed"]["v2"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap().into())
            .collect();
        assert_eq!(exposed, frozen, "{material}: v2 public IDs");

        let qualification = json_file(parent["qualificationPlan"].as_str().unwrap());
        let budget = parent["maxDescriptorPeakBytes"].as_u64().unwrap();
        let outputs: Vec<OutputChannel> = qualification["channels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().parse().unwrap())
            .collect();
        for size in qualification["sizes"].as_array().unwrap() {
            let request = CompileRequest {
                size: [
                    size[0].as_u64().unwrap() as u32,
                    size[1].as_u64().unwrap() as u32,
                ],
                outputs: outputs.clone(),
                ..CompileRequest::default()
            };
            let (old, new) = (
                compile(&v1(material), &request).unwrap(),
                compile(&v2(material), &request).unwrap(),
            );
            assert_eq!(
                new.passes().len(),
                old.passes().len(),
                "{material} {size}: pass count"
            );
            assert!(
                new.estimates().peak_bytes <= budget,
                "{material} {size}: parent peak {} exceeds frozen {budget}",
                new.estimates().peak_bytes
            );
        }
    }
}

#[test]
fn woven_grain_instances_stay_isolated() {
    let new = v2("woven-fabric");
    let (_, defaults) = woven_overrides(&json!({}));
    let mut changed = defaults.clone();
    changed.insert("warpSeed".into(), json!(4294967295_u64));
    let (base, seeded) = (
        graph(&new, &defaults, &BTreeMap::new()).0,
        graph(&new, &changed, &BTreeMap::new()).0,
    );
    let differing: Vec<&String> = base.keys().filter(|id| base[*id] != seeded[*id]).collect();
    assert_eq!(
        differing,
        ["warp--noise"],
        "only the warp instance's noise node changes"
    );
}

#[test]
fn presets_reproduce_their_frozen_cases() {
    let frozen = &plan()["presets"];
    for material in ["brick-paving", "woven-fabric"] {
        let corpus = json_file(&format!("fixtures/recipes/presets/{material}.json"));
        assert_eq!(corpus["version"], 1);
        assert_eq!(corpus["parent"], parent_plan(material)["path"]);
        let names: Vec<&String> = corpus["presets"].as_object().unwrap().keys().collect();
        let mut expected: Vec<String> = frozen[material]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap().into())
            .collect();
        expected.sort();
        assert_eq!(names, expected.iter().collect::<Vec<_>>());
        let new = v2(material);
        for (case, controls) in cases(material) {
            let Some(preset) = corpus["presets"].get(&case) else {
                continue;
            };
            let (_, full) = overrides(material, &controls);
            assert_eq!(
                graph(&new, preset.as_object().unwrap(), &BTreeMap::new()),
                graph(&new, &full, &BTreeMap::new()),
                "{material}/{case}: preset overrides reproduce the frozen case"
            );
        }
    }
}
