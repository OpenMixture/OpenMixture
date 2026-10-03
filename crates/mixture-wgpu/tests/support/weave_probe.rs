//! Sparse analytical oracles and test-only observations through the production helper.
use mixture_wgpu::GpuContext;
use serde_json::{Value, json};
fn near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1. / 1024., "{a} != {b}");
}
fn params(counts: [u32; 2], r: f32, c: f32, mode: u32) -> [u32; 12] {
    [
        counts[0],
        counts[1],
        mode,
        0,
        0.7f32.to_bits(),
        0.7f32.to_bits(),
        0.08f32.to_bits(),
        c.to_bits(),
        r.to_bits(),
        0,
        0,
        0,
    ]
}
fn observe(context: &GpuContext, p: [u32; 12], uvs: &[[f32; 2]], field: &str) -> Vec<[f32; 4]> {
    let mut body = String::from("var uv=vec2<f32>(0.0);switch gid.y*32u+gid.x {");
    for (i, uv) in uvs.iter().enumerate() {
        body.push_str(&format!(
            "case {i}u: {{ uv=vec2<f32>({:.9},{:.9}); }}",
            uv[0], uv[1]
        ));
    }
    body.push_str("default:{} } let s=weave_sample(uv); ");
    body.push_str(&format!(
        "textureStore(output,vec2<i32>(gid.xy),mixture_half4({field}));"
    ));
    let mut result = render(
        context,
        [32, (uvs.len() as u32).div_ceil(32)],
        p,
        Some(&body),
    );
    result.truncate(uvs.len());
    result
}
pub(super) fn run(context: &GpuContext) -> Value {
    let mut centers = 0;
    let mut translations = 0;
    let mut weighted_witnesses = 0;
    for counts in [[8, 8], [12, 8]] {
        for r in [0.25, 0.5, 0.75] {
            for crown in [0., 0.5, 1.] {
                let p = params(counts, r, crown, 0);
                let uvs: Vec<_> = (0..counts[1])
                    .flat_map(|j| {
                        (0..counts[0]).map(move |i| {
                            [
                                (i as f32 + 0.5) / counts[0] as f32,
                                (j as f32 + 0.5) / counts[1] as f32,
                            ]
                        })
                    })
                    .collect();
                let heights = observe(
                    context,
                    p,
                    &uvs,
                    "vec4<f32>(s.heights,s.fields.y,s.fields.x)",
                );
                let weights = observe(
                    context,
                    p,
                    &uvs,
                    "vec4<f32>(s.occupancy,s.fields.z,s.fields.y-s.fields.z)",
                );
                for (index, (h, w)) in heights.iter().zip(&weights).enumerate() {
                    let even = (index as u32 % counts[0] + index as u32 / counts[0]) % 2 == 0;
                    let want = if even { [1., r] } else { [r, 1.] };
                    near(h[0], want[0]);
                    near(h[1], want[1]);
                    assert!((h[0] - h[1]).abs() > 0.24);
                    near(h[2], 1.);
                    near(h[3], 1.);
                    near(w[0], 1.);
                    near(w[1], 1.);
                    near(w[2], if even { 1. } else { 0. });
                    near(w[3], if even { 0. } else { 1. });
                    centers += 1;
                }
                let nw = counts[0] as f32;
                let support = [
                    [0., 0.],
                    [0.5 / nw, 0.],
                    [(0.5 + 0.35) / nw, 0.],
                    [(0.5 + 0.35 - 0.04) / nw, 0.],
                    [(0.5 + 0.1) / nw, 0.],
                    [(0.5 + 0.2) / nw, 0.],
                ];
                let fields = observe(context, p, &support, "vec4<f32>(s.fields,1.0)");
                near(fields[0][0], 0.);
                near(fields[0][1], 0.);
                near(fields[0][2], 0.);
                near(fields[1][1], 1.);
                assert!(fields[1][0] >= r);
                near(fields[2][1], 0.);
                near(fields[3][1], 0.5);
                assert!(fields[1][0] > fields[4][0] && fields[4][0] > fields[5][0]);
                let base = [
                    [0.00001, 0.375],
                    [-0.00001, 0.375],
                    [0.0625, 0.00001],
                    [0.125, 0.1875],
                    [0.173, 0.419],
                ];
                let reference = observe(context, p, &base, "vec4<f32>(s.fields,1.0)");
                for delta in [[1., 0.], [0., 1.], [1., 1.], [-1., -1.]] {
                    let moved: Vec<_> = base
                        .iter()
                        .map(|v| [v[0] + delta[0], v[1] + delta[1]])
                        .collect();
                    let actual = observe(context, p, &moved, "vec4<f32>(s.fields,1.0)");
                    for (a, b) in actual.iter().zip(&reference) {
                        for k in 0..4 {
                            near(a[k], b[k]);
                        }
                        translations += 1;
                    }
                }
                let mut seams = Vec::new();
                for v in [
                    [0., 0.375],
                    [0.5 / nw, 0.],
                    [0.5 / nw, 0.5 / counts[1] as f32],
                    [1. / nw, 0.1875],
                ] {
                    for axis in 0..2 {
                        let mut a = v;
                        let mut b = v;
                        a[axis] -= 1e-5;
                        b[axis] += 1e-5;
                        seams.extend([a, b]);
                    }
                }
                let actual = observe(context, p, &seams, "vec4<f32>(s.fields,1.0)");
                for pair in actual.chunks_exact(2) {
                    for k in 0..3 {
                        assert!((pair[0][k] - pair[1][k]).abs() <= 0.01);
                    }
                }
            }
        }
    }
    // Compare actual public entry-point modes against four independent production-helper observations.
    let size = [19, 11];
    let p = params([12, 8], 0.5, 0.5, 0);
    let mut samples = Vec::new();
    for y in 0..size[1] {
        for x in 0..size[0] {
            for offset in [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75], [0.75, 0.75]] {
                samples.push([
                    (x as f32 + offset[0]) / size[0] as f32,
                    (y as f32 + offset[1]) / size[1] as f32,
                ]);
            }
        }
    }
    let observed = observe(context, p, &samples, "vec4<f32>(s.fields,1.0)");
    let modes: Vec<_> = (0..3)
        .map(|mode| render(context, size, params([12, 8], 0.5, 0.5, mode), None))
        .collect();
    for mode in 0..3 {
        assert_eq!(
            modes[mode],
            render(context, size, params([12, 8], 0.5, 0.5, mode as u32), None)
        );
    }
    for (i, s) in observed.chunks_exact(4).enumerate() {
        let total: [f32; 3] = std::array::from_fn(|k| s.iter().map(|p| p[k]).sum());
        let share = if total[1] > 0. {
            total[2] / total[1]
        } else {
            0.5
        };
        near(modes[0][i][0], total[0] / 4.);
        near(modes[1][i][0], total[1] / 4.);
        near(modes[2][i][0], share);
        assert!((modes[1][i][0] * modes[2][i][0] - total[2] / 4.).abs() <= 2. / 1024.);
        let unweighted = s
            .iter()
            .map(|p| if p[1] > 0. { p[2] / p[1] } else { 0.5 })
            .sum::<f32>()
            / 4.;
        if (share - unweighted).abs() > 0.01 {
            weighted_witnesses += 1;
        }
    }
    assert!(weighted_witnesses > 0);
    // Empty production footprint uses the neutral conditional share, not zero.
    let empty = render(context, [32, 32], params([4, 4], 0.5, 0.5, 2), None);
    near(empty[0][0], 0.5);
    json!({"ok":true,"adapter":context.report().adapter(),"crossingCenters":centers,"translatedEvaluations":translations,"weightedFilterWitnesses":weighted_witnesses,"rawHalfTolerance":1./1024.,"repeatExact":true})
}
fn render(
    context: &GpuContext,
    size: [u32; 2],
    parameters: [u32; 12],
    probe: Option<&str>,
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
    assert!(size[0] <= 32);
    let output = texture(
        "weave probe result",
        wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    );
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("probe parameters"),
        size: 48,
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
    let mut shader = concat!(
        include_str!("../../shaders/precision.wgsl"),
        "\n",
        include_str!("../../shaders/nodes/weave-pattern.wgsl")
    )
    .to_owned();
    if let Some(body) = probe {
        shader.push_str(&format!("\n@compute @workgroup_size(8,8,1) fn probe(@builtin(global_invocation_id) gid:vec3<u32>) {{ let size=textureDimensions(output);if any(gid.xy>=size) {{return;}} {} }}",body));
    }
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("production weave helper"),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("morphology probe"),
        layout: None,
        module: &module,
        entry_point: Some(if probe.is_some() {
            "probe"
        } else {
            "weave_pattern"
        }),
        compilation_options: Default::default(),
        cache: None,
    });
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
