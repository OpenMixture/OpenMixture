//! Hand-specified scalar ramps exercise the real WGSL, without a CPU normal renderer.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};

pub(super) fn run(context: &GpuContext) -> Value {
    let horizontal: Vec<f32> = [0.0, 0.25, 0.5, 0.75].repeat(4);
    let vertical: Vec<f32> = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]
        .into_iter()
        .flat_map(|h| [h; 4])
        .collect();
    // These literal expected directions follow the analytic ramp slope, not an
    // implementation evaluating arbitrary image neighbors on the CPU.
    let horizontal_expected = [
        [185, 128, 242, 255],
        [70, 128, 242, 255],
        [70, 128, 242, 255],
        [185, 128, 242, 255],
    ]
    .repeat(4);
    let vertical_expected: Vec<[u8; 4]> = [
        [128, 21, 198, 255],
        [128, 185, 242, 255],
        [128, 185, 242, 255],
        [128, 185, 242, 255],
        [128, 185, 242, 255],
        [128, 185, 242, 255],
        [128, 185, 242, 255],
        [128, 21, 198, 255],
    ]
    .into_iter()
    .flat_map(|p| [p; 4])
    .collect();
    let mut evidence = Vec::new();
    for (name, size, heights, strength, expected) in [
        (
            "horizontal-ramp-wrap",
            [4, 4],
            horizontal.clone(),
            0.5,
            horizontal_expected,
        ),
        (
            "vertical-ramp-positive-y-up-and-uv-scaling",
            [4, 8],
            vertical,
            0.5,
            vertical_expected,
        ),
        (
            "zero-strength",
            [4, 4],
            horizontal,
            0.0,
            vec![[128, 128, 255, 255]; 16],
        ),
        (
            "one-texel-axis-flat",
            [1, 1],
            vec![0.75],
            8.0,
            vec![[128, 128, 255, 255]],
        ),
    ] {
        let actual = render(context, size, &heights, strength);
        assert_eq!(actual.len(), expected.len());
        for (index, (a, b)) in actual.iter().zip(&expected).enumerate() {
            for (got, want) in a.iter().zip(b) {
                assert!(
                    got.abs_diff(*want) <= 1,
                    "{name} pixel {index}: got {a:?}, expected {b:?}"
                );
            }
        }
        evidence.push(json!({"case":name,"size":size,"strength":strength,"expected":expected,"actual":actual,"maxTolerance":1}));
    }
    json!({"ok":true,"adapter":context.report().adapter(),"kind":"literal analytic-ramp probes through production WGSL","cases":evidence})
}

fn render(context: &GpuContext, size: [u32; 2], heights: &[f32], strength: f32) -> Vec<[u8; 4]> {
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
        "normal probe result",
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
        label: Some("probe strength"),
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
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production height-to-normal"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../shaders/nodes/height-to-normal.wgsl").into(),
        ),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("normal probe"),
        layout: None,
        module: &module,
        entry_point: Some("height_to_normal"),
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
                        let mut rgba = [0; 4];
                        for (channel, bytes) in p.as_chunks::<2>().0.iter().enumerate() {
                            let value =
                                half::f16::from_bits(u16::from_le_bytes([bytes[0], bytes[1]]))
                                    .to_f32();
                            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
                            rgba[channel] = (value * 255.).round() as u8;
                        }
                        rgba
                    })
            })
            .collect()
    };
    buffer.unmap();
    result
}
