# .mix v1 格式夹具

[English](./README.md) | 简体中文

PR-005 源文件验证夹具遵循[文件格式](../../docs/file-format.zh-CN.md)与 [M2 节点契约](../../docs/node-contracts.zh-CN.md)。它们是 JSON 输入／诊断夹具，不是像素基准。

| 夹具 | 用途 |
| --- | --- |
| [valid/checker.mix](./valid/checker.mix) | 最小棋盘格 → baseColor、默认参数及默认频率暴露 |
| [valid/all-m2.mix](./valid/all-m2.mix) | 全部六个 M2 契约、levels／blend 遮罩、显式粗糙度 |
| [invalid/duplicate-key.mix](./invalid/duplicate-key.mix) | 即使值相同也拒绝重复 JSON 键 |
| [invalid/unknown-field.mix](./invalid/unknown-field.mix) | 拒绝未声明的顶层布局字段 |
| [invalid/unsupported-version.mix](./invalid/unsupported-version.mix) | 拒绝文档版本 2，不进行迁移 |
| [invalid/duplicate-ids.mix](./invalid/duplicate-ids.mix) | 拒绝有歧义的节点 ID |
| [invalid/duplicate-edges.mix](./invalid/duplicate-edges.mix) | 拒绝重复边身份和单输入冲突 |
| [invalid/unknown-port.mix](./invalid/unknown-port.mix) | 拒绝未声明的源端口 |
| [invalid/type-mismatch.mix](./invalid/type-mismatch.mix) | Scalar 不能连接要求 Color 的 baseColor |
| [invalid/cycle.mix](./invalid/cycle.mix) | 不连通的 a → b → a 环，以及一个不属于环的下游节点 |
| [invalid/cycle.diagnostics.json](./invalid/cycle.diagnostics.json) | 已审查、不含文件路径的精确环诊断 |
| [invalid/missing-base-color.mix](./invalid/missing-base-color.mix) | 必填连接不能由默认值代替 |
| [invalid/invalid-parameters.mix](./invalid/invalid-parameters.mix) | 参数范围、颜色形状和未知名称错误 |
| [invalid/invalid-exposed.mix](./invalid/invalid-exposed.mix) | 节点版本不能作为可变参数暴露 |

运行 `cargo xtask test-format`。核心测试还构造字节／数量边界、深度嵌套、重复转义键、错误端口方向、自环、源数据顺序置换及参数／公开绑定变体。CLI 测试读取夹具但不修改它们，并断言退出码和文件路径。尚未实现夹具更新命令。
