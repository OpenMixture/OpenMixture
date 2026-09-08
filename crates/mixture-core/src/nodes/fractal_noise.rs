use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "fractal-noise",
    version: 1,
    label: "Fractal Noise",
    description: "Seeded periodic value or cellular noise with octave doubling and normalized accumulation.",
    inputs: &[],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "seed",
            kind: ParameterKind::Integer {
                min: 0,
                max: u32::MAX,
            },
            default: None,
        },
        ParameterContract {
            id: "scale",
            kind: ParameterKind::Integer { min: 1, max: 128 },
            default: Some(ParameterDefault::Integer(8)),
        },
        ParameterContract {
            id: "octaves",
            kind: ParameterKind::Integer { min: 1, max: 6 },
            default: Some(ParameterDefault::Integer(4)),
        },
        ParameterContract {
            id: "persistence",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: Some(ParameterDefault::Float(0.5)),
        },
        ParameterContract {
            id: "basis",
            kind: ParameterKind::Enum {
                values: &["value", "cellular"],
            },
            default: Some(ParameterDefault::Enum("value")),
        },
    ],
};
