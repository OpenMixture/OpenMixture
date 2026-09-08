use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "constant-color",
    version: 1,
    label: "Constant color",
    description: "Uniform linear RGBA color in [0, 1].",
    inputs: &[],
    outputs: &[PortContract {
        id: "color",
        kind: PortKind::Color,
        default: None,
    }],
    parameters: &[ParameterContract {
        id: "value",
        kind: ParameterKind::Color,
        default: Some(ParameterDefault::Color([1.0, 1.0, 1.0, 1.0])),
    }],
};
