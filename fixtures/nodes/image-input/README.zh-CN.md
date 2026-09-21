# image-input v1 — Core 夹具

[English](./README.md) | 简体中文

[height.mix](./height.mix)表达已批准的外部高度／噪声交叉混合及高度／法线输出。必需 `resourceId` 为 `heightSource`，使用同尺寸紧密 `rgba8-linear` 字节。`detailWeight` 保持 scalar-blend 的交叉混合语义。[资源合同](../../../docs/m6a-resource-contract.zh-CN.md)定义输入方向、R 解释、预算、接缝与版本策略。

M6A-02 通过 `cargo xtask test-core` 验证源码，使用 2×2 按行字节 `[0,11,22,0, 64,33,44,1, 128,55,66,2, 255,77,88,3]`。G/B/A 刻意不同；Core 测试绑定字节而不计算像素。非法／边界／覆盖／预算夹具是 [Core 资源测试](../../../crates/mixture-core/tests/resources.rs)中的显式用例。Native GPU 上传及像素资格待 M6A-03；`test-node image-input` 尚不是已实现的 GPU 验收目标。本次未接受像素金图。
