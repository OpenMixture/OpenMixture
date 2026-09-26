//! The sole graph executor: explicit context, bounded pipeline cache, typed passes.
use crate::{
    AdapterDiagnostics, AllocationReport, ExecutionTimings, GpuContext, GpuOperationError,
    kernels::{PipelineCache, PipelineCacheReport},
    operation::checked,
    readback::ReadbackLayout,
    resources::{self, Resources},
};
use mixture_core::{
    InputSource, OutputChannel, PreparedRender, RenderPlan, ResourceSnapshot, Stage,
    plan::{KernelInvocation, PLAN_VERSION, PlanEstimates, PlanHash},
    registry::PortKind,
};
use serde::Serialize;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::Instant;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use web_time::Instant;

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
/// Owns one context and bounded pipelines. Physical slots are local to each render.
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
        let kernels: Vec<_> = plan.passes().iter().map(|p| &p.kernel).collect();
        require_resource_free(&kernels)
            .map_err(|error| error.in_context(&self.context).in_plan(plan))?;
        self.render_inner(plan, &[]).await
    }
    /// Execute Core's immutable plan/snapshot pairing. Each selected image is uploaded
    /// once per call; retaining the prepared request retains only its CPU snapshots.
    /// Outputs own their pixels independently of the request and renderer.
    pub async fn render_prepared(
        &mut self,
        prepared: &PreparedRender,
    ) -> Result<RenderOutput, GpuOperationError> {
        self.render_inner(prepared.plan(), prepared.resources())
            .await
    }
    async fn render_inner(
        &mut self,
        plan: &RenderPlan,
        snapshots: &[ResourceSnapshot],
    ) -> Result<RenderOutput, GpuOperationError> {
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
        let slots: Vec<_> = plan
            .allocation()
            .resource_slots()
            .iter()
            .map(|slot| slot.index() as usize)
            .collect();
        let result = execute_prepared(
            &self.context,
            &mut self.cache,
            plan.size(),
            &kernels,
            &mappings,
            snapshots,
            &slots,
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
fn require_resource_free(kernels: &[&KernelInvocation]) -> Result<(), GpuOperationError> {
    if let Some(resource_id) = kernels.iter().find_map(|kernel| match kernel {
        KernelInvocation::ImageInput { resource_id } => Some(resource_id),
        _ => None,
    }) {
        return Err(mixture_core::Diagnostic::error(
            mixture_core::DiagnosticCode::ResourceMissing,
            Stage::Compile,
            "Image plans require their immutable prepared resources.",
        )
        .with_evidence("resourceId", resource_id.as_str())
        .with_suggestion("Call Renderer::render_prepared with the Core prepared request.")
        .into());
    }
    Ok(())
}
// The fixed probe also uses this exact dispatch, upload, allocation and readback path.
pub(crate) async fn execute(
    context: &GpuContext,
    cache: &mut PipelineCache,
    size: [u32; 2],
    kernels: &[&KernelInvocation],
    outputs: &[(usize, PortKind)],
) -> Result<Executed, GpuOperationError> {
    require_resource_free(kernels).map_err(|error| error.in_context(context))?;
    // Fixed checker/internal failure probes have explicit one-result storage;
    // only compiler-produced graph plans select reusable slots.
    let slots: Vec<_> = (0..kernels.len()).collect();
    execute_prepared(context, cache, size, kernels, outputs, &[], &slots).await
}
async fn execute_prepared(
    context: &GpuContext,
    cache: &mut PipelineCache,
    size: [u32; 2],
    kernels: &[&KernelInvocation],
    outputs: &[(usize, PortKind)],
    snapshots: &[ResourceSnapshot],
    slots: &[usize],
) -> Result<Executed, GpuOperationError> {
    let total = Instant::now();
    let mut resources = Resources::default();
    let result = async {
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
        // Deliver pending native destruction callbacks before touching the pipeline
        // cache or allocating resources. An already-recorded loss does not poll again.
        context.ensure_available(Stage::GpuExecution)?;
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        device.poll(wgpu::PollType::Poll).map_err(|source| {
            GpuOperationError::source_error(
                Stage::GpuExecution,
                "Could not poll the GPU before rendering.",
                source,
            )
        })?;
        context.ensure_available(Stage::GpuExecution)?;
        for snapshot in snapshots {
            resources.upload(context, snapshot).await.map_err(|error| {
                error
                    .evidence("operation", "resourceUpload")
                    .evidence("resourceId", snapshot.image().id.as_str())
            })?;
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
        for (index, (kernel, pipeline)) in kernels.iter().zip(&pipelines).enumerate() {
            context.ensure_available(Stage::GpuExecution)?;
            let slot = slots.get(index).ok_or_else(|| {
                GpuOperationError::at(
                    Stage::GpuExecution,
                    "Pass physical slot mapping is missing.",
                )
            })?;
            resources
                .push(device, pipeline, kernel, size, *slot)
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
        context.ensure_available(Stage::GpuExecution)?;
        let execution_ms = execution_started.elapsed().as_secs_f64() * 1000.;
        let readback_started = Instant::now();
        let mut pixels = Vec::new();
        for (resource, kind) in outputs {
            context
                .ensure_available(Stage::Readback)
                .map_err(GpuOperationError::after_compute)?;
            let (texture, allocations) =
                resources.readback_resource(*resource).ok_or_else(|| {
                    GpuOperationError::at(Stage::Readback, "Output resource is missing.")
                        .after_compute()
                })?;
            pixels.push(
                resources::read_texture(
                    device,
                    context.queue(),
                    texture,
                    layout,
                    *kind,
                    allocations,
                )
                .await
                .map_err(GpuOperationError::after_compute)?,
            );
        }
        let readback_ms = readback_started.elapsed().as_secs_f64() * 1000.;
        Ok((
            pixels,
            cache_report,
            ExecutionTimings {
                pipeline_ms,
                execution_ms,
                readback_ms,
                total_ms: total.elapsed().as_secs_f64() * 1000.,
            },
        ))
    }
    .await;
    // Finish the same resource guard on both paths, before publishing success or
    // attaching actual cleanup counters to the first operation failure.
    let allocations = resources.finish();
    let result = result.and_then(|completed| {
        context
            .ensure_available(Stage::Readback)
            .map_err(GpuOperationError::after_compute)
            .map(|()| completed)
    });
    if context.device_loss().is_some() {
        cache.clear();
    }
    match result {
        Ok((pixels, cache, mut timings)) => {
            timings.total_ms = total.elapsed().as_secs_f64() * 1000.;
            Ok(Executed {
                pixels,
                cache,
                allocations,
                timings,
            })
        }
        Err(error) => Err(error.with_allocations(allocations).in_context(context)),
    }
}

#[cfg(test)]
mod allocation_tests {
    use super::*;
    use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, SafetyLimits, compile};

    fn reuse_plan(size: [u32; 2]) -> RenderPlan {
        let document = MaterialDocument::decode(
            include_bytes!("testdata/texture-reuse.mix"),
            &SafetyLimits::default(),
        )
        .unwrap()
        .into_validated(&SafetyLimits::default())
        .unwrap();
        compile(
            &document,
            &CompileRequest {
                size,
                outputs: vec![
                    OutputChannel::Height,
                    OutputChannel::Roughness,
                    OutputChannel::Metallic,
                    OutputChannel::Opacity,
                ],
                ..Default::default()
            },
        )
        .unwrap()
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn graph_gpu_reuse_preserves_pinned_outputs_aliases_repeats_and_owned_pixels() {
        for size in [[1, 1], [1, 17], [19, 11], [65, 3]] {
            let plan = reuse_plan(size);
            assert_eq!(
                plan.allocation()
                    .resource_slots()
                    .iter()
                    .map(|s| s.index())
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3, 2]
            );
            let context =
                pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
            let mut renderer = Renderer::new(context);
            let first = pollster::block_on(renderer.render(&plan)).unwrap();
            let second = pollster::block_on(renderer.render(&plan)).unwrap();
            assert_eq!(first.report().allocations, second.report().allocations);
            assert_eq!(second.report().pipeline_cache.misses, 0);
            let allocations = &first.report().allocations;
            let pixels = u64::from(size[0]) * u64::from(size[1]);
            assert_eq!(allocations.texture_count, 4);
            assert_eq!(allocations.uniform_count, 5);
            assert_eq!(allocations.reused_bytes, pixels * 8);
            assert_eq!(allocations.peak_bytes, plan.estimates().peak_bytes);
            assert_eq!(allocations.live_bytes, 0);
            assert_eq!(allocations.released_bytes, allocations.cumulative_bytes);
            drop(renderer);
            for (index, gray) in [64, 191, 64, 64].into_iter().enumerate() {
                assert_eq!(
                    first.channels()[index].pixels(),
                    [gray, gray, gray, 255].repeat(pixels as usize)
                );
                assert_eq!(
                    first.channels()[index].pixels(),
                    second.channels()[index].pixels()
                );
            }
        }
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn graph_gpu_reuse_rejects_same_pass_overlap_and_cleans_before_retry() {
        let plan = reuse_plan([19, 11]);
        let context =
            pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
        let mut cache = PipelineCache::default();
        let kernels: Vec<_> = plan.passes().iter().map(|p| &p.kernel).collect();
        // Only an internal fault injection can forge this schedule; public plans are immutable.
        let error = pollster::block_on(execute_prepared(
            &context,
            &mut cache,
            plan.size(),
            &kernels,
            &[(4, PortKind::Scalar)],
            &[],
            &[0, 0, 0, 0, 0],
        ))
        .err()
        .unwrap();
        assert_eq!(
            error.diagnostic().message,
            "Pass input and output share a physical slot."
        );
        assert!(!error.compute_completed());
        let allocations = error.allocations().unwrap();
        assert_eq!(allocations.texture_count, 1);
        assert_eq!(allocations.uniform_count, 1);
        assert_eq!(allocations.live_bytes, 0);
        assert_eq!(allocations.released_bytes, allocations.cumulative_bytes);
        // Fail after real reuse and one completed readback, exercising unique
        // physical destruction rather than only the early rejection path.
        let late = pollster::block_on(execute_prepared(
            &context,
            &mut cache,
            plan.size(),
            &kernels,
            &[(4, PortKind::Scalar), (usize::MAX, PortKind::Scalar)],
            &[],
            &[0, 1, 2, 3, 2],
        ))
        .err()
        .unwrap();
        assert_eq!(late.diagnostic().stage, Stage::Readback);
        assert!(late.compute_completed());
        let allocations = late.allocations().unwrap();
        assert_eq!(allocations.texture_count, 4);
        assert_eq!(allocations.staging_count, 1);
        assert_eq!(allocations.reused_bytes, 19 * 11 * 8);
        assert_eq!(allocations.live_bytes, 0);
        assert_eq!(allocations.released_bytes, allocations.cumulative_bytes);
        let mut renderer = Renderer::new(context);
        let result = pollster::block_on(renderer.render(&plan)).unwrap();
        assert_eq!(
            result.channels()[2].pixels(),
            [64, 64, 64, 255].repeat(19 * 11)
        );
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn image_gpu_partial_readback_failure_releases_uploads_and_allows_retry() {
        let source = br#"{"version":1,"nodes":[
            {"id":"c","type":"constant-color","version":1},
            {"id":"i","type":"image-input","version":1,"parameters":{"resourceId":"pixels"}},
            {"id":"o","type":"material-output","version":1}],"edges":[
            {"from":{"nodeId":"c","portId":"color"},"to":{"nodeId":"o","portId":"baseColor"}},
            {"from":{"nodeId":"i","portId":"value"},"to":{"nodeId":"o","portId":"height"}}]}"#;
        let doc = MaterialDocument::decode(source, &SafetyLimits::default())
            .unwrap()
            .into_validated(&SafetyLimits::default())
            .unwrap();
        let pixels = [64, 0, 255, 0].repeat(65 * 3);
        let prepared = mixture_core::prepare(
            &doc,
            &CompileRequest {
                size: [65, 3],
                outputs: vec![OutputChannel::Height],
                ..Default::default()
            },
            &[mixture_core::ImageBinding {
                id: "pixels",
                width: 65,
                height: 3,
                format: "rgba8-linear",
                bytes_per_row: 260,
                data: &pixels,
            }],
            &mixture_core::ResourceLimits::default(),
        )
        .unwrap();
        let context =
            pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
        let mut cache = PipelineCache::default();
        let kernels: Vec<_> = prepared.plan().passes().iter().map(|p| &p.kernel).collect();
        for _ in 0..3 {
            let error = pollster::block_on(execute_prepared(
                &context,
                &mut cache,
                [65, 3],
                &kernels,
                &[(0, PortKind::Scalar), (usize::MAX, PortKind::Scalar)],
                prepared.resources(),
                &[0],
            ))
            .err()
            .unwrap();
            assert_eq!(error.diagnostic().message, "Output resource is missing.");
            let allocations = error.allocations().unwrap();
            assert_eq!(allocations.resource_count, 1);
            assert_eq!(allocations.resource_staging_bytes, 1536);
            assert_eq!(allocations.live_bytes, 0);
            assert_eq!(allocations.released_bytes, allocations.cumulative_bytes);
            let output = pollster::block_on(execute_prepared(
                &context,
                &mut cache,
                [65, 3],
                &kernels,
                &[(0, PortKind::Scalar)],
                prepared.resources(),
                &[0],
            ))
            .unwrap();
            assert_eq!(output.pixels[0], [64, 64, 64, 255].repeat(65 * 3));
            assert_eq!(output.allocations.live_bytes, 0);
            assert_eq!(output.cache.entries, 1);
        }
    }

    #[test]
    fn plain_image_plan_is_rejected_before_gpu_work() {
        let image = KernelInvocation::ImageInput {
            resource_id: "heightSource".into(),
        };
        let error = require_resource_free(&[&image]).unwrap_err();
        assert_eq!(
            error.diagnostic().code,
            mixture_core::DiagnosticCode::ResourceMissing
        );
        assert_eq!(error.diagnostic().stage, Stage::Compile);
        assert!(error.allocations().is_none());
        assert!(require_resource_free(&[&KernelInvocation::Constant { value: [0.; 4] }]).is_ok());
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn graph_gpu_failure_after_readback_releases_resources_without_returning_partial_pixels() {
        let context =
            pollster::block_on(GpuContext::request(crate::test_support::options())).unwrap();
        let mut cache = PipelineCache::default();
        let kernel = KernelInvocation::Constant {
            value: [0.25, 0.0, 0.0, 1.0],
        };
        // Internal invalid mapping: first readback succeeds, then the second
        // fails. Public compiler-produced plans cannot contain this mapping.
        let error = match pollster::block_on(execute(
            &context,
            &mut cache,
            [33, 3],
            &[&kernel],
            &[(0, PortKind::Scalar), (usize::MAX, PortKind::Scalar)],
        )) {
            Err(error) => error,
            Ok(_) => panic!("partial pixels must never become a successful render"),
        };
        assert_eq!(
            error.diagnostic().code,
            mixture_core::DiagnosticCode::ReadbackFailed
        );
        assert_eq!(error.diagnostic().message, "Output resource is missing.");
        assert!(error.compute_completed());
        let measured = error.allocations().unwrap();
        assert_eq!(measured.texture_count, 1);
        assert_eq!(measured.uniform_count, 1);
        assert_eq!(measured.staging_count, 1);
        assert_eq!(measured.live_bytes, 0);
        assert_eq!(measured.released_bytes, measured.cumulative_bytes);
        assert!(measured.cumulative_bytes > 0);
        assert!(error.device_loss().is_none());
        // Ordinary readback failures do not poison this context or its cache.
        let output = pollster::block_on(execute(
            &context,
            &mut cache,
            [33, 3],
            &[&kernel],
            &[(0, PortKind::Scalar)],
        ))
        .unwrap();
        assert!(
            output.pixels[0]
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| *pixel == [64, 64, 64, 255])
        );
        assert_eq!(output.cache.misses, 0);
        assert_eq!(output.cache.hits, 1);
        eprintln!(
            "readback failure cleanup: {}",
            serde_json::to_string(error.diagnostic()).unwrap()
        );
    }

    #[test]
    #[ignore = "requires GPU; cargo xtask gpu-smoke"]
    fn graph_gpu_allocation_accounting_matches_plan_and_releases_aliased_readbacks() {
        let mut document = MaterialDocument::decode(
            include_bytes!("testdata/constant-scalar.mix"),
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
