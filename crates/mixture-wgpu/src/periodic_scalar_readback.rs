//! Shared test-only scalar generator dispatch and exact half readback; no pixel formulas.
use super::GpuContext;
pub(crate) fn render(
    context: &GpuContext,
    size: [u32; 2],
    bytes: &[u8],
    shader: &str,
    entry: &str,
) -> Vec<[u16; 4]> {
    let (device, queue) = (context.device(), context.queue());
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("value-noise probe parameters"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    uniform
        .slice(..)
        .get_mapped_range_mut()
        .unwrap()
        .copy_from_slice(bytes);
    uniform.unmap();
    let extent = wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("value-noise probe output"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production value-noise periodic probe"),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: Some(entry),
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
