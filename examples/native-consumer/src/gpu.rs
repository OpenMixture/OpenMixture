use crate::{Result, Selection, cpu, require};
use mixture_core::{InputSource, OutputChannel, RenderPlan, registry::PortKind};
use mixture_wgpu::{
    ContextReport, GpuContext, GpuOperationError, OutputEncoding, RenderOutput, Renderer,
};
use serde_json::{Value, json};
use std::{error::Error, fmt, fs, io::Read, path::Path, time::Instant};

#[derive(Debug)]
pub struct RenderFailure {
    pub error: GpuOperationError,
    pub context: ContextReport,
}
impl fmt::Display for RenderFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
impl Error for RenderFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

async fn acquire(selection: &Selection) -> Result<GpuContext> {
    let context = GpuContext::request(selection.options).await?;
    let report = context.report();
    let adapter = report
        .adapter()
        .ok_or("acquisition omitted actual adapter")?;
    if selection.options.software_adapter {
        require(
            adapter.device_type == "Cpu",
            "software policy selected a hardware device",
        )?;
    }
    if let Some(expected) = &selection.expected_adapter {
        require(
            adapter.name.contains(expected),
            "selected adapter differs from requested name",
        )?;
    }
    Ok(context)
}

async fn render(renderer: &mut Renderer, plan: &RenderPlan) -> Result<RenderOutput> {
    renderer.render(plan).await.map_err(|error| {
        Box::new(RenderFailure {
            error,
            context: renderer.context().report().clone(),
        }) as Box<dyn Error>
    })
}

fn inspect(output: &RenderOutput, plan: &RenderPlan) -> Result<Value> {
    let report = output.report();
    require(
        &report.plan_hash == plan.hash(),
        "renderer returned a different plan hash",
    )?;
    require(
        report.pass_count == plan.passes().len(),
        "pass count differs from requested plan",
    )?;
    require(
        report.size == plan.size(),
        "render report has wrong dimensions",
    )?;
    require(
        report.allocations.live_bytes == 0
            && report.allocations.released_bytes == report.allocations.cumulative_bytes,
        "completed render retained per-call GPU descriptors",
    )?;
    require(
        !report.adapter.name.is_empty(),
        "render omitted adapter evidence",
    )?;
    require(
        output.channels().len() == plan.outputs().len(),
        "wrong returned channel count",
    )?;
    let mut channels = Vec::new();
    for (actual, expected) in output.channels().iter().zip(plan.outputs()) {
        let encoding = match expected.kind {
            PortKind::Color => OutputEncoding::Srgb,
            PortKind::Scalar | PortKind::Normal => OutputEncoding::Linear,
        };
        require(
            actual.channel == expected.channel
                && actual.kind == expected.kind
                && actual.source == expected.input
                && actual.size == plan.size()
                && actual.encoding == encoding,
            "channel metadata differs from the public plan/encoding contract",
        )?;
        require(
            actual.pixels().len() as u64
                == u64::from(actual.size[0]) * u64::from(actual.size[1]) * 4,
            "returned RGBA8 bytes are not tightly packed",
        )?;
        channels.push(json!({
            "channel": actual.channel, "kind": actual.kind, "source": actual.source,
            "encoding": actual.encoding, "size": actual.size, "bytes": actual.pixels().len(),
        }));
    }
    Ok(json!({"execution": report, "channels": channels}))
}

fn check_literals(output: &RenderOutput) -> Result<()> {
    for channel in output.channels() {
        let pixels = channel.pixels().as_chunks::<4>().0;
        // Literal transfer/default expectations for this consumer-owned fixture;
        // no checker, sRGB, normal, or other pixel algorithm is implemented here.
        let expected = match channel.channel {
            OutputChannel::BaseColor => {
                let a = [137, 188, 225, 255];
                let b = [255, 0, 0, 128];
                require(pixels.first() == Some(&a), "top-left color/alpha differs")?;
                require(
                    pixels.contains(&b),
                    "nontrivial checker color/straight alpha was not observed",
                )?;
                require(
                    pixels.iter().all(|p| *p == a || *p == b),
                    "sRGB color or straight alpha differs from literal expectations",
                )?;
                continue;
            }
            OutputChannel::Normal => [128, 128, 255, 255],
            OutputChannel::Roughness => [64, 64, 64, 255],
            OutputChannel::Height => [0, 0, 0, 255],
            _ => return Err("unexpected consumer fixture channel".into()),
        };
        require(
            pixels.iter().all(|pixel| *pixel == expected),
            "linear scalar/normal output differs from literal expectations",
        )?;
    }
    Ok(())
}

pub async fn check(selection: &Selection) -> Result<Value> {
    let cpu = cpu::check()?;
    let plan = cpu::own_plan(16)?;
    let changed_plan = cpu::own_plan(4)?;
    let context = acquire(selection).await?;
    let context_report = context.report().clone();
    let mut renderer = Renderer::new(context);
    let original = render(&mut renderer, &plan).await?;
    let changed = render(&mut renderer, &changed_plan).await?;
    // The renderer owns the only context. All pixel access below occurs after drop.
    drop(renderer);
    let original_report = inspect(&original, &plan)?;
    let changed_report = inspect(&changed, &changed_plan)?;
    check_literals(&original)?;
    check_literals(&changed)?;
    require(
        original.channels()[0].pixels() != changed.channels()[0].pixels(),
        "exposed repeat override did not change rendered baseColor",
    )?;
    require(
        matches!(original.channels()[1].source, InputSource::Default { .. }),
        "normal default provenance was lost",
    )?;
    Ok(json!({
        "schemaVersion": 1, "ok": true, "mode": "gpu", "gpuExecuted": true,
        "cpu": cpu, "context": context_report, "original": original_report,
        "changedOverride": changed_report, "pixelsCheckedAfterRendererDrop": true,
        "literalPixelsPassed": true, "overrideChangedPixels": true,
        "packagedCratesValidated": false,
    }))
}

pub async fn measure(input: &Path, out: &Path, selection: &Selection) -> Result<Value> {
    let request = cpu::request([1024, 1024]);
    let mut bytes = Vec::new();
    fs::File::open(input)?
        .take(request.limits.decoded_bytes + 1)
        .read_to_end(&mut bytes)?;
    let compile_started = Instant::now();
    let plan = cpu::prepare(&bytes, &request)?;
    let compile_ms = compile_started.elapsed().as_secs_f64() * 1000.;
    // A fresh caller-owned directory prevents any overwrite of existing outputs.
    fs::create_dir(out)?;
    let context_started = Instant::now();
    let context = acquire(selection).await?;
    let context_ms = context_started.elapsed().as_secs_f64() * 1000.;
    let context_report = context.report().clone();
    let mut renderer = Renderer::new(context);
    let first_started = Instant::now();
    let first = render(&mut renderer, &plan).await?;
    let first_ms = first_started.elapsed().as_secs_f64() * 1000.;
    let reused_started = Instant::now();
    let reused = render(&mut renderer, &plan).await?;
    let reused_ms = reused_started.elapsed().as_secs_f64() * 1000.;
    drop(renderer);
    let first_report = inspect(&first, &plan)?;
    let reused_report = inspect(&reused, &plan)?;
    require(
        first
            .channels()
            .iter()
            .zip(reused.channels())
            .all(|(a, b)| a.pixels() == b.pixels()),
        "reusing the renderer changed pixels for an identical plan",
    )?;
    // Export actual owned bytes for independent golden comparison after all timers.
    for (label, output) in [("first", &first), ("reused", &reused)] {
        for channel in output.channels() {
            fs::write(
                out.join(format!("{label}-{}.rgba", channel.channel.as_str())),
                channel.pixels(),
            )?;
        }
    }
    Ok(json!({
        "schemaVersion": 1, "ok": true, "mode": "measure", "gpuExecuted": true,
        "input": input, "outputDirectory": out, "inputBytes": bytes.len(),
        "context": context_report, "compileMs": compile_ms, "contextMs": context_ms,
        "firstCallMs": first_ms, "reusedCallMs": reused_ms,
        "first": first_report, "reused": reused_report,
        "identicalPixels": true, "pixelsCheckedAfterRendererDrop": true,
        "timingScope": "CPU wall times; context acquisition and render calls separate; file reads/writes and inspection excluded; no GPU timestamps or PNG encoding",
        "packagedCratesValidated": false,
    }))
}

/// Deterministic event driver: inject newer requests before delivering a completion.
/// A host can call the same state from its event loop and explicitly owned worker.
pub async fn latest(selection: &Selection) -> Result<Value> {
    use mixture_native_consumer::latest::Latest;
    let context = acquire(selection).await?;
    let context_report = context.report().clone();
    let mut renderer = Renderer::new(context);
    let mut state = Latest::<u32, RenderOutput, Value>::default();
    let mut events = Vec::new();
    // Initial displayed result.
    state.request(16)?;
    let (first, repeat) = state.start().ok_or("missing first request")?;
    let plan = cpu::own_plan(repeat)?;
    let output = render(&mut renderer, &plan).await?;
    check_literals(&output)?;
    events.push(inspect(&output, &plan)?);
    drop(state.complete(first, Ok(output)));
    // While request 2 is active, request 3 replaces the pending slot; 4 replaces 3.
    state.request(4)?;
    let (old, repeat) = state.start().ok_or("missing old request")?;
    state.request(8)?;
    let newest_failed = state.request(0)?;
    require(state.start().is_none(), "started concurrent work")?;
    let plan = cpu::own_plan(repeat)?;
    let output = render(&mut renderer, &plan).await?;
    events.push(inspect(&output, &plan)?);
    drop(state.complete(old, Ok(output)));
    let (generation, repeat) = state.start().ok_or("missing newest request")?;
    require(
        generation == newest_failed && repeat == 0,
        "pending was not replaced",
    )?;
    let error = cpu::own_plan(repeat)
        .err()
        .ok_or("invalid request succeeded")?;
    let failed = crate::failure(error.as_ref());
    drop(state.complete(generation, Err(failed.clone())));
    let (displayed, _, current) = state.displayed().ok_or("lost previous display")?;
    require(
        displayed == first && !current && state.failure().is_some(),
        "failed newest request made old pixels current",
    )?;
    // Only the retained display and this completion coexist during replacement.
    let current = state.request(4)?;
    let (generation, repeat) = state.start().ok_or("missing replacement")?;
    let plan = cpu::own_plan(repeat)?;
    let output = render(&mut renderer, &plan).await?;
    check_literals(&output)?;
    events.push(inspect(&output, &plan)?);
    drop(state.complete(generation, Ok(output)));
    let (generation, output, is_current) = state.displayed().ok_or("no current display")?;
    require(
        generation == current && is_current,
        "replacement not current",
    )?;
    let retained_bytes: usize = output.channels().iter().map(|c| c.pixels().len()).sum();
    drop(renderer);
    check_literals(output)?;
    Ok(
        json!({"schemaVersion":1,"ok":true,"completed":true,"gpuExecuted":true,
        "mode":"latest","context":context_report,"executions":events,
        "startedGenerations":[1,2,4,5],"publishedGenerations":[1,5],
        "staleCompletion":2,"replacedPending":3,"failedGeneration":4,
        "failedRequest":failed,"oldDisplayWasStaleAfterFailure":true,
        "displayedGeneration":generation.value(),"displayedCurrent":is_current,
        "retainedOutputBytes":retained_bytes,"maximumCoexistingOutputs":2,
        "pixelsCheckedAfterRendererDrop":true}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_worker_types_compile_without_acquiring_a_gpu() {
        fn require_send<T: Send>() {}
        fn require_send_value<T: Send>(_: T) {}
        require_send::<Renderer>();
        require_send::<RenderPlan>();
        require_send::<RenderOutput>();
        fn future(renderer: &mut Renderer, plan: &RenderPlan) {
            require_send_value(renderer.render(plan));
        }
        let _compile_only = future;
    }
}
