# Six-node graph rendering

English | [简体中文](./graph-rendering.zh-CN.md)

PR-007 completes the local `.mix → validate → RenderPlan → wgpu → PNG` path for the six M2 contracts. [Renderer](../crates/mixture-wgpu/src/executor.rs) accepts only an immutable compiler-produced plan. It owns one explicitly acquired context and a small pipeline cache. The GPU crate does not parse documents, resolve node defaults, or interpret parameter overrides. [The CLI](../crates/mixture-cli/src/commands/render.rs) handles file I/O, arguments, PNG encoding, and reports.

## Run the examples

```bash
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render examples/levels.mix --size 256 \
  --output baseColor,roughness --set 'gamma=2' --out ./tmp/levels --json
cargo run --locked -p mixture-cli -- render examples/blend.mix --size 256 \
  --output baseColor,roughness,normal --set 'frequency=16' --out ./tmp/blend --json
cargo run --locked -p mixture-cli -- render examples/blend.mix \
  --output roughness --out ./tmp/roughness --backend metal --json
```

[checker.mix](../examples/checker.mix) supplies the checker base color. [levels.mix](../examples/levels.mix) remaps a constant scalar into roughness, with a white base color. [blend.mix](../examples/blend.mix) connects all six contracts: a checker and tint are multiplied with a levels-adjusted mask, and the scalar also feeds roughness. These demonstrate execution and are not realistic golden materials.

`render <file.mix> --out <directory>` defaults to 64×64 and `baseColor` only. `--size`, `--output`, and repeatable `--set` use the same syntax and core semantics as [inspect](./render-plan.md). Overrides must use exposed public IDs; JSON types and cross-parameter constraints are checked before acquiring a GPU. `--backend`, `--power-preference`, and `--software` use the explicit [context policy](./gpu-context.md). Omitted GPU options do not read smoke-test environment variables. An unavailable/disabled adapter fails without changing the execution backend.

Requested files are named exactly `<channel>.png` in material-contract order. The CLI creates the directory after successful rendering, replaces same-named files, and leaves other files alone. Paths beginning with `-` can follow `--`. Source files are never edited. Outputs are written sequentially, not as a multi-file transaction: if a later write fails, the report lists files already completed; a failed file write may leave a partial file.

Exit `0` means every requested PNG was written. Invalid usage/source/compile request returns `2`; input I/O, GPU acquisition, execution/readback, PNG, and output/report I/O failures return `1`. Usage errors go to stderr. Semantic and operational failures use the selected human/JSON report mode. Invalid source/request does not initialize a GPU or create output directories.

## Public Rust consumer

```rust
use mixture_core::{CompileRequest, MaterialDocument, compile};
use mixture_wgpu::{GpuContext, GpuContextOptions, Renderer};

let request = CompileRequest::default();
let source = MaterialDocument::decode(source_bytes, &request.limits)?
    .into_validated(&request.limits)?;
let plan = compile(&source, &request)?;
let context = GpuContext::request(GpuContextOptions::default()).await?;
let mut renderer = Renderer::new(context);
let output = renderer.render(&plan).await?;
assert_eq!(&output.report().plan_hash, plan.hash());
for channel in output.channels() {
    let rgba8: &[u8] = channel.pixels();
}
renderer.clear_pipeline_cache();
```

[Crate doctests](../crates/mixture-wgpu/src/lib.rs) compile the async public consumer; [GPU integration tests](../crates/mixture-wgpu/tests/nodes.rs) execute it. `RenderOutput` owns CPU buffers, so outputs survive renderer/context drop. Each `RenderedChannel` carries channel ID, logical kind, dimensions, transfer encoding, and the connected endpoint or explicit default copied from `PlanOutput`. Alpha is straight, not premultiplied.

## One kernel path

[The exhaustive mapping](../crates/mixture-wgpu/src/kernels.rs) pairs every `KernelId` with exactly one WGSL entry point. Uniform uploads use explicit little-endian values and padding; no unsafe casts or generated shader language are needed.

| Kernel | WGSL and bindings | Uniform |
| --- | --- | --- |
| `constant` | [constant.wgsl](../crates/mixture-wgpu/shaders/nodes/constant.wgsl); uniform 0, storage output 1 | packed RGBA, 16 bytes |
| `checker` | [checker.wgsl](../crates/mixture-wgpu/shaders/nodes/checker.wgsl); uniform 0, storage output 1 | cells and two colors, 48 bytes |
| `levels` | [levels.wgsl](../crates/mixture-wgpu/shaders/nodes/levels.wgsl); uniform 0, output 1, scalar input 2 | five f32 values with padding, 32 bytes |
| `blend` | [blend.wgsl](../crates/mixture-wgpu/shaders/nodes/blend.wgsl); uniform 0, output 1, a/b/mask inputs 2/3/4 | mode code and opacity with padding, 16 bytes |

Constants serve both scalar/color nodes and compiler-generated optional defaults. `material-output` reads mapped resources, with no additional shader. Inputs use unfiltered `textureLoad` at the same pixel coordinate; outputs use `rgba16float` storage. Every kernel uses 8×8 workgroups and bounds-checks partial workgroups. Checker dimensions come from the output texture; frequencies/colors come from the typed invocation. No random values or seeds are involved.

The [node contract formulas](./node-contracts.md) are unchanged. Levels handles input endpoints before division/pow, preserving clamp behavior and avoiding undefined calculations in tiny intervals with no representable half-float input between bounds. Blend implements normal/multiply/screen RGB interpolation with `opacity × mask`, and interpolates alpha independently. There is no source-over compositing, implicit premultiplication, or CPU graph evaluator.

The fixed `render-builtin checker` and `doctor` probe now call the same allocation/dispatch/readback engine and checker shader as graph rendering. Their 8×8 opaque black/white pixels and golden stay unchanged. The shared checker uniform grows from 16 to 48 bytes; the fixed probe's logical allocation estimate and exact budget boundary therefore increase by 32 bytes (64×64: 65,584 bytes). Prior PR-004 evidence remains historical. The fixed API uses an ephemeral pipeline cache; persistent caching is owned by `Renderer`.

## Cache, lifetime, and failures

A renderer retains at most four pipelines, keyed by `KernelId`. Within a renderer, device, shader ABI/version, local workgroup size, and storage format are fixed, so parameter values and output dimensions do not need additional cache keys. Entries are populated only after successful pipeline creation. `cached_pipeline_count()` exposes the size; `clear_pipeline_cache()` and renderer drop release retained handles. Each render report counts per-pass cache hits and misses, including reuse earlier in the same call.

[Per-call resources](../crates/mixture-wgpu/src/resources.rs) implement the [PR-006 lifetime model](./render-plan.md): all pass textures and padded uniforms remain allocated through execution and readback. Each requested channel gets a staging buffer that is mapped, unpacked, unmapped, and destroyed before the next channel. Aliased channels share the producer texture but still read back separately. All per-call buffers/textures are released on success or failure. There is no texture pool, last-consumer optimization, pass fusion, or disk cache.

Before allocation, the executor checks actual device texture dimensions, maximum buffer size, and workgroup count. Graph/request safety ceilings were already checked by the compiler. GPU shader, pipeline, execution, and readback operations use balanced error scopes and shared typed diagnostics; native source errors are retained. Plan failures include their plan hash, and pipeline/allocation failures identify the pass and source node where available. Each native completion/map wait is bounded to 30 seconds. Timings are CPU wall times; peak/cumulative byte figures are logical estimates, excluding driver/pipeline overhead and CPU/PNG buffers.

## Output encoding and precision

The source retains f64, the compiled invocation records f32, and every pass stores f16 components in `rgba16float`. Readback rejects non-finite or out-of-range components and removes 256-byte row padding. Returned data is tightly packed RGBA8, top-left origin.

| Logical kind | RGBA8 conversion | PNG metadata |
| --- | --- | --- |
| Color (`baseColor`, `emissive`) | linear RGB → sRGB; alpha stays linear; round × 255 | sRGB intent |
| Scalar | red component replicated into RGB, opaque alpha; round × 255 | gAMA = 1.0, no sRGB chunk |
| Normal | encoded XYZ retained in RGB, opaque alpha; round × 255 | gAMA = 1.0, no sRGB chunk |

The readback transfer function is `12.92 × c` for `c ≤ 0.0031308`, otherwise `1.055 × c^(1/2.4) − 0.055`. This is output encoding, not a second material executor. PNG writing occurs only in the CLI. Normal `[0.5, 0.5, 1]` becomes `[128, 128, 255, 255]`; scalar `0.5` becomes `[128, 128, 128, 255]`; linear color `[0.25, 0.5, 0.75, 0.25]` becomes approximately `[137, 188, 225, 64]`.

Focused floating-point pixel sentinels allow at most one RGBA8 code value of error after f32/f16/transfer rounding. Endpoint/default-normal/alias cases use zero tolerance where specified. The existing black/white checker is compared exactly over all 16,384 bytes. These checks do not promise universal floating-point byte identity across GPUs.

## Reports

JSON schema version 1 contains `input`, `outputDirectory`, `planHash`, `context`, `execution`, `outputs`, `ok`, and `diagnostics`. Context records actual selection and retains acquisition-only `unverified` status; a successful `execution` report is the evidence of rendering. A source/compile failure leaves context and execution null. A GPU failure retains available context; an encoding/write failure retains completed execution and output entries.

`execution` contains the actual adapter, plan hash, dimensions, executed pass count, pipeline-cache lookups, allocation estimates, tight raw readback bytes, total padded mapped bytes, returned RGBA8 bytes, and stage timings. `outputs` lists only successfully written files with channel/kind/source/size/encoding/path/byte count. The same semantic request has the same plan hash as `inspect`; adapter names, paths, and timings never enter that hash.

## Verification and evidence

```bash
cargo xtask shader-check
cargo xtask test-node constant-scalar
cargo xtask test-node constant-color
cargo xtask test-node checker
cargo xtask test-node levels
cargo xtask test-node blend
cargo xtask test-node material-output
cargo xtask gpu-smoke
cargo xtask test-plan
cargo xtask check
```

`shader-check` validates all WGSL entry points/workgroup sizes and uniform struct spans without a GPU. `test-node` first validates [all node fixture families](../fixtures/nodes/README.md), including invalid overrides/source, without a GPU, then runs exactly the named node's GPU cases. It uses the same `MIXTURE_GPU_BACKEND`, `MIXTURE_GPU_SOFTWARE`, and optional expected-adapter policy as smoke. Reports go to `tmp/node-tests/<backend>/<node>.json`. Unknown nodes and invalid policies fail before running Cargo/GPU work.

`gpu-smoke` retains the fixed checker/doctor golden gate, renders all three graph examples, verifies the graph checker against the same golden, and runs every ignored library/CLI GPU regression. It includes all six node families, odd dimensions, nontrivial colors/alpha, all blend modes, levels extremes, defaults/aliases, executed slicing, cache reuse/clear, device rejection/recovery, invalid shader/pipeline/map/device paths, PNG metadata, file names, and partial-output failure reports. Evidence is saved under `tmp/gpu-smoke/`, including per-node case reports. Ordinary `check`/workspace tests never initialize a GPU.

Local [Apple M5/Metal](./evidence/pr-007-apple-m5.json) and pinned [SwiftShader/Vulkan](./evidence/pr-007-swiftshader.json) pass. Three 256×256 examples were visually inspected. The fixed checker remains byte-identical to its protected golden; no pixel golden was overwritten. No dependency or lockfile version changed.

Reviewed 256×256 previews: [checker](./evidence/pr-007-checker.png), [levels roughness](./evidence/pr-007-levels.png), and [blend base color](./evidence/pr-007-blend.png). These are evidence images, not additional test goldens.

The [remote GPU job](../.github/workflows/gpu-smoke.yml) now covers graph examples and all nodes, but remote Linux/macOS/Windows results remain pending. M2 is implemented and locally verified; the open remote milestone gates are not claimed complete. PR-008 now adds [protected golden tooling and glazed ceramic machine gates](./material-goldens.md), with human approval still open and no realistic-quality claims for these M2 examples.
