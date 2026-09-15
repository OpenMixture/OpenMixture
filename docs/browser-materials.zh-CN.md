# 浏览器材质比较——M5-05

[English](./browser-materials.md) | 简体中文

原生参考生产者通过公开 CLI 执行全部 11 个既有验收用例，尺寸为 1024 × 1024，请求 baseColor／normal／roughness／height，保留原始源码字节和公开覆盖。准备步骤要求显式后端及新目录；相对归档生产者修订存在运行时实现漂移时拒绝继续。源码、夹具和构建身份与计划哈希分开记录。

```bash
MIXTURE_GPU_BACKEND=metal node scripts/browser-runtime/prepare-materials.mjs tmp/browser-native 4b914feb9f3365d292b27ea60c5e0b6004f745e8
# In the independent product checkout:
npm run test:materials -- /absolute/native-reference /absolute/new-browser-output
# Back in the engine checkout:
cargo xtask browser-material-measure tmp/browser-native /absolute/new-browser-output
cargo xtask browser-material-check tmp/browser-native /absolute/new-browser-output
```

Linux 使用已有锁定 SwiftShader 设置及显式 Vulkan／软件策略。产品只消费独立参考清单和已安装 tarball。生产资源在 `/player/` 下静态提供；缺失 WASM 或不可用 WebGPU 均失败。引擎比较解码浏览器 PNG，复用既有材质结构、接缝、非退化、因果性和高度／法线关系检查，不改变原生 golden。

`browser-material-measure` 记录差异并检查结构／语义门槛，但明确不接受像素容差。`browser-material-check` 另执行[逐通道容差](./browser-tolerances.json)。两者在浏览器输出目录写入 `comparison.json` 及逐用例的原生／浏览器／差异接触表。计划比较保持整数精确，规范化 f32 JSON 投影，并要求语义哈希一致。比较前检查原始清单及 PNG 摘要。

## 校准进行中

首轮本地 Metal／Chromium 测量的 11 个计划哈希全部一致。44 个通道中 40 个逐字节一致；四个木材粗糙度输出最多相差一个 RGBA8 单位，变化像素比例为 0.0001783371–0.0003967285。当前候选门槛要求 baseColor／normal／height 精确相等；roughness 最大绝对误差 1、平均绝对误差 0.001、像素阈值 0、变化比例 0.001。这些门槛在锁定 Linux 浏览器矩阵完成测量、校准记录评审前仍属暂定。本工具提交不声称完整 M5-05 验收通过。

新增 CI 矩阵锁定独立产品提交，并保留全部既有原生必需检查。原生 SwiftShader 与 Chromium 自带 SwiftShader 分别记录来源。测量任务通过不等于容差冻结或广泛硬件兼容。发布、公开网站托管、Studio 编辑及 M6 不在范围内。
