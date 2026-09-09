//! Straightforward per-render allocations; all textures/uniforms live until readback.
use crate::{
    allocations::Allocations,
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
    pub allocations: Allocations,
}
impl Drop for Resources {
    fn drop(&mut self) {
        self.release();
    }
}
impl Resources {
    fn release(&mut self) {
        if self.uniforms.is_empty() && self.textures.is_empty() {
            return;
        }
        self.groups.clear();
        for buffer in self.uniforms.drain(..) {
            buffer.destroy();
        }
        for texture in self.textures.drain(..) {
            texture.destroy();
        }
        self.allocations.release_passes();
    }

    pub fn finish(mut self) -> crate::AllocationReport {
        self.release();
        self.allocations.report()
    }
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
        let texture_bytes = u64::from(texture.width())
            .checked_mul(u64::from(texture.height()))
            .and_then(|bytes| bytes.checked_mul(8))
            .ok_or_else(|| {
                GpuOperationError::at(
                    Stage::GpuExecution,
                    "Pass texture descriptor byte count overflowed.",
                )
            })?;
        self.allocations.pass(texture_bytes, uniform.size())?;
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
struct Staging<'a> {
    buffer: wgpu::Buffer,
    allocations: &'a mut Allocations,
}
impl Drop for Staging<'_> {
    fn drop(&mut self) {
        self.buffer.destroy();
        self.allocations.release_staging(self.buffer.size());
    }
}
pub(crate) async fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    layout: ReadbackLayout,
    kind: PortKind,
    allocations: &mut Allocations,
) -> Result<Vec<u8>, GpuOperationError> {
    let buffer = checked(
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
    .await?;
    allocations.staging(buffer.size())?;
    let buffer = Staging {
        buffer,
        allocations,
    };
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
                    buffer: &buffer.buffer,
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
    read_pixels(device, &buffer.buffer, layout, kind).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn readback_gpu_device_loss_releases_staging_and_keeps_the_mapping_failure() {
        let context =
            pollster::block_on(crate::GpuContext::request(crate::test_support::options())).unwrap();
        let layout = ReadbackLayout::new(33, 3).unwrap();
        let buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging before device destruction"),
            size: layout.buffer_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut allocations = Allocations::default();
        allocations.staging(buffer.size()).unwrap();
        let staging = Staging {
            buffer,
            allocations: &mut allocations,
        };
        context.device().destroy();
        let error = pollster::block_on(read_pixels(
            context.device(),
            &staging.buffer,
            layout,
            PortKind::Scalar,
        ))
        .unwrap_err();
        let original = error.diagnostic().clone();
        drop(staging);
        // Some backends fail mapping before its polling step. Deliver the native
        // callback explicitly and prove it cannot replace the first failure.
        context.device().poll(wgpu::PollType::Poll).unwrap();
        let error = error
            .with_allocations(allocations.report())
            .in_context(&context);
        assert_eq!(error.diagnostic().code, original.code);
        assert_eq!(error.diagnostic().stage, Stage::Readback);
        assert_eq!(error.diagnostic().message, original.message);
        assert!(std::error::Error::source(error.diagnostic()).is_some());
        assert_eq!(
            error.device_loss().unwrap().reason,
            crate::DeviceLossReason::Destroyed
        );
        assert_eq!(error.allocations().unwrap().staging_count, 1);
        assert_eq!(error.allocations().unwrap().live_bytes, 0);
        assert_eq!(
            error.allocations().unwrap().released_bytes,
            layout.buffer_bytes
        );
        eprintln!(
            "device-loss mapping cleanup: {}",
            serde_json::to_string(error.diagnostic()).unwrap()
        );
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn readback_gpu_copy_failure_releases_staging_allocation_and_preserves_first_error() {
        let context =
            pollster::block_on(crate::GpuContext::request(crate::test_support::options())).unwrap();
        let texture = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("intentional missing COPY_SRC usage"),
            size: extent([33, 3]),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let layout = ReadbackLayout::new(33, 3).unwrap();
        let mut allocations = Allocations::default();
        let error = pollster::block_on(read_texture(
            context.device(),
            context.queue(),
            &texture,
            layout,
            PortKind::Scalar,
            &mut allocations,
        ))
        .unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::Readback);
        // Backends may validate the copy when encoding it or finishing commands.
        assert!(matches!(
            error.diagnostic().message.as_str(),
            "Could not encode channel readback." | "Could not finish GPU commands."
        ));
        assert!(std::error::Error::source(error.diagnostic()).is_some());
        let report = allocations.report();
        assert_eq!(report.staging_count, 1);
        assert_eq!(report.staging_bytes, layout.buffer_bytes);
        assert_eq!(report.peak_bytes, layout.buffer_bytes);
        assert_eq!(report.released_bytes, layout.buffer_bytes);
        assert_eq!(report.live_bytes, 0);
        texture.destroy();
    }
}
