# 砖墙／铺地砖材质候选

[English](./README.md) | 简体中文

这是尚未接受的 MAT-01c 四通道[材质](./material.mix)，实现[冻结目标](../../../docs/mat-01-structured-materials.zh-CN.md)。十个 pass 生成 baseColor、roughness、height 和 normal。对齐的 brick-pattern 副本分别提供高度幅度、颜色变化及共享表面遮罩；颜色与粗糙度共用遮罩，法线从最终高度派生。

[controls.json](./controls.json) 是概念控制到公开覆盖 ID 的验收夹具映射，不是新运行时 schema 或自动参数传播。调用方对共享布局控制一起修改所有列出的公开覆盖；heightVariation 与 colorVariation 保持独立。编译图中的重复节点没有隐藏联动。未来图／参数复用由 MAT-04 负责。

[qualification-plan.json](./qualification-plan.json) 保留冻结用例和预算。本候选尚无接受的 golden、人工评审或完整跨端结论，不得仅因源码可渲染就安装 golden。MAT-01d 须提供完整矩阵、PBR 评审、包／公开浏览器消费及留存证据。

候选 [Native 矩阵测试](../../../examples/native-consumer/tests/brick_material.rs) 通过公开 API 渲染全部二十组用例／尺寸，保留八十张通道 PNG，验证重复像素精确一致、描述符／pass 预算、独立参数效果和默认降采样，验证包往返的源码／计划／像素精确一致，并在实测适配器匹配冻结硬件／软件目标时执行一次冷渲染和五次热渲染预算。回执只是部分引擎证据，不代表材质验收。候选浏览器消费者渲染相同二十组用例。`node scripts/browser-runtime/check-brick.mjs <candidate-qualification> <fresh-output>` 将八十组对照绑定到源码／构建身份，并以 release 模式运行 Native 工具。`test-node brick-pattern` 将测试专用周期采样原点与生产像素的周期移位对照，并验证原始 f16 范围。Native 矩阵记录 256² 相对 1024² 的 64×64 格压力，不放宽默认门槛。人工 PBR 评审继续单独处理。未知适配器不获得耗时预算认证；debug 耗时不认证 release 性能。

在仓库根目录使用 PowerShell 运行，明确选择后端，每次指定新输出目录：

```powershell
$env:MIXTURE_GPU_BACKEND='vulkan'
$env:MIXTURE_GPU_SOFTWARE='0'
$env:MIXTURE_BRICK_FIXTURE_DIR=(Resolve-Path fixtures/materials/brick-paving).Path
$env:MIXTURE_BRICK_EVIDENCE_DIR=Join-Path $env:TEMP 'mixture-brick-native-run-01'
cargo test --release --locked --manifest-path examples/native-consumer/Cargo.toml --test brick_material brick_material_public_gpu_matrix -- --ignored --nocapture
```

其他原生后端可选 `dx12` 或 `metal`；固定 SwiftShader 须按现有文档设置 Vulkan 驱动并指定软件策略 `1`。普通 CPU 测试只编译而不执行该 GPU 测试。已存在的输出目录会被拒绝，失败不会覆盖先前证据。目录包含精确输入副本及记录真实适配器／计划身份的 `native-matrix.json`；成功回执仅覆盖其明确声明的范围。

通过公开 CLI 预览：

```bash
cargo run --locked -p mixture-cli -- render fixtures/materials/brick-paving/material.mix --size 1024 --output baseColor,normal,roughness,height --out tmp/brick-paving-preview --json
```

`cargo xtask test-material brick-paving` 使用常规显式 GPU 策略运行 release Native 矩阵，向 `tmp/materials/brick-paving/` 写入新证据。它验证性质及包，不安装 golden 像素、不声明人工接受。原有三材质 `golden check` 保持不变；必需软件 GPU CI 运行这一新增材质门槛。

受控 PBR 评审：先执行 `npm ci --ignore-scripts --prefix examples/browser-consumer`，再执行 `node scripts/brick-material-preview.mjs <native-evidence> <fresh-output>`。工具在 Chrome 中使用 WebGPU 显示已有 PNG，固定相机、光照和介电 GGX 参数，生成每个预设的平面／球体及 1×／3× 平铺图；不执行材质图、不做几何位移。回执绑定输入 PNG 与预览源码摘要，不会自动声明人工接受。

已测开发主机限制：GT 1030 Vulkan 下，64×64 格／256² 的高度平均降采样误差约 14.13/255，baseColor 红通道约 7.99/255，超过默认 4/255 门槛。这个设置位于默认质量保证之外；保留测量，不重置门槛或自动修改输入。
