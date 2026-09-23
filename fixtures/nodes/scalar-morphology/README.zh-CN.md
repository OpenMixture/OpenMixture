# Scalar morphology 夹具

[English](./README.md) | 简体中文

[选定 MAT-02 契约](../../../docs/mat-02-layered-weathering.zh-CN.md)定义环绕单轴腐蚀／膨胀。[用例](./cases.json)通过公开图编译器覆盖默认值、半径端点、两轴／两操作及非法值。Core 测试另覆盖必需 Scalar 输入、未知参数、不支持版本、非有限 JSON、确定性哈希与依赖裁剪。

`cargo xtask test-node scalar-morphology` 用固定二值支撑集合执行生产 WGSL，精确比较原始半精度结果：常量、原点／中心脉冲、细线、中心／跨边界矩形、半径 0/1/16、两轴／两操作、1x1/1x17/17x1/19x11 尺寸、周期平移、轴顺序交换及半径单调性。分数／越界字面输入测试精确恒等、极值和夹取。集合并／交仅为独立测试预期，不是 CPU 像素执行器。证据使用 `MIXTURE_NODE_EVIDENCE_DIR`。公开 Native／浏览器验收与材质接受单独进行。

独立浏览器消费者通过已安装的公开 SDK 增加 192 组固定支撑集合用例。`node scripts/browser-runtime/check-morphology.mjs <candidate-qualification> <fresh-output>` 通过公开 Native 准备／渲染 API 执行同名用例，要求计划哈希与 RGBA8 字节精确一致。候选模式必需 19 项浏览器测试；注册表模式保持历史 13 项，不声明新节点能力。Chromium CI 必需此包装器；定义了测试不代表验收通过。

[记录的 Windows 证据](../../../docs/evidence/mat-02-morphology/README.zh-CN.md)保留 Vulkan／DX12 相对 Chrome 各 192 组精确对照、源码身份、所选像素及失败的软件缓存数量门槛；不关闭完整 CI 或材质验收。
