# Built-in checker compute and readback

English | [简体中文](./builtin-checker.zh-CN.md)

PR-004 implements one real headless compute pass before graph work begins. It uses the caller's `GpuContext`, writes an offscreen `rgba16float` texture, copies it to a padded readback buffer, and returns CPU-owned RGBA8 pixels. PNG encoding and file I/O belong to the CLI. The fixed probe needs no graph model, node registry, general renderer, resource pool, or second pixel executor. PR-005 adds separate [source validation and node contracts](./file-format.md); it does not change this probe.

## Run and inspect

```bash
mkdir -p tmp
cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out tmp/checker.png
cargo run --locked -p mixture-cli -- render-builtin checker --size 65 --out tmp/checker-65.png --json
cargo run --locked -p mixture-cli -- doctor --json
cargo run --locked -p mixture-cli -- doctor --skip-probe --json
```

The only built-in name is `checker`. `--size` defaults to 64 and accepts 1 through 2048. `--out` is required, its parent directory must exist, and an existing output file is replaced. The command accepts the same `--backend`, `--power-preference`, and `--software` selection options as [doctor](./gpu-context.md). Unknown names/options, duplicate options, or missing values fail before GPU access.

Human output includes the actual adapter, dimensions, pass count, readback bytes, and CPU wall time. `--json` emits a report containing `schemaVersion: 1`, `request`, `output`, `writtenBytes`, acquisition `context`, completed `execution`, and shared `ok`/`diagnostics` fields. `writtenBytes` is null until the PNG write succeeds; `execution` remains available if later PNG encoding or I/O fails. The nested context is an acquisition snapshot and remains `unverified`; a render report proves execution through its separate `execution` field.

Exit codes are `0` for a written PNG, `1` for GPU/readback/encoding/I/O failure, and `2` for invalid invocation or dimensions/budgets. Usage syntax errors go to stderr even with `--json`; validated-request failures have structured JSON. No output file is created for invalid dimensions or failed GPU execution. PNG is encoded completely in memory before opening the output file; a later filesystem write failure can still leave a partial file.

## Checker v1 contract

| Property | Contract |
| --- | --- |
| Inputs | Positive output width and height; CLI uses a square, Rust permits rectangles |
| Pattern | Eight cells per axis; top-left black, alternating opaque black and white |
| Cell selection | Integer `floor(x * 8 / width)` and `floor(y * 8 / height)`; odd summed parity is white |
| Sampling | Top-left pixel origin, no filtering or randomness; uneven dimensions produce uneven cell widths |
| GPU format | `rgba16float`, one mip, one layer, storage write plus copy source |
| Dispatch | Workgroup `[8, 8, 1]`, rounded-up group counts, explicit out-of-bounds guard |
| Uniform | PR-007 shared ABI: `u32` cells/padding and two f32 RGBA colors; 48 bytes; size from the texture |
| Output | Tight RGBA8, row-major, top-left origin; alpha 255 |
| Color space | Black/white RGB endpoints are identical in linear and sRGB; PNG carries sRGB metadata |
| Tiling | Eight cells form a repeating pattern across each axis; opposing edge pixels can differ at the intended checker boundary |

The single pixel implementation is [checker.wgsl](../crates/mixture-wgpu/shaders/nodes/checker.wgsl). Sizes smaller than eight may undersample cells; 1×1 is opaque black. There is no CPU renderer or generalized color transform. Future colored kernels must define their own color-space conversion contract.

## Public API and limits

[CheckerRequest](../crates/mixture-wgpu/src/checker.rs) contains width and height. `validate(&SafetyLimits)` checks positive dimensions, safe integer cell arithmetic, per-axis output limits, one requested output, and estimated transient GPU bytes before context acquisition. Defaults cap each dimension at 2048 and transient bytes at 512 MiB. The renderer also checks the acquired device's texture and buffer limits before allocation. No request limit is silently raised.

```rust
use mixture_core::SafetyLimits;
use mixture_wgpu::{CheckerOutput, CheckerRequest, GpuContext, GpuContextOptions};

async fn checker() -> Result<CheckerOutput, Box<dyn std::error::Error>> {
    let request = CheckerRequest::default();
    let limits = SafetyLimits::default();
    request.validate(&limits)?;
    let mut context = GpuContext::request(GpuContextOptions::default()).await?;
    Ok(context.render_checker(request, &limits).await?)
}
```

`CheckerOutput::pixels()` borrows the CPU-owned image; `report()` provides actual adapter evidence, dimensions, dispatch/pass count, format/encoding, row stride, byte counts, memory estimate, and stage timings. The acquisition report is immutable. `GpuContext::probe_checker()` returns a new doctor report after executing and checking a 64×64 checker.

Shader and pipeline preparation, execution, readback, and total timings are CPU wall-clock milliseconds. They are not GPU timestamp measurements and are excluded from pixel comparisons. The memory estimate is logical texture bytes plus padded readback bytes plus the 48-byte uniform (PR-007 shared ABI); it excludes driver allocations and pipeline overhead.

## Readback and error behavior

`rgba16float` uses eight bytes per pixel. The copy row stride is `ceil(width * 8 / 256) * 256`; the mapped buffer is stride × height. Readback checks byte counts, removes row padding, decodes half-float channels, rejects NaN/infinity and values outside `[0, 1]`, then rounds each channel × 255. For 64×64, the raw texture/readback is 32,768 bytes, stride is 512, and RGBA8 is 16,384 bytes. For 65×3, stride is 768 and the mapped buffer is 2,304 bytes.

GPU resource, shader, pipeline, command, and map requests use validation, internal-error, and out-of-memory scopes. Scopes are popped before awaiting results, so they do not remain on a thread-local stack across suspension. Shader failures use `MIX_GPU_SHADER_VALIDATION_FAILED` / `gpuShader`; pipeline creation uses `MIX_GPU_EXECUTION_FAILED` / `gpuPipeline`; allocation, upload, dispatch, or submission use `MIX_GPU_EXECUTION_FAILED` / `gpuExecution`; copy, mapping, decoding, or probe-pixel failure use `MIX_READBACK_FAILED` / `readback`. PNG and filesystem failures use `MIX_ENCODING_FAILED` / `encoding`. Native sources are retained beneath stable codes.

Native submission and mapping waits each have a 30-second bound. Mapping views are dropped before unmapping on success and decode failure. Successful calls explicitly destroy buffers/textures; error paths release owned handles through Rust drop. The context is reusable, and returned pixels survive context destruction. There are no process-global resources or implicit recovery attempts.

## Golden, tests, and evidence

The initial [64×64 PNG](../fixtures/nodes/checker/checker-64.png) and [raw RGBA golden](../fixtures/nodes/checker/checker-64.rgba) were generated by SwiftShader revision `694585a05946e1ed49b6bd577ca6537cbb57f025`, visually reviewed, and recorded in [provenance](../fixtures/nodes/checker/provenance.json). They contain 2048 opaque black and 2048 opaque white pixels. Golden comparisons use decoded RGBA bytes, not compressed PNG bytes.

```bash
cargo test --locked -p mixture-wgpu checker
cargo test --locked -p mixture-wgpu readback
cargo test --locked -p mixture-cli
cargo xtask shader-check
cargo xtask check
cargo xtask gpu-smoke
```

Ordinary tests validate requests, layout arithmetic, padding removal, half conversion, portable WGSL through Naga, PNG round trips, and CLI failures without accessing a GPU. Explicit smoke runs doctor with and without its probe, renders and decodes a real PNG, compares every byte against the reviewed golden, and runs ignored GPU tests for odd/partial dimensions, repeated rendering, context drop, bad shader/pipeline, destroyed device, failed mapping, and output I/O failure. A failure is never converted into skipped coverage or a new baseline.

The [software-adapter setup and smoke policy](./gpu-context.md) supports Linux CI and native macOS reproduction. The PR-004 local [Apple M5 / Metal](./evidence/pr-004-apple-m5.json) and [SwiftShader / Vulkan](./evidence/pr-004-swiftshader.json) probes passed and produced identical checker RGBA bytes. This establishes this fixture's result on those tested adapters, not universal floating-point identity. Remote Linux SwiftShader and the non-GPU cross-platform CI matrix were still pending at PR-004; their documented [remote CI gates](./evidence/remote-ci/README.md) are now closed. PR-005 [strict `.mix` decoding and validation](./file-format.md) is implemented; PR-006 [plan compilation](./render-plan.md) is implemented. PR-007 [graph execution](./graph-rendering.md) is implemented and shares the fixed probe dispatch path; the golden pixels are unchanged.
