use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "height-to-normal",
    version: 1,
    label: "Height to Normal",
    description: "Wrap central differences in UV units; encode normalize(-strength*dH/du, +strength*dH/dv, 1), with image v down and tangent +Y up.",
    inputs: &[PortContract {
        id: "in",
        kind: PortKind::Scalar,
        default: None,
    }],
    outputs: &[PortContract {
        id: "normal",
        kind: PortKind::Normal,
        default: None,
    }],
    parameters: &[ParameterContract {
        id: "strength",
        kind: ParameterKind::Float { min: 0.0, max: 8.0 },
        default: Some(ParameterDefault::Float(1.0)),
    }],
};
