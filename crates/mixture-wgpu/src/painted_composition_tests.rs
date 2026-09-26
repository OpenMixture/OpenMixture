//! Scalar composition oracles and production-normal replay from captured half height.
use super::*;
use mixture_core::{CompileRequest, MaterialDocument, compile, plan::KernelId};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn half(value: f32) -> f32 {
    half::f16::from_f32(value).to_f32()
}
fn mix(a: f32, b: f32, t: f32) -> f32 {
    half(if t == 0. {
        a
    } else if t == 1. {
        b
    } else {
        a + (b - a) * t
    })
}

fn request(contract: &Value, preset: &Value) -> CompileRequest {
    let mut controls = contract["defaults"].clone();
    for (key, value) in preset["controls"].as_object().unwrap() {
        controls[key] = value.clone();
    }
    let mut overrides = BTreeMap::new();
    for key in [
        "macroSeed",
        "detailSeed",
        "exposureScale",
        "rustAmount",
        "rustFill",
        "paintColor",
        "substrateColor",
        "rustColor",
        "paintRoughness",
        "substrateRoughness",
        "rustRoughness",
        "normalStrength",
    ] {
        overrides.insert(key.into(), controls[key].clone());
    }
    let amount = controls["exposureAmount"].as_f64().unwrap();
    let endpoint = amount == 0. || amount == 1.;
    let low = if endpoint { 0. } else { 0.8 * (1. - amount) };
    for (key, value) in [
        ("exposureInputMin", low),
        ("exposureInputMax", if endpoint { 1. } else { low + 0.2 }),
        ("exposureOutputMin", if endpoint { amount } else { 0. }),
        ("exposureOutputMax", if endpoint { amount } else { 1. }),
        ("detailMin", 1. - controls["detailAmount"].as_f64().unwrap()),
        (
            "paintHeight",
            0.2 + controls["paintThickness"].as_f64().unwrap(),
        ),
        ("rustHeight", 0.2 + controls["rustRelief"].as_f64().unwrap()),
    ] {
        overrides.insert(key.into(), json!(value));
    }
    let width = controls["edgeWidth"].as_u64().unwrap();
    for (key, size) in [("radiusX", 257), ("radiusY", 129)] {
        overrides.insert(
            key.into(),
            json!(if width == 0 {
                0
            } else {
                ((width * size + 512) / 1024).max(1)
            }),
        );
    }
    CompileRequest {
        size: [257, 129],
        outputs: vec![
            OutputChannel::BaseColor,
            OutputChannel::Normal,
            OutputChannel::Roughness,
            OutputChannel::Metallic,
            OutputChannel::Height,
        ],
        overrides,
        ..Default::default()
    }
}

fn capture(context: &GpuContext, source: &[u8], request: &CompileRequest) -> (Vec<u8>, Vec<u8>) {
    let document = MaterialDocument::decode(source, &request.limits)
        .unwrap()
        .into_validated(&request.limits)
        .unwrap();
    let plan = compile(&document, request).unwrap();
    assert_eq!(plan.passes().len(), 23);
    let kernels: Vec<_> = plan.passes().iter().map(|p| &p.kernel).collect();
    let slots: Vec<_> = plan
        .allocation()
        .resource_slots()
        .iter()
        .map(|s| s.index() as usize)
        .collect();
    let outputs: Vec<_> = [OutputChannel::Height, OutputChannel::Normal]
        .into_iter()
        .map(|channel| {
            (
                plan.outputs()
                    .iter()
                    .find(|o| o.channel == channel)
                    .unwrap()
                    .resource
                    .index() as usize,
                ReadbackFormat::RawHalf,
            )
        })
        .collect();
    let result = pollster::block_on(execute_prepared(
        context,
        &mut PipelineCache::default(),
        plan.size(),
        &kernels,
        &outputs,
        &[],
        &slots,
    ))
    .unwrap();
    assert_eq!(result.allocations.live_bytes, 0);
    let mut pixels = result.pixels.into_iter();
    (pixels.next().unwrap(), pixels.next().unwrap())
}

// A probe of the existing production kernel, never a CPU normal implementation.
fn replay_normal(context: &GpuContext, height: &[u8], size: [u32; 2], strength: f32) -> Vec<u8> {
    let device = context.device();
    let queue = context.queue();
    assert_eq!(height.len(), size[0] as usize * size[1] as usize * 8);
    let extent = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let texture = |usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("captured material height replay"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage,
            view_formats: &[],
        })
    };
    let input = texture(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING);
    let output = texture(wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC);
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &input,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        height,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size[0] * 8),
            rows_per_image: Some(size[1]),
        },
        extent,
    );
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("replay strength"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    uniform
        .slice(..)
        .get_mapped_range_mut()
        .unwrap()
        .copy_from_slice(
            &[strength, 0., 0., 0.]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
    uniform.unmap();
    let (shader, entry) = crate::kernels::shader(KernelId::HeightToNormal);
    let pipeline =
        pollster::block_on(crate::kernels::create_pipeline(device, shader, entry)).unwrap();
    let input_view = input.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("normal replay"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&output_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&input_view),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(size[0].div_ceil(8), size[1].div_ceil(8), 1);
    }
    pollster::block_on(resources::submit(
        device,
        queue,
        encoder,
        Stage::GpuExecution,
    ))
    .unwrap();
    let mut allocations = crate::allocations::Allocations::default();
    let result = pollster::block_on(resources::read_texture(
        device,
        queue,
        &output,
        ReadbackLayout::new(size[0], size[1]).unwrap(),
        ReadbackFormat::RawHalf,
        &mut allocations,
    ))
    .unwrap();
    uniform.destroy();
    input.destroy();
    output.destroy();
    result
}

#[test]
#[ignore = "requires GPU; cargo xtask gpu-smoke"]
fn graph_gpu_painted_composition_and_final_height_normal_replay() {
    let source = std::fs::read(super::painted_mask_tests::SOURCE).unwrap();
    let contract: Value =
        serde_json::from_slice(&std::fs::read(super::painted_mask_tests::CONTRACT).unwrap())
            .unwrap();
    let context = pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
    let mut cases = Vec::new();
    for preset in contract["cases"].as_array().unwrap() {
        let request = request(&contract, preset);
        let fields = super::painted_mask_tests::scalar_fields(
            &context,
            &source,
            request.overrides.clone(),
            &[
                "exposure",
                "rustMask",
                "coatingHeight",
                "height",
                "coatingRoughness",
                "roughness",
                "metallic",
            ],
        );
        let [wear, rust, coat_height, height, coat_rough, rough, metal] = fields.as_slice() else {
            panic!("fields")
        };
        let parameter = |key: &str| request.overrides[key].as_f64().unwrap() as f32;
        for i in 0..wear.len() {
            assert_eq!(
                coat_height[i],
                mix(parameter("paintHeight"), 0.2, wear[i]),
                "coating height {i}"
            );
            assert_eq!(
                coat_rough[i],
                mix(
                    parameter("paintRoughness"),
                    parameter("substrateRoughness"),
                    wear[i]
                ),
                "coating roughness {i}"
            );
            assert_eq!(
                height[i],
                mix(coat_height[i], half(parameter("rustHeight")), rust[i]),
                "height {i}"
            );
            assert_eq!(
                rough[i],
                mix(coat_rough[i], half(parameter("rustRoughness")), rust[i]),
                "roughness {i}"
            );
            assert_eq!(metal[i], mix(wear[i], 0., rust[i]), "metallic {i}");
            assert!(height[i] >= half(0.2) && height[i] <= half(parameter("paintHeight")));
        }
        let (captured_height, normal) = capture(&context, &source, &request);
        assert_eq!(
            captured_height
                .as_chunks::<8>()
                .0
                .iter()
                .map(|p| half::f16::from_bits(u16::from_le_bytes([p[0], p[1]])).to_f32())
                .collect::<Vec<_>>(),
            *height
        );
        for value in contract["causality"]["normalStrengths"].as_array().unwrap() {
            let mut variant = request.clone();
            variant
                .overrides
                .insert("normalStrength".into(), value.clone());
            let (same_height, actual) = capture(&context, &source, &variant);
            assert_eq!(
                same_height, captured_height,
                "normal strength changed stored height"
            );
            assert_eq!(
                actual,
                replay_normal(
                    &context,
                    &captured_height,
                    request.size,
                    value.as_f64().unwrap() as f32
                ),
                "normal replay {}",
                preset["id"]
            );
        }
        assert_eq!(
            normal,
            replay_normal(
                &context,
                &captured_height,
                request.size,
                parameter("normalStrength")
            )
        );
        cases.push(preset["id"].clone());
    }
    assert_eq!(cases.len(), 7);
    eprintln!(
        "painted composition: {}",
        json!({"ok":true,"materialAccepted":false,"cases":cases,"normalStrengths":[0,0.5,1],"rawHalfExact":true,"adapter":context.report().adapter()})
    );
}
