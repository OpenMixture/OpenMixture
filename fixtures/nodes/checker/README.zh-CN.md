# 棋盘格 v1 参考夹具

[English](./README.md) | 简体中文

此 PR-004 夹具属于固定内置探针，先于 M2 节点注册表。见[完整契约](../../../docs/builtin-checker.zh-CN.md)。

- [checker-64.png](./checker-64.png)：GPU 生成并经过查看确认的 64×64 预览。
- [checker-64.rgba](./checker-64.rgba)：16,384 字节紧密排列的 RGBA8，左上角原点；用于比较的基准。
- [provenance.json](./provenance.json)：固定 SwiftShader 提交、平台、格式、哈希及初始审查记录。

来源是唯一的 [WGSL 实现](../../../crates/mixture-wgpu/shaders/nodes/checker.wgsl)。预期像素从 SwiftShader 捕获，没有使用 CPU 参考渲染器生成。图像为 8×8 网格，左上角黑色，alpha 完全不透明，每种颜色各占 2048 像素。Apple M5／Metal 生成相同的解码字节。

`cargo xtask gpu-smoke` 与这些字节比较，将候选输出、报告和比较证据写入已忽略的 `tmp/gpu-smoke/`，不会覆盖基准。修改必须遵循[基准审查策略](../../../AGENTS.zh-CN.md)，包括视觉审查；不得只为使测试通过而复制新输出。通用 `golden update` 命令仍未实现。

定向测试还覆盖 1×1、65×3、非整齐工作组、行填充、零／超大尺寸、预算失败、重复渲染／销毁、错误着色器／管线、已销毁设备、映射失败及 PNG 编码／I/O。普通测试无需 GPU；冒烟测试显式启用默认忽略的 GPU 用例。

PR-007 的 [input.mix](./input.mix) 和 [cases.json](./cases.json)覆盖 `checker` 图契约。用例：`defaults`, `odd-size`, `one-cell`, `max-cells`, `colors`, `256-grid`。清单包含 6 个有效 GPU 用例和 4 个无效源文件／覆盖用例。样本为固定预期 RGBA8 哨点，显式使用 0／1 码值容差，不通过 CPU 图求值器生成。无效用例必须在获取 GPU 前失败。

[契约](../../../docs/node-contracts.zh-CN.md)定义参数／默认值／公式；[图渲染](../../../docs/graph-rendering.zh-CN.md)定义 f32／f16 精度、通道编码及证据。像素路径使用 [checker.wgsl](../../../crates/mixture-wgpu/shaders/nodes/checker.wgsl)。`material-output` 自身仅映射资源，生成的默认值使用常量 kernel。

```bash
cargo xtask test-node checker
```

此命令先在无 GPU 环境验证各夹具，再显式使用选定适配器运行该节点。渲染指南链接本地 Metal 和固定 SwiftShader 结果。像素基准仍受保护，夹具命令不会更新基准。
