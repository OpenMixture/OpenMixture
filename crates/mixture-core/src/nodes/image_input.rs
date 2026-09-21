use crate::registry::*;

pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "image-input",
    version: 1,
    label: "Image Input",
    description: "Read normalized red from a same-size caller-supplied linear RGBA8 image.",
    inputs: &[],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[ParameterContract {
        id: "resourceId",
        kind: ParameterKind::ResourceRef,
        default: None,
    }],
};
