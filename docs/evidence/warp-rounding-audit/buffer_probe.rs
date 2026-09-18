//! Diagnostic buffer readback only; not a material renderer or acceptance command.
use mixture_wgpu::GpuContext;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = args[3].parse().unwrap();
    let inputs: Vec<f32> = if args.len() > 4 {
        std::fs::read(&args[4])
            .unwrap()
            .chunks_exact(4)
            .map(|x| f32::from_le_bytes(x.try_into().unwrap()))
            .collect()
    } else {
        vec![0.0_f32; count]
    };
    assert_eq!(inputs.len(), count);
    let context = pollster::block_on(GpuContext::request(mixture_wgpu::GpuContextOptions {
        backend: if std::env::var("MIXTURE_GPU_BACKEND").as_deref() == Ok("vulkan") {
            mixture_wgpu::BackendPreference::Vulkan
        } else {
            mixture_wgpu::BackendPreference::Dx12
        },
        ..Default::default()
    }))
    .unwrap();
    eprintln!("{}", serde_json::to_string(context.report()).unwrap());
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
        source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(&args[1]).unwrap().into()),
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
        std::fs::write(&args[2], &*mapped).unwrap();
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let command = format!(
            "Get-Process -Id {} | Select-Object -ExpandProperty Modules | Where-Object ModuleName -eq 'dxcompiler.dll' | Select-Object FileName,ModuleName | ConvertTo-Json",
            std::process::id()
        );
        let modules = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", &command])
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(modules.status.success());
        std::fs::write(format!("{}.modules.json", args[2]), modules.stdout).unwrap();
    }
    staging.unmap();
    input.destroy();
    output.destroy();
    staging.destroy();
}
