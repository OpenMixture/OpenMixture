//! Metamorphic checks on the complete material through the public renderer.
use mixture_core::{CompileRequest, OutputChannel, ValidatedDocument, compile};
use mixture_wgpu::{RenderOutput, Renderer};
use serde_json::{Value, json};

pub(super) fn check(
    gpu: &mut Renderer,
    document: &ValidatedDocument,
    request: &CompileRequest,
    baseline: &RenderOutput,
    contract: &Value,
) -> Vec<Value> {
    assert_eq!(json!(request.size), contract["causality"]["size"]);
    let mut cases = Vec::new();
    for name in ["paintColor", "substrateColor", "rustColor"] {
        cases.push((
            name,
            contract["causality"]["colorReplacement"].clone(),
            Some(OutputChannel::BaseColor),
        ));
    }
    for name in ["paintRoughness", "substrateRoughness", "rustRoughness"] {
        cases.push((name, json!(0.), Some(OutputChannel::Roughness)));
    }
    for value in contract["causality"]["normalStrengths"].as_array().unwrap() {
        cases.push(("normalStrength", value.clone(), Some(OutputChannel::Normal)));
    }
    for name in ["macroSeed", "detailSeed"] {
        cases.push((
            name,
            json!(request.overrides[name].as_u64().unwrap() + 1),
            None,
        ));
    }
    cases.into_iter().map(|(name, value, affected)| {
        let mut variant = request.clone();
        variant.overrides.insert(name.to_owned(), value.clone());
        let plan = compile(document, &variant).unwrap();
        let output = pollster::block_on(gpu.render(&plan)).unwrap();
        let repeat = pollster::block_on(gpu.render(&plan)).unwrap();
        let mut changed = Vec::new();
        for channel in output.channels() {
            let before = baseline.channels().iter().find(|c| c.channel == channel.channel).unwrap();
            let again = repeat.channels().iter().find(|c| c.channel == channel.channel).unwrap();
            assert_eq!(channel.pixels(), again.pixels(), "repeat {name}");
            if channel.pixels() != before.pixels() {
                if let Some(expected) = affected {
                    assert_eq!(channel.channel, expected, "{name} changed an unrelated channel");
                }
                changed.push(channel.channel.as_str());
            }
        }
        if request.overrides[name] != value {
            assert!(!changed.is_empty(), "{name} did not affect any delivered pixel");
        } else {
            assert!(changed.is_empty(), "unchanged control changed pixels");
        }
        json!({"control":name,"value":value,"changedChannels":changed,"repeatExact":true,"isolationPassed":true})
    }).collect()
}
