use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "scalar-mask-blend",
    version: 1,
    label: "Scalar Mask Blend",
    description: "Interpolate two normalized scalar fields using a required spatial mask and opacity.",
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
        PortContract {
            id: "mask",
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
        id: "opacity",
        kind: ParameterKind::Float { min: 0.0, max: 1.0 },
        default: Some(ParameterDefault::Float(1.0)),
    }],
};
