//! Immutable backend-neutral plans. These describe computation; they never execute it.

use crate::{CompileError, InputSource, registry::PortKind};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr};

/// Version of the plan structure, lowering rules, memory model, and hash encoding.
pub const PLAN_VERSION: u32 = 1;
/// Domain separator prepended to canonical compact plan JSON when hashing.
pub const PLAN_HASH_DOMAIN: &[u8] = b"mixture-render-plan-v1\0";

/// Material channel request in stable material-contract order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputChannel {
    /// Required color output.
    BaseColor,
    /// Encoded tangent-space normal.
    Normal,
    /// Roughness scalar.
    Roughness,
    /// Metallic scalar.
    Metallic,
    /// Height scalar.
    Height,
    /// Ambient occlusion scalar.
    AmbientOcclusion,
    /// Opacity scalar.
    Opacity,
    /// Linear emissive color.
    Emissive,
}
impl OutputChannel {
    /// Every supported channel, in stable contract order.
    pub const ALL: [Self; 8] = [
        Self::BaseColor,
        Self::Normal,
        Self::Roughness,
        Self::Metallic,
        Self::Height,
        Self::AmbientOcclusion,
        Self::Opacity,
        Self::Emissive,
    ];
    /// Exact public channel name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaseColor => "baseColor",
            Self::Normal => "normal",
            Self::Roughness => "roughness",
            Self::Metallic => "metallic",
            Self::Height => "height",
            Self::AmbientOcclusion => "ambientOcclusion",
            Self::Opacity => "opacity",
            Self::Emissive => "emissive",
        }
    }
}
impl FromStr for OutputChannel {
    type Err = CompileError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|channel| channel.as_str() == value)
            .ok_or_else(|| {
                crate::compiler::invalid("Unknown requested material channel.")
                    .with_evidence("channel", value)
                    .into()
            })
    }
}

/// Zero-based stable compute-pass identity within a plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct PassId(pub(crate) u32);
impl PassId {
    /// The stable numeric index.
    pub fn index(self) -> u32 {
        self.0
    }
}
/// Logical output texture identity; no physical GPU allocation is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ResourceId(pub(crate) u32);
impl ResourceId {
    /// The stable numeric index.
    pub fn index(self) -> u32 {
        self.0
    }
}

/// One portable intermediate storage format in plan version 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum TextureFormat {
    /// Eight bytes per pixel, one mip, one layer, two-dimensional storage.
    #[serde(rename = "rgba16float")]
    Rgba16Float,
}
/// Backend-neutral description of a logical pass output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TextureDesc {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Intermediate pixel storage format.
    pub format: TextureFormat,
    /// Logical interpretation of the stored components.
    pub kind: PortKind,
}
/// Source identity retained for diagnostics and deterministic pass ordering.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeIdentity {
    /// Document-local node ID.
    pub id: String,
    /// Versioned node type ID.
    pub type_id: String,
    /// Node version, independently from the document version.
    pub version: u32,
}
/// Why a pass exists, including explicit lowering of optional-input defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum PassOrigin {
    /// A selected graph node.
    Node {
        /// Source identity.
        node: NodeIdentity,
    },
    /// A constant input required by a selected node or requested material channel.
    InputDefault {
        /// Owner node identity.
        node: NodeIdentity,
        /// Owner input port.
        port: String,
    },
}
/// Compute kernel families; material-output is a mapping, not a pixel pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KernelId {
    /// Shared uniform constant kernel for scalar, color, and default normal data.
    Constant,
    /// Two-color checker.
    Checker,
    /// Scalar levels remapping.
    Levels,
    /// Component color blending.
    Blend,
    /// Periodic explicitly seeded scalar noise.
    FractalNoise,
    /// Scalar-to-color linear gradient.
    GradientMap,
    /// Wrapped height derivative to encoded tangent normal.
    HeightToNormal,
}
/// Fractal noise basis, independent of any backend API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoiseBasis {
    /// Quintic-interpolated periodic lattice values.
    Value,
    /// Gap between nearest two jittered-cell distances in a wrapped 3x3 neighborhood.
    Cellular,
}
/// Supported component blend modes, matching the version 1 node contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BlendMode {
    /// Interpolate toward the second color.
    Normal,
    /// Multiply the two RGB values.
    Multiply,
    /// Screen the two RGB values.
    Screen,
}
/// Typed kernel arguments and logical input bindings, with f32 GPU parameters.
/// There is no arbitrary JSON, shader source, or redundant untyped input list.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "id", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum KernelInvocation {
    /// Uniform RGBA storage value. Scalars use [value, 0, 0, 1]; normals use [xyz, 1].
    Constant {
        /// Packed uniform value.
        value: [f32; 4],
    },
    /// Checker cells and colors. Pixel coordinates and dimensions come from the pass.
    Checker {
        /// Integer cells per axis.
        cells: [u32; 2],
        /// Even-parity linear RGBA.
        color_a: [f32; 4],
        /// Odd-parity linear RGBA.
        color_b: [f32; 4],
    },
    /// Scalar levels with strictly increasing input bounds after f32 lowering.
    Levels {
        /// Scalar input texture.
        input: ResourceId,
        /// Lower input bound.
        input_min: f32,
        /// Upper input bound.
        input_max: f32,
        /// Positive gamma.
        gamma: f32,
        /// Output at zero.
        output_min: f32,
        /// Output at one.
        output_max: f32,
    },
    /// Color blend with two color textures and one scalar mask.
    Blend {
        /// First color texture.
        a: ResourceId,
        /// Second color texture.
        b: ResourceId,
        /// Scalar mask texture.
        mask: ResourceId,
        /// Component blend mode.
        mode: BlendMode,
        /// Overall opacity.
        opacity: f32,
    },
    /// Explicit seed and bounded periodic fractal value-noise parameters.
    FractalNoise {
        /// All 32 bits of the source seed, never lowered through a float.
        seed: u32,
        /// Integer base lattice period per output tile on both axes.
        scale: u32,
        /// Number of octaves; each doubles the lattice period.
        octaves: u32,
        /// Successive octave amplitude multiplier.
        persistence: f32,
        /// Per-octave basis.
        basis: NoiseBasis,
    },
    /// Clamped scalar input interpolated between two linear RGBA endpoints.
    GradientMap {
        /// Scalar input texture.
        input: ResourceId,
        /// Color at zero.
        color_a: [f32; 4],
        /// Color at one.
        color_b: [f32; 4],
    },
    /// Wrapped central differences in UV units, with tangent +Y up.
    HeightToNormal {
        /// Scalar height texture.
        input: ResourceId,
        /// Height amplitude per unit UV tile; zero produces neutral normals.
        strength: f32,
    },
}
impl KernelInvocation {
    /// Exhaustive kernel identity; parameter variants cannot disagree with this ID.
    pub fn id(&self) -> KernelId {
        match self {
            Self::Constant { .. } => KernelId::Constant,
            Self::Checker { .. } => KernelId::Checker,
            Self::Levels { .. } => KernelId::Levels,
            Self::Blend { .. } => KernelId::Blend,
            Self::FractalNoise { .. } => KernelId::FractalNoise,
            Self::GradientMap { .. } => KernelId::GradientMap,
            Self::HeightToNormal { .. } => KernelId::HeightToNormal,
        }
    }
    /// Logical input resources in binding order. Repeated bindings are preserved.
    pub fn inputs(&self) -> impl Iterator<Item = ResourceId> {
        match self {
            Self::Constant { .. } | Self::Checker { .. } | Self::FractalNoise { .. } => {
                [None, None, None]
            }
            Self::Levels { input, .. }
            | Self::GradientMap { input, .. }
            | Self::HeightToNormal { input, .. } => [Some(*input), None, None],
            Self::Blend { a, b, mask, .. } => [Some(*a), Some(*b), Some(*mask)],
        }
        .into_iter()
        .flatten()
    }
    /// Uniform allocation assumed by plan version 1, including struct padding.
    pub fn uniform_bytes(&self) -> u64 {
        match self {
            Self::Constant { .. } | Self::Blend { .. } | Self::HeightToNormal { .. } => 16,
            Self::Checker { .. } => 48,
            Self::Levels { .. } | Self::FractalNoise { .. } | Self::GradientMap { .. } => 32,
        }
    }
}
/// One compute dispatch producing one logical texture. Owned by an immutable plan.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputePass {
    /// Stable pass index.
    pub id: PassId,
    /// Graph or synthesized-default provenance.
    pub origin: PassOrigin,
    /// Typed invocation, including input bindings.
    pub kernel: KernelInvocation,
    /// Logical result texture.
    pub output: ResourceId,
    /// Format, kind, and dimensions of that texture.
    pub output_desc: TextureDesc,
    /// Workgroup counts, using [8, 8, 1] local workgroups.
    pub dispatch: [u32; 3],
}
/// One requested channel, with connection/default provenance and a real resource.
#[derive(Clone, Debug, Serialize)]
pub struct PlanOutput {
    /// Requested material channel.
    pub channel: OutputChannel,
    /// Logical output kind.
    pub kind: PortKind,
    /// Connection or versioned default, never an implicit fallback.
    pub input: InputSource,
    /// Logical texture to read back.
    pub resource: ResourceId,
}
/// Checked logical GPU allocation estimates, not driver measurements.
/// All pass textures and uniforms remain resident; readbacks allocate and release
/// one staging buffer per requested channel in sequence. No pooling or early release.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanEstimates {
    /// Sum of all logical pass textures, also the peak resident texture bytes.
    pub texture_bytes: u64,
    /// Sum of all padded uniform allocations.
    pub uniform_bytes: u64,
    /// 256-byte-aligned staging row stride.
    pub padded_bytes_per_row: u32,
    /// One staging buffer; the maximum simultaneously resident staging bytes.
    pub readback_buffer_bytes: u64,
    /// Tight rgba16float bytes across all requested channel readbacks.
    pub readback_bytes: u64,
    /// Total allocated staging bytes across the sequential readbacks.
    pub cumulative_readback_bytes: u64,
    /// Sum of all texture, uniform, and staging allocations over the plan.
    pub cumulative_bytes: u64,
    /// Conservative simultaneous logical GPU bytes, checked against SafetyLimits.
    pub peak_bytes: u64,
}
/// SHA-256 digest of the domain-separated canonical plan body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct PlanHash(String);
impl PlanHash {
    /// `sha256:` followed by 64 lowercase hexadecimal digits.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for PlanHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlanData {
    pub version: u32,
    pub document_version: u32,
    pub size: [u32; 2],
    pub material_output: NodeIdentity,
    pub passes: Vec<ComputePass>,
    pub outputs: Vec<PlanOutput>,
    pub estimates: PlanEstimates,
}
/// Immutable compiler output. No deserializer or public constructor accepts
/// forged passes, resource references, estimates, or hashes.
#[derive(Clone, Debug, Serialize)]
pub struct RenderPlan {
    #[serde(flatten)]
    data: PlanData,
    hash: PlanHash,
}
impl RenderPlan {
    pub(crate) fn new(data: PlanData) -> Result<Self, CompileError> {
        let mut bytes = PLAN_HASH_DOMAIN.to_vec();
        bytes.extend(serde_json::to_vec(&data).map_err(crate::compiler::serialization_error)?);
        let digest = Sha256::digest(&bytes);
        let hexadecimal: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        let hash = PlanHash(format!("sha256:{hexadecimal}"));
        Ok(Self { data, hash })
    }
    /// Plan schema and lowering version.
    pub fn version(&self) -> u32 {
        self.data.version
    }
    /// Actual output dimensions.
    pub fn size(&self) -> [u32; 2] {
        self.data.size
    }
    /// Stable compute order; includes selected input/default constant passes.
    pub fn passes(&self) -> &[ComputePass] {
        &self.data.passes
    }
    /// Requested channel mappings in canonical contract order.
    pub fn outputs(&self) -> &[PlanOutput] {
        &self.data.outputs
    }
    /// Checked estimates for the declared naive lifetime model.
    pub fn estimates(&self) -> &PlanEstimates {
        &self.data.estimates
    }
    /// Stable semantic plan digest.
    pub fn hash(&self) -> &PlanHash {
        &self.hash
    }
    /// Complete reproducible hash input: domain separator plus compact plan JSON,
    /// excluding the hash itself. Field order is fixed by plan version 1.
    pub fn hash_input(&self) -> Result<Vec<u8>, CompileError> {
        let mut bytes = PLAN_HASH_DOMAIN.to_vec();
        bytes.extend(serde_json::to_vec(&self.data).map_err(crate::compiler::serialization_error)?);
        Ok(bytes)
    }
}
