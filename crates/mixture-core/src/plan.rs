//! Immutable backend-neutral plans. These describe computation; they never execute it.

use crate::{CompileError, InputSource, registry::PortKind};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr};

/// Version of the plan structure, lowering rules, memory model, and hash encoding.
pub const PLAN_VERSION: u32 = 2;
/// Domain separator prepended to canonical compact plan JSON when hashing.
pub const PLAN_HASH_DOMAIN: &[u8] = b"mixture-render-plan-v2\0";

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

/// One portable intermediate storage format in plan version 2.
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
    /// Periodic rectangular block profiles with deterministic per-cell amplitude.
    BrickPattern,
    /// Read caller-supplied linear RGBA8 red into a Scalar intermediate.
    ImageInput,
    /// Shared uniform constant kernel for scalar, color, and default normal data.
    Constant,
    /// Two-color checker.
    Checker,
    /// Scalar levels remapping.
    Levels,
    /// Component color blending.
    Blend,
    /// Normalized scalar interpolation.
    ScalarBlend,
    /// Spatial masked normalized scalar interpolation.
    ScalarMaskBlend,
    /// Wrapped single-axis neighborhood minimum or maximum.
    ScalarMorphology,
    /// Periodic explicitly seeded scalar noise.
    FractalNoise,
    /// Scalar-to-color linear gradient.
    GradientMap,
    /// Wrapped height derivative to encoded tangent normal.
    HeightToNormal,
    /// Periodic scalar transform with integer scales and clockwise quarter turns.
    Transform2d,
    /// Periodic scalar resampling displaced by a scalar field.
    Warp,
}
/// Fractal noise basis, independent of any backend API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoiseBasis {
    /// Quintic-interpolated periodic lattice values.
    Value,
    /// Version 2 value noise with deterministic Q0.24 evaluation and rounding.
    StableValue,
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
/// Scalar neighborhood reduction, independent of backend discriminants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MorphologyOperation {
    /// Minimum of normalized neighborhood samples.
    Erode,
    /// Maximum of normalized neighborhood samples.
    Dilate,
}
/// Axis of a separable scalar neighborhood.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MorphologyAxis {
    /// Horizontal texel offsets.
    X,
    /// Vertical texel offsets.
    Y,
}
/// Typed kernel arguments and logical input bindings, with f32 GPU parameters.
/// There is no arbitrary JSON, shader source, or redundant untyped input list.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "id", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum KernelInvocation {
    /// Four-sample periodic brick profile; explicit seed retains all 32 bits.
    BrickPattern {
        /// Columns and rows per tile.
        cells: [u32; 2],
        /// Explicit per-cell random seed.
        seed: u32,
        /// Alternating row displacement in cell widths.
        row_offset: f32,
        /// Horizontal and vertical gap fractions.
        mortar: [f32; 2],
        /// Inward smooth profile width in cell coordinates.
        bevel: f32,
        /// Seeded amplitude variation in [0, 1].
        variation: f32,
    },
    /// Immutable content identity is recorded in the plan resource table.
    ImageInput {
        /// Logical caller resource ID, resolved by Core preparation.
        resource_id: String,
    },
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
    /// Pointwise interpolation of two normalized scalar textures.
    ScalarBlend {
        /// First scalar texture.
        a: ResourceId,
        /// Second scalar texture.
        b: ResourceId,
        /// Effective f32 weight in [0, 1].
        weight: f32,
    },
    /// Scalar blend with a required spatial mask.
    ScalarMaskBlend {
        /// First scalar texture.
        a: ResourceId,
        /// Second scalar texture.
        b: ResourceId,
        /// Scalar spatial mask.
        mask: ResourceId,
        /// Overall opacity in [0, 1].
        opacity: f32,
    },
    /// Wrapped separable neighborhood reduction with integer texel radius.
    ScalarMorphology {
        /// Required scalar source.
        input: ResourceId,
        /// Minimum or maximum reduction.
        operation: MorphologyOperation,
        /// Axis along which samples are taken.
        axis: MorphologyAxis,
        /// Inclusive distance in output texels, from zero through sixteen.
        radius: u32,
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
    /// Inverse quarter rotation, then integer scaling, then a sampling UV offset.
    Transform2d {
        /// Scalar input texture.
        input: ResourceId,
        /// Source sample periods on the two source axes per output UV tile.
        scale: [u32; 2],
        /// Visible clockwise quarter rotations, in top-left image coordinates.
        quarter_turns: u32,
        /// Source-coordinate UV offset after rotation and scaling.
        offset: [f32; 2],
    },
    /// Sample input at UV plus centered scalar displacement times UV strength.
    Warp {
        /// Scalar input texture sampled with repeat-bilinear interpolation.
        input: ResourceId,
        /// Scalar field read at each output texel; 0.5 is neutral.
        displacement: ResourceId,
        /// Signed UV displacement at field value one, independently per axis.
        strength: [f32; 2],
    },
}
impl KernelInvocation {
    /// Exhaustive kernel identity; parameter variants cannot disagree with this ID.
    pub fn id(&self) -> KernelId {
        match self {
            Self::BrickPattern { .. } => KernelId::BrickPattern,
            Self::ImageInput { .. } => KernelId::ImageInput,
            Self::Constant { .. } => KernelId::Constant,
            Self::Checker { .. } => KernelId::Checker,
            Self::Levels { .. } => KernelId::Levels,
            Self::Blend { .. } => KernelId::Blend,
            Self::ScalarBlend { .. } => KernelId::ScalarBlend,
            Self::ScalarMaskBlend { .. } => KernelId::ScalarMaskBlend,
            Self::ScalarMorphology { .. } => KernelId::ScalarMorphology,
            Self::FractalNoise { .. } => KernelId::FractalNoise,
            Self::GradientMap { .. } => KernelId::GradientMap,
            Self::HeightToNormal { .. } => KernelId::HeightToNormal,
            Self::Transform2d { .. } => KernelId::Transform2d,
            Self::Warp { .. } => KernelId::Warp,
        }
    }
    /// Logical input resources in binding order. Repeated bindings are preserved.
    pub fn inputs(&self) -> impl Iterator<Item = ResourceId> {
        match self {
            Self::ImageInput { .. }
            | Self::BrickPattern { .. }
            | Self::Constant { .. }
            | Self::Checker { .. }
            | Self::FractalNoise { .. } => [None, None, None],
            Self::Levels { input, .. }
            | Self::ScalarMorphology { input, .. }
            | Self::GradientMap { input, .. }
            | Self::HeightToNormal { input, .. }
            | Self::Transform2d { input, .. } => [Some(*input), None, None],
            Self::Warp {
                input,
                displacement,
                ..
            } => [Some(*input), Some(*displacement), None],
            Self::ScalarBlend { a, b, .. } => [Some(*a), Some(*b), None],
            Self::Blend { a, b, mask, .. } | Self::ScalarMaskBlend { a, b, mask, .. } => {
                [Some(*a), Some(*b), Some(*mask)]
            }
        }
        .into_iter()
        .flatten()
    }
    /// Uniform allocation assumed by plan version 2, including struct padding.
    pub fn uniform_bytes(&self) -> u64 {
        match self {
            Self::ImageInput { .. }
            | Self::Constant { .. }
            | Self::ScalarBlend { .. }
            | Self::ScalarMaskBlend { .. }
            | Self::ScalarMorphology { .. }
            | Self::Blend { .. }
            | Self::HeightToNormal { .. }
            | Self::Warp { .. } => 16,
            Self::Checker { .. } | Self::BrickPattern { .. } => 48,
            Self::Levels { .. }
            | Self::FractalNoise { .. }
            | Self::GradientMap { .. }
            | Self::Transform2d { .. } => 32,
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
    /// Unique selected external images, independent of node reference count.
    pub resource_count: u64,
    /// Packed bytes uploaded for selected external images.
    pub resource_upload_bytes: u64,
    /// External rgba8unorm texture descriptor bytes, separate from pass textures.
    pub resource_texture_bytes: u64,
    /// Conservative 256-byte-row-aligned upload staging bytes.
    pub resource_staging_bytes: u64,
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
    pub image_resources: Vec<crate::ImageResource>,
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
    /// Source document schema version retained by the compiled plan.
    pub fn document_version(&self) -> u32 {
        self.data.document_version
    }
    /// Selected external image identities, sorted by logical ID; never raw pixels.
    pub fn image_resources(&self) -> &[crate::ImageResource] {
        &self.data.image_resources
    }
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
    /// excluding the hash itself. Field order is fixed by plan version 2.
    pub fn hash_input(&self) -> Result<Vec<u8>, CompileError> {
        let mut bytes = PLAN_HASH_DOMAIN.to_vec();
        bytes.extend(serde_json::to_vec(&self.data).map_err(crate::compiler::serialization_error)?);
        Ok(bytes)
    }
}
