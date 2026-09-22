# 砖墙／铺地砖材质候选

[English](./README.md) | 简体中文

这是尚未接受的 MAT-01c 四通道[材质](./material.mix)，实现[冻结目标](../../../docs/mat-01-structured-materials.zh-CN.md)。十个 pass 生成 baseColor、roughness、height 和 normal。对齐的 brick-pattern 副本分别提供高度幅度、颜色变化及共享表面遮罩；颜色与粗糙度共用遮罩，法线从最终高度派生。

[controls.json](./controls.json) 是概念控制到公开覆盖 ID 的验收夹具映射，不是新运行时 schema 或自动参数传播。调用方对共享布局控制一起修改所有列出的公开覆盖；heightVariation 与 colorVariation 保持独立。编译图中的重复节点没有隐藏联动。未来图／参数复用由 MAT-04 负责。

[qualification-plan.json](./qualification-plan.json) 保留冻结用例和预算。本候选尚无接受的 golden、人工评审或完整跨端结论，不得仅因源码可渲染就安装 golden。MAT-01d 须提供完整矩阵、PBR 评审、包／公开浏览器消费及留存证据。

通过公开 CLI 预览：

```bash
cargo run --locked -p mixture-cli -- render fixtures/materials/brick-paving/material.mix --size 1024 --output baseColor,normal,roughness,height --out tmp/brick-paving-preview --json
```

现有 `test-material` 分发尚未接纳该候选；MAT-01d 先加入材质专项验证器，再声明该命令可用。
