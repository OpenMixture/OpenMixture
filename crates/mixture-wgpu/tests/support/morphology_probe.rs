//! Finite support-set oracles through the production WGSL, not a CPU renderer.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};
use std::collections::BTreeSet;
type Support = BTreeSet<(u32, u32)>;
fn shift(s: &Support, size: [u32; 2], dx: i32, dy: i32) -> Support {
    s.iter()
        .map(|&(x, y)| {
            (
                ((x as i32 + dx).rem_euclid(size[0] as i32)) as u32,
                ((y as i32 + dy).rem_euclid(size[1] as i32)) as u32,
            )
        })
        .collect()
}
fn oracle(s: &Support, size: [u32; 2], operation: u32, axis: u32, radius: u32) -> Support {
    let mut result = s.clone();
    for k in -(radius as i32)..=radius as i32 {
        let moved = shift(
            s,
            size,
            if axis == 0 { k } else { 0 },
            if axis == 1 { k } else { 0 },
        );
        result = if operation == 0 {
            result.intersection(&moved).copied().collect()
        } else {
            result.union(&moved).copied().collect()
        };
    }
    result
}
fn field(s: &Support, size: [u32; 2]) -> Vec<f32> {
    (0..size[1])
        .flat_map(|y| (0..size[0]).map(move |x| if s.contains(&(x, y)) { 1. } else { 0. }))
        .collect()
}
fn check(actual: &[[f32; 4]], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (i, (got, want)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*got, [*want, 0., 0., 1.], "raw half pixel {i}");
    }
}
pub(super) fn run(context: &GpuContext) -> Value {
    let mut cases = 0;
    for size in [[1, 1], [1, 17], [17, 1], [19, 11]] {
        let [w, h] = size;
        let rect: Support = (h / 3..(h / 3 + 1).max(2 * h / 3))
            .flat_map(|y| (w / 3..(w / 3 + 1).max(2 * w / 3)).map(move |x| (x, y)))
            .collect();
        let wrapped: Support = [0, h - 1]
            .into_iter()
            .flat_map(|y| [0, w - 1].into_iter().map(move |x| (x, y)))
            .collect();
        let fields: Vec<Support> = vec![
            [(0, 0)].into(),
            [(w / 2, h / 2)].into(),
            (0..h).map(|y| (w / 2, y)).collect(),
            rect,
            wrapped,
        ];
        for operation in [0, 1] {
            for axis in [0, 1] {
                for radius in [0, 1, 16] {
                    for constant in [0., 0.5, 1.] {
                        let input = vec![constant; (w * h) as usize];
                        check(
                            &render(context, size, &input, [operation, axis, radius, 0]),
                            &input,
                        );
                        cases += 1;
                    }
                    for support in &fields {
                        let expected = oracle(support, size, operation, axis, radius);
                        check(
                            &render(
                                context,
                                size,
                                &field(support, size),
                                [operation, axis, radius, 0],
                            ),
                            &field(&expected, size),
                        );
                        cases += 1;
                        for (dx, dy) in [(1, 0), (0, 1), (1, 1)] {
                            check(
                                &render(
                                    context,
                                    size,
                                    &field(&shift(support, size, dx, dy), size),
                                    [operation, axis, radius, 0],
                                ),
                                &field(&shift(&expected, size, dx, dy), size),
                            );
                            cases += 1;
                        }
                    }
                }
            }
        }
        // Exact production-kernel compositions test axis commutation and radius monotonicity.
        for support in &fields {
            for operation in [0, 1] {
                let input = field(support, size);
                let x = render(context, size, &input, [operation, 0, 1, 0]);
                let y = render(context, size, &input, [operation, 1, 1, 0]);
                let xy = render(
                    context,
                    size,
                    &x.iter().map(|p| p[0]).collect::<Vec<_>>(),
                    [operation, 1, 1, 0],
                );
                let yx = render(
                    context,
                    size,
                    &y.iter().map(|p| p[0]).collect::<Vec<_>>(),
                    [operation, 0, 1, 0],
                );
                assert_eq!(xy, yx);
                cases += 1;
                for axis in [0, 1] {
                    let narrow = render(context, size, &input, [operation, axis, 1, 0]);
                    let wide = render(context, size, &input, [operation, axis, 16, 0]);
                    for ((n, w), original) in narrow.iter().zip(&wide).zip(&input) {
                        if operation == 0 {
                            assert!(w[0] <= n[0] && n[0] <= *original);
                        } else {
                            assert!(w[0] >= n[0] && n[0] >= *original);
                        }
                    }
                    cases += 1;
                }
            }
        }
    }
    // Exactly representable fractional extrema and saturation are checked before RGBA8 conversion.
    for (parameters, expected) in [
        ([0, 0, 0, 0], vec![0., 0.25, 0.5, 1.]),
        ([0, 0, 1, 0], vec![0., 0., 0.25, 0.]),
        ([1, 0, 1, 0], vec![1., 0.5, 1., 1.]),
    ] {
        check(
            &render(context, [4, 1], &[-1., 0.25, 0.5, 2.], parameters),
            &expected,
        );
        cases += 1;
    }
    json!({"ok":true,"adapter":context.report().adapter(),"cases":cases,"comparison":"exact raw rgba16float","sizes":[[1,1],[1,17],[17,1],[19,11]],"radii":[0,1,16]})
}

fn render(
    context: &GpuContext,
    size: [u32; 2],
    heights: &[f32],
    parameters: [u32; 4],
) -> Vec<[f32; 4]> {
    let device = context.device();
    let queue = context.queue();
    let extent = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let texture = |label, usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage,
            view_formats: &[],
        })
    };
    let input = texture(
        "literal scalar probe",
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let output = texture(
        "morphology probe result",
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let data: Vec<_> = heights
        .iter()
        .flat_map(|h| {
            [*h, 0., 0., 1.]
                .into_iter()
                .flat_map(|v| half::f16::from_f32(v).to_bits().to_le_bytes())
        })
        .collect();
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &input,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size[0] * 8),
            rows_per_image: Some(size[1]),
        },
        extent,
    );
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("probe parameters"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    uniform
        .slice(..)
        .get_mapped_range_mut()
        .unwrap()
        .copy_from_slice(
            &parameters
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
    uniform.unmap();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production scalar-morphology"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("../../shaders/precision.wgsl"),
                "\n",
                include_str!("../../shaders/nodes/scalar-morphology.wgsl")
            )
            .into(),
        ),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("morphology probe"),
        layout: None,
        module: &module,
        entry_point: Some("scalar_morphology"),
        compilation_options: Default::default(),
        cache: None,
    });
    let input_view = input.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("probe bindings"),
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
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("probe readback"),
        size: u64::from(256 * size[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(size[0].div_ceil(8), size[1].div_ceil(8), 1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &output,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(size[1]),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .unwrap();
    receiver.recv().unwrap().unwrap();
    let result = {
        let view = buffer.slice(..).get_mapped_range().unwrap();
        view.as_chunks::<256>()
            .0
            .iter()
            .flat_map(|row| {
                row[..size[0] as usize * 8]
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|p| {
                        let mut rgba = [0.; 4];
                        for (channel, bytes) in p.as_chunks::<2>().0.iter().enumerate() {
                            let value =
                                half::f16::from_bits(u16::from_le_bytes([bytes[0], bytes[1]]))
                                    .to_f32();
                            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
                            rgba[channel] = value;
                        }
                        rgba
                    })
            })
            .collect()
    };
    buffer.unmap();
    result
}
