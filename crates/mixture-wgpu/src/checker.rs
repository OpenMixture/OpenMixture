//! One fixed checker compute pass. No graph, registry, cache, or general renderer.

use crate::{
    AdapterDiagnostics, GpuContext,
    operation::{GpuOperationError, checked},
    readback::{ReadbackLayout, read_rgba8},
};
use mixture_core::{SafetyLimits, Stage, limits::LimitKind};
use serde::Serialize;
use std::time::{Duration, Instant};

const SHADER: &str = include_str!("../shaders/nodes/checker.wgsl");

/// Output dimensions for an opaque, eight-by-eight black/white checker.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct CheckerRequest {
    /// Output width in pixels, positive and within explicit safety/device limits.
    pub width: u32,
    /// Output height in pixels, positive and within explicit safety/device limits.
    pub height: u32,
}
impl Default for CheckerRequest {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
        }
    }
}
impl CheckerRequest {
    /// Validate dimensions and estimated GPU allocation before acquiring a context.
    pub fn validate(self, limits: &SafetyLimits) -> Result<(), GpuOperationError> {
        if self.width == 0
            || self.height == 0
            || self.width.checked_mul(8).is_none()
            || self.height.checked_mul(8).is_none()
        {
            return Err(GpuOperationError::at(
                Stage::Validation,
                "Checker dimensions must be positive and support eight cells per axis.",
            )
            .evidence("width", u64::from(self.width))
            .evidence("height", u64::from(self.height)));
        }
        for size in [self.width, self.height] {
            limits
                .check(LimitKind::OutputDimension, u64::from(size))
                .map_err(|error| GpuOperationError::from(error.diagnostic(Stage::Validation)))?;
        }
        limits
            .check(LimitKind::RequestedOutputs, 1)
            .map_err(|error| GpuOperationError::from(error.diagnostic(Stage::Validation)))?;
        let layout = ReadbackLayout::new(self.width, self.height)?;
        limits
            .check(LimitKind::TransientBytes, estimated_bytes(layout)?)
            .map_err(|error| GpuOperationError::from(error.diagnostic(Stage::Validation)))?;
        Ok(())
    }
}

/// Measured CPU wall-clock milliseconds; no GPU timestamp query is implied.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionTimings {
    /// Shader and pipeline preparation.
    pub pipeline_ms: f64,
    /// Command encoding, submission, and waiting for completion.
    pub execution_ms: f64,
    /// Mapping, row unpacking, and RGBA conversion.
    pub readback_ms: f64,
    /// Complete render call, including allocation and validation.
    pub total_ms: f64,
}

/// Evidence for one completed checker dispatch and readback.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReport {
    /// Actual adapter identity and capabilities.
    pub adapter: AdapterDiagnostics,
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Offscreen GPU storage format.
    pub texture_format: &'static str,
    /// Image-ready output encoding; black/white RGB endpoints are also valid sRGB.
    pub rgba_encoding: &'static str,
    /// Number of executed compute passes.
    pub pass_count: u32,
    /// Submitted workgroup counts.
    pub dispatch: [u32; 3],
    /// Tight rgba16float bytes, excluding copy-row padding.
    pub readback_bytes: u64,
    /// GPU copy row stride, aligned to 256 bytes.
    pub padded_bytes_per_row: u32,
    /// Entire mapped buffer, including row padding.
    pub mapped_bytes: u64,
    /// Tight RGBA8 output bytes.
    pub rgba_bytes: u64,
    /// Logical texture, readback buffer, and uniform bytes; excludes driver overhead.
    pub estimated_gpu_bytes: u64,
    /// CPU wall-clock stage durations.
    pub timings: ExecutionTimings,
}

/// CPU-owned image and evidence. No GPU resource is retained after return.
#[derive(Debug)]
pub struct CheckerOutput {
    pixels: Vec<u8>,
    report: ExecutionReport,
}
impl CheckerOutput {
    /// Tight row-major RGBA8 pixels, top-left origin, with opaque alpha.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    /// Actual execution and readback evidence.
    pub fn report(&self) -> &ExecutionReport {
        &self.report
    }
}

impl GpuContext {
    /// Execute and read back one checker using this explicit context.
    ///
    /// Resources belong to this call and are dropped on success or error. The
    /// native GPU wait is bounded to 30 seconds; this is not a browser API.
    pub async fn render_checker(
        &mut self,
        request: CheckerRequest,
        limits: &SafetyLimits,
    ) -> Result<CheckerOutput, GpuOperationError> {
        let total = Instant::now();
        request.validate(limits)?;
        let layout = ReadbackLayout::new(request.width, request.height)?;
        let device = self.device();
        let max_dimension = device.limits().max_texture_dimension_2d;
        if request.width > max_dimension
            || request.height > max_dimension
            || layout.buffer_bytes > device.limits().max_buffer_size
        {
            return Err(GpuOperationError::at(
                Stage::GpuExecution,
                "Checker request exceeds the acquired device limits.",
            )
            .evidence("maxTextureDimension2D", u64::from(max_dimension))
            .evidence("maxBufferSize", device.limits().max_buffer_size));
        }
        let prepare = Instant::now();
        let pipeline = checker_pipeline(device, SHADER).await?;
        let pipeline_ms = prepare.elapsed().as_secs_f64() * 1000.0;
        let extent = wgpu::Extent3d {
            width: layout.width,
            height: layout.height,
            depth_or_array_layers: 1,
        };
        let (texture, uniform, readback, group) = checked(
            device,
            Stage::GpuExecution,
            "Could not allocate checker resources.",
            || {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("checker rgba16float"),
                    size: extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba16Float,
                    usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("checker parameters"),
                    size: 16,
                    usage: wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: true,
                });
                let readback = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("checker readback"),
                    size: layout.buffer_bytes,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let view = texture.create_view(&Default::default());
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("checker bindings"),
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
                (texture, uniform, readback, group)
            },
        )
        .await?;
        let parameters: Vec<_> = [request.width, request.height, 8, 8]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect();
        // Avoid DeviceExt::create_buffer_init: it panics when mapping an invalid
        // device allocation. Each native failure must retain a Mixture stage.
        let upload = match uniform.get_mapped_range_mut(..) {
            Ok(mut view) => {
                view.slice(..).copy_from_slice(&parameters);
                Ok(())
            }
            Err(source) => Err(GpuOperationError::source_error(
                Stage::GpuExecution,
                "Could not upload checker parameters.",
                source,
            )),
        };
        let unmap = checked(
            device,
            Stage::GpuExecution,
            "Could not unmap checker parameters.",
            || uniform.unmap(),
        )
        .await;
        upload?;
        unmap?;
        let execution = Instant::now();
        let dispatch = [request.width.div_ceil(8), request.height.div_ceil(8), 1];
        let mut encoder = checked(
            device,
            Stage::GpuExecution,
            "Could not encode checker compute pass.",
            || {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("checker commands"),
                });
                {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("checker"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &group, &[]);
                    pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
                }
                encoder
            },
        )
        .await?;
        checked(
            device,
            Stage::Readback,
            "Could not encode checker texture readback.",
            || {
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
                            bytes_per_row: Some(layout.padded_row_bytes),
                            rows_per_image: Some(request.height),
                        },
                    },
                    extent,
                );
            },
        )
        .await?;
        let commands = checked(
            device,
            Stage::GpuExecution,
            "Could not finish checker commands.",
            || encoder.finish(),
        )
        .await?;
        let submission = checked(
            device,
            Stage::GpuExecution,
            "Could not submit checker commands.",
            || self.queue().submit([commands]),
        )
        .await?;
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(|source| {
                GpuOperationError::source_error(
                    Stage::GpuExecution,
                    "Checker execution did not complete.",
                    source,
                )
            })?;
        let execution_ms = execution.elapsed().as_secs_f64() * 1000.0;
        let readback_started = Instant::now();
        let pixels = read_rgba8(device, &readback, layout)
            .await
            .map_err(GpuOperationError::after_compute)?;
        let readback_ms = readback_started.elapsed().as_secs_f64() * 1000.0;
        // No pooling: explicitly release each allocation after the mapping is closed.
        readback.destroy();
        uniform.destroy();
        texture.destroy();
        let report = ExecutionReport {
            adapter: AdapterDiagnostics::from_adapter(self.adapter()),
            width: request.width,
            height: request.height,
            texture_format: "rgba16float",
            rgba_encoding: "rgba8-srgb",
            pass_count: 1,
            dispatch,
            readback_bytes: layout.texture_bytes,
            padded_bytes_per_row: layout.padded_row_bytes,
            mapped_bytes: layout.buffer_bytes,
            rgba_bytes: layout.rgba_bytes,
            estimated_gpu_bytes: estimated_bytes(layout)?,
            timings: ExecutionTimings {
                pipeline_ms,
                execution_ms,
                readback_ms,
                total_ms: total.elapsed().as_secs_f64() * 1000.0,
            },
        };
        Ok(CheckerOutput { pixels, report })
    }
}

fn estimated_bytes(layout: ReadbackLayout) -> Result<u64, GpuOperationError> {
    layout
        .texture_bytes
        .checked_add(layout.buffer_bytes)
        .and_then(|bytes| bytes.checked_add(16))
        .ok_or_else(|| {
            GpuOperationError::at(Stage::Validation, "Checker memory estimate overflowed.")
        })
}

pub(crate) async fn checker_pipeline(
    device: &wgpu::Device,
    source: &str,
) -> Result<wgpu::ComputePipeline, GpuOperationError> {
    let module = checked(
        device,
        Stage::GpuShader,
        "Checker shader validation failed.",
        || {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("checker shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
        },
    )
    .await?;
    checked(
        device,
        Stage::GpuPipeline,
        "Checker compute pipeline creation failed.",
        || {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("checker pipeline"),
                layout: None,
                module: &module,
                entry_point: Some("checker"),
                compilation_options: Default::default(),
                cache: None,
            })
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checker_validation_rejects_bad_dimensions_and_budget_before_gpu() {
        let limits = SafetyLimits::default();
        CheckerRequest::default().validate(&limits).unwrap();
        for (width, height) in [(0, 64), (64, 0), (2049, 64), (64, 2049), (u32::MAX, 1)] {
            assert_eq!(
                CheckerRequest { width, height }
                    .validate(&limits)
                    .unwrap_err()
                    .diagnostic()
                    .stage,
                Stage::Validation
            );
        }
        let no_outputs = CheckerRequest::default()
            .validate(&SafetyLimits {
                requested_outputs: 0,
                ..limits
            })
            .unwrap_err();
        assert_eq!(
            no_outputs.diagnostic().code,
            mixture_core::DiagnosticCode::LimitRequestedOutputsExceeded
        );
        let required = 64 * 64 * 16 + 16;
        CheckerRequest::default()
            .validate(&SafetyLimits {
                transient_bytes: required,
                ..limits
            })
            .unwrap();
        let error = CheckerRequest::default()
            .validate(&SafetyLimits {
                transient_bytes: required - 1,
                ..limits
            })
            .unwrap_err();
        assert_eq!(
            error.diagnostic().code,
            mixture_core::DiagnosticCode::LimitTransientBytesExceeded
        );
    }
}

impl GpuContext {
    /// Run the actual 64x64 checker and validate fixed samples and channel counts.
    /// Only a completed and verified compute/readback can produce `healthy`.
    pub async fn probe_checker(&mut self) -> crate::ContextReport {
        let mut report = self.report().clone();
        match self
            .render_checker(CheckerRequest::default(), &SafetyLimits::default())
            .await
        {
            Ok(output) => match verify_probe(&output.pixels) {
                Ok(()) => report.probe_passed(output.report),
                Err(error) => report.probe_failed(error.diagnostic().clone(), true),
            },
            Err(error) => {
                report.probe_failed(error.diagnostic().clone(), error.compute_completed())
            }
        }
        report
    }
}

fn verify_probe(pixels: &[u8]) -> Result<(), GpuOperationError> {
    let failed = || {
        GpuOperationError::at(
            Stage::Readback,
            "Checker probe pixels do not match the fixed 64x64 contract.",
        )
    };
    if pixels.len() != 64 * 64 * 4 {
        return Err(failed().evidence("observedBytes", pixels.len() as u64));
    }
    let (mut black, mut white) = (0, 0);
    for pixel in pixels.as_chunks::<4>().0 {
        match pixel {
            [0, 0, 0, 255] => black += 1,
            [255, 255, 255, 255] => white += 1,
            _ => return Err(failed()),
        }
    }
    if black != 2048 || white != 2048 {
        return Err(failed()
            .evidence("blackPixels", black as u64)
            .evidence("whitePixels", white as u64));
    }
    // Fixed sentinels, not a CPU implementation of the shader.
    for (x, y, value) in [
        (0, 0, 0),
        (7, 7, 0),
        (8, 0, 255),
        (0, 8, 255),
        (8, 8, 0),
        (63, 0, 255),
        (63, 63, 0),
    ] {
        let offset = (y * 64 + x) * 4;
        if pixels[offset..offset + 4] != [value, value, value, 255] {
            return Err(failed().evidence("x", x as u64).evidence("y", y as u64));
        }
    }
    Ok(())
}

#[cfg(test)]
mod shader_tests {
    use super::*;
    #[test]
    fn checker_shader_validates_without_a_gpu() {
        let module = naga::front::wgsl::parse_str(SHADER).expect("checker WGSL parses");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("portable checker WGSL validates");
        assert_eq!(module.entry_points.len(), 1);
        assert_eq!(module.entry_points[0].name, "checker");
        assert_eq!(module.entry_points[0].workgroup_size, [8, 8, 1]);
    }

    #[test]
    fn checker_probe_rejects_empty_and_uniform_images() {
        assert!(verify_probe(&[]).is_err());
        assert!(verify_probe(&vec![255; 64 * 64 * 4]).is_err());
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn checker_gpu_shader_pipeline_and_device_failures_preserve_stage() {
        let mut context =
            pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
        let error = pollster::block_on(checker_pipeline(context.device(), "this is invalid WGSL"))
            .unwrap_err();
        assert_eq!(
            error.diagnostic().code,
            mixture_core::DiagnosticCode::GpuShaderValidationFailed
        );
        assert_eq!(error.diagnostic().stage, Stage::GpuShader);
        assert!(std::error::Error::source(error.diagnostic()).is_some());
        let wrong_entry = SHADER.replace("fn checker(", "fn different_entry(");
        let error =
            pollster::block_on(checker_pipeline(context.device(), &wrong_entry)).unwrap_err();
        assert_eq!(error.diagnostic().stage, Stage::GpuPipeline);
        assert_eq!(
            error.diagnostic().code,
            mixture_core::DiagnosticCode::GpuExecutionFailed
        );
        // Errors must not poison or leak error scopes into the next operation.
        pollster::block_on(
            context.render_checker(CheckerRequest::default(), &SafetyLimits::default()),
        )
        .unwrap();
        context.device().destroy();
        let error = pollster::block_on(
            context.render_checker(CheckerRequest::default(), &SafetyLimits::default()),
        )
        .unwrap_err();
        assert!(matches!(
            error.diagnostic().stage,
            Stage::GpuShader | Stage::GpuPipeline | Stage::GpuExecution
        ));
        assert!(!error.compute_completed());
    }
}
