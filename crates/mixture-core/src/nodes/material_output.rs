use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "material-output",
    version: 1,
    label: "Material output",
    description: "Exactly one material sink; baseColor is required, other channels have versioned defaults.",
    inputs: &[
        PortContract {
            id: "baseColor",
            kind: PortKind::Color,
            default: None,
        },
        PortContract {
            id: "normal",
            kind: PortKind::Normal,
            default: Some(PortDefault::Normal([0.5, 0.5, 1.0])),
        },
        PortContract {
            id: "roughness",
            kind: PortKind::Scalar,
            default: Some(PortDefault::Scalar(1.0)),
        },
        PortContract {
            id: "metallic",
            kind: PortKind::Scalar,
            default: Some(PortDefault::Scalar(0.0)),
        },
        PortContract {
            id: "height",
            kind: PortKind::Scalar,
            default: Some(PortDefault::Scalar(0.0)),
        },
        PortContract {
            id: "ambientOcclusion",
            kind: PortKind::Scalar,
            default: Some(PortDefault::Scalar(1.0)),
        },
        PortContract {
            id: "opacity",
            kind: PortKind::Scalar,
            default: Some(PortDefault::Scalar(1.0)),
        },
        PortContract {
            id: "emissive",
            kind: PortKind::Color,
            default: Some(PortDefault::Color([0.0, 0.0, 0.0, 1.0])),
        },
    ],
    outputs: &[],
    parameters: &[],
};
