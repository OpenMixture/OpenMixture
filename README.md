# Mixture

English | [简体中文](./README.zh-CN.md)

> A small material-graph compiler and headless texture renderer built in Rust on top of `wgpu`.

**Status:** pre-alpha; Rust crates are unpublished. Source manifests own the current build versions, and [release status](./docs/release.md) owns published packages, qualification scope and hardware limits. The [roadmap](./ROADMAP.md) owns current priorities.

[M6-A](./docs/m6a-resource-contract.md) provides external image resources, immutable prepared requests and plan v2. [M6-B](./docs/m6b-portable-assets.md) provides the shared CPU asset codec, CLI/browser adapters and bounded qualification. [Stable noise](./docs/stable-noise.md) supports explicit `fractal-noise@2` migration; old documents retain their node versions.

**Implemented:** PR-001 through PR-015 delivered the first eleven nodes and accepted ceramic, leather and [wood](./fixtures/materials/wood/README.md) appearances. The [M3 review](./docs/m3-review.md) records 1K release timings and bounded 2K allocation evidence. The [M4 train](./M4_PRS.md) verifies [public Rust consumption](./docs/native-sdk.md), [CLI reports and exit codes](./docs/cli-contract.md), [GPU failure and cleanup contracts](./docs/gpu-failures.md), [latest-result publication](./docs/stale-results.md), and [isolated Cargo package consumption](./docs/package-consumption.md). [M4 acceptance](./docs/release.md) includes the completed [three-platform CPU and Linux SwiftShader CI gates](./docs/evidence/remote-ci/README.md). The [M5 browser runtime](./docs/browser-runtime.md) passed the [recorded macOS/Linux Chromium material matrix](./docs/evidence/m5-05/README.md); the independent [Studio](https://github.com/OpenMixture/Studio) MVP passed its [recorded macOS saved-file qualification](./docs/evidence/studio-qualification/README.md). Current-candidate CI independently builds and installs a fresh package for browser qualification; every delivery candidate must still bind its own source and archive results.

`PR-001` through `PR-015` are historical implementation batch identifiers, not GitHub pull request numbers. New changes follow the [repository governance](./docs/governance.md) and [evidence retention](./docs/evidence-policy.md) policies.

Mixture is designed to read a versioned `.mix` material document, validate and compile its directed acyclic graph, execute the resulting compute passes through one `wgpu` renderer, and return requested PBR texture channels.

The project deliberately starts as an engine and command-line tool rather than a full material authoring product.

## Getting started

Install [rustup](https://rustup.rs/) and the native linker for your platform, then run from the repository root:

```bash
cargo xtask check
cargo run --locked -p mixture-cli -- --help
```

The repository pins Rust 1.98.1 / edition 2024 and includes `Cargo.lock`. The check covers formatting, dependency boundaries, Clippy, workspace and independent-consumer tests, rustdoc, and local document links without a GPU. See [development instructions](./docs/development.md) for all implemented commands and platform prerequisites.

The core's [diagnostics and safety-limit API](./docs/diagnostics.md) now provides typed errors, deterministic JSON reports, and seven explicit resource ceilings. Try its public example with `cargo run --locked -p mixture-core --example diagnostics`.

The [GPU context and doctor guide](./docs/gpu-context.md) documents adapter selection, structured failures, exit codes, and the explicit `cargo xtask gpu-smoke` check. Run `cargo run --locked -p mixture-cli -- doctor --json` to inspect your environment.

Try [the built-in checker](./docs/builtin-checker.md): `cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out checker.png`. Doctor runs its compute/readback probe by default; use `--skip-probe` for acquisition only.

Validate the [checker document](./examples/checker.mix) without a GPU: `cargo run --locked -p mixture-cli -- validate examples/checker.mix --json`. See the [strict file format](./docs/file-format.md) and [node contracts](./docs/node-contracts.md).

## Mission

Mixture exists to make this path reliable and embeddable:

```text
.mix JSON
  -> parse
  -> validate
  -> compile
  -> RenderPlan
  -> wgpu compute passes
  -> Base Color / Normal / Roughness / Height / ...
  -> PNG or raw pixels
```

The first supported consumer is the native CLI. Browser WebGPU through WebAssembly comes only after the native contract is stable and an external consumer is ready to use it.

## Core guarantees

Mixture is designed around a small set of non-negotiable guarantees:

1. **Rust owns the document model, graph rules, node contracts, compilation, and diagnostics.**
2. **`wgpu` is the only pixel execution backend.** There is no second CPU renderer and no independent TypeScript WebGPU renderer.
3. **Execution is explicit.** GPU initialization, adapter selection, compilation, requested outputs, size, and parameter overrides are caller-visible inputs.
4. **No hidden semantic fallback.** A failed GPU request returns a structured error; it does not silently switch to another renderer.
5. **Inputs are deterministic by construction.** Randomized nodes require explicit seeds, graph traversal is stable, and compiled plans have stable hashes.
6. **The public surface stays small.** New crates, nodes, file-format features, or optimization layers require evidence from a real material or consumer.
7. **Visual quality is part of correctness.** Three golden materials must pass structural, tiling, GPU, and human-review gates before the node vocabulary can expand.

## Explicit non-goals

The following are intentionally outside the initial roadmap:

- a React or desktop node editor;
- Gallery, Player, marketplace, accounts, or collaboration;
- a CPU pixel renderer;
- a separately maintained WebGL2 renderer;
- automatic `WebGPU -> WebGL2 -> CPU` fallback;
- subgraphs, function graphs, custom WGSL, or a plugin system;
- `.mixar`, a binary container, or embedded resources;
- engine-specific export profiles inside the core;
- 3D baking, mesh processing, or a PBR preview scene;
- cloud rendering or AI material generation.

These may be reconsidered only after a real consumer demonstrates that the simpler system is insufficient.

## Command surface

`check`, `validate`, `inspect --plan`, `render`, `doctor`, `render-builtin checker` and `asset pack|inspect|render` are implemented. See [plan inspection](./docs/render-plan.md), [graph rendering](./docs/graph-rendering.md) and [asset adapters](./docs/m6b-04-adapters.md) for options, channel encoding, and reports.

```bash
# Repository verification
cargo xtask check

# Environment and adapter diagnostics
cargo run -p mixture-cli -- doctor
cargo run -p mixture-cli -- doctor --json

# Document and plan inspection
cargo run -p mixture-cli -- validate examples/checker.mix --json
cargo run -p mixture-cli -- inspect examples/checker.mix --plan --json

# Headless rendering
cargo run -p mixture-cli -- render examples/blend.mix \
  --size 512 \
  --output baseColor,normal,roughness,height \
  --out ./out
```

Targeted development commands are documented in [AGENTS.md](./AGENTS.md).

## `.mix` v1 source

The first executable schema is defined in [the file-format guide](./docs/file-format.md). This checker validates today:

```json
{
  "version": 1,
  "nodes": [
    {
      "id": "checker",
      "type": "checker",
      "version": 1
    },
    {
      "id": "out",
      "type": "material-output",
      "version": 1
    }
  ],
  "edges": [
    {
      "from": {
        "nodeId": "checker",
        "portId": "color"
      },
      "to": {
        "nodeId": "out",
        "portId": "baseColor"
      }
    }
  ],
  "exposedParameters": [
    {
      "id": "frequency",
      "nodeId": "checker",
      "parameterId": "cellsX"
    }
  ]
}
```

UI layout, metadata, thumbnails, presets, resources, and engine targets are rejected. `baseColor` requires a valid Color connection; optional channels use the [documented material defaults](./docs/node-contracts.md).

## Architecture at a glance

```text
                         no wgpu dependency
+-------------------+   -------------------->   +---------------------+
|   mixture-core    |                           |    mixture-wgpu     |
|                   |   RenderPlan              |                     |
| document          | ------------------------> | explicit GpuContext |
| validation        |                           | compute pipelines    |
| node contracts    |                           | resource lifetime    |
| compiler          |                           | readback             |
| diagnostics       |                           | execution metrics    |
+---------+---------+                           +----------+----------+
          ^                                                ^
          |                                                |
          +--------------------+  +------------------------+
                               |  |
                         +-----+--+------+
                         | mixture-cli  |
                         | thin I/O and |
                         | orchestration|
                         +-------------+
```

`mixture-wasm` (with the `packages/runtime` npm facade) and the CPU-only `mixture-asset` codec are thin layers over the same core compiler and `wgpu` renderer.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full ownership and execution model.

## Repository layout

The current layout uses five product crates plus one private tooling crate:

```text
mixture/
├── .cargo/config.toml      # exposes `cargo xtask`
├── crates/
│   ├── mixture-core/       # document, validation, node contracts, compiler
│   ├── mixture-wgpu/       # the only pixel executor
│   ├── mixture-asset/      # CPU-only .mixpack codec
│   ├── mixture-cli/        # validate, inspect, doctor, render, asset
│   └── mixture-wasm/       # thin browser bindings
├── packages/runtime/       # public npm ESM runtime over mixture-wasm
├── xtask/                  # repository-only automation
├── scripts/                # browser and material qualification scripts
├── fixtures/
│   ├── nodes/              # focused node cases
│   ├── materials/          # golden material acceptance assets
│   └── packages/           # .mixpack acceptance and rejection corpus
├── examples/               # small .mix documents and independent consumers
├── docs/                   # focused design and usage guides
├── AGENTS.md
├── ARCHITECTURE.md
└── ROADMAP.md
```

Do not add a new crate merely to create a conceptual boundary. A new crate requires a runtime, publication, dependency, or build boundary that cannot be represented cleanly inside the existing product crates.

## Golden materials

M3 accepted these three materials; their evidence and regression gates remain required as the catalog evolves under the [node admission rules](./ARCHITECTURE.md#72-reviewed-node-catalog-and-admission):

- a glazed ceramic/checker material for graph and tiling fundamentals;
- a leather-like material for micro-height, roughness, and normal behavior;
- an anisotropic wood-like material for directional structure and warp behavior.

Each golden material must include:

- a readable `.mix` source;
- fixed parameter variants;
- expected outputs from a pinned software adapter;
- tolerant cross-adapter comparisons;
- tiling and non-degeneracy measurements;
- a short human-review record explaining why the result is acceptable.

A passing unit test is not sufficient evidence that a material looks correct.

## Design inspirations

Mixture borrows a few focused ideas without copying the surrounding product scope:

- [`wgpu`](https://github.com/gfx-rs/wgpu): one cross-platform Rust GPU API for native and WebAssembly targets;
- [`vgpu`](https://github.com/vercel-labs/vgpu): explicit GPU context, small public API, executable diagnostics, and agent-discoverable documentation;
- [`vinext`](https://github.com/cloudflare/vinext): clear agent instructions, task-to-test mapping, thin adapters, and small reviewable changes.

## Project documents

- [AGENTS.md](./AGENTS.md) — operating rules for coding agents and contributors.
- [ARCHITECTURE.md](./ARCHITECTURE.md) — system boundaries, invariants, data flow, and testing model.
- [ROADMAP.md](./ROADMAP.md) — milestone outcomes, exit criteria, and stop rules.
- [INITIAL_PRS.md](./INITIAL_PRS.md) — historical M0–M3 implementation batches and their acceptance requirements.
- [M4_PRS.md](./M4_PRS.md) — completed native-consumer implementation batches and their historical evidence.
- [M5_PRS.md](./M5_PRS.md) — completed browser runtime, npm package consumption and independent Player work items.
- [Agent playbooks](./docs/agent-playbooks.md) — task-specific procedures for nodes, formats, shaders, goldens, GPU debugging and performance.
- [Glossary](./docs/glossary.md) — milestone and work-item IDs and evidence terms.
- [Browser SDK contract](./docs/browser-sdk.md) — package, initialization, input/output and lifetime requirements, with the implemented checker slice and remaining acceptance distinguished.
- [Repository governance](./docs/governance.md) — integration branches, actual GitHub pull requests, and required-check policy.
- [Evidence retention](./docs/evidence-policy.md) — accepted records, temporary run output, and artifact availability.
- [Documentation index](./docs/README.md) — development instructions, architecture decisions, and supporting guides.

## License

Licensed under either [Apache-2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT), at your option. Current pre-alpha packages retain `publish = false`; publication requires a separate release decision.
