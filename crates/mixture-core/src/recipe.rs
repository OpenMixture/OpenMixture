//! Caller-supplied recipe documents, `.mix` v2 parents and Core-owned expansion.
//!
//! A recipe is a versioned graph fragment with a typed interface. A parent
//! instantiates recipes beside ordinary nodes; Core validates the exact closure
//! and expands every instance into ordinary built-in nodes before normalization,
//! so compilation, RenderPlan v3 and pixel execution are unchanged.

use crate::{
    Diagnostic, DiagnosticCode as Code, DiagnosticReport, LimitKind, SafetyLimits, Stage,
    ValidatedDocument,
    document::{
        DocumentError, Edge, Endpoint, ExposedParameter, MaterialDocument, Node,
        PARENT_FORMAT_VERSION,
        decode::{self, Bounded, Ceiling},
    },
    registry::{ParameterContract, ParameterKind, PortKind, node_contract_version},
    validation::{
        GraphShape, Supplied, at_node, at_parameter, at_port, failure, identifier, validate_graph,
    },
};
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeSeed, IgnoredAny, MapAccess, Visitor},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// The only supported recipe document format version.
pub const RECIPE_FORMAT_VERSION: u32 = 1;
/// Separator between instance and recipe node IDs in an expanded graph.
const SEPARATOR: &str = "--";

/// Caller-owned recipe ceilings beside [`SafetyLimits`]; zero is a valid bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeLimits {
    /// Supplied recipe documents, including any unused one; default 16.
    pub recipe_count: u64,
    /// Instances in one parent document; default 32.
    pub instance_count: u64,
    /// Sum of supplied recipe byte lengths; default 2 MiB.
    pub recipe_bytes: u64,
}
impl Default for RecipeLimits {
    fn default() -> Self {
        Self {
            recipe_count: 16,
            instance_count: 32,
            recipe_bytes: 2_097_152,
        }
    }
}

/// One mutable parameter of a recipe node.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParameterTarget {
    /// Recipe-local node identifier.
    pub node_id: String,
    /// Parameter identifier from that node's contract.
    pub parameter_id: String,
}

/// Interface input fanned out to one or more unconnected internal input ports.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeInput {
    /// Interface port identifier.
    pub id: String,
    /// Internal input ports of one exact kind.
    pub targets: Vec<Endpoint>,
}

/// Interface output naming exactly one internal output port.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeOutput {
    /// Interface port identifier.
    pub id: String,
    /// Internal output port.
    pub from: Endpoint,
}

/// Interface parameter assigned directly to internal parameters with one contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeParameter {
    /// Interface parameter identifier.
    pub id: String,
    /// Internal parameters receiving the same value.
    pub targets: Vec<ParameterTarget>,
}

/// Editable `.mixrecipe` source. Decode untrusted bytes with [`RecipeDocument::decode`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeDocument {
    /// Recipe document format version; only 1.
    pub version: u32,
    /// Recipe identity, `[a-z][a-z0-9-]{0,63}`.
    pub id: String,
    /// Recipe version; identity is `id@recipeVersion`.
    pub recipe_version: u32,
    /// Built-in nodes of the fragment.
    pub nodes: Vec<Node>,
    /// Internal connections.
    pub edges: Vec<Edge>,
    /// Interface inputs.
    pub inputs: Vec<RecipeInput>,
    /// Interface outputs.
    pub outputs: Vec<RecipeOutput>,
    /// Interface parameters.
    pub parameters: Vec<RecipeParameter>,
}

/// Exact identity and byte digest of one supplied recipe.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeIdentity {
    /// Recipe identity.
    pub id: String,
    /// Recipe version.
    pub recipe_version: u32,
    /// Lowercase hexadecimal SHA-256 of the exact supplied bytes.
    pub sha256: String,
    /// Exact supplied byte length.
    pub byte_length: u64,
}

/// One recipe instance in a parent document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Instance {
    /// Unique ID shared with the parent's node namespace; never contains `--`.
    pub id: String,
    /// Referenced recipe identity.
    pub recipe: String,
    /// Referenced recipe version.
    pub recipe_version: u32,
    /// Interface parameter values; omitted ones use the interface default.
    #[serde(default, deserialize_with = "decode::parameters")]
    pub parameters: BTreeMap<String, Value>,
}

/// Editable `.mix` v2 parent source. Validate it against a [`RecipeSet`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentDocument {
    /// Required format version 2.
    pub version: u32,
    /// Ordinary built-in nodes, including the one material-output.
    pub nodes: Vec<Node>,
    /// Connections between node ports and instance interface ports.
    pub edges: Vec<Edge>,
    /// Recipe instances.
    pub instances: Vec<Instance>,
    /// Public bindings to node parameters or instance interface parameters.
    pub exposed_parameters: Vec<ExposedParameter>,
}

/// A validated input of a recipe interface.
#[derive(Clone, Debug)]
struct InputSpec {
    kind: PortKind,
    required: bool,
    targets: Vec<Endpoint>,
}
/// A validated parameter of a recipe interface.
#[derive(Clone, Debug)]
struct ParameterSpec {
    contract: &'static ParameterContract,
    default: Option<Value>,
    targets: Vec<ParameterTarget>,
}
/// Validated interface inputs, outputs and parameters keyed by interface ID.
type Interface = (
    BTreeMap<String, InputSpec>,
    BTreeMap<String, (Endpoint, PortKind)>,
    BTreeMap<String, ParameterSpec>,
);
#[derive(Clone, Debug)]
struct ValidatedRecipe {
    document: RecipeDocument,
    identity: RecipeIdentity,
    inputs: BTreeMap<String, InputSpec>,
    outputs: BTreeMap<String, (Endpoint, PortKind)>,
    parameters: BTreeMap<String, ParameterSpec>,
}

/// Validated, duplicate-free recipes keyed by `id@recipeVersion`.
#[derive(Clone, Debug, Default)]
pub struct RecipeSet {
    recipes: BTreeMap<(String, u32), ValidatedRecipe>,
}

impl RecipeDocument {
    /// Decode strict UTF-8 JSON under explicit limits. Interface arrays are bounded
    /// by `exposedParameters`; graph semantics are checked by [`Self::validate`].
    pub fn decode(bytes: &[u8], limits: &SafetyLimits) -> Result<Self, DocumentError> {
        let document = decode::strict(
            bytes,
            limits,
            "Recipe input does not conform to .mixrecipe v1 JSON.",
            |parser, exceeded| RecipeSeed { limits, exceeded }.deserialize(parser),
        )?;
        if document.version != RECIPE_FORMAT_VERSION {
            return Err(unsupported(document.version, RECIPE_FORMAT_VERSION, "recipe").into());
        }
        Ok(document)
    }

    /// Validate the fragment, its interface and collection limits.
    pub fn validate(&self, limits: &SafetyLimits) -> DiagnosticReport {
        DiagnosticReport::new(self.check(limits).err().unwrap_or_default())
    }

    fn check(&self, limits: &SafetyLimits) -> Result<Interface, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if self.version != RECIPE_FORMAT_VERSION {
            diagnostics.push(unsupported(self.version, RECIPE_FORMAT_VERSION, "recipe"));
        }
        for (kind, count) in [
            (LimitKind::Nodes, self.nodes.len()),
            (LimitKind::Edges, self.edges.len()),
            (LimitKind::ExposedParameters, self.inputs.len()),
            (LimitKind::ExposedParameters, self.outputs.len()),
            (LimitKind::ExposedParameters, self.parameters.len()),
        ] {
            if let Err(error) = limits.check(kind, count as u64) {
                diagnostics.push(error.diagnostic(Stage::Validation));
            }
        }
        if !diagnostics.is_empty() {
            return Err(self.annotate(diagnostics));
        }
        if !recipe_identifier(&self.id) {
            diagnostics.push(
                failure(
                    Code::FormatInvalidDocument,
                    "Recipe ID must match [a-z][a-z0-9-]{0,63}.",
                )
                .with_suggestion("Use a lowercase kebab-case recipe ID of at most 64 bytes."),
            );
        }
        if self.recipe_version == 0 {
            diagnostics.push(
                failure(
                    Code::FormatInvalidDocument,
                    "Recipe version must be at least 1.",
                )
                .with_suggestion(
                    "Start recipe versions at 1 and increment them for semantic changes.",
                ),
            );
        }
        let mut nodes: BTreeMap<&str, Vec<&Node>> = BTreeMap::new();
        for node in &self.nodes {
            nodes.entry(&node.id).or_default().push(node);
            if matches!(node.type_id.as_str(), "material-output" | "image-input") {
                diagnostics.push(
                    at_node(Code::RecipeForbiddenNode, &node.id, "Recipes may not contain this node type.")
                        .with_evidence("type", node.type_id.as_str())
                        .with_suggestion("Keep material-output and image-input in the parent document; connect them through interface ports."),
                );
            }
        }
        let unique = |id: &str| {
            nodes
                .get(id)
                .filter(|group| group.len() == 1)
                .and_then(|group| group.first().copied())
        };
        let connected: BTreeSet<&Endpoint> = self.edges.iter().map(|edge| &edge.to).collect();
        let mut supplied = Supplied::default();

        let mut inputs = BTreeMap::new();
        let mut input_ids = BTreeSet::new();
        for input in &self.inputs {
            interface_id(&input.id, "input", &mut input_ids, &mut diagnostics);
            if input.targets.is_empty() {
                diagnostics.push(interface(
                    &input.id,
                    "Interface input needs at least one target.",
                ));
            }
            let mut kind = None;
            let mut required = false;
            for target in &input.targets {
                let port = unique(&target.node_id)
                    .and_then(|node| node_contract_version(&node.type_id, node.version))
                    .and_then(|contract| contract.input(&target.port_id));
                let Some(port) = port else {
                    diagnostics.push(at_target(
                        interface(
                            &input.id,
                            "Interface input target is not an input port of one supported node.",
                        ),
                        target,
                    ));
                    continue;
                };
                required |= port.default.is_none();
                if kind.is_some_and(|kind| kind != port.kind) {
                    diagnostics.push(
                        at_target(
                            interface(
                                &input.id,
                                "All targets of an interface input must have the same port kind.",
                            ),
                            target,
                        )
                        .with_evidence("targetKind", port.kind.as_str()),
                    );
                }
                kind.get_or_insert(port.kind);
                if connected.contains(target) {
                    diagnostics.push(at_target(
                        interface(
                            &input.id,
                            "Interface input target is already connected by an internal edge.",
                        ),
                        target,
                    ));
                }
                if !supplied.inputs.insert(target.clone()) {
                    diagnostics.push(at_target(
                        interface(
                            &input.id,
                            "An internal input may be targeted by only one interface input, once.",
                        ),
                        target,
                    ));
                }
            }
            if let Some(kind) = kind {
                inputs.insert(
                    input.id.clone(),
                    InputSpec {
                        kind,
                        required,
                        targets: input.targets.clone(),
                    },
                );
            }
        }

        let mut outputs = BTreeMap::new();
        let mut output_ids = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for output in &self.outputs {
            interface_id(&output.id, "output", &mut output_ids, &mut diagnostics);
            let port = unique(&output.from.node_id)
                .and_then(|node| node_contract_version(&node.type_id, node.version))
                .and_then(|contract| contract.output(&output.from.port_id));
            match port {
                None => diagnostics.push(at_target(
                    interface(
                        &output.id,
                        "Interface output is not an output port of one supported node.",
                    ),
                    &output.from,
                )),
                Some(port) => {
                    if !sources.insert(&output.from) {
                        diagnostics.push(at_target(
                            interface(
                                &output.id,
                                "Two interface outputs may not alias one internal port.",
                            ),
                            &output.from,
                        ));
                    }
                    outputs.insert(output.id.clone(), (output.from.clone(), port.kind));
                }
            }
        }

        let mut parameters = BTreeMap::new();
        let mut parameter_ids = BTreeSet::new();
        for parameter in &self.parameters {
            interface_id(
                &parameter.id,
                "parameter",
                &mut parameter_ids,
                &mut diagnostics,
            );
            if parameter.targets.is_empty() {
                diagnostics.push(interface(
                    &parameter.id,
                    "Interface parameter needs at least one target.",
                ));
            }
            let mut contract: Option<&'static ParameterContract> = None;
            let mut values = Vec::new();
            let mut complete = true;
            for target in &parameter.targets {
                let found = unique(&target.node_id).and_then(|node| {
                    node_contract_version(&node.type_id, node.version)
                        .and_then(|c| c.parameter(&target.parameter_id))
                        .map(|p| (node, p))
                });
                let Some((node, target_contract)) = found else {
                    diagnostics.push(at_parameter_target(
                        interface(
                            &parameter.id,
                            "Interface parameter target is not a parameter of one supported node.",
                        ),
                        target,
                    ));
                    complete = false;
                    continue;
                };
                if matches!(target_contract.kind, ParameterKind::ResourceRef) {
                    diagnostics.push(at_parameter_target(
                        interface(
                            &parameter.id,
                            "Resource references cannot be interface parameters.",
                        ),
                        target,
                    ));
                }
                if contract.is_some_and(|c| c.kind != target_contract.kind) {
                    diagnostics.push(at_parameter_target(interface(&parameter.id, "All targets of an interface parameter must share one parameter contract."), target)
                        .with_evidence("expected", format!("{:?}", contract.map(|c| c.kind)))
                        .with_evidence("observed", format!("{:?}", target_contract.kind)));
                }
                contract.get_or_insert(target_contract);
                if !supplied
                    .parameters
                    .insert((target.node_id.clone(), target.parameter_id.clone()))
                {
                    diagnostics.push(at_parameter_target(interface(&parameter.id, "An internal parameter may be targeted by only one interface parameter, once."), target));
                }
                values.push(
                    node.parameters
                        .get(&target.parameter_id)
                        .cloned()
                        .or_else(|| target_contract.default.map(|d| d.value())),
                );
            }
            let default = match values.first() {
                Some(first) if values.iter().all(|value| value == first) => first.clone(),
                Some(_) => {
                    diagnostics.push(
                        interface(&parameter.id, "Interface parameter targets must share one source value, or all lack one.")
                            .with_suggestion("Give every target the same explicit value, or split them into separate interface parameters."),
                    );
                    None
                }
                None => None,
            };
            if let (Some(contract), true) = (contract, complete) {
                parameters.insert(
                    parameter.id.clone(),
                    ParameterSpec {
                        contract,
                        default,
                        targets: parameter.targets.clone(),
                    },
                );
            }
        }

        // Explicit seeds and other parameters without defaults are chosen by every
        // instance, never fixed or derived inside a reusable recipe.
        for node in &self.nodes {
            let Some(contract) = node_contract_version(&node.type_id, node.version) else {
                continue;
            };
            for parameter in contract.parameters.iter().filter(|p| p.default.is_none()) {
                if !supplied
                    .parameters
                    .contains(&(node.id.clone(), parameter.id.to_string()))
                {
                    diagnostics.push(
                        at_parameter(Code::RecipeInvalidInterface, &node.id, parameter.id,
                            "A parameter without a contract default must be bound by an interface parameter.")
                            .with_suggestion("Bind seeds to an interface parameter so each instance sets them explicitly."),
                    );
                }
            }
        }
        validate_graph(
            &self.nodes,
            &self.edges,
            &[],
            GraphShape::Recipe,
            &supplied,
            &mut diagnostics,
        );
        if diagnostics.is_empty() {
            Ok((inputs, outputs, parameters))
        } else {
            Err(self.annotate(diagnostics))
        }
    }

    fn annotate(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        diagnostics
            .into_iter()
            .map(|d| {
                d.with_evidence("recipeId", self.id.as_str())
                    .with_evidence("recipeVersion", u64::from(self.recipe_version))
            })
            .collect()
    }
}

impl RecipeSet {
    /// Decode and validate every supplied recipe. The count and total byte
    /// ceilings are checked before decoding; duplicate identities are rejected.
    pub fn decode(
        sources: &[&[u8]],
        limits: &SafetyLimits,
        recipe_limits: &RecipeLimits,
    ) -> Result<Self, DocumentError> {
        let count = sources.len() as u64;
        if count > recipe_limits.recipe_count {
            return Err(budget(
                Code::LimitRecipeCountExceeded,
                Stage::Parse,
                "recipeCount",
                recipe_limits.recipe_count,
                count,
            )
            .into());
        }
        let bytes = sources
            .iter()
            .try_fold(0_u64, |sum, source| sum.checked_add(source.len() as u64))
            .unwrap_or(u64::MAX);
        if bytes > recipe_limits.recipe_bytes {
            return Err(budget(
                Code::LimitRecipeBytesExceeded,
                Stage::Parse,
                "recipeBytes",
                recipe_limits.recipe_bytes,
                bytes,
            )
            .into());
        }
        let mut diagnostics = Vec::new();
        let mut grouped: BTreeMap<(String, u32), Vec<ValidatedRecipe>> = BTreeMap::new();
        for (index, source) in sources.iter().enumerate() {
            let at = |d: Diagnostic| d.with_evidence("recipeIndex", index as u64);
            let document = match RecipeDocument::decode(source, limits) {
                Ok(document) => document,
                Err(error) => {
                    diagnostics.extend(error.report().diagnostics().iter().cloned().map(at));
                    continue;
                }
            };
            match document.check(limits) {
                Err(found) => diagnostics.extend(found.into_iter().map(at)),
                Ok((inputs, outputs, parameters)) => {
                    let digest = Sha256::digest(source);
                    let identity = RecipeIdentity {
                        id: document.id.clone(),
                        recipe_version: document.recipe_version,
                        sha256: digest.iter().map(|b| format!("{b:02x}")).collect(),
                        byte_length: source.len() as u64,
                    };
                    grouped
                        .entry((document.id.clone(), document.recipe_version))
                        .or_default()
                        .push(ValidatedRecipe {
                            document,
                            identity,
                            inputs,
                            outputs,
                            parameters,
                        });
                }
            }
        }
        let mut recipes = BTreeMap::new();
        for (key, mut group) in grouped {
            if group.len() > 1 {
                diagnostics.push(
                    failure(
                        Code::RecipeDuplicate,
                        "Two supplied recipes share one identity.",
                    )
                    .with_evidence("recipeId", key.0.as_str())
                    .with_evidence("recipeVersion", u64::from(key.1))
                    .with_evidence("observed", group.len() as u64)
                    .with_suggestion("Supply each recipe identity exactly once."),
                );
            } else if let Some(recipe) = group.pop() {
                recipes.insert(key, recipe);
            }
        }
        if diagnostics.is_empty() {
            Ok(Self { recipes })
        } else {
            Err(DocumentError::new(diagnostics))
        }
    }

    /// Supplied identities and digests, sorted by identity.
    pub fn identities(&self) -> Vec<RecipeIdentity> {
        self.recipes.values().map(|r| r.identity.clone()).collect()
    }
    /// Number of supplied recipes.
    pub fn len(&self) -> usize {
        self.recipes.len()
    }
    /// Whether no recipe was supplied.
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}

impl ParentDocument {
    /// Decode strict UTF-8 JSON version 2 under explicit limits; the instance
    /// array stops at the first item beyond `instanceCount`.
    pub fn decode(
        bytes: &[u8],
        limits: &SafetyLimits,
        recipe_limits: &RecipeLimits,
    ) -> Result<Self, DocumentError> {
        let document = decode::strict(
            bytes,
            limits,
            "Material input does not conform to .mix v2 JSON.",
            |parser, exceeded| {
                ParentSeed {
                    limits,
                    recipe_limits,
                    exceeded,
                }
                .deserialize(parser)
            },
        )?;
        if document.version != PARENT_FORMAT_VERSION {
            return Err(unsupported(document.version, PARENT_FORMAT_VERSION, "parent").into());
        }
        Ok(document)
    }

    /// Validate the parent against an exact recipe closure, expand it and
    /// validate the expanded graph. Nothing is expanded after a parent error.
    pub fn validate(
        &self,
        recipes: &RecipeSet,
        limits: &SafetyLimits,
        recipe_limits: &RecipeLimits,
    ) -> DiagnosticReport {
        DiagnosticReport::new(
            self.expand(recipes, limits, recipe_limits)
                .err()
                .unwrap_or_default(),
        )
    }

    /// Consume the source after successful validation and return the expanded
    /// graph for the unchanged compiler.
    pub fn into_validated(
        self,
        recipes: &RecipeSet,
        limits: &SafetyLimits,
        recipe_limits: &RecipeLimits,
    ) -> Result<ValidatedDocument, DocumentError> {
        let expanded = self
            .expand(recipes, limits, recipe_limits)
            .map_err(DocumentError::new)?;
        Ok(ValidatedDocument::expanded(expanded, recipes.identities()))
    }

    /// Serialize in lexical node/instance/edge/public-ID order without changing
    /// this source; defaults are not inserted.
    pub fn to_json(&self) -> Result<Vec<u8>, DocumentError> {
        let mut canonical = self.clone();
        canonical
            .nodes
            .sort_by(|a, b| (&a.id, &a.type_id, a.version).cmp(&(&b.id, &b.type_id, b.version)));
        canonical.instances.sort_by(|a, b| a.id.cmp(&b.id));
        canonical.edges.sort();
        canonical.exposed_parameters.sort();
        serde_json::to_vec_pretty(&canonical).map_err(|source| {
            failure(
                Code::FormatInvalidDocument,
                "Could not serialize the parent document.",
            )
            .with_source(source)
            .into()
        })
    }

    fn expand(
        &self,
        recipes: &RecipeSet,
        limits: &SafetyLimits,
        recipe_limits: &RecipeLimits,
    ) -> Result<MaterialDocument, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if self.version != PARENT_FORMAT_VERSION {
            diagnostics.push(unsupported(self.version, PARENT_FORMAT_VERSION, "parent"));
        }
        for (kind, count) in [
            (LimitKind::Nodes, self.nodes.len()),
            (LimitKind::Edges, self.edges.len()),
            (LimitKind::ExposedParameters, self.exposed_parameters.len()),
        ] {
            if let Err(error) = limits.check(kind, count as u64) {
                diagnostics.push(error.diagnostic(Stage::Validation));
            }
        }
        if self.instances.len() as u64 > recipe_limits.instance_count {
            diagnostics.push(budget(
                Code::LimitInstanceCountExceeded,
                Stage::Validation,
                "instanceCount",
                recipe_limits.instance_count,
                self.instances.len() as u64,
            ));
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        // Identity: v2 IDs share one namespace and reserve the expansion separator.
        let mut ids: BTreeMap<&str, u64> = BTreeMap::new();
        for id in self
            .nodes
            .iter()
            .map(|n| n.id.as_str())
            .chain(self.instances.iter().map(|i| i.id.as_str()))
        {
            *ids.entry(id).or_insert(0) += 1;
            if id.contains(SEPARATOR) && identifier(id) {
                diagnostics.push(
                    at_node(
                        Code::NodeInvalidId,
                        id,
                        "Node and instance IDs in a v2 parent may not contain \"--\".",
                    )
                    .with_suggestion(
                        "Rename the ID; \"--\" is reserved for expanded recipe nodes.",
                    ),
                );
            }
        }
        for (id, count) in &ids {
            if *count > 1 {
                diagnostics.push(
                    at_node(
                        Code::NodeDuplicateId,
                        id,
                        "Node and instance IDs must be unique together.",
                    )
                    .with_evidence("observed", *count)
                    .with_suggestion(
                        "Give every node and instance a unique ID and update its references.",
                    ),
                );
            }
        }

        // Exact closure.
        let mut resolved: BTreeMap<&str, &ValidatedRecipe> = BTreeMap::new();
        let mut used = BTreeSet::new();
        for instance in &self.instances {
            if !identifier(&instance.id) {
                diagnostics.push(
                    at_node(
                        Code::NodeInvalidId,
                        &instance.id,
                        "Instance ID must match [A-Za-z][A-Za-z0-9_-]{0,63}.",
                    )
                    .with_evidence("instanceId", instance.id.as_str()),
                );
            }
            let key = (instance.recipe.clone(), instance.recipe_version);
            match recipes.recipes.get(&key) {
                Some(recipe) => {
                    used.insert(key);
                    if ids.get(instance.id.as_str()) == Some(&1) {
                        resolved.insert(&instance.id, recipe);
                    }
                }
                None => {
                    let supplied: Vec<String> = recipes
                        .recipes
                        .keys()
                        .filter(|(id, _)| *id == instance.recipe)
                        .map(|(_, version)| version.to_string())
                        .collect();
                    diagnostics.push(
                        at_node(Code::RecipeMissing, &instance.id, "Instance references a recipe identity that was not supplied.")
                            .with_evidence("instanceId", instance.id.as_str())
                            .with_evidence("recipeId", instance.recipe.as_str())
                            .with_evidence("recipeVersion", u64::from(instance.recipe_version))
                            .with_evidence("suppliedVersions", supplied.join(","))
                            .with_suggestion("Supply the exact recipe identity and version; recipes are never looked up by path."),
                    );
                }
            }
        }
        for (key, recipe) in &recipes.recipes {
            if !used.contains(key) {
                diagnostics.push(
                    failure(
                        Code::RecipeUnused,
                        "A supplied recipe is not used by any instance.",
                    )
                    .with_evidence("recipeId", recipe.identity.id.as_str())
                    .with_evidence("recipeVersion", u64::from(recipe.identity.recipe_version))
                    .with_suggestion("Supply exactly the recipes this parent instantiates."),
                );
            }
        }

        // Instance parameter values.
        for instance in &self.instances {
            let Some(recipe) = resolved.get(instance.id.as_str()) else {
                continue;
            };
            for (id, value) in &instance.parameters {
                let at = |d: Diagnostic| {
                    d.with_evidence("instanceId", instance.id.as_str())
                        .with_evidence("interfaceId", id.as_str())
                };
                match recipe.parameters.get(id) {
                    None => diagnostics.push(at(at_parameter(Code::ParameterUnknown, &instance.id, id, "Unknown interface parameter.")
                        .with_suggestion("Use parameter IDs declared by the recipe interface."))),
                    Some(spec) if !spec.contract.accepts(value) => diagnostics.push(at(at_parameter(Code::ParameterInvalidValue, &instance.id, id,
                        "Instance value does not satisfy the interface parameter's type, range, or enum contract.")
                        .with_evidence("expected", format!("{:?}", spec.contract.kind))
                        .with_evidence("observed", value.to_string())
                        .with_suggestion("Supply a finite value in the documented range and exact JSON type; values are never clamped or coerced."))),
                    Some(_) => {}
                }
            }
            for (id, spec) in &recipe.parameters {
                if spec.default.is_none() && !instance.parameters.contains_key(id) {
                    diagnostics.push(
                        at_parameter(Code::ParameterInvalidValue, &instance.id, id, "Required interface parameter is missing; the recipe defines no default.")
                            .with_evidence("instanceId", instance.id.as_str())
                            .with_evidence("interfaceId", id.as_str())
                            .with_evidence("required", true)
                            .with_evidence("present", false)
                            .with_suggestion("Set this interface parameter explicitly on every instance; seeds are never derived."),
                    );
                }
            }
        }

        // Interface ports on edges.
        let node_port = |endpoint: &Endpoint, output: bool| -> Option<PortKind> {
            let node = self.nodes.iter().find(|n| n.id == endpoint.node_id)?;
            let contract = node_contract_version(&node.type_id, node.version)?;
            let port = if output {
                contract.output(&endpoint.port_id)
            } else {
                contract.input(&endpoint.port_id)
            };
            port.map(|p| p.kind)
        };
        let mut incoming: BTreeMap<&Endpoint, u64> = BTreeMap::new();
        for edge in &self.edges {
            let from = match resolved.get(edge.from.node_id.as_str()) {
                Some(recipe) => match recipe.outputs.get(&edge.from.port_id) {
                    Some((_, kind)) => Some(*kind),
                    None => {
                        diagnostics.push(instance_port(
                            &edge.from,
                            "Instance has no interface output with this name.",
                            true,
                        ));
                        None
                    }
                },
                None => node_port(&edge.from, true),
            };
            let to = match resolved.get(edge.to.node_id.as_str()) {
                Some(recipe) => {
                    *incoming.entry(&edge.to).or_insert(0) += 1;
                    match recipe.inputs.get(&edge.to.port_id) {
                        Some(spec) => Some(spec.kind),
                        None => {
                            diagnostics.push(instance_port(
                                &edge.to,
                                "Instance has no interface input with this name.",
                                false,
                            ));
                            None
                        }
                    }
                }
                None => node_port(&edge.to, false),
            };
            let touches_instance = resolved.contains_key(edge.from.node_id.as_str())
                || resolved.contains_key(edge.to.node_id.as_str());
            if let (true, Some(from), Some(to)) = (touches_instance, from, to)
                && from != to
            {
                diagnostics.push(
                    at_port(
                        Code::PortTypeMismatch,
                        &edge.to,
                        "Connected port kinds must match exactly.",
                    )
                    .with_evidence("sourceNode", edge.from.node_id.as_str())
                    .with_evidence("sourcePort", edge.from.port_id.as_str())
                    .with_evidence("sourceKind", from.as_str())
                    .with_evidence("targetKind", to.as_str())
                    .with_suggestion(
                        "Connect an output of the required kind; recipes never insert conversions.",
                    ),
                );
            }
        }
        for (target, count) in &incoming {
            if *count > 1 {
                diagnostics.push(
                    at_port(
                        Code::GraphMultipleInputs,
                        target,
                        "An interface input may have at most one incoming edge.",
                    )
                    .with_evidence("instanceId", target.node_id.as_str())
                    .with_evidence("observed", *count),
                );
            }
        }
        for (id, recipe) in &resolved {
            for (input, spec) in &recipe.inputs {
                let endpoint = Endpoint {
                    node_id: (*id).to_string(),
                    port_id: input.clone(),
                };
                if spec.required && !incoming.contains_key(&endpoint) {
                    diagnostics.push(
                        at_port(
                            Code::PortRequiredConnection,
                            &endpoint,
                            "Required interface input has no connection.",
                        )
                        .with_evidence("instanceId", *id)
                        .with_suggestion("Connect one compatible output to this interface input."),
                    );
                }
            }
        }

        // Public bindings: unique IDs and targets at the parent level.
        let mut public: BTreeMap<&str, u64> = BTreeMap::new();
        let mut bound: BTreeMap<(&str, &str), u64> = BTreeMap::new();
        for binding in &self.exposed_parameters {
            *public.entry(&binding.id).or_insert(0) += 1;
            *bound
                .entry((&binding.node_id, &binding.parameter_id))
                .or_insert(0) += 1;
            if let Some(recipe) = resolved.get(binding.node_id.as_str())
                && !recipe.parameters.contains_key(&binding.parameter_id)
            {
                diagnostics.push(
                    at_parameter(
                        Code::ExposedParameterInvalid,
                        &binding.node_id,
                        &binding.parameter_id,
                        "Exposed parameter target is not an interface parameter of this instance.",
                    )
                    .with_evidence("publicId", binding.id.as_str())
                    .with_evidence("instanceId", binding.node_id.as_str()),
                );
            }
        }
        for (id, count) in public {
            if count > 1 {
                diagnostics.push(
                    failure(
                        Code::ExposedParameterInvalid,
                        "Public parameter IDs must be unique.",
                    )
                    .with_evidence("publicId", id)
                    .with_evidence("observed", count)
                    .with_suggestion("Give each exposed parameter a unique public ID."),
                );
            }
        }
        for ((node, parameter), count) in bound {
            if count > 1 {
                diagnostics.push(
                    at_parameter(
                        Code::ExposedParameterInvalid,
                        node,
                        parameter,
                        "Multiple public bindings target the same parameter.",
                    )
                    .with_evidence("observed", count)
                    .with_suggestion("Expose this target once; parameter aliases are unsupported."),
                );
            }
        }

        // Expanded size, checked before materializing any expanded node.
        let mut node_total = self.nodes.len() as u64;
        let mut edge_total = 0_u64;
        for recipe in resolved.values() {
            node_total += recipe.document.nodes.len() as u64;
            edge_total += recipe.document.edges.len() as u64;
        }
        for edge in &self.edges {
            edge_total += match resolved
                .get(edge.to.node_id.as_str())
                .and_then(|r| r.inputs.get(&edge.to.port_id))
            {
                Some(spec) => spec.targets.len() as u64,
                None => 1,
            };
        }
        for (kind, count) in [
            (LimitKind::Nodes, node_total),
            (LimitKind::Edges, edge_total),
        ] {
            if let Err(error) = limits.check(kind, count) {
                diagnostics.push(
                    error
                        .diagnostic(Stage::Validation)
                        .with_evidence("expanded", true),
                );
            }
        }
        for (id, recipe) in &resolved {
            for node in &recipe.document.nodes {
                let expanded = expanded_id(id, &node.id);
                if !identifier(&expanded) {
                    diagnostics.push(
                        at_node(
                            Code::NodeInvalidId,
                            &expanded,
                            "Expanded node ID exceeds the node ID grammar.",
                        )
                        .with_evidence("instanceId", *id)
                        .with_evidence("recipeNodeId", node.id.as_str())
                        .with_suggestion(
                            "Shorten the instance ID so instance--node stays within 64 bytes.",
                        ),
                    );
                }
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        // Expansion.
        let mut nodes = self.nodes.clone();
        let mut edges = Vec::new();
        let mut exposed = Vec::new();
        for instance in &self.instances {
            let recipe = resolved[instance.id.as_str()];
            let mut values: BTreeMap<(&str, &str), &Value> = BTreeMap::new();
            for (id, value) in &instance.parameters {
                for target in &recipe.parameters[id].targets {
                    values.insert((&target.node_id, &target.parameter_id), value);
                }
            }
            for node in &recipe.document.nodes {
                let mut node = node.clone();
                for ((target, parameter), value) in &values {
                    if *target == node.id {
                        node.parameters
                            .insert((*parameter).to_string(), (*value).clone());
                    }
                }
                node.id = expanded_id(&instance.id, &node.id);
                nodes.push(node);
            }
            for edge in &recipe.document.edges {
                edges.push(Edge {
                    from: rename(&instance.id, &edge.from),
                    to: rename(&instance.id, &edge.to),
                });
            }
        }
        for edge in &self.edges {
            let from = match resolved.get(edge.from.node_id.as_str()) {
                Some(recipe) => rename(&edge.from.node_id, &recipe.outputs[&edge.from.port_id].0),
                None => edge.from.clone(),
            };
            match resolved.get(edge.to.node_id.as_str()) {
                Some(recipe) => {
                    for target in &recipe.inputs[&edge.to.port_id].targets {
                        edges.push(Edge {
                            from: from.clone(),
                            to: rename(&edge.to.node_id, target),
                        });
                    }
                }
                None => edges.push(Edge {
                    from,
                    to: edge.to.clone(),
                }),
            }
        }
        for binding in &self.exposed_parameters {
            match resolved.get(binding.node_id.as_str()) {
                Some(recipe) => {
                    for target in &recipe.parameters[&binding.parameter_id].targets {
                        exposed.push(ExposedParameter {
                            id: binding.id.clone(),
                            node_id: expanded_id(&binding.node_id, &target.node_id),
                            parameter_id: target.parameter_id.clone(),
                        });
                    }
                }
                None => exposed.push(binding.clone()),
            }
        }
        let expanded = MaterialDocument {
            version: PARENT_FORMAT_VERSION,
            nodes,
            edges,
            exposed_parameters: exposed,
        };
        let report = expanded.validate_shape(limits, GraphShape::Expanded);
        if !report.is_ok() {
            return Err(report
                .diagnostics()
                .iter()
                .cloned()
                .map(attribute)
                .collect());
        }
        Ok(expanded)
    }
}

/// Name the instance of a diagnostic raised on an expanded node.
pub(crate) fn attribute(diagnostic: Diagnostic) -> Diagnostic {
    let instance = diagnostic
        .node_id
        .as_deref()
        .and_then(|id| id.split_once(SEPARATOR))
        .map(|(instance, _)| instance.to_string());
    match instance {
        Some(instance) => diagnostic.with_evidence("instanceId", instance),
        None => diagnostic,
    }
}
fn expanded_id(instance: &str, node: &str) -> String {
    format!("{instance}{SEPARATOR}{node}")
}
fn rename(instance: &str, endpoint: &Endpoint) -> Endpoint {
    Endpoint {
        node_id: expanded_id(instance, &endpoint.node_id),
        port_id: endpoint.port_id.clone(),
    }
}
fn recipe_identifier(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn unsupported(observed: u32, supported: u32, document: &str) -> Diagnostic {
    Diagnostic::error(
        Code::FormatUnsupportedVersion,
        Stage::Parse,
        format!("Unsupported {document} format version."),
    )
    .with_evidence("observed", u64::from(observed))
    .with_evidence("supported", u64::from(supported))
    .with_evidence("document", document)
    .with_suggestion(
        "Use a supported document version; documents are never migrated automatically.",
    )
}
fn budget(code: Code, stage: Stage, name: &str, configured: u64, observed: u64) -> Diagnostic {
    Diagnostic::error(code, stage, format!("{name} limit exceeded: configured {configured}, observed {observed}"))
        .with_evidence("limit", name)
        .with_evidence("configured", configured)
        .with_evidence("observed", observed)
        .with_suggestion("Reduce the request to the configured limit; raise limits only through an explicit caller policy.")
}
fn interface(id: &str, message: &str) -> Diagnostic {
    failure(Code::RecipeInvalidInterface, message)
        .with_evidence("interfaceId", id)
        .with_suggestion(
            "Bind each interface ID to existing, compatible, otherwise unconnected targets.",
        )
}
fn interface_id(
    id: &str,
    kind: &str,
    seen: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !identifier(id) {
        diagnostics.push(
            interface(id, "Interface ID must match [A-Za-z][A-Za-z0-9_-]{0,63}.")
                .with_evidence("interface", kind),
        );
    }
    if !seen.insert(id.to_string()) {
        diagnostics.push(
            interface(
                id,
                "Interface IDs must be unique within inputs, outputs and parameters.",
            )
            .with_evidence("interface", kind),
        );
    }
}
fn at_target(mut diagnostic: Diagnostic, target: &Endpoint) -> Diagnostic {
    diagnostic.node_id = Some(target.node_id.clone());
    diagnostic.port_id = Some(target.port_id.clone());
    diagnostic
}
fn at_parameter_target(mut diagnostic: Diagnostic, target: &ParameterTarget) -> Diagnostic {
    diagnostic.node_id = Some(target.node_id.clone());
    diagnostic.parameter_id = Some(target.parameter_id.clone());
    diagnostic
}
fn instance_port(endpoint: &Endpoint, message: &str, output: bool) -> Diagnostic {
    at_port(Code::PortUnknown, endpoint, message)
        .with_evidence("instanceId", endpoint.node_id.as_str())
        .with_evidence("direction", if output { "output" } else { "input" })
        .with_suggestion("Use a port declared by the recipe interface.")
}

struct RecipeSeed<'a> {
    limits: &'a SafetyLimits,
    exceeded: &'a mut Option<Diagnostic>,
}
impl<'de> DeserializeSeed<'de> for RecipeSeed<'_> {
    type Value = RecipeDocument;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
const RECIPE_FIELDS: &[&str] = &[
    "version",
    "id",
    "recipeVersion",
    "nodes",
    "edges",
    "inputs",
    "outputs",
    "parameters",
];
impl<'de> Visitor<'de> for RecipeSeed<'_> {
    type Value = RecipeDocument;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a .mixrecipe document object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut version = None;
        let mut id = None;
        let mut recipe_version = None;
        let mut nodes = None;
        let mut edges = None;
        let mut inputs = None;
        let mut outputs = None;
        let mut parameters = None;
        while let Some(key) = map.next_key::<String>()? {
            let duplicate = || de::Error::custom(format!("duplicate field {key:?}"));
            match key.as_str() {
                "version" => {
                    if version.replace(map.next_value::<u32>()?).is_some() {
                        return Err(duplicate());
                    }
                }
                "id" => {
                    if id.replace(map.next_value::<String>()?).is_some() {
                        return Err(duplicate());
                    }
                }
                "recipeVersion" => {
                    if recipe_version.replace(map.next_value::<u32>()?).is_some() {
                        return Err(duplicate());
                    }
                }
                "nodes" if nodes.is_none() => {
                    nodes = Some(map.next_value_seed(Bounded::<Node>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::Nodes,
                    ))?)
                }
                "edges" if edges.is_none() => {
                    edges = Some(map.next_value_seed(Bounded::<Edge>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::Edges,
                    ))?)
                }
                "inputs" if inputs.is_none() => {
                    inputs = Some(map.next_value_seed(Bounded::<RecipeInput>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::ExposedParameters,
                    ))?)
                }
                "outputs" if outputs.is_none() => {
                    outputs = Some(map.next_value_seed(Bounded::<RecipeOutput>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::ExposedParameters,
                    ))?)
                }
                "parameters" if parameters.is_none() => {
                    parameters = Some(map.next_value_seed(Bounded::<RecipeParameter>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::ExposedParameters,
                    ))?)
                }
                "nodes" | "edges" | "inputs" | "outputs" | "parameters" => return Err(duplicate()),
                "instances" => {
                    map.next_value::<IgnoredAny>()?;
                    return Err(de::Error::custom(
                        "recipes cannot contain instances; recipe nesting is unsupported",
                    ));
                }
                _ => return Err(de::Error::unknown_field(&key, RECIPE_FIELDS)),
            }
        }
        Ok(RecipeDocument {
            version: version.ok_or_else(|| de::Error::missing_field("version"))?,
            id: id.ok_or_else(|| de::Error::missing_field("id"))?,
            recipe_version: recipe_version
                .ok_or_else(|| de::Error::missing_field("recipeVersion"))?,
            nodes: nodes.ok_or_else(|| de::Error::missing_field("nodes"))?,
            edges: edges.ok_or_else(|| de::Error::missing_field("edges"))?,
            inputs: inputs.unwrap_or_default(),
            outputs: outputs.ok_or_else(|| de::Error::missing_field("outputs"))?,
            parameters: parameters.unwrap_or_default(),
        })
    }
}

struct ParentSeed<'a> {
    limits: &'a SafetyLimits,
    recipe_limits: &'a RecipeLimits,
    exceeded: &'a mut Option<Diagnostic>,
}
impl<'de> DeserializeSeed<'de> for ParentSeed<'_> {
    type Value = ParentDocument;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
const PARENT_FIELDS: &[&str] = &[
    "version",
    "nodes",
    "edges",
    "instances",
    "exposedParameters",
];
impl<'de> Visitor<'de> for ParentSeed<'_> {
    type Value = ParentDocument;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a .mix v2 document object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let (mut version, mut nodes, mut edges, mut instances, mut exposed) =
            (None, None, None, None, None);
        while let Some(key) = map.next_key::<String>()? {
            let duplicate = || de::Error::custom(format!("duplicate field {key:?}"));
            match key.as_str() {
                "version" => {
                    if version.replace(map.next_value::<u32>()?).is_some() {
                        return Err(duplicate());
                    }
                }
                "nodes" if nodes.is_none() => {
                    nodes = Some(map.next_value_seed(Bounded::<Node>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::Nodes,
                    ))?)
                }
                "edges" if edges.is_none() => {
                    edges = Some(map.next_value_seed(Bounded::<Edge>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::Edges,
                    ))?)
                }
                "instances" if instances.is_none() => {
                    instances = Some(map.next_value_seed(Bounded::<Instance>::with(
                        Ceiling::Instances,
                        self.recipe_limits.instance_count,
                        self.exceeded,
                    ))?)
                }
                "exposedParameters" if exposed.is_none() => {
                    exposed = Some(map.next_value_seed(Bounded::<ExposedParameter>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::ExposedParameters,
                    ))?)
                }
                "nodes" | "edges" | "instances" | "exposedParameters" => return Err(duplicate()),
                _ => return Err(de::Error::unknown_field(&key, PARENT_FIELDS)),
            }
        }
        Ok(ParentDocument {
            version: version.ok_or_else(|| de::Error::missing_field("version"))?,
            nodes: nodes.ok_or_else(|| de::Error::missing_field("nodes"))?,
            edges: edges.ok_or_else(|| de::Error::missing_field("edges"))?,
            instances: instances.unwrap_or_default(),
            exposed_parameters: exposed.unwrap_or_default(),
        })
    }
}
