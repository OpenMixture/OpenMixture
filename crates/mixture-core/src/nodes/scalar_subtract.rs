use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "scalar-subtract",
    version: 1,
    label: "Scalar Subtract",
    description: "Subtract normalized scalar b from a, saturating at zero.",
    inputs: &[
        PortContract {
            id: "a",
            kind: PortKind::Scalar,
            default: None,
        },
        PortContract {
            id: "b",
            kind: PortKind::Scalar,
            default: None,
        },
    ],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[],
};
