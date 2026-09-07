//! Explicit GPU integration checks, excluded from ordinary workspace tests.

use mixture_core::SafetyLimits;
use mixture_wgpu::{
    BackendPreference, CheckerRequest, DoctorVerdict, GpuContext, GpuContextOptions,
};

fn options() -> GpuContextOptions {
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("vulkan") => BackendPreference::Vulkan,
        Ok("metal") => BackendPreference::Metal,
        Ok("dx12") => BackendPreference::Dx12,
        Err(std::env::VarError::NotPresent) | Ok("auto") => BackendPreference::Auto,
        other => panic!("invalid smoke backend {other:?}"),
    };
    GpuContextOptions {
        backend,
        software_adapter: std::env::var("MIXTURE_GPU_SOFTWARE").is_ok_and(|value| value == "1"),
        ..Default::default()
    }
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn checker_gpu_odd_dimensions_row_padding_repeated_render_and_drop() {
    let mut context = pollster::block_on(GpuContext::request(options())).expect("GPU context");
    let request = CheckerRequest {
        width: 65,
        height: 3,
    };
    let first = pollster::block_on(context.render_checker(request, &SafetyLimits::default()))
        .expect("odd-size checker");
    assert_eq!(first.pixels().len(), 65 * 3 * 4);
    assert_eq!(first.report().readback_bytes, 65 * 3 * 8);
    assert_eq!(first.report().padded_bytes_per_row, 768);
    assert_eq!(first.report().mapped_bytes, 768 * 3);
    assert_eq!(first.report().dispatch, [9, 1, 1]);
    // Fixed sentinels cover a partial workgroup, a padded row, and an uneven cell.
    for (x, y, value) in [
        (0, 0, 0),
        (8, 0, 0),
        (9, 0, 255),
        (64, 0, 255),
        (64, 1, 255),
        (0, 2, 255),
        (64, 2, 0),
    ] {
        let offset = (y * 65 + x) * 4;
        assert_eq!(
            &first.pixels()[offset..offset + 4],
            &[value, value, value, 255]
        );
    }
    let second = pollster::block_on(context.render_checker(request, &SafetyLimits::default()))
        .expect("repeat after unmap/release");
    assert_eq!(first.pixels(), second.pixels());
    let tiny = pollster::block_on(context.render_checker(
        CheckerRequest {
            width: 1,
            height: 1,
        },
        &SafetyLimits::default(),
    ))
    .unwrap();
    assert_eq!(tiny.pixels(), [0, 0, 0, 255]);
    let doctor = pollster::block_on(context.probe_checker());
    assert_eq!(doctor.verdict(), DoctorVerdict::Healthy);
    assert_eq!(doctor.compute_probe(), "passed");
    assert_eq!(doctor.readback_probe(), "passed");
    assert_eq!(
        context.report().verdict(),
        DoctorVerdict::Unverified,
        "acquisition snapshot is immutable"
    );
    drop(context);
    assert_eq!(
        first.pixels(),
        second.pixels(),
        "CPU output survives GPU context drop"
    );
}
