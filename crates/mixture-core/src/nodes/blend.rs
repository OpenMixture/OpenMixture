use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "blend",
    version: 1,
    label: "Blend",
    description: "Blend colors a and b using a scalar mask, opacity, and an explicit mode.",
    inputs: &[
        PortContract {
            id: "a",
            kind: PortKind::Color,
            default: None,
        },
        PortContract {
            id: "b",
            kind: PortKind::Color,
            default: None,
        },
        PortContract {
            id: "mask",
            kind: PortKind::Scalar,
            default: Some(PortDefault::Scalar(1.0)),
        },
    ],
    outputs: &[PortContract {
        id: "color",
        kind: PortKind::Color,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "mode",
            kind: ParameterKind::Enum {
                values: &["normal", "multiply", "screen"],
            },
            default: ParameterDefault::Enum("normal"),
        },
        ParameterContract {
            id: "opacity",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: ParameterDefault::Float(1.0),
        },
    ],
};
