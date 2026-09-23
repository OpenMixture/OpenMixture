use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "scalar-morphology",
    version: 1,
    label: "Scalar Morphology",
    description: "Wrapped neighborhood minimum or maximum along one axis of a normalized scalar field.",
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
            id: "operation",
            kind: ParameterKind::Enum {
                values: &["erode", "dilate"],
            },
            default: Some(ParameterDefault::Enum("erode")),
        },
        ParameterContract {
            id: "axis",
            kind: ParameterKind::Enum {
                values: &["x", "y"],
            },
            default: Some(ParameterDefault::Enum("x")),
        },
        ParameterContract {
            id: "radius",
            kind: ParameterKind::Integer { min: 0, max: 16 },
            default: Some(ParameterDefault::Integer(2)),
        },
    ],
};
