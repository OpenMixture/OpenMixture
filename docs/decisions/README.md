# Architecture decisions

English | [简体中文](./README.zh-CN.md)

These decisions adopt the supplied greenfield architecture for M0 / PR-001. They record project design choices; they do not claim the runtime is implemented.

| ADR | Decision | Status |
| --- | --- | --- |
| [0001](./0001-rust-owns-graph-semantics.md) | Rust owns graph semantics | Accepted for the foundation |
| [0002](./0002-wgpu-only-pixel-backend.md) | `wgpu` is the only pixel backend | Accepted for the foundation |
| [0003](./0003-readable-mix-dag.md) | `.mix` v1 is one readable DAG | Accepted for the foundation |
| [0004](./0004-no-implicit-semantic-fallback.md) | No implicit semantic fallback | Accepted for the foundation |
| [0006](./0006-browser-quality-gates.md) | Bounded browser texture agreement | Implemented decision; qualification recorded separately |
| [0007](./0007-external-image-resources.md) | Caller-owned external image input | Implemented and qualified within recorded scope in M6A-02–05 |
| [0008](./0008-portable-assets.md) | Optional byte-only USTAR assets and CPU codec boundary | Implemented and qualified within recorded scope in M6B-03–05 |
| [0009](./0009-transient-texture-reuse.md) | Per-render last-use planning and compatible texture slots | Selected PERF-MAT design; runtime unimplemented |

Future ADRs must include context, decision, alternatives, consequences, migration, and verification. Number them sequentially, link them here, and update [the architecture](../../ARCHITECTURE.md) and [roadmap](../../ROADMAP.md) when their contracts change. Supersede an accepted decision explicitly rather than rewriting its history.
