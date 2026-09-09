//! Default tests never initialize a GPU. The ignored test is run by gpu-smoke.

use mixture_core::error::{DiagnosticCode, Stage};
use mixture_wgpu::{BackendPreference, DoctorVerdict, GpuContext, GpuContextOptions};

#[test]
fn context_disabled_policy_fails_without_a_gpu_and_preserves_policy() {
    let options = GpuContextOptions {
        backend: BackendPreference::None,
        software_adapter: true,
        ..Default::default()
    };
    let error =
        pollster::block_on(GpuContext::request(options)).expect_err("no backend is permitted");
    assert_eq!(
        error.diagnostic().code,
        DiagnosticCode::GpuAdapterUnavailable
    );
    assert_eq!(error.diagnostic().stage, Stage::GpuAdapter);
    assert_eq!(error.report().verdict(), DoctorVerdict::Unhealthy);
    assert_eq!(error.report().requested().options, options);
    let report = serde_json::to_value(error.report()).unwrap();
    assert_eq!(report["ok"], false);
    assert_eq!(
        report["requested"]["effectiveBackends"],
        serde_json::json!([])
    );
    assert!(report["adapter"].is_null());
    assert!(report["device"].is_null());
    assert_eq!(report["computeProbe"], "notRun");
    assert_eq!(report["readbackProbe"], "notRun");
}

#[test]
#[ignore = "requires a native GPU or configured software Vulkan adapter; cargo xtask gpu-smoke"]
fn context_gpu_smoke_owns_independent_contexts() {
    // These variables belong exclusively to the explicit smoke-test harness.
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("auto") => BackendPreference::Auto,
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("metal") => BackendPreference::Metal,
        Ok("dx12") => BackendPreference::Dx12,
        other => panic!("invalid smoke backend: {other:?}"),
    };
    let software_adapter = match std::env::var("MIXTURE_GPU_SOFTWARE").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("0") => false,
        Ok("1") => true,
        other => panic!("invalid smoke software policy: {other:?}"),
    };
    let options = GpuContextOptions {
        backend,
        software_adapter,
        ..Default::default()
    };
    let first =
        pollster::block_on(GpuContext::request(options)).expect("first context acquisition");
    let second =
        pollster::block_on(GpuContext::request(options)).expect("independent context acquisition");
    assert!(first.device_loss().is_none());
    assert!(second.device_loss().is_none());
    // Handle equality uses instance-local IDs, so it cannot prove independence.
    // Destroying one device must signal loss only on that context.
    first.device().destroy();
    first
        .device()
        .poll(wgpu::PollType::Poll)
        .expect("deliver destruction callback");
    assert_eq!(
        first
            .device_loss()
            .expect("first device destruction")
            .reason,
        mixture_wgpu::DeviceLossReason::Destroyed
    );
    drop(first);
    second
        .device()
        .poll(wgpu::PollType::Poll)
        .expect("second device remains live");
    assert!(second.device_loss().is_none());
    assert_eq!(second.report().verdict(), DoctorVerdict::Unverified);
    assert!(second.report().diagnostics().is_ok());
    assert_eq!(
        second.adapter().get_info().name,
        second.report().adapter().unwrap().name
    );
    assert_eq!(
        second.device().features().bits(),
        wgpu::Features::empty().bits()
    );
    assert!(
        second
            .report()
            .requested()
            .required_limits
            .check_limits(&second.adapter().limits())
    );
    let _instance = second.instance();
    let json = serde_json::to_value(second.report()).unwrap();
    assert_eq!(json["computeProbe"], "notRun");
    assert_eq!(json["readbackProbe"], "notRun");
    if software_adapter {
        assert_eq!(json["adapter"]["deviceType"], "Cpu");
    }
}
