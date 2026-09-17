//! Compare the production WGSL conversion to the independent `half` conversion.
//! This tests numeric storage conversion, not a CPU material executor.
use mixture_wgpu::GpuContext;

pub(super) fn run(context: &GpuContext) {
    let mut inputs = vec![0.0_f32, -0.0, 1.0, -1.0];
    for bits in 0_u16..0x3c00 {
        let lower = half::f16::from_bits(bits).to_f32();
        let upper = half::f16::from_bits(bits + 1).to_f32();
        let middle = (lower + upper) * 0.5;
        for value in [
            lower,
            f32::from_bits(middle.to_bits() - 1),
            middle,
            f32::from_bits(middle.to_bits() + 1),
        ] {
            inputs.extend([value, -value]);
        }
    }
    let (device, queue) = (context.device(), context.queue());
    let bytes: Vec<u8> = inputs.iter().flat_map(|v| v.to_le_bytes()).collect();
    let input = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("half boundary inputs"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: true,
    });
    input
        .slice(..)
        .get_mapped_range_mut()
        .unwrap()
        .copy_from_slice(&bytes);
    input.unmap();
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("half boundary results"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("half boundary readback"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production half conversion probe"),
        source: wgpu::ShaderSource::Wgsl(concat!(
            include_str!("../../shaders/precision.wgsl"),
            "\n@group(0) @binding(0) var<storage, read> inputs: array<f32>;\n",
            "@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;\n",
            "@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id: vec3<u32>) {\n",
            "if id.x < arrayLength(&inputs) { outputs[id.x] = mixture_half(inputs[id.x]); } }\n"
        ).into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("half conversion"),
        layout: None,
        module: &module,
        entry_point: Some("probe"),
        compilation_options: Default::default(),
        cache: None,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups((inputs.len() as u32).div_ceil(64), 1, 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &staging, 0, bytes.len() as u64);
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    staging
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
    {
        let mapped = staging.slice(..).get_mapped_range().unwrap();
        for (value, bytes) in inputs.iter().zip(mapped.as_chunks::<4>().0) {
            let actual = u32::from_le_bytes(*bytes);
            let expected = half::f16::from_f32(*value).to_f32().to_bits();
            assert_eq!(actual, expected, "half boundary {value:?}");
        }
    }
    staging.unmap();
    input.destroy();
    output.destroy();
    staging.destroy();
}
