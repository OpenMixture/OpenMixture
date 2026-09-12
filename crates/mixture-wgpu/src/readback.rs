//! Checked rgba16float row layout, mapping, and conversion to tight RGBA8.

use crate::operation::{GpuOperationError, checked};
use mixture_core::{Stage, registry::PortKind};
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::{sync::mpsc, time::Duration};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadbackLayout {
    pub width: u32,
    pub height: u32,
    pub row_bytes: u32,
    pub padded_row_bytes: u32,
    pub buffer_bytes: u64,
    pub texture_bytes: u64,
    pub rgba_bytes: u64,
}
impl ReadbackLayout {
    pub fn new(width: u32, height: u32) -> Result<Self, GpuOperationError> {
        let invalid = || {
            GpuOperationError::at(
                Stage::Readback,
                "Invalid or overflowing readback dimensions.",
            )
        };
        if width == 0 || height == 0 {
            return Err(invalid());
        }
        let row_bytes = width.checked_mul(8).ok_or_else(invalid)?;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes =
            row_bytes.checked_add(alignment - 1).ok_or_else(invalid)? / alignment * alignment;
        let buffer_bytes = u64::from(padded_row_bytes)
            .checked_mul(u64::from(height))
            .ok_or_else(invalid)?;
        let texture_bytes = u64::from(row_bytes) * u64::from(height);
        let rgba_bytes = texture_bytes / 2;
        // Allocations and slice offsets must fit this host's address space too.
        usize::try_from(buffer_bytes).map_err(|_| invalid())?;
        usize::try_from(rgba_bytes).map_err(|_| invalid())?;
        Ok(Self {
            width,
            height,
            row_bytes,
            padded_row_bytes,
            buffer_bytes,
            texture_bytes,
            rgba_bytes,
        })
    }
}

pub(crate) async fn read_pixels(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    layout: ReadbackLayout,
    kind: PortKind,
) -> Result<Vec<u8>, GpuOperationError> {
    let result = async {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        let (sender, receiver) = mpsc::channel();
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        let (sender, receiver) = futures_channel::oneshot::channel();
        checked(
            device,
            Stage::Readback,
            "Could not map the texture readback buffer.",
            || {
                buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = sender.send(result);
                    });
            },
        )
        .await?;
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(|source| {
                GpuOperationError::source_error(
                    Stage::Readback,
                    "Timed out while mapping texture pixels.",
                    source,
                )
            })?;
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        let mapping = receiver.try_recv().map_err(|source| {
            GpuOperationError::source_error(
                Stage::Readback,
                "Readback mapping callback did not complete.",
                source,
            )
        });
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        let mapping = receiver.await.map_err(|source| {
            // Resolving the map callback requires returning to the event loop.
            GpuOperationError::source_error(
                Stage::Readback,
                "Readback mapping callback was dropped.",
                source,
            )
        });
        mapping?.map_err(|source| {
            GpuOperationError::source_error(Stage::Readback, "Readback mapping failed.", source)
        })?;
        // The view is dropped before the cleanup scope, including on invalid data.
        match buffer.slice(..).get_mapped_range() {
            Ok(view) => rgba16float_to_rgba8(&view, layout, kind),
            Err(source) => Err(GpuOperationError::source_error(
                Stage::Readback,
                "Could not access mapped texture pixels.",
                source,
            )),
        }
    }
    .await;
    // Unmapping a device-destroyed buffer can itself raise a validation error.
    // Always attempt cleanup, but never turn that secondary error into a panic
    // or replace the original map/access/conversion failure.
    let unmapped = checked(
        device,
        Stage::Readback,
        "Could not unmap the texture readback buffer.",
        || buffer.unmap(),
    )
    .await;
    match (result, unmapped) {
        (Err(error), Err(cleanup)) => Err(error.with_cleanup_error(cleanup)),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Ok(pixels), Ok(())) => Ok(pixels),
    }
}

fn rgba16float_to_rgba8(
    bytes: &[u8],
    layout: ReadbackLayout,
    kind: PortKind,
) -> Result<Vec<u8>, GpuOperationError> {
    if bytes.len() as u64 != layout.buffer_bytes {
        return Err(GpuOperationError::at(
            Stage::Readback,
            "Readback byte count does not match the padded layout.",
        )
        .evidence("expected", layout.buffer_bytes)
        .evidence("observed", bytes.len() as u64));
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(layout.rgba_bytes as usize)
        .map_err(|source| {
            GpuOperationError::source_error(
                Stage::Readback,
                "Could not allocate the RGBA output buffer.",
                source,
            )
        })?;
    for row in bytes.chunks_exact(layout.padded_row_bytes as usize) {
        for pixel in row[..layout.row_bytes as usize].as_chunks::<8>().0 {
            let mut values = [0.0; 4];
            for (component, pair) in pixel.as_chunks::<2>().0.iter().enumerate() {
                let value = half::f16::from_bits(u16::from_le_bytes(*pair)).to_f32();
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    return Err(GpuOperationError::at(
                        Stage::Readback,
                        "Texture readback contains a non-finite or out-of-range component.",
                    )
                    .evidence("component", (output.len() + component) as u64)
                    .evidence("value", value.to_string()));
                }
                values[component] = value;
            }
            match kind {
                PortKind::Color => {
                    for value in &mut values[..3] {
                        *value = linear_to_srgb(*value);
                    }
                }
                PortKind::Scalar => values = [values[0], values[0], values[0], 1.0],
                PortKind::Normal => {}
            }
            output.extend(values.map(|v| (v * 255.0).round() as u8));
        }
    }
    Ok(output)
}

// Transfer encoding at the readback boundary, never material computation.
fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readback_alignment_and_sizes_include_padding_without_overflow() {
        for (width, row, padded) in [
            (1, 8, 256),
            (31, 248, 256),
            (32, 256, 256),
            (33, 264, 512),
            (65, 520, 768),
        ] {
            let layout = ReadbackLayout::new(width, 3).unwrap();
            assert_eq!((layout.row_bytes, layout.padded_row_bytes), (row, padded));
            assert_eq!(layout.buffer_bytes, u64::from(padded) * 3);
            assert_eq!(layout.rgba_bytes, u64::from(width) * 3 * 4);
        }
        for (width, height) in [(0, 1), (1, 0), (u32::MAX, u32::MAX)] {
            assert!(ReadbackLayout::new(width, height).is_err());
        }
    }
    #[test]
    fn readback_strips_padding_preserves_rows_and_converts_half_values() {
        let layout = ReadbackLayout::new(1, 2).unwrap();
        let mut bytes = vec![0xff; layout.buffer_bytes as usize];
        for (row, values) in [(0, [0.0, 0.5, 1.0, 1.0]), (1, [1.0, 0.25, 0.0, 1.0])] {
            for (channel, value) in values.into_iter().enumerate() {
                let offset = row * layout.padded_row_bytes as usize + channel * 2;
                bytes[offset..offset + 2]
                    .copy_from_slice(&half::f16::from_f32(value).to_bits().to_le_bytes());
            }
        }
        assert_eq!(
            rgba16float_to_rgba8(&bytes, layout, PortKind::Normal).unwrap(),
            [0, 128, 255, 255, 255, 64, 0, 255]
        );
        assert!(rgba16float_to_rgba8(&bytes[..511], layout, PortKind::Normal).is_err());
        for invalid in [f32::NAN, f32::INFINITY, -0.5, 2.0] {
            bytes[..2].copy_from_slice(&half::f16::from_f32(invalid).to_bits().to_le_bytes());
            let error = rgba16float_to_rgba8(&bytes, layout, PortKind::Normal).unwrap_err();
            assert_eq!(
                error.diagnostic().code,
                mixture_core::DiagnosticCode::ReadbackFailed
            );
            assert_eq!(error.diagnostic().stage, Stage::Readback);
        }
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;
    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn readback_gpu_mapping_failure_is_typed_and_does_not_poison_context() {
        let mut context =
            pollster::block_on(crate::GpuContext::request(crate::test_support::options())).unwrap();
        let buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("intentional missing MAP_READ usage"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let error = pollster::block_on(read_pixels(
            context.device(),
            &buffer,
            ReadbackLayout::new(1, 1).unwrap(),
            PortKind::Color,
        ))
        .unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::Readback);
        assert_eq!(
            error.diagnostic().code,
            mixture_core::DiagnosticCode::ReadbackFailed
        );
        assert!(std::error::Error::source(error.diagnostic()).is_some());
        let report = pollster::block_on(context.probe_checker());
        assert_eq!(report.verdict(), crate::DoctorVerdict::Healthy);
    }
}
