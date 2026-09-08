# Mixture

English | [简体中文](./README.zh-CN.md)

> A small material-graph compiler and headless texture renderer built in Rust on top of `wgpu`.

**Status:** greenfield, pre-alpha, no compatibility promises yet.

**Implemented:** PR-001 through PR-010 are locally implemented, with eleven nodes and accepted ceramic, leather and [wood](./fixtures/materials/wood/README.md) appearances. The [M3 review](./docs/m3-review.md) records 1K release timings and bounded 2K allocation evidence. [M4 PR-011](./M4_PRS.md) now verifies [independent public Rust consumption](./docs/native-sdk.md), including owned GPU outputs and reused-renderer measurements; PR-012 verifies [CLI reports and exit codes](./docs/cli-contract.md) and fixes human diagnostic context. PR-013–015 remain planned. Remote CI remains deferred.

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

Validate the [checker document](./examples/checker.mix) without a GPU: `cargo run --locked -p mixture-cli -- validate examples/checker.mix --json`. See the [strict file format](./docs/file-format.md) and [eleven node contracts](./docs/node-contracts.md).

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

`check`, `validate`, `inspect --plan`, `render`, `doctor`, and `render-builtin checker` are implemented. See [plan inspection](./docs/render-plan.md) and [graph rendering](./docs/graph-rendering.md) for options, channel encoding, and reports.

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

A future `mixture-wasm` crate must remain a thin binding over the same core compiler and `wgpu` renderer.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the full ownership and execution model.

## Repository layout

The target layout uses three product crates plus one private tooling crate:

```text
mixture/
├── .cargo/config.toml      # exposes `cargo xtask`
├── crates/
│   ├── mixture-core/       # document, validation, node contracts, compiler
│   ├── mixture-wgpu/       # the only pixel executor
│   └── mixture-cli/        # validate, inspect, doctor, render
├── xtask/                  # repository-only automation
├── fixtures/
│   ├── nodes/              # focused node cases
│   └── materials/          # golden material acceptance assets
├── examples/               # small user-facing .mix documents
├── docs/                   # focused design and usage guides
├── AGENTS.md
├── ARCHITECTURE.md
├── ROADMAP.md
└── INITIAL_PRS.md
```

Do not add a new crate merely to create a conceptual boundary. A new crate requires a runtime, publication, dependency, or build boundary that cannot be represented cleanly inside the existing three product crates.

## Golden materials

Before the built-in node vocabulary can exceed twelve node types, Mixture must produce and preserve three accepted materials:

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
- [INITIAL_PRS.md](./INITIAL_PRS.md) — the first implementation train, ready to turn into issues and stacked pull requests.
- [Documentation index](./docs/README.md) — development instructions, architecture decisions, and supporting guides.

## License

Licensed under either [Apache-2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT), at your option. Package publication is disabled during the foundation milestone.
