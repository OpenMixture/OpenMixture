use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "brick-pattern",
    version: 1,
    label: "Brick Pattern",
    description: "Periodic beveled rectangular cells with seeded amplitude variation and alternating row offsets.",
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
            id: "columns",
            kind: ParameterKind::Integer { min: 1, max: 64 },
            default: Some(ParameterDefault::Integer(8)),
        },
        ParameterContract {
            id: "rows",
            kind: ParameterKind::Integer { min: 1, max: 64 },
            default: Some(ParameterDefault::Integer(8)),
        },
        ParameterContract {
            id: "rowOffset",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: Some(ParameterDefault::Float(0.5)),
        },
        ParameterContract {
            id: "mortarX",
            kind: ParameterKind::Float {
                min: 0.0,
                max: 0.45,
            },
            default: Some(ParameterDefault::Float(0.08)),
        },
        ParameterContract {
            id: "mortarY",
            kind: ParameterKind::Float {
                min: 0.0,
                max: 0.45,
            },
            default: Some(ParameterDefault::Float(0.08)),
        },
        ParameterContract {
            id: "bevel",
            kind: ParameterKind::Float {
                min: 0.0,
                max: 0.25,
            },
            default: Some(ParameterDefault::Float(0.08)),
        },
        ParameterContract {
            id: "variation",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: Some(ParameterDefault::Float(0.15)),
        },
    ],
};
