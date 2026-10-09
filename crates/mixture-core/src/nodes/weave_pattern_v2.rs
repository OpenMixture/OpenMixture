//! Approved @2 ownership/stacking contract; @1 remains independently addressable.
use crate::registry::*;
pub(crate) static CONTRACT: NodeContract = NodeContract {
    type_id: "weave-pattern",
    version: 2,
    label: "Weave Pattern",
    description: "Periodic occupancy-owned plain weave with continuous stacking height.",
    inputs: &[],
    outputs: super::weave_pattern::CONTRACT.outputs,
    parameters: super::weave_pattern::CONTRACT.parameters,
};
