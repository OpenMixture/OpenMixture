//! Fixed checker probe using the same kernel and execution path as graph rendering.

use crate::{
    AdapterDiagnostics, GpuContext, operation::GpuOperationError, readback::ReadbackLayout,
};
use mixture_core::{SafetyLimits, Stage, limits::LimitKind};
use mixture_core::{plan::KernelInvocation, registry::PortKind};
use serde::Serialize;

#[cfg(test)]
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
        request.validate(limits)?;
        let layout = ReadbackLayout::new(request.width, request.height)?;
        let kernel = KernelInvocation::Checker {
            cells: [8, 8],
            color_a: [0., 0., 0., 1.],
            color_b: [1., 1., 1., 1.],
        };
        let result = crate::executor::execute(
            self,
            &mut crate::kernels::PipelineCache::default(),
            [request.width, request.height],
            &[&kernel],
            &[(0, PortKind::Color)],
        )
        .await?;
        let pixels =
            result.pixels.into_iter().next().ok_or_else(|| {
                GpuOperationError::at(Stage::Readback, "Checker output is missing.")
            })?;
        let report = ExecutionReport {
            adapter: AdapterDiagnostics::from_adapter(self.adapter()),
            width: request.width,
            height: request.height,
            texture_format: "rgba16float",
            rgba_encoding: "rgba8-srgb",
            pass_count: 1,
            dispatch: [request.width.div_ceil(8), request.height.div_ceil(8), 1],
            readback_bytes: layout.texture_bytes,
            padded_bytes_per_row: layout.padded_row_bytes,
            mapped_bytes: layout.buffer_bytes,
            rgba_bytes: layout.rgba_bytes,
            estimated_gpu_bytes: estimated_bytes(layout)?,
            timings: result.timings,
        };
        Ok(CheckerOutput { pixels, report })
    }
}

fn estimated_bytes(layout: ReadbackLayout) -> Result<u64, GpuOperationError> {
    layout
        .texture_bytes
        .checked_add(layout.buffer_bytes)
        .and_then(|bytes| bytes.checked_add(48))
        .ok_or_else(|| {
            GpuOperationError::at(Stage::Validation, "Checker memory estimate overflowed.")
        })
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
        let required = 64 * 64 * 16 + 48;
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
    async fn checker_pipeline(
        device: &wgpu::Device,
        source: &str,
    ) -> Result<wgpu::ComputePipeline, GpuOperationError> {
        crate::kernels::create_pipeline(device, source, "checker").await
    }
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
