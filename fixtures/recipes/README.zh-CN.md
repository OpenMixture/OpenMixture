# 配方 fixture

[English](./README.md) | 简体中文

已接受的 [MAT-04a 契约](../../docs/mat-04a-recipe-contract.zh-CN.md)下的 MAT-04 可复用配方输入。这些是 JSON 源 fixture，不是像素 golden。

| Fixture | 用途 |
| --- | --- |
| [masked-layer-surface.mixrecipe](./masked-layer-surface.mixrecipe) | 契约中的跨族配方：一个遮罩在底层颜色／粗糙度与连接进来的上层输入之间选择 |
| [acceptance-plan.json](./acceptance-plan.json) | 契约冻结验收一节的机器可读形式：四个配方、两个父图、暴露 ID、预算、门槛和预设 |

`cargo test -p mixture-core --test recipes` 解码这些 fixture，把展开后的父图与其手工展开的 v1 图比较，并按其引用的已验收砖块和机织物 fixture 检查计划。其余配方、两个父图和预设语料随 MAT-04c 提供；届时计划中的值不变。
