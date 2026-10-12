//! MAT-04b recipe documents, `.mix` v2 parents, exact closures and expansion.
use mixture_core::{
    CompileRequest, DocumentError, MaterialDocument, OutputChannel, ParentDocument, RecipeLimits,
    RecipeSet, SafetyLimits, ValidatedDocument, compile, normalize,
};
use serde_json::{Value, json};

fn masked() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../fixtures/recipes/masked-layer-surface.mixrecipe"
    ))
    .unwrap()
}
/// Seeded grain: the seed has no contract default, so the interface requires it.
fn grain() -> Value {
    json!({
        "version": 1, "id": "grain", "recipeVersion": 1,
        "nodes": [
            {"id":"noise","type":"fractal-noise","version":2},
            {"id":"levels","type":"levels","version":1,"parameters":{"inputMin":0.3,"inputMax":0.7}}
        ],
        "edges": [{"from":{"nodeId":"noise","portId":"value"},"to":{"nodeId":"levels","portId":"in"}}],
        "outputs": [{"id":"value","from":{"nodeId":"levels","portId":"value"}}],
        "parameters": [
            {"id":"seed","targets":[{"nodeId":"noise","parameterId":"seed"}]},
            {"id":"high","targets":[{"nodeId":"levels","parameterId":"inputMax"}]}
        ]
    })
}
/// One interface parameter fanned out to two nodes with equal source values.
fn twin() -> Value {
    json!({
        "version": 1, "id": "twin", "recipeVersion": 1,
        "nodes": [
            {"id":"a","type":"fractal-noise","version":2,"parameters":{"scale":8}},
            {"id":"b","type":"fractal-noise","version":2,"parameters":{"scale":8,"octaves":3}}
        ],
        "edges": [],
        "outputs": [
            {"id":"a","from":{"nodeId":"a","portId":"value"}},
            {"id":"b","from":{"nodeId":"b","portId":"value"}}
        ],
        "parameters": [
            {"id":"seed","targets":[{"nodeId":"a","parameterId":"seed"},{"nodeId":"b","parameterId":"seed"}]},
            {"id":"scale","targets":[{"nodeId":"a","parameterId":"scale"},{"nodeId":"b","parameterId":"scale"}]}
        ]
    })
}
/// Two independent pass-through paths, for cycles through and not through an instance.
fn passes() -> Value {
    json!({
        "version": 1, "id": "passes", "recipeVersion": 1,
        "nodes": [
            {"id":"first","type":"levels","version":1},
            {"id":"second","type":"levels","version":1}
        ],
        "edges": [],
        "inputs": [
            {"id":"a","targets":[{"nodeId":"first","portId":"in"}]},
            {"id":"b","targets":[{"nodeId":"second","portId":"in"}]}
        ],
        "outputs": [
            {"id":"a","from":{"nodeId":"first","portId":"value"}},
            {"id":"b","from":{"nodeId":"second","portId":"value"}}
        ]
    })
}
fn parent() -> Value {
    json!({
        "version": 2,
        "nodes": [
            {"id":"mask","type":"constant-scalar","version":1,"parameters":{"value":0.3}},
            {"id":"over","type":"constant-color","version":1,"parameters":{"value":[0.4,0.2,0.1,1]}},
            {"id":"overRoughness","type":"constant-scalar","version":1,"parameters":{"value":0.4}},
            {"id":"out","type":"material-output","version":1}
        ],
        "instances": [
            {"id":"layer","recipe":"masked-layer-surface","recipeVersion":1,"parameters":{"underRoughness":0.95}}
        ],
        "edges": [
            {"from":{"nodeId":"mask","portId":"value"},"to":{"nodeId":"layer","portId":"mask"}},
            {"from":{"nodeId":"over","portId":"color"},"to":{"nodeId":"layer","portId":"overColor"}},
            {"from":{"nodeId":"overRoughness","portId":"value"},"to":{"nodeId":"layer","portId":"overRoughness"}},
            {"from":{"nodeId":"layer","portId":"baseColor"},"to":{"nodeId":"out","portId":"baseColor"}},
            {"from":{"nodeId":"layer","portId":"roughness"},"to":{"nodeId":"out","portId":"roughness"}}
        ],
        "exposedParameters": [
            {"id":"backing","nodeId":"layer","parameterId":"underColor"},
            {"id":"maskValue","nodeId":"mask","parameterId":"value"}
        ]
    })
}
/// The hand-flattened v1 graph, written with the expanded node IDs.
fn flattened() -> Value {
    json!({
        "version": 1,
        "nodes": [
            {"id":"mask","type":"constant-scalar","version":1,"parameters":{"value":0.3}},
            {"id":"over","type":"constant-color","version":1,"parameters":{"value":[0.4,0.2,0.1,1]}},
            {"id":"overRoughness","type":"constant-scalar","version":1,"parameters":{"value":0.4}},
            {"id":"out","type":"material-output","version":1},
            {"id":"layer--underColor","type":"constant-color","version":1,"parameters":{"value":[0.16,0.15,0.13,1]}},
            {"id":"layer--underRoughness","type":"constant-scalar","version":1,"parameters":{"value":0.95}},
            {"id":"layer--color","type":"blend","version":1},
            {"id":"layer--roughness","type":"scalar-mask-blend","version":1}
        ],
        "edges": [
            {"from":{"nodeId":"layer--underColor","portId":"color"},"to":{"nodeId":"layer--color","portId":"a"}},
            {"from":{"nodeId":"layer--underRoughness","portId":"value"},"to":{"nodeId":"layer--roughness","portId":"a"}},
            {"from":{"nodeId":"mask","portId":"value"},"to":{"nodeId":"layer--color","portId":"mask"}},
            {"from":{"nodeId":"mask","portId":"value"},"to":{"nodeId":"layer--roughness","portId":"mask"}},
            {"from":{"nodeId":"over","portId":"color"},"to":{"nodeId":"layer--color","portId":"b"}},
            {"from":{"nodeId":"overRoughness","portId":"value"},"to":{"nodeId":"layer--roughness","portId":"b"}},
            {"from":{"nodeId":"layer--color","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}},
            {"from":{"nodeId":"layer--roughness","portId":"value"},"to":{"nodeId":"out","portId":"roughness"}}
        ],
        "exposedParameters": [
            {"id":"backing","nodeId":"layer--underColor","parameterId":"value"},
            {"id":"maskValue","nodeId":"mask","parameterId":"value"}
        ]
    })
}

fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn set(recipes: &[Value]) -> Result<RecipeSet, DocumentError> {
    let sources: Vec<Vec<u8>> = recipes.iter().map(bytes).collect();
    let slices: Vec<&[u8]> = sources.iter().map(Vec::as_slice).collect();
    RecipeSet::decode(&slices, &SafetyLimits::default(), &RecipeLimits::default())
}
fn expand(parent: &Value, recipes: &[Value]) -> Result<ValidatedDocument, DocumentError> {
    let limits = SafetyLimits::default();
    let recipe_limits = RecipeLimits::default();
    let recipes = set(recipes)?;
    ParentDocument::decode(&bytes(parent), &limits, &recipe_limits)?.into_validated(
        &recipes,
        &limits,
        &recipe_limits,
    )
}
fn codes(error: &DocumentError) -> Vec<String> {
    error
        .report()
        .diagnostics()
        .iter()
        .map(|d| d.code.as_str().to_string())
        .collect()
}
fn diagnostics(error: &DocumentError) -> Vec<Value> {
    error
        .report()
        .diagnostics()
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect()
}
fn rejects(result: Result<ValidatedDocument, DocumentError>, code: &str) -> Vec<Value> {
    let error = result.expect_err("expected a structured failure");
    assert!(
        codes(&error).iter().any(|c| c == code),
        "expected {code}, got {:?}",
        codes(&error)
    );
    diagnostics(&error)
        .into_iter()
        .filter(|d| d["code"] == code)
        .collect()
}
fn request(overrides: Value) -> CompileRequest {
    CompileRequest {
        size: [16, 16],
        outputs: vec![OutputChannel::BaseColor, OutputChannel::Roughness],
        overrides: serde_json::from_value(overrides).unwrap(),
        ..CompileRequest::default()
    }
}

#[test]
fn expanded_parent_compiles_like_its_hand_flattened_v1_graph() {
    let expanded = expand(&parent(), &[masked()]).unwrap();
    let v1 = MaterialDocument::decode(&bytes(&flattened()), &SafetyLimits::default())
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
    for overrides in [
        json!({}),
        json!({"backing":[0.02,0.01,0.01,1],"maskValue":0.8}),
    ] {
        let request = request(overrides);
        let parent_plan = compile(&expanded, &request).unwrap();
        let v1_plan = compile(&v1, &request).unwrap();
        assert_eq!(parent_plan.document_version(), 2);
        assert_eq!(v1_plan.document_version(), 1);
        let parent_body = String::from_utf8(parent_plan.hash_input().unwrap()).unwrap();
        let v1_body = String::from_utf8(v1_plan.hash_input().unwrap()).unwrap();
        assert_eq!(
            parent_body.replace("\"documentVersion\":2", "\"documentVersion\":1"),
            v1_body,
            "only the document version may differ"
        );
        assert_ne!(parent_plan.hash(), v1_plan.hash());
    }
    let closure = expanded.recipe_closure();
    assert_eq!(closure.len(), 1);
    assert_eq!(closure[0].id, "masked-layer-surface");
    assert_eq!(closure[0].byte_length, bytes(&masked()).len() as u64);
    assert_eq!(closure[0].sha256.len(), 64);
}

#[test]
fn interface_parameters_fan_out_and_instances_stay_isolated() {
    let parent = json!({
        "version": 2,
        "nodes": [
            {"id":"mix","type":"scalar-blend","version":1},
            {"id":"tint","type":"gradient-map","version":1},
            {"id":"out","type":"material-output","version":1}
        ],
        "instances": [
            {"id":"warp","recipe":"twin","recipeVersion":1,"parameters":{"seed":7}},
            {"id":"weft","recipe":"twin","recipeVersion":1,"parameters":{"seed":9,"scale":16}}
        ],
        "edges": [
            {"from":{"nodeId":"warp","portId":"a"},"to":{"nodeId":"mix","portId":"a"}},
            {"from":{"nodeId":"weft","portId":"b"},"to":{"nodeId":"mix","portId":"b"}},
            {"from":{"nodeId":"mix","portId":"value"},"to":{"nodeId":"tint","portId":"in"}},
            {"from":{"nodeId":"tint","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}
        ],
        "exposedParameters": [{"id":"warpSeed","nodeId":"warp","parameterId":"seed"}]
    });
    let expanded = expand(&parent, &[twin()]).unwrap();
    let seed = |doc: &ValidatedDocument, id: &str| doc.parameter(id, "seed").unwrap();
    assert_eq!(seed(&expanded, "warp--a"), json!(7));
    assert_eq!(seed(&expanded, "warp--b"), json!(7));
    assert_eq!(seed(&expanded, "weft--a"), json!(9));
    assert_eq!(expanded.parameter("warp--a", "scale").unwrap(), json!(8));
    assert_eq!(expanded.parameter("weft--b", "scale").unwrap(), json!(16));
    // Recipe-fixed parameters keep their source values in every instance.
    assert_eq!(expanded.parameter("weft--b", "octaves").unwrap(), json!(3));

    let mut request = CompileRequest::default();
    request.overrides.insert("warpSeed".into(), json!(1234));
    let normalized = normalize(&expanded, &request).unwrap();
    let value = |id: &str| {
        normalized
            .document()
            .nodes
            .iter()
            .find(|n| n.id == id)
            .unwrap()
            .parameters["seed"]
            .clone()
    };
    assert_eq!(value("warp--a"), json!(1234));
    assert_eq!(value("warp--b"), json!(1234));
    assert_eq!(value("weft--a"), json!(9));
    assert_eq!(value("weft--b"), json!(9));

    request.overrides.insert("warpSeed".into(), json!(-1));
    let error = normalize(&expanded, &request).unwrap_err();
    let failures = error.report().diagnostics();
    assert_eq!(
        failures.len(),
        1,
        "one diagnostic per public ID, not per target"
    );
    let failure = serde_json::to_value(&failures[0]).unwrap();
    assert_eq!(failure["code"], "MIX_PARAMETER_INVALID_VALUE");
    assert_eq!(failure["evidence"]["instanceId"], "warp");
}

#[test]
fn v1_entry_points_reject_v2_parents_by_version() {
    let limits = SafetyLimits::default();
    let error = MaterialDocument::decode(&bytes(&parent()), &limits).unwrap_err();
    assert_eq!(codes(&error), ["MIX_FORMAT_UNSUPPORTED_VERSION"]);
    let mut v1 = flattened();
    v1["instances"] = json!([]);
    let error = MaterialDocument::decode(&bytes(&v1), &limits).unwrap_err();
    assert_eq!(codes(&error), ["MIX_FORMAT_INVALID_DOCUMENT"]);
    let error = ParentDocument::decode(&bytes(&flattened()), &limits, &RecipeLimits::default())
        .unwrap_err();
    assert_eq!(codes(&error), ["MIX_FORMAT_UNSUPPORTED_VERSION"]);
}

#[test]
fn recipe_documents_are_strict_and_cannot_nest() {
    let limits = SafetyLimits::default();
    let decode = |v: &Value| mixture_core::RecipeDocument::decode(&bytes(v), &limits);
    let mut unknown = masked();
    unknown["layout"] = json!({});
    assert_eq!(
        codes(&decode(&unknown).unwrap_err()),
        ["MIX_FORMAT_INVALID_DOCUMENT"]
    );
    let mut nested = masked();
    nested["instances"] = json!([]);
    let error = decode(&nested).unwrap_err();
    assert_eq!(codes(&error), ["MIX_FORMAT_INVALID_DOCUMENT"]);
    assert!(
        diagnostics(&error)[0]["evidence"]["sourceMessage"]
            .as_str()
            .unwrap()
            .contains("nesting")
    );
    let mut version = masked();
    version["version"] = json!(2);
    assert_eq!(
        codes(&decode(&version).unwrap_err()),
        ["MIX_FORMAT_UNSUPPORTED_VERSION"]
    );
    let duplicate = br#"{"version":1,"version":1,"id":"x","recipeVersion":1,"nodes":[],"edges":[],"outputs":[]}"#;
    assert_eq!(
        codes(&mixture_core::RecipeDocument::decode(duplicate, &limits).unwrap_err()),
        ["MIX_FORMAT_INVALID_DOCUMENT"]
    );
}

#[test]
fn recipe_interfaces_reject_invalid_bindings() {
    let invalid = |recipe: Value| -> Vec<Value> {
        let error = set(&[recipe]).expect_err("invalid recipe");
        diagnostics(&error)
    };
    let has = |found: &[Value], code: &str| found.iter().any(|d| d["code"] == code);

    let mut forbidden = masked();
    forbidden["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"out","type":"material-output","version":1}));
    assert!(has(&invalid(forbidden), "MIX_RECIPE_FORBIDDEN_NODE"));

    let mut kinds = masked();
    kinds["inputs"][1]["targets"]
        .as_array_mut()
        .unwrap()
        .push(json!({"nodeId":"roughness","portId":"b"}));
    let found = invalid(kinds);
    assert!(has(&found, "MIX_RECIPE_INVALID_INTERFACE"));
    assert!(
        found
            .iter()
            .all(|d| d["evidence"]["recipeId"] == "masked-layer-surface")
    );

    let mut connected = masked();
    connected["inputs"][0]["targets"] = json!([{"nodeId":"color","portId":"a"}]);
    assert!(has(&invalid(connected), "MIX_RECIPE_INVALID_INTERFACE"));

    let mut unbound_seed = grain();
    unbound_seed["parameters"].as_array_mut().unwrap().remove(0);
    assert!(has(&invalid(unbound_seed), "MIX_RECIPE_INVALID_INTERFACE"));

    let mut unequal = twin();
    unequal["nodes"][1]["parameters"]["scale"] = json!(4);
    assert!(has(&invalid(unequal), "MIX_RECIPE_INVALID_INTERFACE"));

    let mut alias = masked();
    alias["outputs"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"again","from":{"nodeId":"color","portId":"color"}}));
    assert!(has(&invalid(alias), "MIX_RECIPE_INVALID_INTERFACE"));

    let mut identity = masked();
    identity["id"] = json!("Masked");
    assert!(has(&invalid(identity), "MIX_FORMAT_INVALID_DOCUMENT"));
}

#[test]
fn closures_must_be_exact_and_bounded() {
    let found = rejects(expand(&parent(), &[]), "MIX_RECIPE_MISSING");
    assert_eq!(found[0]["evidence"]["instanceId"], "layer");
    let mut other_version = masked();
    other_version["recipeVersion"] = json!(2);
    let found = rejects(expand(&parent(), &[other_version]), "MIX_RECIPE_MISSING");
    assert_eq!(found[0]["evidence"]["suppliedVersions"], "2");
    rejects(expand(&parent(), &[masked(), grain()]), "MIX_RECIPE_UNUSED");
    let error = set(&[masked(), masked()]).unwrap_err();
    assert_eq!(codes(&error), ["MIX_RECIPE_DUPLICATE"]);

    let source = bytes(&masked());
    let limits = SafetyLimits::default();
    let count = RecipeLimits {
        recipe_count: 0,
        ..RecipeLimits::default()
    };
    let error = RecipeSet::decode(&[&source], &limits, &count).unwrap_err();
    assert_eq!(codes(&error), ["MIX_LIMIT_RECIPE_COUNT_EXCEEDED"]);
    let size = RecipeLimits {
        recipe_bytes: source.len() as u64 - 1,
        ..RecipeLimits::default()
    };
    let error = RecipeSet::decode(&[&source], &limits, &size).unwrap_err();
    assert_eq!(codes(&error), ["MIX_LIMIT_RECIPE_BYTES_EXCEEDED"]);
    let exact = RecipeLimits {
        recipe_count: 1,
        recipe_bytes: source.len() as u64,
        ..RecipeLimits::default()
    };
    assert!(RecipeSet::decode(&[&source], &limits, &exact).is_ok());

    let instances = RecipeLimits {
        instance_count: 0,
        ..RecipeLimits::default()
    };
    let error = ParentDocument::decode(&bytes(&parent()), &limits, &instances).unwrap_err();
    assert_eq!(codes(&error), ["MIX_LIMIT_INSTANCE_COUNT_EXCEEDED"]);
}

#[test]
fn parent_instances_are_checked_before_expansion() {
    let mut reserved = parent();
    reserved["nodes"][0]["id"] = json!("mask--a");
    rejects(expand(&reserved, &[masked()]), "MIX_NODE_INVALID_ID");

    let mut shared = parent();
    shared["instances"][0]["id"] = json!("mask");
    rejects(expand(&shared, &[masked()]), "MIX_NODE_DUPLICATE_ID");

    let mut unknown = parent();
    unknown["instances"][0]["parameters"]["tint"] = json!(1);
    let found = rejects(expand(&unknown, &[masked()]), "MIX_PARAMETER_UNKNOWN");
    assert_eq!(found[0]["evidence"]["instanceId"], "layer");

    let mut value = parent();
    value["instances"][0]["parameters"]["underRoughness"] = json!(2);
    rejects(expand(&value, &[masked()]), "MIX_PARAMETER_INVALID_VALUE");

    let seedless = json!({
        "version": 2,
        "nodes": [{"id":"tint","type":"gradient-map","version":1},{"id":"out","type":"material-output","version":1}],
        "instances": [{"id":"g","recipe":"grain","recipeVersion":1}],
        "edges": [
            {"from":{"nodeId":"g","portId":"value"},"to":{"nodeId":"tint","portId":"in"}},
            {"from":{"nodeId":"tint","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}
        ]
    });
    let found = rejects(expand(&seedless, &[grain()]), "MIX_PARAMETER_INVALID_VALUE");
    assert_eq!(found[0]["evidence"]["interfaceId"], "seed");

    let mut port = parent();
    port["edges"][0]["to"]["portId"] = json!("weight");
    rejects(expand(&port, &[masked()]), "MIX_PORT_UNKNOWN");

    let mut kind = parent();
    kind["edges"][1]["to"]["portId"] = json!("mask");
    kind["edges"].as_array_mut().unwrap().remove(0);
    rejects(expand(&kind, &[masked()]), "MIX_PORT_TYPE_MISMATCH");

    let mut required = parent();
    required["edges"].as_array_mut().unwrap().remove(0);
    let found = rejects(
        expand(&required, &[masked()]),
        "MIX_PORT_REQUIRED_CONNECTION",
    );
    assert_eq!(found[0]["portId"], "mask");

    let mut twice = parent();
    twice["edges"].as_array_mut().unwrap().push(json!({"from":{"nodeId":"overRoughness","portId":"value"},"to":{"nodeId":"layer","portId":"mask"}}));
    rejects(expand(&twice, &[masked()]), "MIX_GRAPH_MULTIPLE_INPUTS");

    let mut exposed = parent();
    exposed["exposedParameters"][0]["parameterId"] = json!("value");
    rejects(
        expand(&exposed, &[masked()]),
        "MIX_EXPOSED_PARAMETER_INVALID",
    );
    let mut public = parent();
    public["exposedParameters"][1]["id"] = json!("backing");
    rejects(
        expand(&public, &[masked()]),
        "MIX_EXPOSED_PARAMETER_INVALID",
    );

    let mut long = parent();
    let id = "l".repeat(60);
    long["instances"][0]["id"] = json!(id);
    for edge in long["edges"].as_array_mut().unwrap() {
        for end in ["from", "to"] {
            if edge[end]["nodeId"] == "layer" {
                edge[end]["nodeId"] = json!(id);
            }
        }
    }
    long["exposedParameters"][0]["nodeId"] = json!(id);
    let found = rejects(expand(&long, &[masked()]), "MIX_NODE_INVALID_ID");
    let mut nodes: Vec<_> = found
        .iter()
        .map(|d| d["evidence"]["recipeNodeId"].clone())
        .collect();
    nodes.sort_by_key(|n| n.to_string());
    assert_eq!(
        nodes,
        [
            json!("color"),
            json!("roughness"),
            json!("underColor"),
            json!("underRoughness")
        ]
    );

    let limits = SafetyLimits {
        nodes: 7,
        ..SafetyLimits::default()
    };
    let recipes = set(&[masked()]).unwrap();
    let source =
        ParentDocument::decode(&bytes(&parent()), &limits, &RecipeLimits::default()).unwrap();
    let error = source
        .into_validated(&recipes, &limits, &RecipeLimits::default())
        .unwrap_err();
    let found = diagnostics(&error);
    assert_eq!(found[0]["code"], "MIX_LIMIT_NODES_EXCEEDED");
    assert_eq!(found[0]["evidence"]["observed"], 8);
    assert_eq!(found[0]["evidence"]["expanded"], true);
}

#[test]
fn cycles_are_detected_on_the_expanded_graph() {
    let graph = |back: Value| {
        json!({
            "version": 2,
            "nodes": [
                {"id":"source","type":"constant-scalar","version":1},
                {"id":"tint","type":"gradient-map","version":1},
                {"id":"out","type":"material-output","version":1}
            ],
            "instances": [{"id":"p","recipe":"passes","recipeVersion":1}],
            "edges": [
                {"from":{"nodeId":"source","portId":"value"},"to":{"nodeId":"p","portId":"a"}},
                back,
                {"from":{"nodeId":"p","portId":"b"},"to":{"nodeId":"tint","portId":"in"}},
                {"from":{"nodeId":"tint","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}
            ]
        })
    };
    // a's output feeding b's input passes through the instance without a cycle.
    let through =
        graph(json!({"from":{"nodeId":"p","portId":"a"},"to":{"nodeId":"p","portId":"b"}}));
    assert!(expand(&through, &[passes()]).is_ok());
    // b's output feeding b's input is a real cycle inside the instance.
    let mut cyclic = through.clone();
    cyclic["edges"][1] =
        json!({"from":{"nodeId":"p","portId":"b"},"to":{"nodeId":"p","portId":"b"}});
    let found = rejects(expand(&cyclic, &[passes()]), "MIX_GRAPH_CYCLE");
    assert_eq!(found[0]["evidence"]["cycle"], "p--second -> p--second");
    assert_eq!(found[0]["evidence"]["instanceId"], "p");
}

#[test]
fn parent_serialization_is_canonical_and_round_trips() {
    let limits = SafetyLimits::default();
    let recipe_limits = RecipeLimits::default();
    let source = ParentDocument::decode(&bytes(&parent()), &limits, &recipe_limits).unwrap();
    let encoded = source.to_json().unwrap();
    let decoded = ParentDocument::decode(&encoded, &limits, &recipe_limits).unwrap();
    assert_eq!(decoded.to_json().unwrap(), encoded);
    assert!(
        decoded
            .validate(&set(&[masked()]).unwrap(), &limits, &recipe_limits)
            .is_ok()
    );
}

#[test]
fn acceptance_plan_matches_the_accepted_materials_it_replaces() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |path: &str| -> Value {
        serde_json::from_slice(&std::fs::read(root.join(path)).unwrap()).unwrap()
    };
    let plan = read("fixtures/recipes/acceptance-plan.json");
    assert_eq!(plan["contract"], "docs/mat-04a-recipe-contract.md");
    for parent in plan["parents"].as_array().unwrap() {
        let material = read(parent["replaces"].as_str().unwrap());
        assert_eq!(
            material["exposedParameters"].as_array().unwrap().len() as u64,
            parent["exposed"]["v1"].as_u64().unwrap()
        );
        let qualification = read(parent["qualificationPlan"].as_str().unwrap());
        let text = qualification.to_string();
        let budget = parent["maxDescriptorPeakBytes"].as_u64().unwrap();
        assert!(text.contains(&format!("\"maxDescriptorPeakBytes\":{budget}")));
        let presets = &plan["presets"][parent["material"].as_str().unwrap()];
        for preset in presets.as_array().unwrap() {
            assert!(
                text.contains(&format!("\"id\":{preset}")),
                "{preset} is a frozen case"
            );
        }
    }
    let recipes: Vec<_> = plan["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        recipes,
        [
            "masked-layer-surface",
            "brick-structure",
            "weave-structure",
            "directional-grain"
        ]
    );
    // The fixture recipe is the contract's example, unchanged.
    assert!(set(&[masked()]).is_ok());
}

#[test]
fn decode_with_recipes_dispatches_by_version_and_keeps_closures_exact() {
    let (limits, recipe_limits) = (SafetyLimits::default(), RecipeLimits::default());
    let recipes = set(&[masked()]).unwrap();
    let decode = |source: &Value, recipes: &RecipeSet| {
        mixture_core::decode_with_recipes(&bytes(source), recipes, &limits, &recipe_limits)
    };
    assert_eq!(decode(&parent(), &recipes).unwrap().document().version, 2);
    let v1 = decode(&flattened(), &recipes).unwrap_err();
    assert_eq!(codes(&v1), ["MIX_RECIPE_UNUSED"]);
    assert_eq!(
        decode(&flattened(), &RecipeSet::default())
            .unwrap()
            .document()
            .version,
        1
    );
    let mut future = parent();
    future["version"] = json!(3);
    assert_eq!(
        codes(&decode(&future, &recipes).unwrap_err()),
        ["MIX_FORMAT_UNSUPPORTED_VERSION"]
    );
}
