# ENG-04 验收 — Scalar 场组合

[English](./README.md) | 简体中文

[PR #29](https://github.com/OpenMixture/OpenMixture/pull/29)实现[已批准的 Scalar 契约](../../eng-04-scalar-blend.zh-CN.md)。这些本地结果绑定原始源码，不绑定后续文档提交。原生 CLI 图像来自干净提交 `a4588f2946861e8f3f2d4b84ff9c7439e43d5276`；候选浏览器及直接原生／浏览器比较来自干净提交 `4eb8f215e14821ac700574b649fb6ff3ad919530`。[review.json](./review.json)绑定图和生产 shader 字节；两个提交之间仅增加像素捕获／比较测试，未改变运行时语义。注册表消费使用较早的干净消费者 `a4588f2`，原公开包身份单独记录。

## 结果与审查内容

- [候选包](./candidate.json)：未发布 `0.2.0-alpha.0`，全部 9 项浏览器测试通过，无跳过／不稳定结果。[浏览器 Scalar 结果](./candidate-scalar.json)保留四组 1K 计划哈希、输出哈希、范围、边界比例、变化、适配器报告及浏览器版本。
- [注册表包](./registry.json)：已发布 `0.1.0-alpha.0`，全部 9 项测试通过；[新图被明确拒绝](./registry-scalar.json)，错误为 `MIX_NODE_UNKNOWN_TYPE`。这是旧运行时拒绝验证，不是新功能的像素验收。
- [直接比较](./comparison.json)：四种权重 × 高度／法线，每张为 1024×1024 RGBA8。8 张输出的最大分量差均为 1，沿用比较前选定的既有上限。每张 4,194,304 个分量仅有 3–15 个不同，不声称跨后端逐字节一致。各执行器内部，端点与直接输入场完全一致；每次请求后描述符存活字节为零，销毁后仍持有输出。
- 权重 0、0.25、0.5、1 的高度范围分别为 218、218、222、254 级。原生边界步长／内部步长比例分别为 0、0.001493、0.001230、0.000839，均小于夹具门槛 2。相邻权重通过固定的变化字节 >10% 门槛。
- 保留 [CLI 报告](./cli.json)、8 张原生和 8 张浏览器 PNG、精确依赖锁及[文件哈希／大小](./files.json)。[平铺接触表](./contact.png)以 2×2 重复显示所有原生高度／法线变体。[代理审查](./review.json)接受此有界线性混合：细节逐渐增加、低频结构减弱、法线响应，未见明显平铺接缝。浏览器中点法线另以完整分辨率检查。这是代理目视检查，不是独立人工评审或新的通用材质质量保证。

主机信息保留在审查回执。原生使用报告中的 NVIDIA GeForce GT 1030／DX12；自动化 Chrome `153.0.8010.48` 显式使用 `--enable-unsafe-webgpu --ignore-gpu-blocklist`。浏览器隐藏硬件身份，不得由原生适配器推断浏览器适配器。这些结果不证明普通浏览器配置支持。PR 独立执行固定 Linux SwiftShader、原有三材质 golden／v2 门槛、CPU 平台及包检查。未替换 golden。

## 复现与保留

在对应干净源码修订上，使用仓库固定工具链及新的输出目录：

```powershell
cargo xtask test-node scalar-blend
$env:WASM_BINDGEN = '<wasm-bindgen 0.2.128 可执行文件>'
node scripts/browser-runtime/build.mjs
$env:MIXTURE_BROWSER_CHANNEL = 'chrome'
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
$env:MIXTURE_GPU_BACKEND = 'dx12'
$env:MIXTURE_GPU_SOFTWARE = '0'
node scripts/browser-runtime/check-scalar.mjs tmp/sdk-candidate tmp/sdk-scalar-comparison
```

各权重使用 `mixture render fixtures/nodes/scalar-blend/two-noise.mix --size 1024 --output height,normal --set detailWeight=0.5 --backend dx12 --out tmp/eng-04/visual/0.5 --json`，将权重替换为 0／0.25／0.5／1。[contact.py](./contact.py)使用 Pillow 拼接这些已有 PNG。渲染和证据生成不更新 golden，也不发布包。

初步失败仍记为失败：首个测试按数组位置取到法线而非按名称选择高度，修正测试时未改变门槛；首次隔离 Cargo 检查发现夹具路径超出消费者目录，现已内置夹具并检查源码一致性；一次浏览器尝试通过像素断言但无法序列化 BigInt 证据，修正报告后重新通过。以上均未用于放宽容差或改变 shader 语义。

支持此有限决策的内容保留在 Git。完整日志、Playwright 报告及候选归档仍属于本地／30 天 CI 临时输出；哈希无法恢复过期原件。复现产生新运行。后续受保护 CI 结果绑定各自修订，在 PR 中链接，不改写这些回执。
