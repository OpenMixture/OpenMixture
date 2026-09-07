use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "checker",
    version: 1,
    label: "Checker",
    description: "Integer cells per axis; even parity selects colorA at the top-left origin.",
    inputs: &[],
    outputs: &[PortContract {
        id: "color",
        kind: PortKind::Color,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "cellsX",
            kind: ParameterKind::Integer { min: 1, max: 1024 },
            default: ParameterDefault::Integer(8),
        },
        ParameterContract {
            id: "cellsY",
            kind: ParameterKind::Integer { min: 1, max: 1024 },
            default: ParameterDefault::Integer(8),
        },
        ParameterContract {
            id: "colorA",
            kind: ParameterKind::Color,
            default: ParameterDefault::Color([0.0, 0.0, 0.0, 1.0]),
        },
        ParameterContract {
            id: "colorB",
            kind: ParameterKind::Color,
            default: ParameterDefault::Color([1.0, 1.0, 1.0, 1.0]),
        },
    ],
};
