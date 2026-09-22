//! Check production fixed-point primitives against independent u64/f64 arithmetic.
//! This tests arithmetic operations, not a CPU noise or material executor.
use mixture_wgpu::GpuContext;

pub(super) fn run(context: &GpuContext) {
    const Q: u32 = 1 << 24;
    let mut inputs = Vec::new();
    for a in [0, 1, 4095, 4096, Q / 2 - 1, Q / 2, Q / 2 + 1, Q - 1, Q] {
        for b in [0, 1, 4095, 4096, Q / 2, Q - 1, Q] {
            for t in [0, 1, Q / 2, Q - 1, Q] {
                inputs.push([a, b, t, 0]);
            }
        }
    }
    let mut state = 0x71e52acdu32;
    for _ in 0..4096 {
        let mut row = [0; 4];
        for v in &mut row[..3] {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *v = state % (Q + 1);
        }
        inputs.push(row);
    }
    let (device, queue) = (context.device(), context.queue());
    let bytes: Vec<u8> = inputs
        .iter()
        .flatten()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let input = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fixed arithmetic inputs"),
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
        label: Some("fixed arithmetic results"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fixed arithmetic readback"),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production fixed arithmetic probe"),
        source: wgpu::ShaderSource::Wgsl(concat!(
            include_str!("../../shaders/precision.wgsl"), "\n",
            include_str!("../../shaders/nodes/fractal-noise.wgsl"), "\n",
            "@group(0) @binding(3) var<storage, read> inputs: array<vec4<u32>>;\n",
            "@group(0) @binding(4) var<storage, read_write> outputs: array<vec4<u32>>;\n",
            "@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id: vec3<u32>) {\n",
            "if id.x < arrayLength(&inputs) { let v=inputs[id.x]; outputs[id.x] = vec4<u32>(qmul(v.x,v.y),qmix(v.x,v.y,v.z),qratio(v.x,Q+v.y),qfade(v.z)); } }\n"
        ).into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("fixed arithmetic"),
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
                binding: 3,
                resource: input.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
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
        for (v, bytes) in inputs.iter().zip(mapped.as_chunks::<16>().0) {
            let actual: Vec<u32> = bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| u32::from_le_bytes(*b))
                .collect();
            let round = |n: u64| {
                let q = n / u64::from(Q);
                let r = n % u64::from(Q);
                (q + u64::from(r > u64::from(Q / 2) || (r == u64::from(Q / 2) && q % 2 == 1)))
                    as u32
            };
            let product = round(u64::from(v[0]) * u64::from(v[1]));
            let delta = round(u64::from(v[0].abs_diff(v[1])) * u64::from(v[2]));
            let blend = if v[1] >= v[0] {
                v[0] + delta
            } else {
                v[0] - delta
            };
            let ratio = ((u64::from(v[0]) << 24) / u64::from(Q + v[1])) as u32;
            assert_eq!(
                &actual[..3],
                &[product, blend, ratio],
                "fixed operations {v:?}"
            );
            let t = f64::from(v[2]) / f64::from(Q);
            let quintic = t * t * t * (t * (6. * t - 15.) + 10.);
            assert!(
                (f64::from(actual[3]) - quintic * f64::from(Q)).abs() <= 16.,
                "quintic error {v:?}: {}",
                actual[3]
            );
            assert!(actual[3] <= Q);
        }
    }

    staging.unmap();
    input.destroy();
    output.destroy();
    staging.destroy();
}
