# 尚未解决的涂漆金属软件执行一致性失败

[English](./README.md) | 简体中文

PR #59 的 head `611e9c4afad7e4b6558a1c5a4bf538d13196650e`、实测合并源码 `9df804336d295c74b72f32b81371dc81a1b425b9` 在 [Chromium 运行](https://github.com/OpenMixture/OpenMixture/actions/runs/35836054345)中未通过冻结的涂漆金属 Native／浏览器法线门槛：1K 最大差异 4/255、2K 为 8/255，超过保持不变的 ≤1/255 上限。成本和时间检查通过不能验收像素。后续原始／noise-v2 浏览器材质步骤未执行。候选仍未接受、未发布。

[receipt.json](./receipt.json)以大小和 SHA-256 绑定留存报告及图片，记录源码身份、产物到期时间，并区分 Windows 对照与 Linux 验收。[ci-native.json](./ci-native.json)记录失败比较。本目录保留两种分辨率的 Native／浏览器法线和高度图供检查；完整普通输出位于产物 `10739483613`，到期时间为 2026-10-23。输入仍为未改动的[优化前夹具](../perf-mat-before/material.mix)。

Windows 对照使用 Chrome 153.0.8010.53 自带的软件 Vulkan 驱动：源码 `ddb2e831e0397ccc0e07eebb27c8684166072a11`（0.7 release）与 `611e9c4afad7e4b6558a1c5a4bf538d13196650e`（0.8 debug）生成的五通道 1K PNG 字节完全相同。构建配置不同，不能比较时间。解码后的 Windows 像素也与留存的 Linux 浏览器像素完全相同。[解码对照](./decoded-comparison.json)显示与 Linux Native 相比有 422 个法线像素不同，最大差异 4。此对照**不能**证明 Linux 优化前后等价，也未定位最早发生差异的节点，不能单独据此决定着色器修复或精度策略。

下一步限定诊断由既有 Chromium 任务在复用门槛失败后调用 [probe-painted-metal.py](../../../scripts/probe-painted-metal.py)。它通过生产 CLI、同一固定 Native 适配器，将十一处固定 Scalar 阶段分别连接高度和法线，以 1K 渲染。各派生 `.mix`、命令、计划哈希、CLI 报告及图片保留在该运行产物的 `tmp/painted-metal-stages/`。裁剪会改变分配，因此这些结果用于定位可能的差异，不能证明完整图行为不变。诊断不构成通过的门槛、替代渲染器或平台验收；不修改容差、golden、必需检查或着色器。

本地复现需要明确配置软件 Vulkan 驱动和新构建的 CLI：

```bash
python scripts/probe-painted-metal.py --cli target/release/mixture --output tmp/painted-metal-stages
```

Windows 使用实际 CLI 路径；输出目录不得已存在。脚本只依赖 Python 标准库。WSL 实验已停止，不将其构建或结果作为验收证据。Agent Guide 影响：未改变操作规则；此记录保留失败证据，诊断保持既有六项门槛。
