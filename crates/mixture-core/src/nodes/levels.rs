use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "levels",
    version: 1,
    label: "Levels",
    description: "Clamp and remap a scalar using inputMin < inputMax and positive gamma.",
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
            id: "inputMin",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: ParameterDefault::Float(0.0),
        },
        ParameterContract {
            id: "inputMax",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: ParameterDefault::Float(1.0),
        },
        ParameterContract {
            id: "gamma",
            kind: ParameterKind::Float {
                min: 0.01,
                max: 100.0,
            },
            default: ParameterDefault::Float(1.0),
        },
        ParameterContract {
            id: "outputMin",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: ParameterDefault::Float(0.0),
        },
        ParameterContract {
            id: "outputMax",
            kind: ParameterKind::Float { min: 0.0, max: 1.0 },
            default: ParameterDefault::Float(1.0),
        },
    ],
};
