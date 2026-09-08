use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, RenderPlan, compile};
use mixture_wgpu::{GpuContext, GpuContextOptions, RenderOutput, Renderer};

// Compile-check the consumer's complete public render and result-handling path.
// The read-only M3 audit intentionally does not execute a GPU workload.
#[allow(dead_code)]
async fn render(plan: &RenderPlan) -> Result<RenderOutput, Box<dyn std::error::Error>> {
    let context = GpuContext::request(GpuContextOptions::default()).await?;
    let mut renderer = Renderer::new(context);
    let output = renderer.render(plan).await?;
    assert_eq!(&output.report().plan_hash, plan.hash());
    let metrics = serde_json::to_value(output.report())?;
    assert_eq!(metrics["allocations"]["liveBytes"], 0);
    drop(renderer);
    for channel in output.channels() {
        assert_eq!(
            channel.pixels().len(),
            (channel.size[0] * channel.size[1] * 4) as usize
        );
        let _metadata = (
            channel.channel,
            channel.kind,
            &channel.source,
            channel.encoding,
        );
    }
    Ok(output)
}

#[allow(dead_code)]
fn check_worker_types(renderer: &mut Renderer, plan: &RenderPlan) {
    fn require_send<T: Send>(_: T) {}
    fn require_send_type<T: Send>() {}
    require_send_type::<Renderer>();
    require_send_type::<RenderPlan>();
    require_send_type::<RenderOutput>();
    require_send(renderer.render(plan));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = CompileRequest::default();
    request.size = [65, 3];
    request.outputs = vec![OutputChannel::BaseColor, OutputChannel::Roughness];
    request
        .overrides
        .insert("repeat".into(), serde_json::json!(16));
    let bytes = include_bytes!("../input.mix");
    let document = MaterialDocument::decode(bytes, &request.limits)?;
    assert!(document.validate(&request.limits).is_ok());
    let validated = document.into_validated(&request.limits)?;
    let plan = compile(&validated, &request)?;
    assert_eq!(plan.outputs().len(), 2);
    assert_eq!(plan.passes().len(), 2);
    assert_eq!(plan.hash(), compile(&validated, &request)?.hash());
    request
        .overrides
        .insert("repeat".into(), serde_json::json!(0));
    let diagnostic = compile(&validated, &request).expect_err("zero cells must be rejected");
    assert!(!diagnostic.report().is_ok());
    println!(
        "{}",
        serde_json::json!({
            "schemaVersion": 1,
            "kind": "m3-native-public-api-probe",
            "ok": true,
            "gpuPathValidation": "compile-only",
            "nativeWorkerTypesValidation": "compile-only",
            "packagedCratesValidated": false,
            "planHash": plan.hash(),
            "channels": plan.outputs().len(),
            "passes": plan.passes().len(),
            "gpuExecuted": false,
            "invalidOverride": diagnostic.report(),
        })
    );
    Ok(())
}
