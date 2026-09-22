//! Deterministic validation of all source nodes and edges, without GPU access.

use crate::{
    Diagnostic, DiagnosticCode as Code, DiagnosticReport, LimitKind, SafetyLimits, Stage,
    document::{DocumentError, Endpoint, FORMAT_VERSION, MaterialDocument, Node},
    registry::{NodeContract, PortDefault, PortKind, node_contract, node_contract_version},
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A source document whose version, graph, contracts, and collection limits passed.
/// No deserializer or mutable accessor can bypass that validation.
#[derive(Clone, Debug)]
pub struct ValidatedDocument {
    document: MaterialDocument,
}

/// How a validated input obtains its value; no pixels are evaluated here.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum InputSource {
    /// A validated edge supplies this input.
    Connected {
        /// Source output endpoint.
        from: Endpoint,
    },
    /// The versioned contract supplies an unconnected input.
    Default {
        /// Typed constant, owned by the node contract.
        value: PortDefault,
    },
}
/// One material channel and its connected/default source, in contract order.
#[derive(Clone, Debug, Serialize)]
pub struct MaterialChannel {
    /// Stable material channel identifier.
    pub id: &'static str,
    /// Exact port kind.
    pub kind: PortKind,
    /// Connected endpoint or documented default.
    pub input: InputSource,
}
impl ValidatedDocument {
    /// Borrow the original source; validation never fills or reorders its fields.
    pub fn document(&self) -> &MaterialDocument {
        &self.document
    }
    /// Resolve an explicit or default parameter without mutating the document.
    pub fn parameter(&self, node_id: &str, parameter_id: &str) -> Option<Value> {
        let node = self.document.nodes.iter().find(|n| n.id == node_id)?;
        resolved_parameter(
            node,
            node_contract_version(&node.type_id, node.version)?,
            parameter_id,
        )
    }
    /// Resolve a single input to a real connection or its versioned default.
    pub fn input_source(&self, node_id: &str, port_id: &str) -> Option<InputSource> {
        let node = self.document.nodes.iter().find(|n| n.id == node_id)?;
        let port = node_contract_version(&node.type_id, node.version)?.input(port_id)?;
        if let Some(edge) = self
            .document
            .edges
            .iter()
            .find(|edge| edge.to.node_id == node_id && edge.to.port_id == port_id)
        {
            return Some(InputSource::Connected {
                from: edge.from.clone(),
            });
        }
        port.default.map(|value| InputSource::Default { value })
    }
    /// Describe all eight material channels, including optional unconnected ones.
    pub fn material_channels(&self) -> Vec<MaterialChannel> {
        let Some(output) = self
            .document
            .nodes
            .iter()
            .find(|node| node.type_id == "material-output")
        else {
            return Vec::new();
        };
        crate::nodes::material_output::CONTRACT
            .inputs
            .iter()
            .filter_map(|port| {
                self.input_source(&output.id, port.id)
                    .map(|input| MaterialChannel {
                        id: port.id,
                        kind: port.kind,
                        input,
                    })
            })
            .collect()
    }
}

impl MaterialDocument {
    /// Validate source semantics and collection limits without editing the source.
    /// Byte limits apply in `decode`; render-request limits belong to the caller's
    /// compiler request. Over-budget collections stop graph analysis.
    pub fn validate(&self, limits: &SafetyLimits) -> DiagnosticReport {
        let mut diagnostics = Vec::new();
        if self.version != FORMAT_VERSION {
            diagnostics.push(unsupported_version(self.version));
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
        if !diagnostics.is_empty() {
            return DiagnosticReport::new(diagnostics);
        }
        let mut nodes: BTreeMap<&str, Vec<&Node>> = BTreeMap::new();
        for node in &self.nodes {
            nodes.entry(&node.id).or_default().push(node);
            validate_node(node, &mut diagnostics);
        }
        for (id, group) in &nodes {
            if group.len() > 1 {
                diagnostics.push(
                    at_node(Code::NodeDuplicateId, id, "Node IDs must be unique.")
                        .with_evidence("observed", group.len() as u64)
                        .with_suggestion("Give every node a unique ID and update its references."),
                );
            }
        }
        let outputs = self
            .nodes
            .iter()
            .filter(|node| node.type_id == "material-output")
            .count();
        if outputs != 1 {
            diagnostics.push(
                failure(
                    Code::GraphMaterialOutputCount,
                    "A v1 material must contain exactly one material-output node.",
                )
                .with_evidence("observed", outputs as u64)
                .with_evidence("required", 1_u64)
                .with_suggestion("Keep one material-output node and connect its baseColor input."),
            );
        }
        let mut edges = BTreeMap::new();
        let mut incoming = BTreeMap::new();
        for edge in &self.edges {
            *edges.entry(edge).or_insert(0_u64) += 1;
            *incoming.entry(&edge.to).or_insert(0_u64) += 1;
        }
        let mut connected = BTreeSet::new();
        for (edge, count) in &edges {
            if *count > 1 {
                diagnostics.push(
                    at_port(
                        Code::GraphDuplicateEdge,
                        &edge.to,
                        "An edge identity occurs more than once.",
                    )
                    .with_evidence("sourceNode", edge.from.node_id.as_str())
                    .with_evidence("sourcePort", edge.from.port_id.as_str())
                    .with_evidence("observed", *count)
                    .with_suggestion(
                        "Remove duplicate edge records; each connection is declared once.",
                    ),
                );
            }
            let from = edge_port(&nodes, &edge.from, true, &mut diagnostics);
            let to = edge_port(&nodes, &edge.to, false, &mut diagnostics);
            if let (Some(from), Some(to)) = (from, to) {
                if from != to {
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
                            "Connect an output of the required kind; v1 never inserts conversions.",
                        ),
                    );
                } else {
                    connected.insert(&edge.to);
                }
            }
        }
        for (target, count) in incoming {
            if count > 1 {
                diagnostics.push(
                    at_port(
                        Code::GraphMultipleInputs,
                        target,
                        "An input may have at most one incoming edge.",
                    )
                    .with_evidence("observed", count)
                    .with_suggestion(
                        "Choose one source or combine sources through an explicit supported node.",
                    ),
                );
            }
        }
        for group in nodes.values().filter(|group| group.len() == 1) {
            if let Some(node) = group.first()
                && let Some(contract) = supported_contract(node)
            {
                for input in contract
                    .inputs
                    .iter()
                    .filter(|input| input.default.is_none())
                {
                    let target = Endpoint {
                        node_id: node.id.clone(),
                        port_id: input.id.into(),
                    };
                    if !connected.contains(&target) {
                        diagnostics.push(
                            at_port(
                                Code::PortRequiredConnection,
                                &target,
                                "Required input has no valid connection.",
                            )
                            .with_suggestion(
                                "Connect one compatible output to this required input.",
                            ),
                        );
                    }
                }
            }
        }
        // Cycles in unused nodes are invalid too. Check topology independently of
        // port/type errors, excluding ambiguous or unknown node identities.
        let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = nodes
            .iter()
            .filter(|(_, group)| group.len() == 1)
            .map(|(id, _)| (*id, BTreeSet::new()))
            .collect();
        for edge in edges.keys() {
            if adjacency.contains_key(edge.to.node_id.as_str())
                && let Some(children) = adjacency.get_mut(edge.from.node_id.as_str())
            {
                children.insert(edge.to.node_id.as_str());
            }
        }
        detect_cycles(&adjacency, &mut diagnostics);
        validate_exposed(self, &nodes, &mut diagnostics);
        DiagnosticReport::new(diagnostics)
    }

    /// Consume source data only after successful validation; the returned wrapper
    /// exposes read-only access and cannot be forged by deserializing JSON.
    pub fn into_validated(self, limits: &SafetyLimits) -> Result<ValidatedDocument, DocumentError> {
        let report = self.validate(limits);
        if !report.is_ok() {
            return Err(DocumentError::new(report.diagnostics().iter().cloned()));
        }
        Ok(ValidatedDocument { document: self })
    }
}

pub(crate) fn unsupported_version(version: u32) -> Diagnostic {
    failure(
        Code::FormatUnsupportedVersion,
        "Unsupported .mix format version.",
    )
    .with_evidence("observed", u64::from(version))
    .with_evidence("supported", u64::from(FORMAT_VERSION))
    .with_suggestion(
        "Use a version 1 document. Legacy import and version migration are not implemented.",
    )
}
fn failure(code: Code, message: &str) -> Diagnostic {
    Diagnostic::error(code, Stage::Validation, message)
}
fn at_node(code: Code, id: &str, message: &str) -> Diagnostic {
    let mut diagnostic = failure(code, message);
    diagnostic.node_id = Some(id.into());
    diagnostic
}
fn at_port(code: Code, target: &Endpoint, message: &str) -> Diagnostic {
    let mut diagnostic = at_node(code, &target.node_id, message);
    diagnostic.port_id = Some(target.port_id.clone());
    diagnostic
}
fn at_parameter(code: Code, node: &str, parameter: &str, message: &str) -> Diagnostic {
    let mut diagnostic = at_node(code, node, message);
    diagnostic.parameter_id = Some(parameter.into());
    diagnostic
}
fn identifier(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn supported_contract(node: &Node) -> Option<&'static NodeContract> {
    node_contract_version(&node.type_id, node.version)
}
fn resolved_parameter(node: &Node, contract: &NodeContract, id: &str) -> Option<Value> {
    let parameter = contract.parameter(id)?;
    node.parameters
        .get(id)
        .cloned()
        .or_else(|| parameter.default.map(|default| default.value()))
}
fn validate_node(node: &Node, diagnostics: &mut Vec<Diagnostic>) {
    if !identifier(&node.id) {
        diagnostics.push(
            at_node(
                Code::NodeInvalidId,
                &node.id,
                "Node ID must match [A-Za-z][A-Za-z0-9_-]{0,63}.",
            )
            .with_suggestion(
                "Use a nonempty ASCII identifier of at most 64 bytes starting with a letter.",
            ),
        );
    }
    let Some(latest) = node_contract(&node.type_id) else {
        diagnostics.push(
            at_node(
                Code::NodeUnknownType,
                &node.id,
                "Unknown built-in node type.",
            )
            .with_evidence("type", node.type_id.as_str())
            .with_suggestion("Use a documented built-in node type."),
        );
        return;
    };
    let Some(contract) = node_contract_version(&node.type_id, node.version) else {
        diagnostics.push(
            at_node(
                Code::NodeUnsupportedVersion,
                &node.id,
                "Unsupported node version.",
            )
            .with_evidence("observed", u64::from(node.version))
            .with_evidence("supported", u64::from(latest.version))
            .with_suggestion(
                "Use a documented supported node version; migrate explicitly before changing pixel semantics.",
            ),
        );
        return;
    };
    for parameter in contract.parameters {
        if parameter.default.is_none() && !node.parameters.contains_key(parameter.id) {
            diagnostics.push(
                at_parameter(Code::ParameterInvalidValue, &node.id, parameter.id,
                    "Required parameter is missing; this contract defines no default.")
                    .with_evidence("required", true)
                    .with_evidence("present", false)
                    .with_suggestion("Declare the required parameter explicitly in the .mix source; randomized nodes require an unsigned integer seed."),
            );
        }
    }
    for (id, value) in &node.parameters {
        let Some(parameter) = contract.parameter(id) else {
            diagnostics.push(
                at_parameter(
                    Code::ParameterUnknown,
                    &node.id,
                    id,
                    "Unknown node parameter.",
                )
                .with_suggestion("Use parameter IDs from this node's versioned contract."),
            );
            continue;
        };
        if !parameter.accepts(value) {
            diagnostics.push(at_parameter(Code::ParameterInvalidValue, &node.id, id, "Parameter does not satisfy its JSON type, range, or enum contract.")
                .with_evidence("expected", format!("{:?}", parameter.kind)).with_evidence("observed", value.to_string())
                .with_suggestion("Supply a finite value in the documented range and exact JSON type; values are never clamped or coerced."));
        }
    }
    if node.type_id == "brick-pattern" {
        let rows = resolved_parameter(node, contract, "rows").and_then(|v| v.as_u64());
        let offset = resolved_parameter(node, contract, "rowOffset").and_then(|v| v.as_f64());
        if let (Some(rows), Some(offset)) = (rows, offset)
            && rows % 2 != 0
            && offset != 0.0
        {
            diagnostics.push(at_parameter(
                Code::ParameterInvalidValue,
                &node.id,
                "rows",
                "A nonzero brick rowOffset requires an even number of rows for periodic alternation.",
            ).with_suggestion("Use an even rows value, or set rowOffset to zero."));
        }
    }
    if node.type_id == "levels" {
        let low = resolved_parameter(node, contract, "inputMin").and_then(|v| v.as_f64());
        let high = resolved_parameter(node, contract, "inputMax").and_then(|v| v.as_f64());
        if let (Some(low), Some(high)) = (low, high)
            && low >= high
        {
            diagnostics.push(
                at_parameter(
                    Code::ParameterInvalidValue,
                    &node.id,
                    "inputMin",
                    "levels requires inputMin < inputMax, including defaulted values.",
                )
                .with_evidence("inputMin", low.to_string())
                .with_evidence("inputMax", high.to_string())
                .with_suggestion("Choose distinct input bounds with inputMin below inputMax."),
            );
        }
    }
}
fn edge_port(
    nodes: &BTreeMap<&str, Vec<&Node>>,
    endpoint: &Endpoint,
    output: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<PortKind> {
    let Some(group) = nodes.get(endpoint.node_id.as_str()) else {
        diagnostics.push(
            at_port(
                Code::GraphUnknownNode,
                endpoint,
                "Edge endpoint refers to an unknown node.",
            )
            .with_evidence("direction", if output { "output" } else { "input" })
            .with_suggestion("Use the ID of an existing node in this document."),
        );
        return None;
    };
    if group.len() != 1 {
        return None;
    }
    let contract = supported_contract(group.first()?)?;
    let port = if output {
        contract.output(&endpoint.port_id)
    } else {
        contract.input(&endpoint.port_id)
    };
    if port.is_none() {
        diagnostics.push(
            at_port(
                Code::PortUnknown,
                endpoint,
                "Node has no port with this name and direction.",
            )
            .with_evidence("direction", if output { "output" } else { "input" })
            .with_suggestion(
                "Use a named output as the source and a named input as the destination.",
            ),
        );
    }
    port.map(|port| port.kind)
}
fn detect_cycles(adjacency: &BTreeMap<&str, BTreeSet<&str>>, diagnostics: &mut Vec<Diagnostic>) {
    let mut color = BTreeMap::new();
    for root in adjacency.keys().copied() {
        if color.contains_key(root) {
            continue;
        }
        color.insert(root, 1);
        let mut stack = vec![(
            root,
            adjacency
                .get(root)
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            0,
        )];
        while let Some((node, children, next)) = stack.last_mut() {
            let Some(target) = children.get(*next).copied() else {
                color.insert(*node, 2);
                stack.pop();
                continue;
            };
            *next += 1;
            match color.get(target) {
                Some(1) => {
                    let mut path: Vec<_> = stack
                        .iter()
                        .skip_while(|(node, _, _)| *node != target)
                        .map(|(node, _, _)| *node)
                        .collect();
                    path.push(target);
                    diagnostics.push(
                        at_node(
                            Code::GraphCycle,
                            target,
                            "Material graph contains a directed cycle.",
                        )
                        .with_evidence("cycle", path.join(" -> "))
                        .with_suggestion(
                            "Remove an edge in the reported cycle; feedback loops are unsupported.",
                        ),
                    );
                }
                Some(_) => {}
                None => {
                    color.insert(target, 1);
                    stack.push((
                        target,
                        adjacency
                            .get(target)
                            .into_iter()
                            .flatten()
                            .copied()
                            .collect(),
                        0,
                    ));
                }
            }
        }
    }
}
fn validate_exposed(
    document: &MaterialDocument,
    nodes: &BTreeMap<&str, Vec<&Node>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut ids = BTreeMap::new();
    let mut targets = BTreeMap::new();
    for binding in &document.exposed_parameters {
        *ids.entry(binding.id.as_str()).or_insert(0_u64) += 1;
        *targets
            .entry((&binding.node_id, &binding.parameter_id))
            .or_insert(0_u64) += 1;
        let invalid = |message| {
            at_parameter(Code::ExposedParameterInvalid, &binding.node_id, &binding.parameter_id, message)
            .with_evidence("publicId", binding.id.as_str())
            .with_suggestion("Bind one unique public ID to one existing mutable node parameter; aliases are unsupported.")
        };
        if !identifier(&binding.id) {
            diagnostics.push(invalid(
                "Public parameter ID must match [A-Za-z][A-Za-z0-9_-]{0,63}.",
            ));
        }
        let target = nodes
            .get(binding.node_id.as_str())
            .filter(|group| group.len() == 1)
            .and_then(|group| group.first())
            .and_then(|node| supported_contract(node))
            .and_then(|contract| contract.parameter(&binding.parameter_id));
        if target.is_none() {
            diagnostics.push(invalid(
                "Exposed parameter target does not resolve to one supported mutable parameter.",
            ));
        }
    }
    for (id, count) in ids {
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
    for ((node, parameter), count) in targets {
        if count > 1 {
            diagnostics.push(
                at_parameter(
                    Code::ExposedParameterInvalid,
                    node,
                    parameter,
                    "Multiple public bindings target the same node parameter.",
                )
                .with_evidence("observed", count)
                .with_suggestion("Expose this target once; v1 does not support parameter aliases."),
            );
        }
    }
}
