# ENG-04 — Scalar 场组合能力

[English](./eng-04-scalar-blend.md) | 简体中文

ENG-04 实现 `scalar-blend` v1。[已批准设计 PR](https://github.com/OpenMixture/OpenMixture/pull/27)保留原始提案；本页描述实际实现。验收记录随实现 PR 及绑定源码的证据提供。

## 用例与契约

组合独立种子的低频结构（种子 11、尺度 4）和高频细节（种子 29、尺度 32），从组合场生成高度及法线。[完整夹具](../fixtures/nodes/scalar-blend/two-noise.mix)暴露 `detailWeight`。现有 Color `blend` 和单输入 Scalar `levels` 无法表达此双 Scalar 操作。

必需 Scalar 输入 `a`、`b`，输出 `value: Scalar`。有限 Float 参数 `weight` 范围 [0,1]，默认 0.5。各有限输入样本先钳制到 [0,1]。按既有 f64→f32 规则降低参数精度后，权重 0 精确选择 a，1 精确选择 b，否则计算 `clamp(a + (b-a)*weight, 0, 1)`。使用 `mixture_half4` 在 rgba16float 中存储 `(value,0,0,1)`。RGBA8 读回不是无损表示，不新增跨适配器末位一致保证。

这是线性混合：细节增加时低频贡献降低，不是叠加位移。端点仍要求并验证两个输入。整数 texel 读取保持兼容平铺，但不修复输入接缝。不引入随机状态、遮罩、新采样、颜色转换、CPU 像素执行器或新 crate。

## 兼容性与交付

源码 Rust 包升级到 0.2.0：公开且穷尽的 `KernelId`／`KernelInvocation` 枚举新增变体，可能破坏下游穷尽匹配。消费者须处理 `ScalarBlend`，不顺带增加 `non_exhaustive`。独立 Rust 消费者针对新 API 编译。

浏览器候选升级到 0.2.0-alpha.0。API schema 1、`.mix` v1、计划版本／哈希域及已有变体序列化保持不变，旧计划哈希快照继续作为回归测试。公开 npm 0.1.0-alpha.0 保持不可变，独立注册表测试要求对此新图返回 `MIX_NODE_UNKNOWN_TYPE`。不支持的节点版本、输入类型及权重继续返回结构化错误。

候选资格验证仅替换独立宿主的 runtime 依赖／版本／完整性。固定的一次性 Studio CI 宿主有两项明确的生产方兼容调整：目录数量 11→12、runtime 版本 0.1.0-alpha.0→0.2.0-alpha.0。暂存脚本在任一原始断言变化时失败，保留调整前后测试源码及哈希；生命周期、像素、材质和部署检查全部保留。不修改 Studio 仓库，不升级或部署产品。这是经过审查的版本契约更新，不是永久兼容旧目录数量。

[后续浏览器发行](./evidence/npm-020-alpha/README.zh-CN.md)已发布 0.2.0-alpha.0，并将注册表夹具升级为此精确版本，包含 Scalar 渲染。下文 0.1.0-alpha.0 拒绝新图描述实现时验收。Rust 包仍未发布，下游升级属于单独工作。

## 验证

- `cargo xtask test-core` 与 `test-plan`：默认值、边界、缺失／错误输入、版本、未使用分支覆盖、确定性排序／默认值／哈希及输出切片。
- `cargo xtask shader-check` 与 `test-node scalar-blend`：固定端点／中点、相同／反向输入、65×3 和单像素轴；注入有限越界样本，通过生产 WGSL 验证钳制。
- `cargo xtask test-consumer` 与 `gpu-smoke`：独立 Rust API 消费，1K 双噪声权重 0／0.25／0.5／1，与直接输入的端点高度／法线完全一致、非退化输出、权重因果变化、周期边界指标，以及 renderer 销毁后的输出所有权。
- 独立浏览器候选消费执行相同夹具及结构门槛，注册表消费明确拒绝新图。全部 9 项浏览器测试必须执行，无跳过或不稳定结果。原有三材质／v2 比较及固定软件 GPU 检查保持必需。
- `cargo xtask check` 和 6 项受保护 CI 检查控制集成。已有 golden 像素不更新。视觉检查使用 1K 高度／法线变体及平铺接触表；[夹具门槛](../fixtures/nodes/scalar-blend/README.zh-CN.md)在验收前固定。

空间遮罩、混合模式／数学节点族、叠加位移、HDR 域、图重写、图像资源、便携打包、UI 编辑及发布均不在范围内。

[验收证据](./evidence/eng-04/README.zh-CN.md)保留完整 1K 原生／浏览器图像、四组计划和直接像素比较。`node scripts/browser-runtime/check-scalar.mjs <候选验收目录> <新输出目录>` 在显式 GPU 策略下比较两个通道，沿用单分量差值上限 1；已纳入必需浏览器 CI。
