# 受控观感评审

[English](./README.md) | 简体中文

此可选的离线 Blender 场景消费 [expected/](../expected/) 中经过验证的十二张 PNG。平面通道色块只能展示棋盘，无法展示粗糙度或反射光，本场景用于补齐这项证据。它属于夹具的评审工具，不读取 `.mix`、不实现 Mixture 节点，也不是 Rust 运行时或 `cargo xtask check` 的依赖。

## 复现

使用[官方发布目录](https://download.blender.org/release/Blender4.5/)中的 **Blender 4.5.13**，并按公开的 `blender-4.5.13.sha256` 校验下载文件。本次 macOS arm64 DMG 的 SHA-256 为 `663ce944257c61ff1d6aa09e15c8f57bbd8d59023adb2fa7edde33a9ed960b53`。解包后的本地应用即可运行，无需系统安装。

在仓库根目录运行，并将 `MIXTURE_REVIEW_BLENDER` 指向可执行文件：

```bash
"$MIXTURE_REVIEW_BLENDER" --background --factory-startup --python-exit-code 1 \
  --python fixtures/materials/glazed-ceramic/review/render.py -- \
  --expected fixtures/materials/glazed-ceramic/expected \
  --out tmp/ceramic-pbr-review --device METAL --samples 512
```

输出目录必须尚不存在。`METAL` 明确要求 Cycles Metal 设备并记录其身份，不自动改用 CPU。在其他平台可显式指定 `--device CPU` 运行此外部场景评审；这不改变 Mixture 唯一的 `wgpu` 贴图执行路径。可选 `--size` 和 `--samples` 默认分别为 1000×1000、128 个样本；固定种子 8，关闭降噪与自适应采样。保留证据采用 512 个样本，以减轻可见采样噪点。脚本要求固定 Blender 版本、十二张 1K 输入及匹配的清单哈希，并在渲染后再次检查输入身份。

三张 PNG 与 `review.json` 仅写入新的输出目录，不更新基准、不暂存文件、不授予人工批准。使用 Python 与 **Pillow 12.3.0** 生成带标签的对照图：

```bash
python3 fixtures/materials/glazed-ceramic/review/compare.py \
  --review tmp/ceramic-pbr-review
```

对照脚本验证渲染图哈希，以 Lanczos 缩小并加上说明，生成 `comparison.png` 与 `comparison.json`；不重新打光或修图。报告以 SHA-256 绑定脚本、实际输入及输出。PBR 图是补充评审证据，不是精确贴图基准；Cycles 或平台差异可能改变这些图像。保留证据位于 [reports/pbr/](../reports/pbr/)。

## 场景能证明什么

三个用例共用相机、三盏中性矩形面积灯、环境、球体、平面样板与 AgX 显示变换，实际使用全部四个导出通道：

| 通道 | 消费端连接 |
| --- | --- |
| baseColor | sRGB 贴图 → Principled Base Color。 |
| roughness | Non-Color 贴图 → Roughness，不重新映射。 |
| normal | Non-Color 贴图 → 切线空间 Normal Map → Bump Normal。 |
| height | Non-Color 贴图 → Bump Height，距离 0.02；常量零输入没有梯度。 |

共享的电介质 BRDF 采用 IOR 1.5、metallic 0，不添加 coat、透射、次表面散射、噪声或程序表面细节。这些是消费端展示假设，不是新增的 `.mix` 字段。参见 Blender 的 [Principled BSDF 文档](https://docs.blender.org/manual/en/4.5/render/shader_nodes/shader/principled.html)及[法线贴图文档](https://docs.blender.org/manual/en/4.5/render/shader_nodes/vector/normal_map.html)。

比较默认与哑光版：baseColor、normal、height 逐字节相同，只有 roughness 从 43/255 升到 112/255。因此，更宽、更柔和的反射是在此 BRDF 下验证导出粗糙度的响应。细格版保留默认粗糙度，将图案密度加倍。材质的公共 `glaze` 参数控制颜色混合，不是消费端 coat 层。

球体曲率、样板厚度及底座圆角来自展示几何体，不是生成的法线、填缝或高度细节。平面顶部展示一次 UV 重复；球体使用经纬 UV 投影，极点附近的格子会变形。中性法线及零高度使当前材质保持光滑。带图案的光滑电介质本身不能唯一证明物理材质种类或写实陶瓷效果，这项判断仍由人工审查者作出。

## 验证

渲染后查看三张图及对照图，再保留完整报告和图像。输入／覆盖保护可单独验证：

```bash
python3 fixtures/materials/glazed-ceramic/review/test_inputs.py \
  --blender "$MIXTURE_REVIEW_BLENDER"
cargo xtask check
```

原有机器门禁仍为 `cargo xtask golden check` 与 `cargo xtask test-material glazed-ceramic`，参见[基准工作流](../../../../docs/material-goldens.zh-CN.md)。评审工具不改变材质像素，也不会关闭[人工评审记录](../reports/human-review.json)、远端 CI、M3 或 M4 外部消费端里程碑。
