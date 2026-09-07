# Node fixtures

English | [简体中文](./README.zh-CN.md)

Reserved for focused built-in node fixtures beginning in M2. No nodes or pixel outputs are implemented in M0.

Each node will include defaults, boundaries, invalid parameters, a non-trivial case, and seed/tiling cases where relevant. A node is delivered with its Rust contract, one WGSL implementation, documentation, and targeted tests.

See [the node workflow](../../AGENTS.md) and [initial PR sequence](../../INITIAL_PRS.md). `cargo xtask test-node <id>` becomes available with its implementing PR; it is not a passing placeholder today.
