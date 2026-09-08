# 受控皮革评审

[English](./README.md) | 简体中文

此可选夹具消费者用固定外部 Blender 场景渲染实际十六张 1K PNG。[render.py](./render.py)复用未变的[陶瓷场景辅助模块](../../glazed-ceramic/review/render.py)，验证每个清单哈希并记录两个脚本哈希。它不读取 `.mix` 或执行节点公式。使用 Blender 4.5.13，以及[陶瓷复现说明](../../glazed-ceramic/review/README.zh-CN.md)中的已验证便携环境。

```bash
"$MIXTURE_REVIEW_BLENDER" --background --factory-startup --python-exit-code 1 \
  --python fixtures/materials/leather/review/render.py -- \
  --expected fixtures/materials/leather/expected \
  --out tmp/leather-pbr-review --device METAL --samples 512
python3 fixtures/materials/leather/review/compare.py --review tmp/leather-pbr-review
python3 fixtures/materials/leather/review/test_inputs.py --blender "$MIXTURE_REVIEW_BLENDER"
```

从仓库根目录运行。输出目录须全新且位于 `expected/` 外。`--device METAL` 要求显式 Metal 设备，不自动重试；其他平台可显式用 `--device CPU` 运行此外部场景消费者。默认 1000×1000、512 样本、固定种子 8，无降噪或自适应采样。Python 与 Pillow 12.3.0 仅用 Lanczos 缩小并加标签，不修图。输入／覆盖保护测试与 Rust 检查分开运行。

BaseColor 按 sRGB 读取，roughness 和 normal 按 Non-Color。生成法线通过切线 Normal Map（强度 1）直接连接 Principled Normal。高度加载供诊断检查，不额外连接 bump 或位移，其坡度已由法线表达。固定 BRDF 使用 IOR 1.5、metallic 0、coat 0、transmission 0、subsurface 0，不添加噪声、sheen 或凹凸。

所有用例共享相机、三个区域光、UV 球体、单次重复平面样本、底座几何及 AgX 显示变换。对比顺序为 detail 0、0.35、1，最后为 scale 32。更多细节增加精细结构，但降低大颗粒对比；更粗尺度拓宽颗粒。球体极点附近 UV 变形及少量 Cycles 采样颗粒属于展示效果。这是补充观感证据，不是逐像素基准或 M4 验收。[人工决定](../reports/human-review.json)单独记录。
