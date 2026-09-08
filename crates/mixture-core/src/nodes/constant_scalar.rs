use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "constant-scalar",
    version: 1,
    label: "Constant scalar",
    description: "Uniform scalar in [0, 1].",
    inputs: &[],
    outputs: &[PortContract {
        id: "value",
        kind: PortKind::Scalar,
        default: None,
    }],
    parameters: &[ParameterContract {
        id: "value",
        kind: ParameterKind::Float { min: 0.0, max: 1.0 },
        default: Some(ParameterDefault::Float(0.0)),
    }],
};
