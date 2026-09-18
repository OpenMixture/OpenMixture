# 着色器数值稳定性评估 — 2026-09-17

[English](./README.md) | 简体中文

**蜂窝噪声采用局部坐标能明显改善数值稳定性，但不能作为保持现有像素不变的直接替换。** 生产着色器和冻结参照继续保留。本记录落实[算术缩减调查](../chromium-arithmetic-reduction/README.zh-CN.md)的下一步，不认证新的浏览器包，也不采用新的编译策略。

## 受控改动

测量前确定三个诊断版本：原写法、仅局部坐标、局部坐标加平方根差有理化。[隔离的一行补丁](./diagnostic-local.patch)将

```wgsl
vec2<f32>(neighbor) + vec2<f32>(0.2) + 0.6 * jitter - p
```

替换为

```wgsl
vec2<f32>(vec2<i32>(x, y)) + vec2<f32>(0.2) + 0.6 * jitter - fract(p)
```

整数格点身份、周期环绕、种子、九点搜索、octave 权重和半精度舍入均不变。小扰动不再先加到最大可达 4095 的绝对格点坐标上，再减去大坐标。这是实数数学上的等价，不是浮点逐位等价。第三个版本还把 `sqrt(second)-sqrt(nearest)` 改成 `(second-nearest)/(sqrt(second)+sqrt(nearest))`；它没有稳定优于更小的改动，因此没有进入完整材质实验。

## 稳定性与跨端测量

1024² 的默认皮革噪声缓冲区探针使用 seed 271828、scale 64、三个 octave、persistence 0.35。将原 registry 依赖的原生 Release／DXC 与每个普通浏览器比较：

| 探针写法 | 不同的原始 f32 数量 | 不同的半精度舍入值数量 | 原始值最大差 |
|---|---:|---:|---:|
| 原写法 | 1,032,165 | 6,298 | 9.0971589e-6 |
| 局部坐标 | 730,400 | 223 | 2.9802322e-7 |

Chrome 和 Edge 的统计相同。半精度差异数量减少 96.46%，原始值最大差约降为原来的 1/30.5。**半精度舍入后的最大差仍为 0.00048828125**，差异更少不意味着最坏半精度步长界限更小。两种写法都没有达到跨实现逐位一致。见 [browser-grid.json](./browser-grid.json)。本机隔离的严格原生诊断构建给出相同的网格比较统计。有理化版本进一步把半精度差异数量小幅降到 202，但没有降低原始值最大差，部分标量样本的误差反而增加。

[样本计划](./sample-plan.json)覆盖六组配置、每组 256 个可复现采样点，包括图像四角、零／最大种子、scale 1／128、octave 1／6、persistence 0／0.01／0.35／1，以及 129×65、1024²、2048² 尺寸。三个版本均在普通原生、严格原生、普通 Chrome 和 Edge 上执行。

初始[自行生成坐标的扫描](./measurements-samples.json)混合了 UV 除法与蜂窝算术差异，非二次幂尺寸尤其明显。后续实验给各实现提供**完全相同、已存储的 f32 UV 输入**，再将少量标量输出与使用相同 f32 常数、精确二次幂坐标缩放的 binary64 估算比较。它是较高精度诊断参照，不是精确 oracle、CPU 材质渲染器、新 golden 或替换后的验收目标。[固定输入测量](./measurements-fixed.json)保留全部结果、最坏样本及退步案例：

| 固定输入配置 | 原写法原生最大绝对误差 | 局部坐标原生最大绝对误差 |
|---|---:|---:|
| 默认皮革 | 3.39554e-6 | 1.06089e-7 |
| 最大周期、奇数尺寸 | 4.36555e-5 | 7.42572e-8 |
| 最大周期、2K | 4.43897e-5 | 8.57328e-8 |
| 最小周期 | 1.10067e-7 | 1.27004e-7 |

最小周期案例略有退步：原样本与 Chrome 没有半精度差异，局部坐标新增一个；低 persistence 案例也仍有一个。因此，这是大坐标幅度下的明显改善，不是所有输入均更优。未固定输入的奇数尺寸扫描，局部化后仍保留约 1.2e-4 的坐标生成误差，不能全部归因于距离公式。受测全网格的原始值和舍入值均有限且位于 [0,1]。

## 完整材质与 golden 影响

仅局部坐标改动被应用到源码 `909de3e2ecce1d1f02da07031b820c35a30b5554` 的隔离检出，使用原 registry 依赖。编辑前，GT 1030／DX12 上的原 fractal-noise 节点和皮革材质检查通过。编辑后，着色器验证、fractal-noise fixtures 和全部四组皮革硬件材质案例通过。这些原生材质检查包含现有 golden 容差及结构／关系门槛。日志保留于本目录，常规详细运行仍为临时数据。

独立的原版／局部坐标 **Release** 执行文件分别渲染全部 11 组材质、44 个通道，请求与计划哈希一致。[前后比较](./material-comparison.json)：

- 陶瓷与木材：全部 28 个通道逐像素相同。
- 皮革：16 个通道全部有变化，最大差为一个 RGBA8 单位。默认 height 改变 363 个像素，默认 normal 改变 885 个；各案例变化数量为 54 至 1,299。
- 将未改变的浏览器数值范围应用于这次“原生修改前／原生修改后”比较，32/44 在范围内。这是兼容性诊断，**不是浏览器验收结果**。12 个皮革通道超出该范围，尽管原生硬件质量／golden 检查通过。

[前后及差异图](./contact.png)使用 ×64 的 RGB 绝对差，并保留默认 height 的[原版](./original-default/height.png)与[局部坐标](./local-default/height.png)完整分辨率图片，以及 color、normal、roughness 对照。Agent 在显示比例下未发现明显结构变化；稀疏量化差异仍可测量，不能忽略。没有人类验收或 golden 更新。没有执行固定 SwiftShader 的精确 golden 检查，也没有执行修改后候选包的完整浏览器矩阵；硬件测试通过不能代替它们。

## 评估结论与决策

局部蜂窝坐标是最值得单独审查的数值改动候选，但不能为了浏览器门槛变绿而直接合入：它改变现有输出，也仍有跨实现差异。采用前，需要在固定软件适配器上评估原有 golden、审查像素变化原因、完成含 Firefox 的实际候选包回归，并明确兼容性决策。本次评估不授权替换基线。

平方根差有理化没有得到充分收益支持，不建议加入。其他着色器风险应分别处理：

| 范围 | 证据及后续边界 |
|---|---|
| UV 生成／采样 | 非二次幂除法是独立的精度来源，局部距离坐标不会解决它。 |
| gradient-map、blend、双线性插值 | 乘加及 `mix` 允许算术差异，前次扫描已有实证；这些 kernel 本次未修改，也没有新增认证。 |
| levels | 非线性映射会传播或放大上游差异；现有端点保护保留，本次没有确立 `pow` 精度修复。 |
| height-to-normal | 邻点差与分辨率缩放传播高度扰动，实测 normal 变化像素多于 height；这不能独立证明 normalize 有缺陷。 |
| 半精度存储 | 整数辅助函数对自身输入一致舍入；不同的舍入前值可能位于中点两侧。 |

WGSL 允许重排／融合，不承诺跨后端逐位一致（[规范](https://gpuweb.github.io/gpuweb/wgsl/#reassociation-and-fusion)）。本评估没有确立上游编译器缺陷，也不构成维护依赖分支的理由。本次未做性能基准，不声称提速。主工作副本的产品 `.mix` 语义、源码／着色器、依赖政策、归档身份、参照和容差保持不变。Chrome／Edge 完整材质认证仍待完成。

## 复现与保留

[provenance.json](./provenance.json)绑定受测源码、执行文件、原输入摘要及保留文件。原生缓冲区执行器和普通浏览器执行器沿用[前次保留的程序](../chromium-arithmetic-reduction/README.zh-CN.md)；模块记录包含实际加载的编译器路径，本目录保留新的浏览器启动记录。不通过主机 GPU 推断浏览器后端。隔离实验后重新构建正常产品执行文件，并检查其原始输出。

以下命令从新的仓库检出根目录执行，生成数据使用忽略的 `tmp/shader-stability`。复用该目录之前先保留旧本地运行。

1. 分别运行 `node docs/evidence/shader-stability/generate-grid.mjs`、该目录下的 `generate-samples.mjs` 和 `generate-fixed-input.mjs`，生成三个版本和可复现输入；不会修改产品着色器。
2. 使用之前的原生执行器，以 count 2097152 运行 original／local／local_rational 的 `<variant>.wgsl`，输出命名为 `<variant>-regular.bin`、`<variant>-strict.bin`。以 count 3072 执行 `<variant>-samples.wgsl`，不带输入文件；同样 count 执行 `<variant>-fixed.wgsl`，带 `uv-input.bin`。输出分别命名为 `-samples-<mode>.bin`、`-fixed-<mode>.bin`。普通 Chrome／Edge 重复小型探针；浏览器全网格只测 original／local。按前次流程设置原生子进程 DXC PATH 和 `MIXTURE_PROBE_PRODUCT`。
3. `node docs/evidence/shader-stability/measure.mjs samples` 和 `node docs/evidence/shader-stability/measure.mjs fixed` 重新计算摘要。保留的小缓冲区可独立复算采样结果；大型网格需重新执行。binary64 估算不决定发布通过与否。
4. 材质评估在记录的源码建立隔离检出，应用 `diagnostic-local.patch` 前运行 `cargo xtask test-node fractal-noise` 和 `cargo xtask test-material leather`；之后以显式 DX12 策略执行 shader-check 和这两项检查。原版／局部坐标 CLI 必须使用**独立 target 目录**构建 Release，再复制到文档所示临时路径。`render-materials.mjs original`／`local` 重现全部案例，Python／Pillow 执行 `compare-materials.py` 重建统计及差异图。本诊断绝不执行 `golden update`。

Git 保留小型采样缓冲区、输入、脚本、测量、补丁、选定的完整分辨率 PNG 和差异图。大型网格、完整 44 通道 PNG 集合、执行文件及详细 golden 运行仍是忽略的临时输出，摘要不保留原始内容。本记录是评估，不是新的已接受视觉基线。
