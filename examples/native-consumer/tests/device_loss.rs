//! Public Rust consumer of real device destruction, with no producer-private imports.
use mixture_core::{
    CompileRequest, DiagnosticCode, MaterialDocument, OutputChannel, SafetyLimits, Stage, compile,
};
use mixture_wgpu::{
    BackendPreference, DeviceLossReason, GpuContext, GpuContextOptions, GpuFailureReason, Renderer,
};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs::File,
    io::{Seek, Write},
    path::PathBuf,
};

fn save(file: &mut File, report: &Value) {
    file.rewind().unwrap();
    file.set_len(0).unwrap();
    serde_json::to_writer_pretty(&mut *file, report).unwrap();
    file.flush().unwrap();
}

fn options() -> GpuContextOptions {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("metal") => BackendPreference::Metal,
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("dx12") => BackendPreference::Dx12,
        Ok("auto") => BackendPreference::Auto,
        other => panic!("explicit native backend required: {other:?}"),
    };
    let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        other => panic!("explicit software policy required: {other:?}"),
    };
    GpuContextOptions {
        backend,
        software_adapter,
        ..Default::default()
    }
}

#[test]
#[ignore = "requires explicit GPU policy and a new evidence file; cargo xtask gpu-smoke"]
fn device_loss_contract() {
    let path = PathBuf::from(
        std::env::var_os("MIXTURE_CONSUMER_DEVICE_LOSS_EVIDENCE")
            .expect("new absolute evidence filename"),
    );
    assert!(path.is_absolute());
    let mut file = File::create_new(path).expect("evidence file must not already exist");
    let mut report = json!({"schemaVersion":1,"ok":false,"completed":false,"gpuExecuted":false,"context":null,"cases":[]});
    save(&mut file, &report);
    let options = options();
    let document =
        MaterialDocument::decode(include_bytes!("../input.mix"), &SafetyLimits::default())
            .unwrap()
            .into_validated(&SafetyLimits::default())
            .unwrap();
    let plan = compile(
        &document,
        &CompileRequest {
            size: [33, 3],
            outputs: vec![OutputChannel::Roughness, OutputChannel::BaseColor],
            ..Default::default()
        },
    )
    .unwrap();
    // This independent live context predates both destroyed contexts. It must
    // continue operating; acquiring it is an explicit test action, not recovery.
    let mut live = Renderer::new(pollster::block_on(GpuContext::request(options)).unwrap());
    report["context"] = serde_json::to_value(live.context().report()).unwrap();
    let reference = pollster::block_on(live.render(&plan)).unwrap();
    assert_eq!(reference.channels().len(), 2);
    assert!(
        reference.channels()[1]
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [64, 64, 64, 255])
    );

    for warm_cache in [false, true] {
        let mut doomed = Renderer::new(pollster::block_on(GpuContext::request(options)).unwrap());
        let acquired = serde_json::to_value(doomed.context().report()).unwrap();
        if warm_cache {
            let output = pollster::block_on(doomed.render(&plan)).unwrap();
            assert_eq!(
                output.channels()[0].pixels(),
                reference.channels()[0].pixels()
            );
            assert!(doomed.cached_pipeline_count() > 0);
        }
        let cache_before = doomed.cached_pipeline_count();
        assert!(doomed.context().device_loss().is_none());
        // Use the retained public raw-device escape hatch. The subsequent render
        // must deliver the native callback itself, before new GPU allocations.
        doomed.context().device().destroy();
        let error = pollster::block_on(doomed.render(&plan)).unwrap_err();
        assert_eq!(error.reason(), GpuFailureReason::DeviceLost);
        assert_eq!(error.diagnostic().code, DiagnosticCode::GpuDeviceLost);
        assert_eq!(error.diagnostic().stage, Stage::GpuExecution);
        let loss = error.device_loss().unwrap().clone();
        assert_eq!(loss.reason, DeviceLossReason::Destroyed);
        assert_eq!(doomed.context().device_loss(), Some(&loss));
        assert_eq!(doomed.cached_pipeline_count(), 0);
        assert_eq!(
            serde_json::to_value(doomed.context().report()).unwrap(),
            acquired
        );
        let adapter = error.adapter().unwrap();
        assert_eq!(serde_json::to_value(adapter).unwrap(), acquired["adapter"]);
        if options.software_adapter {
            assert_eq!(adapter.device_type, "Cpu");
        }
        if let Ok(expected) = std::env::var("MIXTURE_GPU_EXPECT_ADAPTER") {
            assert!(adapter.name.contains(&expected));
        }
        let allocation = error.allocations().unwrap();
        assert_eq!(allocation.cumulative_bytes, 0);
        assert_eq!(allocation.live_bytes, 0);
        assert_eq!(allocation.released_bytes, 0);
        let diagnostic = serde_json::to_value(error.diagnostic()).unwrap();
        assert_eq!(diagnostic["evidence"]["planHash"], plan.hash().as_str());
        assert_eq!(diagnostic["evidence"]["deviceLost"], true);
        assert_eq!(diagnostic["evidence"]["deviceLostReason"], "destroyed");
        assert_eq!(
            diagnostic["evidence"]["allocationLiveBytes"].as_u64(),
            Some(0)
        );
        for _ in 0..2 {
            let again = pollster::block_on(doomed.render(&plan)).unwrap_err();
            assert_eq!(
                serde_json::to_value(again.diagnostic()).unwrap(),
                diagnostic
            );
            assert_eq!(again.device_loss(), Some(&loss));
            assert_eq!(doomed.cached_pipeline_count(), 0);
        }
        drop(doomed);
        assert_eq!(error.device_loss(), Some(&loss));
        assert!(error.diagnostic().source().is_some());
        let continued = pollster::block_on(live.render(&plan)).unwrap();
        assert!(live.context().device_loss().is_none());
        assert_eq!(
            continued.channels()[0].pixels(),
            reference.channels()[0].pixels()
        );
        assert_eq!(
            continued.channels()[1].pixels(),
            reference.channels()[1].pixels()
        );
        assert_eq!(continued.report().allocations.live_bytes, 0);
        report["cases"].as_array_mut().unwrap().push(json!({
            "warmCache": warm_cache, "cacheBeforeLoss": cache_before, "cacheAfterLoss":0,
            "repeatedFailures":2, "reason":error.reason(), "diagnostic":diagnostic,
            "deviceLoss":loss, "allocations":allocation, "adapter":adapter,
            "independentExecution":continued.report(),
        }));
        save(&mut file, &report);
    }
    report["ok"] = json!(true);
    report["completed"] = json!(true);
    report["gpuExecuted"] = json!(true);
    save(&mut file, &report);
}
