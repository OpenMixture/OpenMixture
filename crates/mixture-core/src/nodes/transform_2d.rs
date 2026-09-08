use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "transform-2d",
    version: 1,
    label: "Transform 2D",
    description: "Repeat-bilinear scalar transform with integer sample periods, clockwise quarter rotations, and UV sampling offsets.",
    inputs: &[PortContract {
        id: "in",
        kind: PortKind::Scalar,
        default: None,
    }],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "scaleX",
            kind: ParameterKind::Integer { min: 1, max: 64 },
            default: Some(ParameterDefault::Integer(1)),
        },
        ParameterContract {
            id: "scaleY",
            kind: ParameterKind::Integer { min: 1, max: 64 },
            default: Some(ParameterDefault::Integer(1)),
        },
        ParameterContract {
            id: "quarterTurns",
            kind: ParameterKind::Integer { min: 0, max: 3 },
            default: Some(ParameterDefault::Integer(0)),
        },
        ParameterContract {
            id: "offsetX",
            kind: ParameterKind::Float {
                min: -1.0,
                max: 1.0,
            },
            default: Some(ParameterDefault::Float(0.0)),
        },
        ParameterContract {
            id: "offsetY",
            kind: ParameterKind::Float {
                min: -1.0,
                max: 1.0,
            },
            default: Some(ParameterDefault::Float(0.0)),
        },
    ],
};
