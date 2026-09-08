//! The sole graph executor: explicit context, bounded pipeline cache, typed passes.
use crate::{
    AdapterDiagnostics, AllocationReport, ExecutionTimings, GpuContext, GpuOperationError,
    kernels::{PipelineCache, PipelineCacheReport},
    operation::checked,
    readback::ReadbackLayout,
    resources::{self, Resources},
};
use mixture_core::{
    InputSource, OutputChannel, RenderPlan, Stage,
    plan::{KernelInvocation, PLAN_VERSION, PlanEstimates, PlanHash},
    registry::PortKind,
};
use serde::Serialize;
use std::time::Instant;

/// Pixel transfer encoding; alpha is always linear and straight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum OutputEncoding {
    /// Linear RGB converted to sRGB, linear alpha quantized to eight bits.
    #[serde(rename = "rgba8-srgb")]
    Srgb,
    /// Linear scalar replicated to RGB, or encoded normal XYZ, with opaque alpha.
    #[serde(rename = "rgba8-linear")]
    Linear,
}
impl OutputEncoding {
    pub(crate) fn for_kind(kind: PortKind) -> Self {
        match kind {
            PortKind::Color => Self::Srgb,
            PortKind::Scalar | PortKind::Normal => Self::Linear,
        }
    }
}
/// CPU-owned channel pixels. Source/default provenance comes directly from the plan.
#[derive(Debug)]
pub struct RenderedChannel {
    /// Requested material channel.
    pub channel: OutputChannel,
    /// Logical channel interpretation.
    pub kind: PortKind,
    /// Connected endpoint or versioned default that produced this channel.
    pub source: InputSource,
    /// Width and height in pixels.
    pub size: [u32; 2],
    /// Transfer encoding of the returned bytes.
    pub encoding: OutputEncoding,
    pixels: Vec<u8>,
}
impl RenderedChannel {
    /// Tight, top-left, row-major RGBA8 bytes, independent of renderer lifetime.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
/// Actual adapter and execution evidence for a completed graph render.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderReport {
    /// Stable hash of the exact plan executed.
    pub plan_hash: PlanHash,
    /// Actual selected adapter/backend.
    pub adapter: AdapterDiagnostics,
    /// Requested output dimensions.
    pub size: [u32; 2],
    /// Number of compute passes actually encoded and completed.
    pub pass_count: usize,
    /// Lookup counts and retained pipeline count for this renderer.
    pub pipeline_cache: PipelineCacheReport,
    /// Logical peak/cumulative allocation estimates, excluding driver overhead.
    pub estimates: PlanEstimates,
    /// Successful descriptor allocations and destruction, excluding driver overhead.
    pub allocations: AllocationReport,
    /// Tight raw rgba16float channel bytes read from the GPU.
    pub readback_bytes: u64,
    /// Total padded staging bytes mapped, sequentially across channels.
    pub mapped_bytes: u64,
    /// Total returned RGBA8 bytes.
    pub rgba_bytes: u64,
    /// CPU wall times; no GPU timestamp measurements are claimed.
    pub timings: ExecutionTimings,
}
/// Complete CPU-owned render result, with no retained GPU textures or buffers.
#[derive(Debug)]
pub struct RenderOutput {
    channels: Vec<RenderedChannel>,
    report: RenderReport,
}
impl RenderOutput {
    /// Requested outputs in canonical material-contract order.
    pub fn channels(&self) -> &[RenderedChannel] {
        &self.channels
    }
    /// Metrics and actual adapter evidence.
    pub fn report(&self) -> &RenderReport {
        &self.report
    }
}
/// Owns a single context and at most one pipeline per kernel. No resource pool or global cache.
pub struct Renderer {
    context: GpuContext,
    cache: PipelineCache,
}
impl Renderer {
    /// Use the explicitly acquired context without selecting another adapter.
    pub fn new(context: GpuContext) -> Self {
        Self {
            context,
            cache: PipelineCache::default(),
        }
    }
    /// Borrow context ownership and acquisition evidence.
    /// Its raw wgpu escape hatches can still mutate shared device state.
    pub fn context(&self) -> &GpuContext {
        &self.context
    }
    /// Number of retained pipelines, bounded by the built-in kernel count.
    pub fn cached_pipeline_count(&self) -> usize {
        self.cache.len()
    }
    /// Explicitly release cached pipelines; later renders recreate them.
    pub fn clear_pipeline_cache(&mut self) {
        self.cache.clear();
    }
    /// Execute an immutable compiler-produced plan and read only requested channels.
    /// All per-call allocations are released on success or failure. Each native wait
    /// is bounded to 30 seconds. This is a native headless API, not a browser API.
    /// Although expressed as a future, native device polling may block its thread.
    /// Consumers needing a responsive event loop should own the renderer on their
    /// explicitly managed worker. This method does not promise cancellation.
    pub async fn render(&mut self, plan: &RenderPlan) -> Result<RenderOutput, GpuOperationError> {
        if plan.version() != PLAN_VERSION {
            return Err(GpuOperationError::at(
                Stage::GpuExecution,
                "Unsupported render plan version.",
            ));
        }
        let kernels: Vec<_> = plan.passes().iter().map(|p| &p.kernel).collect();
        let mappings: Vec<_> = plan
            .outputs()
            .iter()
            .map(|o| (o.resource.index() as usize, o.kind))
            .collect();
        let result = execute(
            &self.context,
            &mut self.cache,
            plan.size(),
            &kernels,
            &mappings,
        )
        .await
        .map_err(|error| error.in_plan(plan))?;
        let channels = plan
            .outputs()
            .iter()
            .zip(result.pixels)
            .map(|(output, pixels)| RenderedChannel {
                channel: output.channel,
                kind: output.kind,
                source: output.input.clone(),
                size: plan.size(),
                encoding: OutputEncoding::for_kind(output.kind),
                pixels,
            })
            .collect();
        let report = RenderReport {
            plan_hash: plan.hash().clone(),
            adapter: AdapterDiagnostics::from_adapter(self.context.adapter()),
            size: plan.size(),
            pass_count: kernels.len(),
            pipeline_cache: result.cache,
            estimates: plan.estimates().clone(),
            allocations: result.allocations,
            readback_bytes: plan.estimates().readback_bytes,
            mapped_bytes: plan.estimates().cumulative_readback_bytes,
            rgba_bytes: plan.estimates().readback_bytes / 2,
            timings: result.timings,
        };
        Ok(RenderOutput { channels, report })
    }
}

pub(crate) struct Executed {
    pub pixels: Vec<Vec<u8>>,
    pub cache: PipelineCacheReport,
    pub timings: ExecutionTimings,
    pub allocations: AllocationReport,
}
// The fixed probe also uses this exact dispatch, upload, allocation and readback path.
pub(crate) async fn execute(
    context: &GpuContext,
    cache: &mut PipelineCache,
    size: [u32; 2],
    kernels: &[&KernelInvocation],
    outputs: &[(usize, PortKind)],
) -> Result<Executed, GpuOperationError> {
    let total = Instant::now();
    let layout = ReadbackLayout::new(size[0], size[1])?;
    let device = context.device();
    let limits = device.limits();
    let dispatch = [size[0].div_ceil(8), size[1].div_ceil(8), 1];
    if size.iter().any(|n| *n > limits.max_texture_dimension_2d)
        || layout.buffer_bytes > limits.max_buffer_size
        || dispatch
            .iter()
            .any(|n| *n > limits.max_compute_workgroups_per_dimension)
    {
        return Err(GpuOperationError::at(
            Stage::GpuExecution,
            "Render request exceeds acquired device limits.",
        )
        .evidence(
            "maxTextureDimension2D",
            u64::from(limits.max_texture_dimension_2d),
        )
        .evidence("maxBufferSize", limits.max_buffer_size)
        .evidence(
            "maxComputeWorkgroupsPerDimension",
            u64::from(limits.max_compute_workgroups_per_dimension),
        ));
    }
    let pipeline_started = Instant::now();
    let mut cache_report = PipelineCacheReport {
        entries: cache.len(),
        ..Default::default()
    };
    let mut pipelines = Vec::new();
    for (index, kernel) in kernels.iter().enumerate() {
        pipelines.push(
            cache
                .get(device, kernel.id(), &mut cache_report)
                .await
                .map_err(|error| {
                    error
                        .evidence("passIndex", index as u64)
                        .evidence("kernel", format!("{:?}", kernel.id()))
                })?,
        );
    }
    let pipeline_ms = pipeline_started.elapsed().as_secs_f64() * 1000.;
    let mut resources = Resources::default();
    for (index, (kernel, pipeline)) in kernels.iter().zip(&pipelines).enumerate() {
        resources
            .push(device, pipeline, kernel, size)
            .await
            .map_err(|error| {
                error
                    .evidence("passIndex", index as u64)
                    .evidence("kernel", format!("{:?}", kernel.id()))
            })?;
    }
    let execution_started = Instant::now();
    let encoder = checked(
        device,
        Stage::GpuExecution,
        "Could not encode compute passes.",
        || {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("material graph"),
            });
            for (pipeline, group) in pipelines.iter().zip(&resources.groups) {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("material pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, group, &[]);
                pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
            }
            encoder
        },
    )
    .await?;
    resources::submit(device, context.queue(), encoder, Stage::GpuExecution).await?;
    let execution_ms = execution_started.elapsed().as_secs_f64() * 1000.;
    let readback_started = Instant::now();
    let mut pixels = Vec::new();
    for (resource, kind) in outputs {
        let texture = resources.textures.get(*resource).ok_or_else(|| {
            GpuOperationError::at(Stage::Readback, "Output resource is missing.").after_compute()
        })?;
        pixels.push(
            resources::read_texture(
                device,
                context.queue(),
                texture,
                layout,
                *kind,
                &mut resources.allocations,
            )
            .await
            .map_err(GpuOperationError::after_compute)?,
        );
    }
    let readback_ms = readback_started.elapsed().as_secs_f64() * 1000.;
    let allocations = resources.finish();
    Ok(Executed {
        pixels,
        cache: cache_report,
        allocations,
        timings: ExecutionTimings {
            pipeline_ms,
            execution_ms,
            readback_ms,
            total_ms: total.elapsed().as_secs_f64() * 1000.,
        },
    })
}

#[cfg(test)]
mod allocation_tests {
    use super::*;
    use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn graph_gpu_allocation_accounting_matches_plan_and_releases_aliased_readbacks() {
        let mut document = MaterialDocument::decode(
            include_bytes!("../../../fixtures/nodes/constant-scalar/input.mix"),
            &SafetyLimits::default(),
        )
        .unwrap();
        document.edges.push(mixture_core::document::Edge {
            from: mixture_core::document::Endpoint {
                node_id: "scalar".into(),
                port_id: "value".into(),
            },
            to: mixture_core::document::Endpoint {
                node_id: "out".into(),
                port_id: "height".into(),
            },
        });
        let document = document.into_validated(&SafetyLimits::default()).unwrap();
        let plan = compile(
            &document,
            &CompileRequest {
                size: [33, 3],
                outputs: vec![
                    OutputChannel::BaseColor,
                    OutputChannel::Roughness,
                    OutputChannel::Height,
                ],
                ..Default::default()
            },
        )
        .unwrap();
        let context =
            pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
        let mut renderer = Renderer::new(context);
        for _ in 0..2 {
            let output = pollster::block_on(renderer.render(&plan)).unwrap();
            let measured = &output.report().allocations;
            let estimated = plan.estimates();
            assert_eq!(measured.texture_count, 2);
            assert_eq!(measured.uniform_count, 2);
            assert_eq!(measured.staging_count, 3);
            assert_eq!(measured.texture_bytes, estimated.texture_bytes);
            assert_eq!(measured.uniform_bytes, estimated.uniform_bytes);
            assert_eq!(measured.staging_bytes, estimated.cumulative_readback_bytes);
            assert_eq!(measured.peak_staging_bytes, estimated.readback_buffer_bytes);
            assert_eq!(measured.cumulative_bytes, estimated.cumulative_bytes);
            assert_eq!(measured.peak_bytes, estimated.peak_bytes);
            assert_eq!(measured.released_bytes, measured.cumulative_bytes);
            assert_eq!(measured.live_bytes, 0);
            assert_eq!(measured.reused_bytes, 0);
            assert_eq!(estimated.padded_bytes_per_row, 512);
            assert_eq!(output.channels()[1].pixels(), output.channels()[2].pixels());
        }
    }
}
