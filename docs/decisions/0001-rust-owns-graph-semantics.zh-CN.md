# ADR 0001：Rust 负责图语义

[English](./0001-rust-owns-graph-semantics.md) | 简体中文

状态：M0 基础工程已接受。来源：[架构第 3–4 节](../../ARCHITECTURE.zh-CN.md)。

## 背景

原生和未来浏览器使用方必须对文档含义、节点契约、验证和编译保持一致。多套语义实现会产生漂移。

## 决策

`mixture-core` 通过常规的强类型 Rust 模块负责这些语义和后端无关诊断。`mixture-wgpu` 使用编译后的计划，`mixture-cli` 负责 I/O 和编排。未来 WebAssembly crate 保持为同一组库之上的轻量绑定层。

## 备选方案

TypeScript 图运行时或归 CLI 所有的编译器会产生第二个语义归属。单一的大型 crate 则会让纯图测试与平台和命令行依赖耦合。

## 影响

核心测试不需要适配器、CLI 或浏览器环境。产品依赖方向指向 core。M0 建立空的 crate 根模块，不设计推测性的文档 API。

## 迁移

这是不承诺兼容性的新仓库。旧版 Mixture 代码属于研究材料，不需要自动导入或兼容层。

## 验证

`cargo xtask deps` 约束 M0 依赖图，`cargo xtask check` 编译全部 crate。后续格式和计划测试归 core 负责，必须能够在没有 GPU 的环境中运行。
