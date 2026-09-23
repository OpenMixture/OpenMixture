# Scalar 饱和减法夹具

[English](./README.md) | 简体中文

[MAT-02 契约](../../../docs/mat-02-layered-weathering.zh-CN.md)准入 `scalar-subtract@1`，保留会在三节点半精度组合中消失的 W-I 边缘带。[用例](./cases.json)覆盖默认、零／一／相等／有序输入、正差、退化尺寸与未知参数拒绝。[放大图](./amplified.mix)用已有 levels 将精确 1/4096 差值映射为白色，独立验证真实图保留微小边缘，而不是由 RGBA8 舍入掩盖。

`cargo xtask test-node scalar-subtract` 还用负值／大于一字面量、分数相等场、半精度可表示差值及周期平移执行生产 WGSL，在输出编码前精确比较原始 rgba16float。Core 测试验证必需输入种类、不支持版本、无参数、有序／重复绑定、源码不变与裁剪哈希。包内 ABI 测试要求两个输入及 16 个零字节。测试不增加 CPU 像素执行器。

独立浏览器／Native 消费者各定义 64 组用例：八输入对 × 四尺寸 × 直接／放大输出，要求精确重复和解析像素预期。`node scripts/browser-runtime/check-subtract.mjs <candidate-qualification> <fresh-output>` 使用两个公开运行时精确比较计划哈希及字节；Chromium CI 必须运行。候选模式必需 19 项测试，已发布注册表消费仍为 13 项并排除尚不可用的节点。运行时 0.7 未发布；节点验收与完整涂漆金属验收单独进行，不改写原 golden 或历史失败。
