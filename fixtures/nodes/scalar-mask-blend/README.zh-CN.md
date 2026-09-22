# Scalar 遮罩混合夹具

[English](./README.md) | 简体中文

[MAT-01 契约](../../../docs/mat-01-structured-materials.zh-CN.md)定义必需 Scalar a/b/mask 输入与 opacity。[用例](./cases.json)验证精确遮罩／opacity 端点、部分 opacity、矩形及退化尺寸。Core 测试在 opacity 为零时也拒绝缺失／类型错误的遮罩和错误版本，保留重复绑定并验证裁剪哈希。

`cargo xtask test-node scalar-mask-blend` 还把负数／大于一的字面遮罩和输入纹素送入生产着色器，验证饱和与精确端点，不提供 CPU 渲染器。既有统一权重 scalar-blend 测试只共享 GPU 上传／回读辅助代码，原有期望保持不变。
