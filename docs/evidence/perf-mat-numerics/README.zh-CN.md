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

## 首个差异阶段 — 2026-09-23

[运行 35869537767](https://github.com/OpenMixture/OpenMixture/actions/runs/35869537767)复现失败门槛，并成功采集全部十一阶段。[源码／产物记录](./stages/receipt.json)与[解码对照](./stages/comparison.json)绑定 Linux Native release 输出及此前 Windows Native 软件执行对照。派生图对象和计划哈希均相同。Macro、exposure、erodeX/Y、band、rustSpread、detailNoise、coatingHeight 的高度和法线像素完全一致。首个观察到差异的阶段是 `detail`：高度有 5,269 个像素不同（最大 1），法线有 62,663 个不同（最大 16）。随后 RustMask 出现差异，最终高度／法线在 1K 重现完整材质的 3／422 个差异像素、最大差异 1／4。这是 Native 软件执行诊断，不是另一份已接受的浏览器矩阵。

`detail` 节点是 gamma=1、[0,1] → [0.5,1] 映射的 `levels@1`。留存的 [Linux](./stages/linux-detail.mix)与 [Windows](./stages/windows-detail.mix)请求及其高度／法线图片保存首个差异证据。其余完整阶段 PNG 是产物 `10754816938` 中的普通诊断输出（2026-10-23 到期）；留存的[比较脚本](./stages/compare.py)需要 Pillow 及两个完整阶段目录才能重现解码比较。哈希不能替代未留存的图片。

gamma=1 的 `pow` 在半精度舍入中点附近求值是当前假设，尚未由 Linux 独立用例确认。诊断现在另渲染十二个不含噪声的 1x1 字面量对照：精确端点，以及仿射结果落在舍入中点的半精度可表示输入。独立常量参考采用 binary16 最近偶数舍入；两个饱和减法及精确端点放大器分别将正、负半精度步长误差暴露到高度／粗糙度。预期误差像素为 `[0,0,0,255]`。Windows 软件执行的十二例均为零，Linux 结果待采集。这些报告仍属诊断，此处不改变生产着色器、版本、容差或迁移策略。
