# 材质夹具

[English](./README.md) | 简体中文

PR-008 添加[釉面陶瓷](./glazed-ceramic/README.zh-CN.md)、四个 1K 通道、两个参数变体及受保护基准工具。PR-009 添加[皮革候选](./leather/README.zh-CN.md)、三个新节点及受控观感证据，人工验收已获用户接受。方向性木纹仍属 PR-010。人工审查与机器检查分别记录，远端 CI 保持暂缓。

每种材质目录包含 `material.mix`、`README.md`、`acceptance.json`、`variants/`、`expected/` 和 `reports/`。关闭 M3 前，机器检查和人工视觉验收必须一致。

基准更新要求显式接受标志，必须拒绝在 CI 中运行，并产生可审查的修改前／修改后／差异证据；不暂存或提交文件。参阅[已实现工作流](../../docs/material-goldens.zh-CN.md)、[验收 schema](./acceptance.schema.json)、[基准规则](../../AGENTS.zh-CN.md)和[M3 路线图](../../ROADMAP.zh-CN.md)。
