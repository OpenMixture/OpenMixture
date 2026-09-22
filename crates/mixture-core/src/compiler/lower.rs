//! Backward dependency slicing, lexical Kahn ordering, and concrete plan lowering.
use super::*;
use crate::{
    InputSource, Node,
    plan::*,
    registry::{PortDefault, PortKind, node_contract_version},
};

pub(crate) fn compile<D: crate::resources::ImageData>(
    normalized: &NormalizedDocument,
    request: &CompileRequest,
    bindings: &[crate::AdapterImageBinding<'_, D>],
    resource_limits: &crate::ResourceLimits,
) -> Result<crate::PreparedRender, CompileError> {
    let document = normalized.document();
    let nodes: BTreeMap<_, _> = document
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let sink = document
        .nodes
        .iter()
        .find(|node| node.type_id == "material-output")
        .ok_or_else(|| invariant("Validated material output is missing."))?;
    let mut channels = request.outputs.clone();
    channels.sort();
    let mut pending = Vec::new();
    for channel in &channels {
        if let Some(InputSource::Connected { from }) =
            normalized.inner.input_source(&sink.id, channel.as_str())
        {
            pending.push(from.node_id);
        }
    }
    let mut selected = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !selected.insert(id.clone()) {
            continue;
        }
        let node = nodes
            .get(id.as_str())
            .ok_or_else(|| invariant("Selected node is missing."))?;
        let contract = node_contract_version(&node.type_id, node.version)
            .ok_or_else(|| invariant("Selected contract is missing."))?;
        for port in contract.inputs {
            if let Some(InputSource::Connected { from }) =
                normalized.inner.input_source(&node.id, port.id)
            {
                pending.push(from.node_id);
            }
        }
    }
    let snapshots =
        crate::resources::capture(normalized, request, bindings, resource_limits, &selected)?;
    // Dependencies are sets of producer nodes, so binding one producer twice
    // does not leave an artificial indegree that Kahn's algorithm cannot clear.
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = selected
        .iter()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect();
    let mut consumers: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for edge in &document.edges {
        if selected.contains(&edge.to.node_id) {
            let inputs = dependencies
                .get_mut(&edge.to.node_id)
                .ok_or_else(|| invariant("Selected dependency entry is missing."))?;
            inputs.insert(edge.from.node_id.clone());
            consumers
                .entry(&edge.from.node_id)
                .or_default()
                .insert(&edge.to.node_id);
        }
    }
    let mut ready: BTreeSet<String> = dependencies
        .iter()
        .filter(|(_, deps)| deps.is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    let mut builder = Builder {
        normalized,
        size: request.size,
        passes: Vec::new(),
        resources: BTreeMap::new(),
    };
    let mut visited = 0;
    while let Some(id) = ready.pop_first() {
        let node = nodes
            .get(id.as_str())
            .ok_or_else(|| invariant("Ready node is missing."))?;
        builder.node(node)?;
        visited += 1;
        for child in consumers.get(id.as_str()).into_iter().flatten() {
            let deps = dependencies
                .get_mut(*child)
                .ok_or_else(|| invariant("Consumer dependency entry is missing."))?;
            deps.remove(&id);
            if deps.is_empty() {
                ready.insert((*child).into());
            }
        }
    }
    if visited != selected.len() {
        return Err(invariant(
            "Validated dependency slice has no complete topological order.",
        ));
    }
    let mut outputs = Vec::new();
    for channel in channels {
        let port = node_contract_version(&sink.type_id, sink.version)
            .and_then(|contract| contract.input(channel.as_str()))
            .ok_or_else(|| invariant("Material channel contract is missing."))?;
        let input = normalized
            .inner
            .input_source(&sink.id, port.id)
            .ok_or_else(|| invariant("Requested material channel has no source."))?;
        let resource = builder.input(sink, port.id, port.kind, &input)?;
        outputs.push(PlanOutput {
            channel,
            kind: port.kind,
            input,
            resource,
        });
    }
    let mut estimates = estimates(request.size, &builder.passes, outputs.len())?;
    for snapshot in &snapshots {
        let image = snapshot.image();
        let texture = mul(mul(u64::from(image.width), u64::from(image.height))?, 4)?;
        let staging = mul(
            mul(add(image.bytes_per_row, 255)? / 256, 256)?,
            u64::from(image.height),
        )?;
        estimates.resource_count = add(estimates.resource_count, 1)?;
        estimates.resource_upload_bytes = add(estimates.resource_upload_bytes, texture)?;
        estimates.resource_texture_bytes = add(estimates.resource_texture_bytes, texture)?;
        estimates.resource_staging_bytes = add(estimates.resource_staging_bytes, staging)?;
        estimates.peak_bytes = add(estimates.peak_bytes, add(texture, staging)?)?;
        estimates.cumulative_bytes = add(estimates.cumulative_bytes, add(texture, staging)?)?;
    }
    request
        .limits
        .check(LimitKind::TransientBytes, estimates.peak_bytes)
        .map_err(|error| CompileError::from(error.diagnostic(Stage::Compile)))?;
    let plan = RenderPlan::new(PlanData {
        version: PLAN_VERSION,
        document_version: document.version,
        size: request.size,
        material_output: identity(sink),
        passes: builder.passes,
        outputs,
        estimates,
        image_resources: snapshots.iter().map(|s| s.image().clone()).collect(),
    })?;
    Ok(crate::resources::prepared(plan, snapshots))
}
struct Builder<'a> {
    normalized: &'a NormalizedDocument,
    size: [u32; 2],
    passes: Vec<ComputePass>,
    resources: BTreeMap<String, ResourceId>,
}
impl Builder<'_> {
    fn push(
        &mut self,
        origin: PassOrigin,
        kind: PortKind,
        kernel: KernelInvocation,
    ) -> Result<ResourceId, CompileError> {
        let index = u32::try_from(self.passes.len()).map_err(|_| overflow())?;
        let output = ResourceId(index);
        self.passes.push(ComputePass {
            id: PassId(index),
            origin,
            kernel,
            output,
            output_desc: TextureDesc {
                width: self.size[0],
                height: self.size[1],
                format: TextureFormat::Rgba16Float,
                kind,
            },
            dispatch: [self.size[0].div_ceil(8), self.size[1].div_ceil(8), 1],
        });
        Ok(output)
    }
    fn input(
        &mut self,
        owner: &Node,
        port: &str,
        kind: PortKind,
        input: &InputSource,
    ) -> Result<ResourceId, CompileError> {
        match input {
            InputSource::Connected { from } => self
                .resources
                .get(&from.node_id)
                .copied()
                .ok_or_else(|| invariant("Input producer has not been lowered.")),
            InputSource::Default { value } => {
                let value = match *value {
                    PortDefault::Scalar(v) => [v as f32, 0.0, 0.0, 1.0],
                    PortDefault::Color(v) => v.map(|v| v as f32),
                    PortDefault::Normal([x, y, z]) => [x as f32, y as f32, z as f32, 1.0],
                };
                self.push(
                    PassOrigin::InputDefault {
                        node: identity(owner),
                        port: port.into(),
                    },
                    kind,
                    KernelInvocation::Constant { value },
                )
            }
        }
    }
    fn node(&mut self, node: &Node) -> Result<(), CompileError> {
        let contract = node_contract_version(&node.type_id, node.version)
            .ok_or_else(|| invariant("Selected node contract is missing."))?;
        let mut bindings = BTreeMap::new();
        for port in contract.inputs {
            let source = self
                .normalized
                .inner
                .input_source(&node.id, port.id)
                .ok_or_else(|| invariant("Selected input source is missing."))?;
            let resource = self.input(node, port.id, port.kind, &source)?;
            bindings.insert(port.id, resource);
        }
        let binding = |port: &str| {
            bindings
                .get(port)
                .copied()
                .ok_or_else(|| invariant("Required typed input binding is missing."))
        };
        let kernel = match node.type_id.as_str() {
            "image-input" => KernelInvocation::ImageInput {
                resource_id: parameter(node, "resourceId")?
                    .as_str()
                    .ok_or_else(|| invariant("Normalized resource reference is invalid."))?
                    .into(),
            },
            "constant-scalar" => KernelInvocation::Constant {
                value: [number(node, "value")?, 0.0, 0.0, 1.0],
            },
            "constant-color" => KernelInvocation::Constant {
                value: color(node, "value")?,
            },
            "checker" => {
                let cells = [integer(node, "cellsX")?, integer(node, "cellsY")?];
                if self
                    .size
                    .into_iter()
                    .zip(cells)
                    .any(|(dimension, cells)| dimension.checked_mul(cells).is_none())
                {
                    return Err(
                        invalid("Checker cell coordinate arithmetic would overflow u32.")
                            .with_evidence("nodeId", node.id.as_str())
                            .into(),
                    );
                }
                KernelInvocation::Checker {
                    cells,
                    color_a: color(node, "colorA")?,
                    color_b: color(node, "colorB")?,
                }
            }
            "levels" => {
                let input_min = number(node, "inputMin")?;
                let input_max = number(node, "inputMax")?;
                if input_min >= input_max {
                    let mut diagnostic = Diagnostic::error(DiagnosticCode::ParameterInvalidValue,Stage::Compile,"levels input bounds collapse after f32 parameter lowering.")
                        .with_suggestion("Separate inputMin and inputMax enough to remain distinct in the f32 GPU representation.");
                    diagnostic.node_id = Some(node.id.clone());
                    diagnostic.parameter_id = Some("inputMin".into());
                    return Err(diagnostic.into());
                }
                KernelInvocation::Levels {
                    input: binding("in")?,
                    input_min,
                    input_max,
                    gamma: number(node, "gamma")?,
                    output_min: number(node, "outputMin")?,
                    output_max: number(node, "outputMax")?,
                }
            }
            "scalar-blend" => KernelInvocation::ScalarBlend {
                a: binding("a")?,
                b: binding("b")?,
                weight: number(node, "weight")?,
            },
            "blend" => {
                let mode = match parameter(node, "mode")?.as_str() {
                    Some("normal") => BlendMode::Normal,
                    Some("multiply") => BlendMode::Multiply,
                    Some("screen") => BlendMode::Screen,
                    _ => return Err(invariant("Validated blend mode is unsupported.")),
                };
                KernelInvocation::Blend {
                    a: binding("a")?,
                    b: binding("b")?,
                    mask: binding("mask")?,
                    mode,
                    opacity: number(node, "opacity")?,
                }
            }
            "fractal-noise" => KernelInvocation::FractalNoise {
                seed: integer(node, "seed")?,
                scale: integer(node, "scale")?,
                octaves: integer(node, "octaves")?,
                persistence: number(node, "persistence")?,
                basis: match parameter(node, "basis")?.as_str() {
                    Some("value") if node.version == 2 => NoiseBasis::StableValue,
                    Some("value") => NoiseBasis::Value,
                    Some("cellular") => NoiseBasis::Cellular,
                    _ => return Err(invariant("Validated noise basis is unsupported.")),
                },
            },
            "gradient-map" => KernelInvocation::GradientMap {
                input: binding("in")?,
                color_a: color(node, "colorA")?,
                color_b: color(node, "colorB")?,
            },
            "height-to-normal" => KernelInvocation::HeightToNormal {
                input: binding("in")?,
                strength: number(node, "strength")?,
            },
            "transform-2d" => KernelInvocation::Transform2d {
                input: binding("in")?,
                scale: [integer(node, "scaleX")?, integer(node, "scaleY")?],
                quarter_turns: integer(node, "quarterTurns")?,
                offset: [number(node, "offsetX")?, number(node, "offsetY")?],
            },
            "warp" => KernelInvocation::Warp {
                input: binding("in")?,
                displacement: binding("displacement")?,
                strength: [number(node, "strengthX")?, number(node, "strengthY")?],
            },
            _ => {
                return Err(invariant(
                    "Selected node has no supported pixel invocation in plan version 2.",
                ));
            }
        };
        let kind = contract
            .outputs
            .first()
            .ok_or_else(|| invariant("Selected pixel node has no output."))?
            .kind;
        let output = self.push(
            PassOrigin::Node {
                node: identity(node),
            },
            kind,
            kernel,
        )?;
        self.resources.insert(node.id.clone(), output);
        Ok(())
    }
}
fn identity(node: &Node) -> NodeIdentity {
    NodeIdentity {
        id: node.id.clone(),
        type_id: node.type_id.clone(),
        version: node.version,
    }
}
fn parameter<'a>(node: &'a Node, id: &str) -> Result<&'a Value, CompileError> {
    node.parameters
        .get(id)
        .ok_or_else(|| invariant("Normalized parameter is missing."))
}
fn number(node: &Node, id: &str) -> Result<f32, CompileError> {
    let value = parameter(node, id)?
        .as_f64()
        .ok_or_else(|| invariant("Normalized numeric parameter is invalid."))?
        as f32;
    if !value.is_finite() {
        return Err(invalid("Parameter cannot be represented as finite f32.")
            .with_evidence("parameterId", id)
            .into());
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}
fn integer(node: &Node, id: &str) -> Result<u32, CompileError> {
    parameter(node, id)?
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| invariant("Normalized integer parameter is invalid."))
}
fn color(node: &Node, id: &str) -> Result<[f32; 4], CompileError> {
    let values = parameter(node, id)?
        .as_array()
        .ok_or_else(|| invariant("Normalized color parameter is invalid."))?;
    let mut result = [0.0; 4];
    if values.len() != 4 {
        return Err(invariant("Normalized color must have four components."));
    }
    for (target, value) in result.iter_mut().zip(values) {
        let value = value
            .as_f64()
            .ok_or_else(|| invariant("Normalized color component is invalid."))?
            as f32;
        if !value.is_finite() {
            return Err(invalid("Color component cannot be represented as finite f32.").into());
        }
        *target = if value == 0.0 { 0.0 } else { value };
    }
    Ok(result)
}
fn overflow() -> CompileError {
    invalid("Plan allocation arithmetic exceeds the supported integer representation.").into()
}
fn add(a: u64, b: u64) -> Result<u64, CompileError> {
    a.checked_add(b).ok_or_else(overflow)
}
fn mul(a: u64, b: u64) -> Result<u64, CompileError> {
    a.checked_mul(b).ok_or_else(overflow)
}
fn estimates(
    size: [u32; 2],
    passes: &[ComputePass],
    outputs: usize,
) -> Result<PlanEstimates, CompileError> {
    let tight_row = mul(u64::from(size[0]), 8)?;
    let row = mul(add(tight_row, 255)? / 256, 256)?;
    let padded_bytes_per_row = u32::try_from(row).map_err(|_| overflow())?;
    let texture = mul(tight_row, u64::from(size[1]))?;
    let texture_bytes = mul(texture, passes.len() as u64)?;
    let uniform_bytes = passes
        .iter()
        .try_fold(0, |bytes, pass| add(bytes, pass.kernel.uniform_bytes()))?;
    let readback_buffer_bytes = mul(row, u64::from(size[1]))?;
    let readback_bytes = mul(texture, outputs as u64)?;
    let cumulative_readback_bytes = mul(readback_buffer_bytes, outputs as u64)?;
    let resident = add(texture_bytes, uniform_bytes)?;
    Ok(PlanEstimates {
        resource_count: 0,
        resource_upload_bytes: 0,
        resource_texture_bytes: 0,
        resource_staging_bytes: 0,
        texture_bytes,
        uniform_bytes,
        padded_bytes_per_row,
        readback_buffer_bytes,
        readback_bytes,
        cumulative_readback_bytes,
        cumulative_bytes: add(resident, cumulative_readback_bytes)?,
        peak_bytes: add(resident, readback_buffer_bytes)?,
    })
}
