use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "warp",
    version: 1,
    label: "Warp",
    description: "Repeat-bilinear scalar sampling displaced in UV units by a centered scalar field.",
    inputs: &[
        PortContract {
            id: "in",
            kind: PortKind::Scalar,
            default: None,
        },
        PortContract {
            id: "displacement",
            kind: PortKind::Scalar,
            default: None,
        },
    ],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "strengthX",
            kind: ParameterKind::Float {
                min: -1.0,
                max: 1.0,
            },
            default: Some(ParameterDefault::Float(0.05)),
        },
        ParameterContract {
            id: "strengthY",
            kind: ParameterKind::Float {
                min: -1.0,
                max: 1.0,
            },
            default: Some(ParameterDefault::Float(0.0)),
        },
    ],
};
