# 稳定 value noise — NUM-01

[English](./stable-noise.md) | 简体中文

维护者在 2026-09-22 的 [Windows 调查](./evidence/windows-numerics/README.zh-CN.md)之后，授权显式迁移噪声语义并重新验收材质。本变更在未发布的 **0.4.0-alpha.0** 候选中实现 `fractal-noise@2`，不发布该候选，也不追认 0.3.0 的硬件像素。

## 契约与兼容性

`.mix` 仍为版本 1，plan/API schema 仍为 2。已评审目录仍有十三种类型、十一个 kernel。最新 `fractal-noise` 契约为版本 2；`node_contract_version` 解析显式版本。版本 1 继续用于冻结回归、已有消费者和前后对照。验证和渲染都不改写文档，版本 3 被拒绝。新源码必须显式选择版本 2 才能获得变更后的 value-noise 语义。

`basis=value` 在 v2 中降低为 `NoiseBasis::StableValue`，v1 仍为 `Value`。计划包含节点版本及不同 basis，防止复用旧计划身份；旧 v1 计划哈希不变。`basis=cellular` 在两个版本中均保留原浮点计算和精度限制。稳定 value-noise 算术不意味着 cellular、warp 或任意完整图在所有硬件上数值一致。

显式 seed、哈希、周期 lattice、像素中心坐标、五次插值、octave 倍增及归一化加权和的数学设计不变。参数名、默认值、范围不变。只有 value basis 采用以下有限精度契约：

1. 使用范围 `[0, 2^24]` 的 Q0.24 整数，包含两个端点。Lattice 值仍取原哈希的高 24 位。
2. 用整数有理数 `(2*pixel+1)*period/(2*size)` 计算中心坐标，获得精确整数 cell，并将小数部分向下取整为 Q0.24。
3. 用 12 位分段组装精确 48 位乘积，以最近值、半值取偶舍入 Q0.24 乘法。插值按方向加或减舍入后的非负差值乘积。
4. 按 shader 中规定的顺序，对控制值 `[0,0,0,1,1,1]` 做 de Casteljau 插值，计算五次 fade。所有中间值均处于 `[0,1]`，避免有符号多项式消减。独立 f64 多项式探针将相对理想 fade 的误差限定在 16 个 Q 单位。
5. 已降低的 f32 persistence 乘 `2^24` 后截断到 u32，得到 Q0.24。后续权重及各加权值使用相同舍入乘法。以整数累计，再通过 24 步整数除法将归一化结果向下取整为 Q0.24。
6. 最终整数精确转换为 f32，乘精确的 `2^-24`，再使用现有整数半值取偶 f16 转换与纹理存储。Value basis 的舍入决策不依赖硬件浮点除法、多项式或乘加。

最多六个 octave，使累计值和总权重不超过 `6*2^24`；除法余数翻倍后小于 `12*2^24`。在运行时设备维度上限 8192 下，中心坐标分子小于 `2*8192*4096`，均适合 u32。首个权重为一，归一化分母非零。二次幂转换在所含上端点也精确。不引入可选 u64/f64 GPU 特性、第二渲染器、新依赖或隐藏适配器回退。

## 显式迁移与证据

迁移文档时，只将选定 `fractal-noise` 节点版本从 1 改为 2，保留原文档，并在采用前评审新高度/法线像素。仓库在三个原材质 fixture 旁提供 `material-noise-v2.mix`。原 `material.mix` 和 golden 保持为 v1 回归输入，不覆盖它们来吸收语义变化。公共 Native 资源及 Scalar fixture 现选择 v2。独立浏览器消费者对 0.4 候选使用显式 v2 文件，对精确已发布 0.3 registry 基线使用未改动的 v1 文件。

引擎拥有的材质准备工具支持显式迁移选项：

```sh
node scripts/browser-runtime/prepare-materials.mjs tmp/browser-native-v2 <full-tested-commit> --noise-v2
```

它选择已提交的迁移源码，不静默转换输入。清单记录 `noiseVersion:2` 和精确源码哈希。`browser-material-check` 对迁移文件核对哈希，应用相同的冻结结构、参数因果、通道关系和数值门禁，并拒绝未知迁移身份。浏览器 CI 在同一个固定临时消费者中运行原始 11 用例/44 通道矩阵及迁移矩阵，不修改 Studio。`candidate.mjs verify-noise-v2` 将独立记录绑定到候选/archive 和原有公共接口/部署门禁。

## 验证与验收边界

`cargo xtask test-node fractal-noise` 现覆盖两个源码版本、重复性、seed 因果、octave/persistence 等价关系、小尺寸矩形/退化轴、新的精确 v2 129×65 基线，以及生产算术与独立 u64/f64 运算的对照。这一算术 oracle 不渲染 CPU 材质像素。新 v2 基线与原有两个 v1 基线分开。`cargo xtask test-core`、`test-plan`、`shader-check`、`gpu-smoke`、`check` 的含义不变。

手动数值探针使用 basis 字段 2 选择新的 value 路径；六组 1K Windows Vulkan/Chrome 诊断用例的原始 f16 高度/法线完全一致，包括此前失败的 scale 7。这些观测不能替代干净候选包验收、DX12 覆盖、迁移材质评审或 CI。验收记录需分别绑定每个被测试的源码/archive。历史 Windows 失败对 v1 及已发布的 0.3 仍成立。

不在范围内：自动改写文档、发布 0.4、cellular/warp 数值重设计、纹理格式变更、放宽阈值、Studio 升级与部署。
