//! Freeze MAT-01 material budget and public-override wiring before pixel acceptance.
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};
use serde_json::Value;
#[test]
fn material_cases_compile_through_public_overrides_within_frozen_budget() {
    let document = MaterialDocument::decode(
        include_bytes!("../../../fixtures/materials/brick-paving/material.mix"),
        &SafetyLimits::default(),
    )
    .unwrap()
    .into_validated(&SafetyLimits::default())
    .unwrap();
    let acceptance: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/materials/brick-paving/acceptance.json"
    ))
    .unwrap();
    let controls: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/materials/brick-paving/controls.json"
    ))
    .unwrap();
    for case in acceptance["cases"].as_array().unwrap() {
        for size in acceptance["sizes"].as_array().unwrap() {
            let mut request = CompileRequest {
                size: [
                    size[0].as_u64().unwrap() as u32,
                    size[1].as_u64().unwrap() as u32,
                ],
                outputs: vec![
                    OutputChannel::BaseColor,
                    OutputChannel::Normal,
                    OutputChannel::Roughness,
                    OutputChannel::Height,
                ],
                ..Default::default()
            };
            for (control, value) in case["overrides"].as_object().unwrap() {
                for id in controls[control].as_array().unwrap() {
                    request
                        .overrides
                        .insert(id.as_str().unwrap().into(), value.clone());
                }
            }
            let plan = compile(&document, &request).unwrap();
            assert_eq!(plan.passes().len(), 10);
            assert!(plan.passes().len() as u64 <= acceptance["maxPasses"].as_u64().unwrap());
            assert!(
                plan.estimates().peak_bytes
                    <= acceptance["maxDescriptorPeakBytes"].as_u64().unwrap()
            );
            assert_eq!(plan.outputs().len(), 4);
        }
    }
}
