use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "gradient-map",
    version: 1,
    label: "Gradient Map",
    description: "Map a clamped scalar through two endpoints, interpolating straight linear RGBA.",
    inputs: &[PortContract {
        id: "in",
        kind: PortKind::Scalar,
        default: None,
    }],
    outputs: &[PortContract {
        id: "color",
        kind: PortKind::Color,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "colorA",
            kind: ParameterKind::Color,
            default: Some(ParameterDefault::Color([0.0, 0.0, 0.0, 1.0])),
        },
        ParameterContract {
            id: "colorB",
            kind: ParameterKind::Color,
            default: Some(ParameterDefault::Color([1.0, 1.0, 1.0, 1.0])),
        },
    ],
};
