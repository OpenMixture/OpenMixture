# Mixture

> A small material-graph compiler and headless texture renderer built in Rust on top of `wgpu`.

**Status:** greenfield, pre-alpha, no compatibility promises yet.

**Implemented:** M0 / PR-001 repository foundation. The CLI exposes help and version only; material parsing, GPU acquisition, and rendering are not implemented yet. The cross-platform milestone gate awaits CI evidence.

Mixture is designed to read a versioned `.mix` material document, validate and compile its directed acyclic graph, execute the resulting compute passes through one `wgpu` renderer, and return requested PBR texture channels.

The project deliberately starts as an engine and command-line tool rather than a full material authoring product.

## Getting started

Install [rustup](https://rustup.rs/) and the native linker for your platform, then run from the repository root:

```bash
cargo xtask check
cargo run --locked -p mixture-cli -- --help
```

The repository pins Rust 1.98.1 / edition 2024 and includes `Cargo.lock`. The check covers formatting, dependency boundaries, Clippy, tests, rustdoc, and local document links without a GPU. See [development instructions](./docs/development.md) for all implemented commands and platform prerequisites.

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

## Planned command surface

The repository begins at roadmap milestone M0. Commands below are the stable target interface and become available as their milestones land.

```bash
# Repository verification
cargo xtask check

# Environment and adapter diagnostics
cargo run -p mixture-cli -- doctor
cargo run -p mixture-cli -- doctor --json

# Document and plan inspection
cargo run -p mixture-cli -- validate examples/wood.mix --json
cargo run -p mixture-cli -- inspect examples/wood.mix --plan --json

# Headless rendering
cargo run -p mixture-cli -- render examples/wood.mix \
  --size 512 \
  --output baseColor,normal,roughness,height \
  --out ./out
```

Targeted development commands are documented in [AGENTS.md](./AGENTS.md).

## `.mix` v1 direction

The first format is intentionally human-readable and editor-agnostic. This is a planned M2/M3 example, not an executable M0 fixture:

```json
{
  "version": 1,
  "nodes": [
    {
      "id": "noise",
      "type": "fractal-noise",
      "nodeVersion": 1,
      "parameters": {
        "scaleX": 3.0,
        "scaleY": 22.0,
        "seed": 17
      }
    },
    {
      "id": "color",
      "type": "gradient-map",
      "nodeVersion": 1,
      "parameters": {
        "gradient": [
          { "position": 0.0, "color": [0.08, 0.03, 0.01, 1.0] },
          { "position": 1.0, "color": [0.62, 0.35, 0.12, 1.0] }
        ]
      }
    },
    {
      "id": "output",
      "type": "material-output",
      "nodeVersion": 1,
      "parameters": {}
    }
  ],
  "edges": [
    {
      "from": { "node": "noise", "port": "value" },
      "to": { "node": "color", "port": "input" }
    },
    {
      "from": { "node": "color", "port": "color" },
      "to": { "node": "output", "port": "baseColor" }
    }
  ],
  "exposedParameters": [
    {
      "id": "grainScale",
      "label": "Grain Scale",
      "target": { "node": "noise", "parameter": "scaleY" }
    }
  ]
}
```

UI layout, thumbnails, presets, engine targets, and local editor state do not belong in `.mix` v1. The initial material output requires a connected `baseColor`; other PBR channels may use documented neutral defaults, and reports must identify connected versus default outputs.

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

## License

Licensed under either [Apache-2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT), at your option. Package publication is disabled during the foundation milestone.
