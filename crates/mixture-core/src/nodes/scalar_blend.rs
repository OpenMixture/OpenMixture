use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "scalar-blend",
    version: 1,
    label: "Scalar Blend",
    description: "Linearly interpolate two normalized scalar fields with one weight.",
    inputs: &[
        PortContract {
            id: "a",
            kind: PortKind::Scalar,
            default: None,
        },
        PortContract {
            id: "b",
            kind: PortKind::Scalar,
            default: None,
        },
    ],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[ParameterContract {
        id: "weight",
        kind: ParameterKind::Float { min: 0.0, max: 1.0 },
        default: Some(ParameterDefault::Float(0.5)),
    }],
};
