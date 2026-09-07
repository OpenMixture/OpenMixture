//! Straightforward per-render allocations; all textures/uniforms live until readback.
use crate::{
    operation::{GpuOperationError, checked},
    readback::{ReadbackLayout, read_pixels},
};
use mixture_core::{Stage, plan::KernelInvocation, registry::PortKind};
use std::time::Duration;

#[derive(Default)]
pub(crate) struct Resources {
    pub textures: Vec<wgpu::Texture>,
    uniforms: Vec<wgpu::Buffer>,
    pub groups: Vec<wgpu::BindGroup>,
}
impl Drop for Resources {
    fn drop(&mut self) {
        for buffer in &self.uniforms {
            buffer.destroy();
        }
        for texture in &self.textures {
            texture.destroy();
        }
    }
}
impl Resources {
    pub async fn push(
        &mut self,
        device: &wgpu::Device,
        pipeline: &wgpu::ComputePipeline,
        kernel: &KernelInvocation,
        size: [u32; 2],
    ) -> Result<(), GpuOperationError> {
        let bytes = crate::kernels::parameters(kernel);
        let inputs = kernel
            .inputs()
            .map(|id| {
                self.textures.get(id.index() as usize).ok_or_else(|| {
                    GpuOperationError::at(
                        Stage::GpuExecution,
                        "Plan input resource has no preceding producer.",
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (texture, uniform, group) = checked(
            device,
            Stage::GpuExecution,
            "Could not allocate compute resources.",
            || {
                let input_views: Vec<_> = inputs
                    .iter()
                    .map(|texture| texture.create_view(&Default::default()))
                    .collect();
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("plan rgba16float"),
                    size: extent(size),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba16Float,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("typed kernel parameters"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: true,
                });
                let output_view = texture.create_view(&Default::default());
                let mut entries = vec![
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&output_view),
                    },
                ];
                entries.extend(input_views.iter().enumerate().map(|(index, view)| {
                    wgpu::BindGroupEntry {
                        binding: index as u32 + 2,
                        resource: wgpu::BindingResource::TextureView(view),
                    }
                }));
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("plan bindings"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &entries,
                });
                (texture, uniform, group)
            },
        )
        .await?;
        // Retain allocations before fallible upload so the guard also cleans errors.
        self.textures.push(texture);
        self.uniforms.push(uniform);
        self.groups.push(group);
        let uniform = self.uniforms.last().ok_or_else(|| {
            GpuOperationError::at(Stage::GpuExecution, "Missing uniform allocation.")
        })?;
        let uploaded = match uniform.get_mapped_range_mut(..) {
            Ok(mut view) => {
                view.slice(..).copy_from_slice(&bytes);
                Ok(())
            }
            Err(source) => Err(GpuOperationError::source_error(
                Stage::GpuExecution,
                "Could not upload typed parameters.",
                source,
            )),
        };
        let unmapped = checked(
            device,
            Stage::GpuExecution,
            "Could not unmap typed parameters.",
            || uniform.unmap(),
        )
        .await;
        uploaded?;
        unmapped
    }
}
fn extent(size: [u32; 2]) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    }
}
pub(crate) async fn submit(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: wgpu::CommandEncoder,
    stage: Stage,
) -> Result<(), GpuOperationError> {
    let commands = checked(device, stage, "Could not finish GPU commands.", || {
        encoder.finish()
    })
    .await?;
    let submission = checked(device, stage, "Could not submit GPU commands.", || {
        queue.submit([commands])
    })
    .await?;
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(30)),
        })
        .map_err(|source| {
            GpuOperationError::source_error(stage, "GPU commands did not complete.", source)
        })?;
    Ok(())
}
struct Staging(wgpu::Buffer);
impl Drop for Staging {
    fn drop(&mut self) {
        self.0.destroy();
    }
}
pub(crate) async fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    layout: ReadbackLayout,
    kind: PortKind,
) -> Result<Vec<u8>, GpuOperationError> {
    let buffer = Staging(
        checked(
            device,
            Stage::Readback,
            "Could not allocate staging buffer.",
            || {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("channel readback"),
                    size: layout.buffer_bytes,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            },
        )
        .await?,
    );
    let encoder = checked(
        device,
        Stage::Readback,
        "Could not encode channel readback.",
        || {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("channel copy"),
            });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer.0,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(layout.padded_row_bytes),
                        rows_per_image: Some(layout.height),
                    },
                },
                extent([layout.width, layout.height]),
            );
            encoder
        },
    )
    .await?;
    submit(device, queue, encoder, Stage::Readback).await?;
    read_pixels(device, &buffer.0, layout, kind).await
}
