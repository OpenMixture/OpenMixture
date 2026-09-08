use crate::{Result, failure, require};
use mixture_core::{
    CompileRequest, InputSource, MaterialDocument, OutputChannel, RenderPlan, compile,
};
use serde_json::{Value, json};

// This asset belongs to this independent application and ships with its source.
pub const INPUT: &[u8] = include_bytes!("../input.mix");
pub const CHANNELS: [OutputChannel; 4] = [
    OutputChannel::BaseColor,
    OutputChannel::Normal,
    OutputChannel::Roughness,
    OutputChannel::Height,
];

pub fn request(size: [u32; 2]) -> CompileRequest {
    CompileRequest {
        size,
        outputs: CHANNELS.to_vec(),
        ..Default::default()
    }
}

pub fn prepare(bytes: &[u8], request: &CompileRequest) -> Result<RenderPlan> {
    let document = MaterialDocument::decode(bytes, &request.limits)?;
    let validated = document.into_validated(&request.limits)?;
    Ok(compile(&validated, request)?)
}

pub fn own_plan(repeat: u32) -> Result<RenderPlan> {
    let mut request = request([65, 3]);
    request.overrides.insert("repeat".into(), json!(repeat));
    prepare(INPUT, &request)
}

pub fn check() -> Result<Value> {
    let plan = own_plan(16)?;
    let repeated = own_plan(16)?;
    require(
        plan.hash() == repeated.hash(),
        "identical requests changed hash",
    )?;
    require(
        plan.hash() != own_plan(4)?.hash(),
        "override did not change plan",
    )?;
    require(
        plan.passes().len() == 4,
        "unexpected four-channel pass count",
    )?;
    require(
        plan.outputs()
            .iter()
            .map(|output| output.channel)
            .eq(CHANNELS),
        "output channels are not in canonical order",
    )?;
    require(
        matches!(plan.outputs()[0].input, InputSource::Connected { .. })
            && matches!(plan.outputs()[1].input, InputSource::Default { .. })
            && matches!(plan.outputs()[2].input, InputSource::Connected { .. })
            && matches!(plan.outputs()[3].input, InputSource::Default { .. }),
        "connected/default provenance changed",
    )?;
    let mut subset = request([65, 3]);
    subset.outputs = vec![OutputChannel::Roughness];
    let sliced = prepare(INPUT, &subset)?;
    require(
        sliced.passes().len() == 1,
        "unused checker branch was not sliced",
    )?;
    let invalid = match own_plan(0) {
        Ok(_) => return Err("invalid exposed override was accepted".into()),
        Err(error) => error,
    };
    let invalid = failure(invalid.as_ref());
    let diagnostic = &invalid["diagnostics"][0];
    require(
        diagnostic["code"] == "MIX_PARAMETER_INVALID_VALUE"
            && diagnostic["stage"] == "compile"
            && diagnostic["nodeId"] == "pattern"
            && diagnostic["parameterId"] == "cellsX"
            && diagnostic["evidence"]["publicId"] == "repeat",
        "invalid override lost public parameter context",
    )?;
    Ok(json!({
        "schemaVersion": 1, "ok": true, "mode": "check", "gpuExecuted": false,
        "source": "consumer-owned input.mix", "planHash": plan.hash(),
        "size": plan.size(), "passes": plan.passes().len(), "outputs": plan.outputs(),
        "slicedPasses": sliced.passes().len(), "invalidOverride": invalid,
        "packagedCratesValidated": false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mixture_core::{DiagnosticCode, SafetyLimits, Stage};

    #[test]
    fn consumer_overrides_channels_hashes_and_diagnostics_need_no_gpu() {
        assert_eq!(check().unwrap()["gpuExecuted"], false);
    }

    #[test]
    fn source_bytes_round_trip_without_external_assets_or_semantic_repair() {
        let limits = SafetyLimits::default();
        let original = MaterialDocument::decode(INPUT, &limits).unwrap();
        let encoded = original.to_json().unwrap();
        let decoded = MaterialDocument::decode(&encoded, &limits).unwrap();
        assert_eq!(encoded, decoded.to_json().unwrap());
        assert_eq!(
            prepare(INPUT, &request([65, 3])).unwrap().hash(),
            prepare(&encoded, &request([65, 3])).unwrap().hash()
        );
    }

    #[test]
    fn malformed_source_and_missing_port_keep_typed_diagnostics() {
        let limits = SafetyLimits::default();
        let error = MaterialDocument::decode(b"{", &limits).unwrap_err();
        assert!(
            error
                .report()
                .diagnostics()
                .iter()
                .all(|d| d.stage == Stage::Parse)
        );
        let mut source = MaterialDocument::decode(INPUT, &limits).unwrap();
        source.edges[0].to.port_id = "missing".into();
        let error = source.into_validated(&limits).unwrap_err();
        assert!(error.report().diagnostics().iter().any(|d| {
            d.code == DiagnosticCode::PortUnknown && d.port_id.as_deref() == Some("missing")
        }));
    }

    #[test]
    fn output_order_normalizes_but_duplicates_and_exceeded_limits_are_rejected() {
        let ordered = request([65, 3]);
        let mut reordered = ordered.clone();
        reordered.outputs.reverse();
        assert_eq!(
            prepare(INPUT, &ordered).unwrap().hash(),
            prepare(INPUT, &reordered).unwrap().hash()
        );
        reordered.outputs.push(OutputChannel::BaseColor);
        let duplicate = prepare(INPUT, &reordered).unwrap_err();
        let report = failure(duplicate.as_ref());
        assert_eq!(
            report["diagnostics"][0]["code"],
            "MIX_COMPILE_INVALID_REQUEST"
        );
        assert_eq!(report["diagnostics"][0]["evidence"]["channel"], "baseColor");
        let mut limited = request([65, 3]);
        limited.limits.output_dimension = 64;
        let error = prepare(INPUT, &limited).unwrap_err();
        let report = failure(error.as_ref());
        assert_eq!(
            report["diagnostics"][0]["code"],
            "MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED"
        );
        assert_eq!(report["diagnostics"][0]["evidence"]["configured"], 64);
        assert_eq!(report["diagnostics"][0]["evidence"]["observed"], 65);
    }
}
