use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "brick-pattern",
    version: 1,
    label: "Brick Pattern",
    description: "Periodic brick height with mortar and linear bevels; half-offset requires even rows.",
    inputs: &[],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "columns",
            kind: ParameterKind::Integer { min: 1, max: 256 },
            default: Some(ParameterDefault::Integer(4)),
        },
        ParameterContract {
            id: "rows",
            kind: ParameterKind::Integer { min: 1, max: 256 },
            default: Some(ParameterDefault::Integer(8)),
        },
        ParameterContract {
            id: "layout",
            kind: ParameterKind::Enum {
                values: &["aligned", "half-offset"],
            },
            default: Some(ParameterDefault::Enum("half-offset")),
        },
        ParameterContract {
            id: "gap",
            kind: ParameterKind::Float { min: 0.0, max: 0.5 },
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
    ],
};
