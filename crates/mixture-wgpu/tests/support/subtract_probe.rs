//! Literal exact subtraction differences through production WGSL before RGBA8 conversion.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};
pub(super) fn run(context: &GpuContext) -> Value {
    let a = [0., 1., 0., 1., 0.5, 0.5, 0.25, -1., 2., 0.5];
    let b = [0., 0., 1., 1., 0.5, 0.5 - 1.0 / 4096.0, 0.75, 2., -1., 0.];
    let expected = [0., 1., 0., 0., 0., 1.0 / 4096.0, 0., 0., 1., 0.5];
    let mut rows = Vec::new();
    for size in [[10, 1], [1, 10], [10, 3]] {
        let repeats = (size[0] * size[1] / 10) as usize;
        let actual = render(
            context,
            size,
            &a.repeat(repeats),
            &b.repeat(repeats),
            [0; 4],
        );
        for (got, want) in actual.iter().zip(expected.repeat(repeats)) {
            assert_eq!(*got, [want, 0., 0., 1.]);
        }
        rows.push(json!({"size":size,"actual":actual}));
    }
    // Shifted literal periodic fields retain exact differences at every texel.
    for offset in 0..10 {
        let mut left = a;
        let mut right = b;
        let mut want = expected;
        left.rotate_left(offset);
        right.rotate_left(offset);
        want.rotate_left(offset);
        let actual = render(context, [10, 1], &left, &right, [0; 4]);
        for (got, want) in actual.iter().zip(want) {
            assert_eq!(*got, [want, 0., 0., 1.]);
        }
        rows.push(json!({"offset":offset,"actual":actual}));
    }
    json!({"ok":true,"adapter":context.report().adapter(),"comparison":"exact raw rgba16float","cases":rows})
}

fn render(
    context: &GpuContext,
    size: [u32; 2],
    heights: &[f32],
    second: &[f32],
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
        "subtraction probe result",
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
    let input_b = texture(
        "second literal scalar",
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let data: Vec<_> = second
        .iter()
        .flat_map(|h| {
            [*h, 0., 0., 1.]
                .into_iter()
                .flat_map(|v| half::f16::from_f32(v).to_bits().to_le_bytes())
        })
        .collect();
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &input_b,
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
        label: Some("production scalar-subtract"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("../../shaders/precision.wgsl"),
                "\n",
                include_str!("../../shaders/nodes/scalar-subtract.wgsl")
            )
            .into(),
        ),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("subtraction probe"),
        layout: None,
        module: &module,
        entry_point: Some("scalar_subtract"),
        compilation_options: Default::default(),
        cache: None,
    });
    let input_view = input.create_view(&Default::default());
    let second_view = input_b.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("probe bindings"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&second_view),
            },
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
