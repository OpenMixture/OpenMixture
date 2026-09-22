//! Versioned built-in contracts only. Pixel execution belongs to mixture-wgpu.

use serde::Serialize;
use serde_json::{Value, json};

/// Exact connection kind; v1 has no implicit conversions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PortKind {
    /// One logical scalar per pixel.
    Scalar,
    /// Linear RGBA color.
    Color,
    /// Encoded tangent-space normal.
    Normal,
}
impl PortKind {
    /// Stable wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Color => "color",
            Self::Normal => "normal",
        }
    }
}
/// Versioned constant used only when an optional input is unconnected.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum PortDefault {
    /// Scalar constant.
    Scalar(f64),
    /// Linear RGBA constant.
    Color([f64; 4]),
    /// Encoded XYZ normal; neutral is [0.5, 0.5, 1.0].
    Normal([f64; 3]),
}
/// One input or output port, depending on its contract list.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortContract {
    /// Stable port identifier.
    pub id: &'static str,
    /// Required connection kind.
    pub kind: PortKind,
    /// An input is required when this is None. Outputs never have defaults.
    pub default: Option<PortDefault>,
}
/// Accepted JSON parameter representation and bounds, inclusive at both ends.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ParameterKind {
    /// A case-sensitive caller resource ID using the document identifier grammar.
    ResourceRef,
    /// A finite JSON number within the bounds.
    Float {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },
    /// A JSON unsigned integer token within the bounds; 8.0 is not an integer token.
    Integer {
        /// Minimum value.
        min: u32,
        /// Maximum value.
        max: u32,
    },
    /// Exactly four finite numbers, each between zero and one.
    Color,
    /// One of the explicitly declared strings.
    Enum {
        /// Accepted values, case-sensitive.
        values: &'static [&'static str],
    },
}
/// Typed parameter default; resolves omitted values without editing the source.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(untagged)]
pub enum ParameterDefault {
    /// Float default.
    Float(f64),
    /// Integer default.
    Integer(u32),
    /// RGBA default.
    Color([f64; 4]),
    /// Enum default.
    Enum(&'static str),
}
impl ParameterDefault {
    /// Return the default in the same JSON representation as an explicit value.
    pub fn value(self) -> Value {
        match self {
            Self::Float(v) => json!(v),
            Self::Integer(v) => json!(v),
            Self::Color(v) => json!(v),
            Self::Enum(v) => json!(v),
        }
    }
}
/// A public, mutable parameter owned by a node contract.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ParameterContract {
    /// Stable parameter identifier.
    pub id: &'static str,
    /// JSON type and range.
    pub kind: ParameterKind,
    /// Value for an omitted parameter; None requires an explicit source value.
    pub default: Option<ParameterDefault>,
}
impl ParameterContract {
    /// Check a value without coercion, clamping, or source mutation.
    pub fn accepts(&self, value: &Value) -> bool {
        match self.kind {
            ParameterKind::ResourceRef => value.as_str().is_some_and(crate::resources::valid_id),
            ParameterKind::Float { min, max } => value
                .as_f64()
                .is_some_and(|v| v.is_finite() && (min..=max).contains(&v)),
            ParameterKind::Integer { min, max } => value
                .as_u64()
                .is_some_and(|v| (u64::from(min)..=u64::from(max)).contains(&v)),
            ParameterKind::Color => value.as_array().is_some_and(|v| {
                v.len() == 4
                    && v.iter().all(|v| {
                        v.as_f64()
                            .is_some_and(|v| v.is_finite() && (0.0..=1.0).contains(&v))
                    })
            }),
            ParameterKind::Enum { values } => value.as_str().is_some_and(|v| values.contains(&v)),
        }
    }
}
/// Static schema for one supported node version. No GPU kernel or pixel code.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeContract {
    /// Stable lowercase kebab-case type ID.
    pub type_id: &'static str,
    /// Node version, independently checked from the document version.
    pub version: u32,
    /// Human-readable name.
    pub label: &'static str,
    /// Short semantic description.
    pub description: &'static str,
    /// Named single-input ports.
    pub inputs: &'static [PortContract],
    /// Named output ports.
    pub outputs: &'static [PortContract],
    /// Mutable parameters and their defaults.
    pub parameters: &'static [ParameterContract],
}
impl NodeContract {
    /// Find a named input; output names are not accepted here.
    pub fn input(&self, id: &str) -> Option<&PortContract> {
        self.inputs.iter().find(|p| p.id == id)
    }
    /// Find a named output; input names are not accepted here.
    pub fn output(&self, id: &str) -> Option<&PortContract> {
        self.outputs.iter().find(|p| p.id == id)
    }
    /// Find a mutable parameter.
    pub fn parameter(&self, id: &str) -> Option<&ParameterContract> {
        self.parameters.iter().find(|p| p.id == id)
    }
}
/// The fifteen reviewed node contracts in lexical type-ID order.
pub static BUILT_INS: &[&NodeContract] = &[
    &crate::nodes::blend::CONTRACT,
    &crate::nodes::brick_pattern::CONTRACT,
    &crate::nodes::checker::CONTRACT,
    &crate::nodes::constant_color::CONTRACT,
    &crate::nodes::constant_scalar::CONTRACT,
    &crate::nodes::fractal_noise::CONTRACT,
    &crate::nodes::gradient_map::CONTRACT,
    &crate::nodes::height_to_normal::CONTRACT,
    &crate::nodes::image_input::CONTRACT,
    &crate::nodes::levels::CONTRACT,
    &crate::nodes::material_output::CONTRACT,
    &crate::nodes::scalar_blend::CONTRACT,
    &crate::nodes::scalar_mask_blend::CONTRACT,
    &crate::nodes::transform_2d::CONTRACT,
    &crate::nodes::warp::CONTRACT,
];
/// Find the latest supported contract by exact type ID.
pub fn node_contract(type_id: &str) -> Option<&'static NodeContract> {
    BUILT_INS
        .iter()
        .copied()
        .find(|contract| contract.type_id == type_id)
}

/// Resolve an explicit source version, including retained legacy noise semantics.
pub fn node_contract_version(type_id: &str, version: u32) -> Option<&'static NodeContract> {
    if type_id == "fractal-noise" && version == 1 {
        return Some(&crate::nodes::fractal_noise::LEGACY_CONTRACT);
    }
    node_contract(type_id).filter(|contract| contract.version == version)
}
