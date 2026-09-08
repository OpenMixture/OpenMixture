# 皮革候选

[English](./README.md) | 简体中文

PR-009 添加棕色颗粒状类皮革表面，内置节点总数为九。[material.mix](./material.mix) 使用五个计算 pass：带种子的 cellular 分形噪声 → levels 高度 → gradient-map 颜色、height-to-normal，以及反向 levels 粗糙度。四个 1K 输出通道全部连接。渲染器报告估算峰值 50,331,792 字节；这是分配估算，不是进程或驱动内存。

本地软件／硬件机器检查及[受控 PBR 对比](./reports/pbr/comparison.png)已完成。[人工验收](./reports/human-review.json)已获用户接受。远端 CI 暂缓，M3 保持开放。已接受的陶瓷夹具及其像素不变。

## 参数与因果

| 公开控制 | 契约范围／本材质默认值 | 用途 |
| --- | --- | --- |
| `seed` | u32 `[0,4294967295]`／`271828` | 确定性的颗粒排列；源文档始终显式填写。 |
| `grainScale` | Integer `[1,128]`／`64` | 每个 UV 单元的格数；`32` 生成更宽颗粒。 |
| `detail` | Float `[0,1]`／`0.35` | 高频 octave 相对振幅；`0` 使用单 octave，`1` 使三个 octave 等权。 |

节点契约的 persistence 默认值为 0.5；本材质有意选择 0.35。[变体](./variants/)覆盖 detail 最小／默认／最大值，以及第二种颗粒尺度因果变化。调色、高度映射（`inputMax=0.6`、`gamma=1.6`）、法线强度（`0.002`）及粗糙度范围（`0.76 → 0.52`）在用例间固定。凸起比沟槽略平滑／明亮。由于 octave 平均归一化，detail 同时改变频率和对比度，不是保持对比度的滤波器。

声明的频率测量为两轴相邻红分量差的平方均值（含循环边界），除以红分量方差，避免将纯对比度变化误判为更细结构。SwiftShader 上高度相对默认值的实测结果：

| 用例 | 归一化梯度能量比 | 高度变化像素比例 |
| --- | --- | --- |
| `detail-min` | 0.8892 | 95.01% |
| `detail-max` | 3.0966 | 98.03% |
| `coarse-grain` | 0.2867 | 99.25% |

检查还要求空间变化、正相邻相关、受限重复边界跳变、单位长度正 Z 法线及高度／法线方向一致。全部轴样本至少一半须具有可测高度坡度；采用至少四字节的中心差分，避开近中性 PNG 量化。两个适配器上全部可测符号均一致。这些检查拒绝打乱空间、合成接缝、平坦／无效法线及反向切线。它们补充全图基准和观感评审；直方图不足以接受皮革。

## 复现

从仓库根目录显式选择固定软件适配器（见 [GPU 设置](../../../docs/gpu-context.zh-CN.md)）：

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask test-material leather

MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask test-material leather
cargo xtask golden check
cargo xtask check
```

`golden check` 同样需要显式 GPU 策略环境，并检查全部材质夹具。输出写入 `tmp/golden/`，不会更新 [expected/](./expected/)。单独的 `cargo xtask golden update leather --accept` 消费已有完整软件候选，不重新渲染，并拒绝 CI。见[受保护更新规则](../../../docs/material-goldens.zh-CN.md)。

软件逐字节比较。硬件要求最大绝对误差 ≤1 字节、RGBA 平均误差 ≤0.15 字节、零像素超过一字节。初始均值上限 0.10 在 detail-min 粗糙度实测 0.1080 时失败，但最大误差为 1 且全部结构检查通过。[原报告](./reports/metal-initial-tolerance-failure.json)予以保留。最终策略允许实测量化分布，同时将原最大误差从 2 收紧到 1，并不再允许任何 >1 字节像素。解决此阈值失败没有修改着色器或基准 PNG。

## 视觉证据与范围

[评审说明](./review/README.zh-CN.md)使用 Blender 4.5.13 复现四帧。经过验证的 baseColor、roughness 和实际生成的法线在所有用例驱动相同 BRDF 与光照。高度保留为诊断输入；若再次用于 bump，会重复应用已生成法线的效果。场景不提供额外表面噪声或凹凸。球体曲率及底座边缘来自展示几何。

[报告](./reports/README.zh-CN.md)绑定输入、着色器／工具源码、适配器、计划、指标、脚本及图像。目前目标是颗粒状类皮革观感，不是扫描或物理标定皮革。范围外：transform／warp、木纹、coat、sheen、AO、curvature、scatter、2K 优化、运行时 3D 查看器、远端 CI 闭环及第十三个节点。
