# 文档索引

[English](./README.md) | 简体中文

根目录文档是当前项目契约：

- [项目使命与开始使用](../README.zh-CN.md)
- [贡献者与智能体指南](../AGENTS.zh-CN.md)
- [架构](../ARCHITECTURE.zh-CN.md)
- [路线图](../ROADMAP.zh-CN.md)
- [初始 PR 实施计划](../INITIAL_PRS.zh-CN.md)
- [M3 评审与证据](./m3-review.zh-CN.md)
- [M4 实施计划](../M4_PRS.zh-CN.md)

[公开原生 Rust 消费](./native-sdk.zh-CN.md)定义 PR-011 API 所有权、依赖暴露、独立应用及 release 测量。
[原生 CLI 报告与退出码](./cli-contract.zh-CN.md)定义 PR-012 JSON 字段存在规则／类型、完整人类可读上下文、独立进程测试及部分写入行为。
[GPU 失败原因与上下文生命周期](./gpu-failures.zh-CN.md)定义 PR-013 丢失／OOM 分类、首个错误优先规则、受保护清理及独立销毁检查。
[开发指南](./development.zh-CN.md)说明已实现命令和验证范围。
[诊断与安全限制](./diagnostics.zh-CN.md)定义 PR-002 公共 API、JSON 契约和共享 CLI 退出码策略。
[GPU 上下文与 doctor](./gpu-context.zh-CN.md)说明 PR-003 获取流程、报告字段、退出码和固定软件适配器 CI。
[内置棋盘格](./builtin-checker.zh-CN.md)定义 PR-004 像素、回读、PNG 输出、基准来源和执行证据。
[严格 .mix v1 格式](./file-format.zh-CN.md)定义 PR-005 解码、验证、CLI 行为及源文件夹具。
[十一个内置节点契约](./node-contracts.zh-CN.md)定义图执行之前的类型化端口、参数、默认值及声明的语义。
[确定性 RenderPlan](./render-plan.zh-CN.md)定义 PR-006 请求、规范化、裁剪、类型化资源、估算、哈希和 CLI 检查。
[十一节点图渲染](./graph-rendering.zh-CN.md)定义 PR-007／009／010 执行、缓存、回读／PNG 编码、示例、测试及适配器证据。
[架构决策](./decisions/README.zh-CN.md)记录四项基础设计决策。

原始[审查文档包](../mixture-greenfield-docs/README.md)作为来源材料原样保留。其计划中的命令不代表后续里程碑已经实现。此后根目录文档与代码一起维护；文档包清单仅适用于原始包。

[材质基准](./material-goldens.zh-CN.md)定义 PR-008／009／010 受保护更新、夹具 schema、度量与陶瓷／皮革／木材审查工作流。

[仓库治理](./governance.zh-CN.md)定义 M4.1 主分支／PR 保护、必需检查、CI 触发及启用验证。
[证据保留](./evidence-policy.zh-CN.md)区分不可改写的历史验收、新摘要、临时附件和已验证平台范围。

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

[最新请求与有界消费者状态](./stale-results.zh-CN.md)定义 PR-014 代次处理、明确过期的展示、CLI 目录清理及 CPU／GPU 内存边界。

[本地软件包消费](./package-consumption.zh-CN.md)、[兼容性](./compatibility.zh-CN.md)及 [M4 发布／退出状态](./release.zh-CN.md)定义 PR-015 归档验证与剩余分发及硬件限制。
