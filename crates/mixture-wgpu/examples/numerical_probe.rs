//! Opt-in diagnostic: execute production noise and normal shaders, retain raw f16.
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions};
use std::{path::PathBuf, time::Duration};

fn main() {
    let output = PathBuf::from(std::env::var("MIXTURE_NUMERICAL_OUTPUT").unwrap());
    assert!(output.is_absolute(), "use an absolute output directory");
    std::fs::create_dir(&output).unwrap();
    let backend = match std::env::var("MIXTURE_GPU_BACKEND").as_deref() {
        Ok("dx12") => BackendPreference::Dx12,
        Ok("vulkan") => BackendPreference::Vulkan,
        _ => panic!("explicit vulkan or dx12 backend required"),
    };
    let context = pollster::block_on(GpuContext::request(GpuContextOptions {
        backend,
        ..Default::default()
    }))
    .unwrap();
    let device = context.device();
    let queue = context.queue();
    let extent = wgpu::Extent3d {
        width: 1024,
        height: 1024,
        depth_or_array_layers: 1,
    };
    let texture = || {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("numerical capture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    };
    let noise = texture();
    let normal = texture();
    let noise_view = noise.create_view(&Default::default());
    let normal_view = normal.create_view(&Default::default());
    let capture = |name: &str, shader: &str, entry: &str, words: &[u32], input: bool| {
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (words.len() * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });
        uniform
            .slice(..)
            .get_mapped_range_mut()
            .unwrap()
            .copy_from_slice(
                &words
                    .iter()
                    .flat_map(|w| w.to_le_bytes())
                    .collect::<Vec<_>>(),
            );
        uniform.unmap();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(name),
            layout: None,
            module: &module,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            cache: None,
        });
        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(if input {
                    &normal_view
                } else {
                    &noise_view
                }),
            },
        ];
        if input {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&noise_view),
            });
        }
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * 1024 * 8,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(128, 128, 1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: if input { &normal } else { &noise },
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(8192),
                    rows_per_image: Some(1024),
                },
            },
            extent,
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(30)),
            })
            .unwrap();
        rx.recv().unwrap().unwrap();
        std::fs::write(
            output.join(format!("{name}.rgba16")),
            buffer.slice(..).get_mapped_range().unwrap(),
        )
        .unwrap();
        buffer.unmap();
    };
    let noise_shader = concat!(
        include_str!("../shaders/precision.wgsl"),
        "\n",
        include_str!("../shaders/nodes/fractal-noise.wgsl")
    );
    let experiment = std::env::var("MIXTURE_NUMERICAL_SHADER")
        .ok()
        .map(|path| std::fs::read_to_string(path).unwrap());
    let noise_shader = experiment.as_deref().unwrap_or(noise_shader);
    std::fs::write(output.join("noise.wgsl"), noise_shader).unwrap();
    let normal_shader = concat!(
        include_str!("../shaders/precision.wgsl"),
        "\n",
        include_str!("../shaders/nodes/height-to-normal.wgsl")
    );
    let parameters: [u32; 8] = std::env::var("MIXTURE_NUMERICAL_PARAMETERS")
        .ok()
        .map(|s| serde_json::from_str(&s).unwrap())
        .unwrap_or([29, 32, 4, 0, 0.5f32.to_bits(), 0, 0, 0]);
    assert!((1..=128).contains(&parameters[1]));
    assert!((1..=6).contains(&parameters[2]));
    assert!(parameters[3] <= 1);
    assert!((0.0..=1.0).contains(&f32::from_bits(parameters[4])));
    std::fs::write(
        output.join("parameters.json"),
        serde_json::to_vec(&parameters).unwrap(),
    )
    .unwrap();
    capture("noise", noise_shader, "fractal_noise", &parameters, false);
    capture(
        "normal",
        normal_shader,
        "height_to_normal",
        &[1f32.to_bits(), 0, 0, 0],
        true,
    );
    if let Ok(path) = std::env::var("MIXTURE_NUMERICAL_REPLAY") {
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(bytes.len(), 1024 * 1024 * 8);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &noise,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8192),
                rows_per_image: Some(1024),
            },
            extent,
        );
        capture(
            "replay-normal",
            normal_shader,
            "height_to_normal",
            &[1f32.to_bits(), 0, 0, 0],
            true,
        );
    }
    std::fs::write(
        output.join("adapter.json"),
        serde_json::to_vec_pretty(context.report()).unwrap(),
    )
    .unwrap();
}
