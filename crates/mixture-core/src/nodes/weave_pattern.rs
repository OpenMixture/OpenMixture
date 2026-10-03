use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "weave-pattern",
    version: 1,
    label: "Weave Pattern",
    description: "Periodic plain-weave height, occupancy or coverage-weighted warp share from one analytical structure.",
    inputs: &[],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[
        ParameterContract {
            id: "mode",
            kind: ParameterKind::Enum {
                values: &["height", "coverage", "warp-share"],
            },
            default: Some(ParameterDefault::Enum("height")),
        },
        ParameterContract {
            id: "warpCount",
            kind: ParameterKind::Integer { min: 4, max: 32 },
            default: Some(ParameterDefault::Integer(8)),
        },
        ParameterContract {
            id: "weftCount",
            kind: ParameterKind::Integer { min: 4, max: 32 },
            default: Some(ParameterDefault::Integer(8)),
        },
        ParameterContract {
            id: "warpWidth",
            kind: ParameterKind::Float {
                min: 0.55,
                max: 0.90,
            },
            default: Some(ParameterDefault::Float(0.7)),
        },
        ParameterContract {
            id: "weftWidth",
            kind: ParameterKind::Float {
                min: 0.55,
                max: 0.90,
            },
            default: Some(ParameterDefault::Float(0.7)),
        },
        ParameterContract {
            id: "bevel",
            kind: ParameterKind::Float {
                min: 0.02,
                max: 0.12,
            },
            default: Some(ParameterDefault::Float(0.08)),
        },
        ParameterContract {
            id: "crown",
            kind: ParameterKind::Float {
                min: 0.00,
                max: 1.00,
            },
            default: Some(ParameterDefault::Float(0.5)),
        },
        ParameterContract {
            id: "underRatio",
            kind: ParameterKind::Float {
                min: 0.25,
                max: 0.75,
            },
            default: Some(ParameterDefault::Float(0.5)),
        },
    ],
};
