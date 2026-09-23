//! Periodic sampling-origin probe using the production brick shader and ABI.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};

pub(super) fn run(context: &GpuContext) -> Value {
    let mut cases = Vec::new();
    for size in [[256, 256], [257, 129], [1, 17], [17, 1]] {
        for (cells, offset, seed, variation) in [
            ([8, 8], 0.5, 1729, 0.15),
            ([6, 12], 0.5, u32::MAX, 0.5),
            ([8, 8], 0.0, 1729, 0.15),
            ([1, 1], 0.0, 0, 0.0),
        ] {
            let parameters = (cells, offset, seed, variation);
            let original = render(context, size, parameters, None);
            for origin in [[1, 0], [0, size[1] - 1], [size[0] + 3, size[1] + 5]] {
                let shifted = render(context, size, parameters, Some(origin));
                for y in 0..size[1] {
                    for x in 0..size[0] {
                        let before = (((y + origin[1]) % size[1]) * size[0]
                            + (x + origin[0]) % size[0])
                            as usize;
                        let after = (y * size[0] + x) as usize;
                        assert_eq!(
                            shifted[after], original[before],
                            "periodic sampling {size:?} origin {origin:?} pixel {x},{y}"
                        );
                    }
                }
                cases.push(json!({"size":size,"cells":cells,"rowOffset":offset,"seed":seed,"variation":variation,"origin":origin,"maxHalfComponentDifference":0}));
            }
        }
    }
    json!({"ok":true,"adapter":context.report().adapter(),"scope":"production WGSL with test-only wrapped sampling origin; raw f16 finite/range and exact shifted pixels","cases":cases})
}

fn render(
    context: &GpuContext,
    size: [u32; 2],
    parameters: ([u32; 2], f32, u32, f32),
    origin: Option<[u32; 2]>,
) -> Vec<[u16; 4]> {
    let (device, queue) = (context.device(), context.queue());
    let (cells, offset, seed, variation) = parameters;
    let shift = origin.unwrap_or([0, 0]);
    let mut bytes: Vec<u8> = [cells[0], cells[1], seed, shift[0]]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    bytes.extend(
        [offset, 0.08, 0.08, 0.08, variation, shift[1] as f32, 0., 0.]
            .into_iter()
            .flat_map(f32::to_le_bytes),
    );
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("brick probe parameters"),
        size: 48,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    uniform
        .slice(..)
        .get_mapped_range_mut()
        .unwrap()
        .copy_from_slice(&bytes);
    uniform.unmap();
    let extent = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("brick probe output"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let production = concat!(
        include_str!("../../shaders/precision.wgsl"),
        "\n",
        include_str!("../../shaders/nodes/brick-pattern.wgsl")
    );
    let shader = if origin.is_some() {
        let anchor = "let p = vec2<f32>(gid.xy);";
        assert_eq!(production.matches(anchor).count(), 1);
        production.replace(anchor, "let p = vec2<f32>((gid.xy + vec2<u32>(params.cells_seed.w, u32(params.variation_padding.y))) % size);")
    } else {
        production.to_owned()
    };
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production brick periodic probe"),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: Some("brick_pattern"),
        compilation_options: Default::default(),
        cache: None,
    });
    let view = texture.create_view(&Default::default());
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&view),
            },
        ],
    });
    let stride = (size[0] * 8).div_ceil(256) * 256;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(stride) * u64::from(size[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(size[0].div_ceil(8), size[1].div_ceil(8), 1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(size[1]),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback
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
    receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .unwrap()
        .unwrap();
    let mut pixels = Vec::new();
    {
        let mapped = readback.slice(..).get_mapped_range().unwrap();
        for y in 0..size[1] as usize {
            let row = &mapped[y * stride as usize..y * stride as usize + size[0] as usize * 8];
            for pixel in row.as_chunks::<8>().0 {
                let bits =
                    std::array::from_fn(|i| u16::from_le_bytes([pixel[i * 2], pixel[i * 2 + 1]]));
                let values = bits.map(|v| half::f16::from_bits(v).to_f32());
                assert!(
                    values
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                );
                assert_eq!(&values[1..], &[0., 0., 1.]);
                pixels.push(bits);
            }
        }
    }
    readback.unmap();
    readback.destroy();
    texture.destroy();
    uniform.destroy();
    pixels
}
