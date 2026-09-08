# 节点夹具

[English](./README.md) | 简体中文

PR-007 为每个 M2 节点提供默认值、边界、非平凡像素及无效输入：

- [constant-scalar](./constant-scalar/README.zh-CN.md)
- [constant-color](./constant-color/README.zh-CN.md)
- [checker](./checker/README.zh-CN.md)
- [levels](./levels/README.zh-CN.md)
- [blend](./blend/README.zh-CN.md)
- [material-output](./material-output/README.zh-CN.md)

每个目录包含可读 `.mix` 输入与 `cases.json` 清单。固定 RGBA8 样本坐标／值带有容差，无效用例指定稳定诊断码。公共契约验证无需 GPU；`cargo xtask test-node <id>` 显式运行选定节点的 GPU 用例。冒烟运行全部用例并记录实际适配器／计划证据。PR-009 添加显式种子噪声及其颜色／法线消费者。

现有[棋盘格 PNG／原始像素基准](./checker/README.zh-CN.md)和 SwiftShader 来源记录不变。测试命令不覆盖像素基准。见[图执行与证据](../../docs/graph-rendering.zh-CN.md)、[节点工作流](../../AGENTS.zh-CN.md)及[实施计划](../../INITIAL_PRS.zh-CN.md)。

- [fractal-noise](./fractal-noise/README.zh-CN.md)
- [gradient-map](./gradient-map/README.zh-CN.md)
- [height-to-normal](./height-to-normal/README.zh-CN.md)

PR-010 添加循环标量重采样与字面插值／旋转探针：

- [transform-2d](./transform-2d/README.zh-CN.md)
- [warp](./warp/README.zh-CN.md)
