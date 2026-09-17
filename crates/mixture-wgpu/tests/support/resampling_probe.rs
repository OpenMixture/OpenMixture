//! Literal texels exercise production resampling WGSL, without a CPU pixel executor.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};

struct Probe {
    name: &'static str,
    size: [u32; 2],
    input: Vec<f32>,
    displacement: Option<Vec<f32>>,
    uniform: Vec<u8>,
    parameters: Value,
    expected: Vec<f32>,
}

pub(super) fn run(context: &GpuContext, node: &str) -> Value {
    let probes = match node {
        "transform-2d" => transform_probes(),
        "warp" => warp_probes(),
        _ => panic!("no literal resampling probes for {node}"),
    };
    let mut evidence = Vec::new();
    for probe in probes {
        let actual = render(context, node, &probe);
        assert_eq!(actual.len(), probe.expected.len());
        for (index, (rgba, scalar)) in actual.iter().zip(&probe.expected).enumerate() {
            // Literal expectations specify the binary16 result analytically;
            // no tolerance or CPU resampler is used.
            assert_eq!(
                *rgba,
                [*scalar, 0.0, 0.0, 1.0],
                "{node}/{} pixel {index}",
                probe.name
            );
        }
        evidence.push(json!({
            "case": probe.name,
            "size": probe.size,
            "input": probe.input,
            "displacement": probe.displacement,
            "parameters": probe.parameters,
            "expectedScalar": probe.expected,
            "actualRgba": actual,
            "maxTolerance": 0
        }));
    }
    json!({
        "ok": true,
        "adapter": context.report().adapter(),
        "kind": "literal scalar probes through production WGSL",
        "cases": evidence
    })
}

fn transform_probes() -> Vec<Probe> {
    let horizontal = vec![0.0, 0.25, 0.5, 1.0];
    let square = vec![0.0, 0.25, 0.5, 1.0];
    let vertical = vec![0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 1.0, 1.0];
    // Tuple fields: name, dimensions, literal input, [scaleX, scaleY, turns],
    // offset, literal expected output. These are examples, not a resampler.
    [
        (
            "identity-rectangular-one-texel-y",
            [4, 1],
            horizontal.clone(),
            [1, 1, 0],
            [0.0, 0.0],
            horizontal.clone(),
        ),
        (
            "positive-half-texel-x-wrap",
            [4, 1],
            horizontal.clone(),
            [1, 1, 0],
            [0.125, 0.0],
            vec![0.125, 0.375, 0.75, 0.5],
        ),
        (
            "negative-half-texel-x-wrap",
            [4, 1],
            horizontal.clone(),
            [1, 1, 0],
            [-0.125, 0.0],
            vec![0.5, 0.125, 0.375, 0.75],
        ),
        (
            "positive-full-period-identity",
            [4, 1],
            horizontal.clone(),
            [1, 1, 0],
            [1.0, 1.0],
            horizontal.clone(),
        ),
        (
            "negative-full-period-identity",
            [4, 1],
            horizontal.clone(),
            [1, 1, 0],
            [-1.0, -1.0],
            horizontal.clone(),
        ),
        (
            "integer-scale-center-pivot",
            [4, 1],
            horizontal.clone(),
            [2, 1, 0],
            [0.0, 0.0],
            vec![0.75, 0.125, 0.75, 0.125],
        ),
        (
            "maximum-integer-scale",
            [4, 1],
            horizontal,
            [64, 64, 0],
            [0.0, 0.0],
            vec![0.375; 4],
        ),
        (
            "clockwise-quarter-turn",
            [2, 2],
            square.clone(),
            [1, 1, 1],
            [0.0, 0.0],
            vec![0.5, 0.0, 1.0, 0.25],
        ),
        (
            "half-turn",
            [2, 2],
            square.clone(),
            [1, 1, 2],
            [0.0, 0.0],
            vec![1.0, 0.5, 0.25, 0.0],
        ),
        (
            "three-quarter-turns",
            [2, 2],
            square.clone(),
            [1, 1, 3],
            [0.0, 0.0],
            vec![0.25, 1.0, 0.0, 0.5],
        ),
        (
            "four-neighbor-interpolation",
            [2, 2],
            square,
            [1, 1, 0],
            [0.25, 0.25],
            vec![0.4375; 4],
        ),
        (
            "positive-half-texel-y-rectangular",
            [2, 4],
            vertical.clone(),
            [1, 1, 0],
            [0.0, 0.125],
            vec![0.125, 0.125, 0.375, 0.375, 0.75, 0.75, 0.5, 0.5],
        ),
        (
            "negative-half-texel-y-rectangular",
            [2, 4],
            vertical.clone(),
            [1, 1, 0],
            [0.0, -0.125],
            vec![0.5, 0.5, 0.125, 0.125, 0.375, 0.375, 0.75, 0.75],
        ),
        (
            "integer-y-scale-center-pivot",
            [2, 4],
            vertical,
            [1, 2, 0],
            [0.0, 0.0],
            vec![0.75, 0.75, 0.125, 0.125, 0.75, 0.75, 0.125, 0.125],
        ),
        (
            "rotation-before-anisotropic-scale",
            [4, 2],
            vec![0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0],
            [2, 1, 1],
            [0.0, 0.0],
            // Both output rows sample the same wrapped x seam. Their y
            // coordinates mix the seam means 0.1875 and 0.75 by 1/4 or 3/4.
            vec![
                0.609375, 0.609375, 0.328125, 0.328125, 0.609375, 0.609375, 0.328125, 0.328125,
            ],
        ),
        (
            "one-texel-x-axis",
            [1, 4],
            vec![0.0, 0.25, 0.5, 1.0],
            [64, 1, 0],
            [0.375, 0.125],
            vec![0.125, 0.375, 0.75, 0.5],
        ),
        (
            "single-texel-all-neighbors-wrap",
            [1, 1],
            vec![0.75],
            [64, 64, 3],
            [-0.375, 0.625],
            vec![0.75],
        ),
    ]
    .into_iter()
    .map(|(name, size, input, integers, offset, expected)| {
        let uniform = [integers[0], integers[1], integers[2], 0_u32]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .chain(
                [offset[0], offset[1], 0.0, 0.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes),
            )
            .collect();
        Probe {
            name,
            size,
            input,
            displacement: None,
            uniform,
            parameters: json!({
                "scale": [integers[0], integers[1]],
                "quarterTurns": integers[2],
                "offset": offset
            }),
            expected,
        }
    })
    .collect()
}

fn warp_probes() -> Vec<Probe> {
    let horizontal = vec![0.0, 0.25, 0.5, 1.0];
    let vertical = vec![0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 1.0, 1.0];
    // Tuple fields: name, dimensions, literal input, literal displacement,
    // strength, literal expected output.
    [
        (
            "neutral-displacement-exact-identity",
            [4, 1],
            horizontal.clone(),
            vec![0.5; 4],
            [-1.0, 1.0],
            horizontal.clone(),
        ),
        (
            "zero-strength-exact-identity",
            [4, 1],
            horizontal.clone(),
            vec![0.0, 1.0, 0.25, 0.75],
            [0.0, 0.0],
            horizontal.clone(),
        ),
        (
            "positive-half-texel-x-wrap",
            [4, 1],
            horizontal.clone(),
            vec![1.0; 4],
            [0.125, 0.0],
            vec![0.125, 0.375, 0.75, 0.5],
        ),
        (
            "negative-half-texel-x-wrap",
            [4, 1],
            horizontal.clone(),
            vec![0.0; 4],
            [0.125, 0.0],
            vec![0.5, 0.125, 0.375, 0.75],
        ),
        (
            "negative-strength-reverses-polarity",
            [4, 1],
            horizontal.clone(),
            vec![1.0; 4],
            [-0.125, 0.0],
            vec![0.5, 0.125, 0.375, 0.75],
        ),
        (
            "displacement-clamps-before-centering",
            [4, 1],
            horizontal.clone(),
            vec![-2.0, 0.0, 1.0, 2.0],
            [0.125, 0.0],
            vec![0.5, 0.125, 0.75, 0.5],
        ),
        (
            "spatial-displacement-loaded-at-output-texel",
            [4, 1],
            horizontal.clone(),
            vec![0.0, 0.5, 1.0, 0.5],
            [0.25, 0.0],
            vec![1.0, 0.25, 1.0, 1.0],
        ),
        (
            "maximum-strength-full-period-identity",
            [4, 1],
            horizontal.clone(),
            vec![1.0; 4],
            [-1.0, 1.0],
            horizontal,
        ),
        (
            "positive-half-texel-y-rectangular",
            [2, 4],
            vertical.clone(),
            vec![1.0; 8],
            [0.0, 0.125],
            vec![0.125, 0.125, 0.375, 0.375, 0.75, 0.75, 0.5, 0.5],
        ),
        (
            "negative-half-texel-y-rectangular",
            [2, 4],
            vertical,
            vec![0.0; 8],
            [0.0, 0.125],
            vec![0.5, 0.5, 0.125, 0.125, 0.375, 0.375, 0.75, 0.75],
        ),
        (
            "four-neighbor-interpolation",
            [2, 2],
            vec![0.0, 0.25, 0.5, 1.0],
            vec![1.0; 4],
            [0.25, 0.25],
            vec![0.4375; 4],
        ),
        (
            "one-texel-x-axis",
            [1, 4],
            vec![0.0, 0.25, 0.5, 1.0],
            vec![1.0; 4],
            [0.375, 0.125],
            vec![0.125, 0.375, 0.75, 0.5],
        ),
        // A positive 2^-26 UV shift is 2^-16 texels here. It must not vanish
        // when added to a large absolute UV. Low samples are exactly 2^-16;
        // high samples round from 1-2^-16 to 1 in binary16.
        (
            "sub-ulp-uv-displacement-keeps-local-weight",
            [1024, 1],
            (0..1024)
                .map(|i| if i % 2 == 0 { 0.0 } else { 1.0 })
                .collect(),
            vec![1.0; 1024],
            [1.0 / 67108864.0, 0.0],
            (0..1024)
                .map(|i| if i % 2 == 0 { 1.0 / 65536.0 } else { 1.0 })
                .collect(),
        ),
        (
            "odd-width-positive-three-quarter-texel",
            [3, 1],
            vec![0.0, 0.25, 1.0],
            vec![1.0; 3],
            [0.25, 0.0],
            vec![0.1875, 0.8125, 0.25],
        ),
        (
            "odd-height-negative-three-quarter-texel",
            [1, 3],
            vec![0.0, 0.25, 1.0],
            vec![1.0; 3],
            [0.0, -0.25],
            vec![0.75, 0.0625, 0.4375],
        ),
        (
            "odd-width-negative-full-period",
            [3, 1],
            vec![0.0, 0.25, 1.0],
            vec![1.0; 3],
            [-1.0, 1.0],
            vec![0.0, 0.25, 1.0],
        ),
        (
            "single-texel-all-neighbors-wrap",
            [1, 1],
            vec![0.75],
            vec![1.0],
            [0.75, -0.25],
            vec![0.75],
        ),
    ]
    .into_iter()
    .map(
        |(name, size, input, displacement, strength, expected)| Probe {
            name,
            size,
            input,
            displacement: Some(displacement),
            uniform: [strength[0], strength[1], 0.0, 0.0]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect(),
            parameters: json!({"strength": strength}),
            expected,
        },
    )
    .collect()
}

fn render(context: &GpuContext, node: &str, probe: &Probe) -> Vec<[f32; 4]> {
    let device = context.device();
    let queue = context.queue();
    let extent = wgpu::Extent3d {
        width: probe.size[0],
        height: probe.size[1],
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
    let upload = |label, scalars: &[f32]| {
        assert_eq!(scalars.len(), (probe.size[0] * probe.size[1]) as usize);
        let input = texture(
            label,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let data: Vec<_> = scalars
            .iter()
            .flat_map(|scalar| {
                [*scalar, 0.0, 0.0, 1.0]
                    .into_iter()
                    .flat_map(|value| half::f16::from_f32(value).to_bits().to_le_bytes())
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
                bytes_per_row: Some(probe.size[0] * 8),
                rows_per_image: Some(probe.size[1]),
            },
            extent,
        );
        input
    };
    let input = upload("literal resampling input", &probe.input);
    let displacement = probe
        .displacement
        .as_ref()
        .map(|values| upload("literal displacement input", values));
    let output = texture(
        "resampling probe result",
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("literal resampling parameters"),
        size: probe.uniform.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    uniform
        .slice(..)
        .get_mapped_range_mut()
        .unwrap()
        .copy_from_slice(&probe.uniform);
    uniform.unmap();
    let (source, entry_point) = match node {
        "transform-2d" => (
            concat!(
                include_str!("../../shaders/precision.wgsl"),
                "\n",
                include_str!("../../shaders/nodes/transform-2d.wgsl")
            ),
            "transform_2d",
        ),
        "warp" => (
            concat!(
                include_str!("../../shaders/precision.wgsl"),
                "\n",
                include_str!("../../shaders/nodes/warp.wgsl")
            ),
            "warp",
        ),
        _ => panic!("unknown resampling shader {node}"),
    };
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production resampling shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("literal resampling probe"),
        layout: None,
        module: &module,
        entry_point: Some(entry_point),
        compilation_options: Default::default(),
        cache: None,
    });
    let input_view = input.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let displacement_view = displacement
        .as_ref()
        .map(|texture| texture.create_view(&Default::default()));
    let mut entries = vec![
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
    ];
    if let Some(view) = &displacement_view {
        entries.push(wgpu::BindGroupEntry {
            binding: 3,
            resource: wgpu::BindingResource::TextureView(view),
        });
    }
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("resampling probe bindings"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });
    let row_bytes = (probe.size[0] * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("resampling probe readback"),
        size: u64::from(row_bytes * probe.size[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(probe.size[0].div_ceil(8), probe.size[1].div_ceil(8), 1);
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
                bytes_per_row: Some(row_bytes),
                rows_per_image: Some(probe.size[1]),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap()
        });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .unwrap();
    receiver.recv().unwrap().unwrap();
    let result = {
        let view = buffer.slice(..).get_mapped_range().unwrap();
        view.chunks_exact(row_bytes as usize)
            .flat_map(|row| {
                row[..probe.size[0] as usize * 8]
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|pixel| {
                        let mut rgba = [0.0; 4];
                        for (channel, bytes) in pixel.as_chunks::<2>().0.iter().enumerate() {
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
