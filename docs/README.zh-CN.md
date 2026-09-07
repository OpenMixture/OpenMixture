# 文档索引

[English](./README.md) | 简体中文

根目录文档是当前项目契约：

- [项目使命与开始使用](../README.zh-CN.md)
- [贡献者与智能体指南](../AGENTS.zh-CN.md)
- [架构](../ARCHITECTURE.zh-CN.md)
- [路线图](../ROADMAP.zh-CN.md)
- [初始 PR 实施计划](../INITIAL_PRS.zh-CN.md)

[开发指南](./development.zh-CN.md)说明已实现命令和验证范围。
[诊断与安全限制](./diagnostics.zh-CN.md)定义 PR-002 公共 API、JSON 契约和计划中的 CLI 退出码策略。
[架构决策](./decisions/README.zh-CN.md)记录四项基础设计决策。

原始[审查文档包](../mixture-greenfield-docs/README.md)作为来源材料原样保留。其计划中的命令不代表后续里程碑已经实现。此后根目录文档与代码一起维护；文档包清单仅适用于原始包。

## 配套文档

- [示例说明](../examples/README.zh-CN.md)
- [节点夹具说明](../fixtures/nodes/README.zh-CN.md)
- [材质夹具说明](../fixtures/materials/README.zh-CN.md)
- [PR 描述模板](../.github/pull_request_template.zh-CN.md)

## 语言与同步维护

- 当前项目文档采用同目录配对：英文使用 `*.md`，简体中文使用 `*.zh-CN.md`。各页面顶部提供语言切换链接。
- 修改行为、命令、范围或验收条件时，在同一 PR 中同步更新两种语言。中文文档保留完整要求，不以摘要替代原文。
- 命令、路径、API／类型／节点标识符和可执行示例保持一致。中文正文优先链接中文版；源码、配置、许可证和原始审查包仍指向原文件。
- 原始 `mixture-greenfield-docs/` 是保留的历史来源。使用当前配对的项目文档了解实现状态。若两种语言出现差异，依据[指南中的优先级](../AGENTS.zh-CN.md)核对权威契约和实现，并修正两版。
