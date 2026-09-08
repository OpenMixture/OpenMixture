# 受控类木材评审

[English](./README.md) | 简体中文

此可选夹具消费者用固定外部 Blender 场景渲染十六张经过验证的 1K PNG。[render.py](./render.py)复用未变的[陶瓷场景辅助模块](../../glazed-ceramic/review/render.py)，要求版本为 1 的木材清单，精确包含四个用例各自的四个通道，验证每张 PNG 的哈希并记录两个脚本哈希。它不读取 `.mix` 或执行节点公式。使用 Blender 4.5.13，以及[陶瓷复现说明](../../glazed-ceramic/review/README.zh-CN.md)中的已验证便携环境。

运行观感渲染前，先通过[受保护的基准工作流](../../../../docs/material-goldens.zh-CN.md)创建并接受木材软件基准。添加这些脚本不代表已经生成 PBR 结果或通过人工验收。

```bash
"$MIXTURE_REVIEW_BLENDER" --background --factory-startup --python-exit-code 1 \
  --python fixtures/materials/wood/review/render.py -- \
  --expected fixtures/materials/wood/expected \
  --out tmp/wood-pbr-review --device METAL --samples 512
python3 fixtures/materials/wood/review/compare.py --review tmp/wood-pbr-review
python3 fixtures/materials/wood/review/test_inputs.py --blender "$MIXTURE_REVIEW_BLENDER"
```

从仓库根目录运行。输出目录须全新且位于 `expected/` 外；完成报告写入前会复查输入及两个场景脚本。`--device METAL` 显式要求 Cycles Metal 设备并记录其身份，不自动重试。其他平台可显式使用 `--device CPU` 运行此外部场景消费者；两种设备均要求固定 Blender 版本。默认 1000×1000、512 样本、固定种子 8，无降噪或自适应采样。Python 与 Pillow 12.3.0 在验证木材评审身份、四张渲染图的哈希及尺寸后，仅用 Lanczos 缩小并加标签，不修图。已有对比文件不会被覆盖。

BaseColor 按 sRGB 读取，roughness 和 normal 按 Non-Color。生成法线通过切线 Normal Map（强度 1）直接连接 Principled Normal。高度加载供诊断检查，不额外连接 bump 或位移，其坡度已由法线表达。固定 BRDF 使用 IOR 1.5、metallic 0、coat 0、transmission 0、subsurface 0，不添加噪声、sheen 或凹凸。

所有用例共享相机、三个区域光、UV 球体、单次重复平面样本、底座几何及 AgX 显示变换。[compare.py](./compare.py)按下表顺序排列；只有经过验证的 PNG 输入发生变化：

| 用例 | grainRepeat | warpStrength | orientation | 对比意图 |
| --- | ---: | ---: | ---: | --- |
| `default` | 32 | 0.018 | 0 | 带扭曲纵向纹理的类木材候选。 |
| `coarse-grain` | 16 | 0.018 | 0 | 减少重复次数，使纹理变宽。 |
| `straight-grain` | 32 | 0 | 0 | 零位移去除 warp 扭曲。 |
| `horizontal-grain` | 32 | 0.018 | 1 | 四分之一圈旋转改变主要纹理方向。 |

这些方向定义在纹理空间中；场景透视和球体 UV 变形会改变其显示方向。Cycles 采样颗粒属于展示效果。场景用相同的各向同性消费者 BRDF 展示方向性纹理结构，不添加各向异性着色器、木纤维、节疤或物理木材模型。这是补充观感证据，不是逐像素基准或 M4 验收。类木材结果的人工接受已单独记录于 [human-review.json](../reports/human-review.json)。原始图注与自动报告保留接受前的措辞，作为历史捕获证据。

[输入／覆盖保护测试](./test_inputs.py)使用临时的纯哈希预检夹具，刻意在场景创建或图片加载前停止；不需要木材基准或 PBR 渲染，且与 Rust 检查分开运行。它们验证精确的十六文件清单、输入哈希、输出隔离和已有评审文件的保留。材质机器门槛仍为 `cargo xtask test-material wood` 和 `cargo xtask golden check`。
