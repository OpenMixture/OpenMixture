# Node fixtures

English | [简体中文](./README.zh-CN.md)

The [PR-004 checker fixture](./checker/README.md) contains a reviewed GPU-generated PNG, raw RGBA golden, and pinned SwiftShader provenance. It is a fixed built-in probe, before the M2 node registry.

Each node will include defaults, boundaries, invalid parameters, a non-trivial case, and seed/tiling cases where relevant. A node is delivered with its Rust contract, one WGSL implementation, documentation, and targeted tests.

See [the node workflow](../../AGENTS.md) and [initial PR sequence](../../INITIAL_PRS.md). `cargo xtask test-node <id>` becomes available with its implementing PR; it is not a passing placeholder today.
