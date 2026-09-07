# 节点夹具

[English](./README.md) | 简体中文

[PR-004 棋盘格夹具](./checker/README.zh-CN.md)包含已审查的 GPU 生成 PNG、原始 RGBA 基准和固定 SwiftShader 来源。它是 M2 节点注册表之前的固定内置探针。

每个节点将包含默认值、边界、无效参数、非平凡案例，以及适用的种子／平铺案例。交付节点时，必须同时提供 Rust 契约、唯一 WGSL 实现、文档和针对性测试。

参阅[节点工作流程](../../AGENTS.zh-CN.md)和[初始 PR 顺序](../../INITIAL_PRS.zh-CN.md)。`cargo xtask test-node <id>` 随对应实施 PR 引入，目前不是一个会虚假通过的占位命令。
