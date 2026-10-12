# 配方 fixture

[English](./README.md) | 简体中文

已接受的 [MAT-04a 契约](../../docs/mat-04a-recipe-contract.zh-CN.md)下的 MAT-04 可复用配方输入。这些是 JSON 源 fixture，不是像素 golden。

| Fixture | 用途 |
| --- | --- |
| [masked-layer-surface.mixrecipe](./masked-layer-surface.mixrecipe) | 契约中的跨族配方：一个遮罩在底层颜色／粗糙度与连接进来的上层输入之间选择 |
| [brick-structure.mixrecipe](./brick-structure.mixrecipe) | 三个 `brick-pattern@1` 共享七个布局控制和种子；高度与颜色变化保持独立 |
| [weave-structure.mixrecipe](./weave-structure.mixrecipe) | 三个 `weave-pattern@2`（warp-share、coverage、height）共享七个织纹控制 |
| [directional-grain.mixrecipe](./directional-grain.mixrecipe) | 带种子的噪声、四分之一圈方向、levels 和渐变映射；每个实例必须设置种子 |
| [presets/](./presets/) | 每个父图的覆盖映射，每种材质三个，各自复现一个冻结用例 |
| [acceptance-plan.json](./acceptance-plan.json) | 契约冻结验收一节的机器可读形式：四个配方、两个父图、暴露 ID、预算、门槛和预设 |

父图为 [brick-paving/material-recipes.mix](../materials/brick-paving/material-recipes.mix) 和 [woven-fabric/material-recipes.mix](../materials/woven-fabric/material-recipes.mix)。`cargo test -p mixture-core --test recipes` 覆盖 Core 规则；`--test recipe_materials` 运行 CPU 门槛：对每个冻结用例（及机织物压力用例），展开父图与已验收 v1 图同构，公开 ID 和描述符预算与计划一致，纹理实例保持隔离，预设复现其用例。GPU 适配器上的像素相同属于 MAT-04d。
