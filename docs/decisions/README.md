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

Future ADRs must include context, decision, alternatives, consequences, migration, and verification. Number them sequentially, link them here, and update [the architecture](../../ARCHITECTURE.md) and [roadmap](../../ROADMAP.md) when their contracts change. Supersede an accepted decision explicitly rather than rewriting its history.
