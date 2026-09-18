# 架构决策

[English](./README.md) | 简体中文

这些决策为 M0 / PR-001 采纳了提供的全新架构。它们记录项目设计选择，不代表运行时已经实现。

| ADR | 决策 | 状态 |
| --- | --- | --- |
| [0001](./0001-rust-owns-graph-semantics.zh-CN.md) | Rust 负责图语义 | 基础工程已接受 |
| [0002](./0002-wgpu-only-pixel-backend.zh-CN.md) | `wgpu` 是唯一像素后端 | 基础工程已接受 |
| [0003](./0003-readable-mix-dag.zh-CN.md) | `.mix` v1 是单个可读 DAG | 基础工程已接受 |
| [0004](./0004-no-implicit-semantic-fallback.zh-CN.md) | 不进行隐式语义回退 | 基础工程已接受 |
| [0005](./0005-warp-numerical-compatibility.zh-CN.md) | Warp 数值准确性与兼容边界 | 提议中；不改变运行时或接受状态 |

未来 ADR 必须包含背景、决策、备选方案、影响、迁移和验证。顺序编号，在此添加链接；契约变化时更新[架构](../../ARCHITECTURE.zh-CN.md)和[路线图](../../ROADMAP.zh-CN.md)。明确用新决策取代旧决策，不重写已接受决策的历史。
