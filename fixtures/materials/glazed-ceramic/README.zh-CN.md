# 釉面陶瓷／棋盘格

[English](./README.md) | 简体中文

PR-008 候选是粗糙度可独立调节的光滑双色陶瓷／棋盘目标。针对平面预览只展示棋盘的反馈，现已通过[受控 PBR 评审](./review/README.zh-CN.md)，将真实导出通道用于球体和平面样板，展示亮面与哑光响应；用户已接受受控观感对照。生成的贴图不含填缝、倒角、裂纹、微高度或光照。

## 源图与输出

[material.mix](./material.mix)使用现有六种 M2 节点。`checker` 与 `constant-color` 输入 screen `blend` 生成基础颜色；`constant-scalar` 经 `levels` 生成粗糙度；`material-output` 提供文档规定的中性法线和零高度平面。计划含默认值在内共八个计算 pass，请求四个通道。M2 没有颜色转标量或生成法线的生产者，本项工作没有为模拟它们而新增节点。

| 输出 | 含义与预期默认值 |
| --- | --- |
| `baseColor` | 不透明 8×8 交替格，两色各占一半；已测软件适配器的 sRGB RGBA8 为 `[224,232,226,255]` 与 `[98,128,142,255]`。 |
| `normal` | 默认编码切线空间 +Z，线性 RGBA8 `[128,128,255,255]`。 |
| `roughness` | 已连接的空间均匀釉面响应，线性 RGBA8 `[43,43,43,255]`，约 0.17。 |
| `height` | 文档规定的默认平面，线性 RGBA8 `[0,0,0,255]`；零值是有意的，不宣称具有凹凸。 |

格子数量为偶数、且尺寸可被格子数量整除时，交替图案可连续重复。目前以 1024×1024、每轴 8 或 16 格验收。其他参数仍受现有节点契约约束，本材质验收不保证奇数格子无缝平铺。

## 暴露参数与变体

| 公共控制 | 源参数 | 默认值／作用 |
| --- | --- | --- |
| `tilesX`／`tilesY` | checker `cellsX`／`cellsY` | 8／8；同时修改可调整方格密度。 |
| `glaze` | blend `opacity` | 0.12；在线性颜色空间混向 screen 结果。 |
| `roughness` | scalar `value` | 0.2，由 levels 映射到偏光滑的 `[0.05,0.65]` 范围。 |

[fine-tiles](./variants/fine-tiles.json)将两轴格数设为 16。调色与所有其他通道保持不变，50% 的基础颜色像素切换到另一格色；最低验收比例为 45%，结构检查另行要求更密的周期与跳变数。

[matte](./variants/matte.json)将 roughness 设为 0.65。全部粗糙度像素由 43 升至 112，归一化增幅约 0.2706，验收下限为 0.25；颜色、法线和高度逐字节不变。这些变体验证参数因果关系，不是在 `.mix` 中保存预设。

## 复现与检查

```bash
cargo run --locked -p mixture-cli -- validate fixtures/materials/glazed-ceramic/material.mix --json
cargo run --locked -p mixture-cli -- inspect fixtures/materials/glazed-ceramic/material.mix \
  --plan --size 1024 --output baseColor,normal,roughness,height --json
cargo run --locked -p mixture-cli -- render fixtures/materials/glazed-ceramic/material.mix \
  --size 1024 --output baseColor,normal,roughness,height --out tmp/ceramic --json
cargo xtask test-material glazed-ceramic
```

[基准工作流](../../../docs/material-goldens.zh-CN.md)说明软件适配器准备、精确／容差比较及独立受保护更新。[acceptance.json](./acceptance.json)是可执行验收契约；[schema](../acceptance.schema.json)及 Rust 跨字段检查拒绝不完整配置。

软件参考输出位于 [expected/](./expected/)，审查图与机器证据位于 [reports/](./reports/)。现有朴素生命周期模型报告 1K 逻辑峰值为 75,497,648 字节，约 72 MiB，八个计算 pass。此值不含驱动／管线开销，不是 2K 验收或优化结果。

## 审查状态

[human-review.json](./reports/human-review.json)明确区分智能体检查与人工验收。通过基准比较及 `--accept` 均不会签署该记录。现已用[受控观感对照图](./reports/pbr/comparison.png)补充平面通道色块；[输入及场景记录](./reports/pbr/review.json)绑定十二张未改动 PNG、Blender 版本／设备及脚本。默认／细格版的灯箱和样板反射比哑光版清晰。球体曲率及底座边缘属于消费端几何体；法线仍为中性，高度仍为零。这支持审查光滑釉面棋盘候选，不宣称生成了表面细节，也不能唯一确定物理材质种类。PR-008 人工验收已针对未改动的清单及 PBR 证据记录；完整 M3 保持开放。用户已暂缓远端 CI，本项工作不宣称关闭远端里程碑。

不在范围内：新增节点、噪声、生成法线、凹凸、资源池、2K 优化、运行时查看器、浏览器与远端 CI 验收。
