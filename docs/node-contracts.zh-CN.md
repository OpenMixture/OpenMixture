# 内置节点契约，版本 1

[English](./node-contracts.md) | 简体中文

[mixture-core](../crates/mixture-core/src/registry.rs)中的十一个版本 1 契约降级为类型化计划，并[通过唯一 `wgpu` 路径执行](./graph-rendering.zh-CN.md)。PR-005–007 建立六个 M2 节点；PR-009 添加噪声、渐变映射和高度派生法线；PR-010 添加标量变换和扭曲。常量共享一个 WGSL kernel，material-output 映射资源，固定棋盘格与图棋盘格共享着色器。

## 通用规则

十一个节点类型都要求 `version: 1`。连接必须严格匹配 `Scalar`、`Color` 或 `Normal`；每个输入最多一条入边。没有默认值的输入为必填。省略的参数使用下表默认值；未知名称、类型错误及超范围值均为错误。下文所有参数均可变，允许通过唯一公开绑定暴露。随机节点 `fractal-noise` 要求在源文档中显式填写整数种子，包括未使用分支；参数覆盖不会修复缺失的源种子。

浮点参数接受有限 JSON 数值；整数参数要求无符号整数记号（`8` 有效，`8.0` 和 `8e0` 无效）。颜色必须是四个有限数值组成的数组，各分量在 `[0, 1]` 内，表示线性 RGBA，采用非预乘 alpha。浮点／颜色边界均包含端点。源模型保留 f64 JSON 数值，编译时显式转换为 f32 GPU 参数。参数验证不计算像素，也不转换颜色空间。

坐标以左上角为原点。常量、levels 和 blend 均为逐点操作，不引入坐标变换；其平铺性质取决于输入，常量输出无缝。图执行的 GPU 精度细节与基准证据属于 PR-007。

## constant-scalar

[契约模块](../crates/mixture-core/src/nodes/constant_scalar.rs)。无输入，输出 `value: Scalar`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `value` | 浮点 `[0, 1]` | `0.0` |

定义每个像素相同的标量。粗糙度、高度、遮罩或其他标量通道的含义由消费方赋予。

## constant-color

[契约模块](../crates/mixture-core/src/nodes/constant_color.rs)。无输入，输出 `color: Color`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `value` | RGBA，各分量 `[0, 1]` | `[1, 1, 1, 1]` |

定义每个像素相同的线性 RGBA 颜色。

## checker

[契约模块](../crates/mixture-core/src/nodes/checker.rs)。无输入，输出 `color: Color`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `cellsX` | 整数 `[1, 1024]` | `8` |
| `cellsY` | 整数 `[1, 1024]` | `8` |
| `colorA` | RGBA，各分量 `[0, 1]` | `[0, 0, 0, 1]` |
| `colorB` | RGBA，各分量 `[0, 1]` | `[1, 1, 1, 1]` |

输出像素 `(x, y)` 的格子索引为 `floor(x * cellsX / width)` 和 `floor(y * cellsY / height)`；两者之和为偶数时选取 `colorA`，为奇数时选取 `colorB`。非整齐尺寸会产生不同格宽，小图可能欠采样。某轴格数为偶数时，沿该轴重复可保持交替连续；允许奇数格，但重复时边界会出现相邻同色格。不隐含滤波或随机性。

默认值与 [PR-004 固定棋盘格](./builtin-checker.zh-CN.md)一致。图契约允许固定探针命令未暴露的颜色和频率；PR-007 扩展现有的唯一 WGSL 路径，同时执行固定及图棋盘格调用。

## levels

[契约模块](../crates/mixture-core/src/nodes/levels.rs)。必填输入 `in: Scalar`，输出 `value: Scalar`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `inputMin` | 浮点 `[0, 1]` | `0.0` |
| `inputMax` | 浮点 `[0, 1]` | `1.0` |
| `gamma` | 浮点 `[0.01, 100]` | `1.0` |
| `outputMin` | 浮点 `[0, 1]` | `0.0` |
| `outputMax` | 浮点 `[0, 1]` | `1.0` |

解析默认值后必须满足 `inputMin < inputMax`。声明的运算为 `t = clamp((in - inputMin) / (inputMax - inputMin), 0, 1)`，再计算 `outputMin + pow(t, 1 / gamma) * (outputMax - outputMin)`。允许反转输出上下界以实现反相。正 gamma 和不同输入边界可避免未定义的除法。PR-007 在 WGSL 中实现此公式；Rust 仅验证／降级参数及转换回读编码。

## blend

[契约模块](../crates/mixture-core/src/nodes/blend.rs)。必填输入 `a: Color`、`b: Color`；可选输入 `mask: Scalar`，默认 `1.0`；输出 `color: Color`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `mode` | 枚举 `normal`、`multiply`、`screen` | `normal` |
| `opacity` | 浮点 `[0, 1]` | `1.0` |

定义 `t = opacity * clamp(mask, 0, 1)`。RGB 按 `t` 从 `a.rgb` 线性插值到模式结果：normal 使用 `b.rgb`，multiply 使用 `a.rgb * b.rgb`，screen 使用 `1 - (1 - a.rgb) * (1 - b.rgb)`。所有模式的 alpha 均按 `t` 从 `a.a` 插值到 `b.a`。这是分量混合契约，不是 source-over 合成；不隐含预乘处理或其他混合模式。

## material-output

[契约模块](../crates/mixture-core/src/nodes/material_output.rs)。每份文档必须恰好有一个输出汇聚节点，无输出端口、无参数。输入按以下稳定契约顺序排列：

| 输入 | 种类 | 策略／默认值 |
| --- | --- | --- |
| `baseColor` | Color | 必须有有效连接 |
| `normal` | Normal | 编码中性 XYZ `[0.5, 0.5, 1.0]` |
| `roughness` | Scalar | `1.0` |
| `metallic` | Scalar | `0.0` |
| `height` | Scalar | `0.0` |
| `ambientOcclusion` | Scalar | `1.0` |
| `opacity` | Scalar | `1.0` |
| `emissive` | Color | 线性不透明黑色 `[0, 0, 0, 1]` |

该编码法线对应切线空间 +Z；RGBA 存储的填充分量（alpha 为 1）与此逻辑 XYZ 值分开定义。M2 没有 `Normal` 生成节点，因此有效 M2 文档使用该默认值。颜色输出不能充当法线。`ValidatedDocument::material_channels()` 提供连接／默认状态，不生成纹理。

## 夹具与检查

[双节点棋盘格](../fixtures/format/valid/checker.mix)覆盖默认参数及对默认频率的暴露。[all-m2.mix](../fixtures/format/valid/all-m2.mix)连接全部六个契约，将 levels 应用于标量混合遮罩，并显式提供粗糙度。[无效文档](../fixtures/format/README.zh-CN.md)与公开 API 测试覆盖参数边界、枚举、端口方向和种类、必填／默认输入、图错误及公开绑定。

```bash
cargo test --locked -p mixture-core --test registry
cargo test --locked -p mixture-core --test validation
cargo xtask test-format
```

这些测试无需 GPU，仅验证契约。现有固定棋盘格像素基准保持原样。PR-007 [节点夹具](../fixtures/nodes/README.zh-CN.md)和 `test-node` 现已验证实际图像素，不宣称新增像素基准。

PR-006 [类型化降级](./render-plan.zh-CN.md)将上述源契约映射为 Constant、Checker、Levels 和 Blend 调用；material-output 仍是映射。常量也用于实例化可选默认值。不添加源节点类型。PR-007 已实现穷尽 GPU 映射与像素测试。

## PR-009 新增节点

PR-009 添加三个契约和三个 WGSL kernel，目录共九个节点，渲染器缓存上限为七条管线。原六个契约、M2 计划快照及棋盘格基准不变。`ParameterContract::default` 现在为 `Option<ParameterDefault>`，`None` 表示源文档必填。这是发布前有意进行的 Rust API 变更，下游调用者须处理 `None`；已有默认值仍为 `Some`。缺失种子报告 `MIX_PARAMETER_INVALID_VALUE`，包含节点／参数证据。`.mix v1` 结构不变，新增版本化节点类型属于增量扩展。

### fractal-noise

[契约](../crates/mixture-core/src/nodes/fractal_noise.rs)、[WGSL](../crates/mixture-wgpu/shaders/nodes/fractal-noise.wgsl)、[夹具](../fixtures/nodes/fractal-noise/README.zh-CN.md)。无输入，输出 `value: Scalar`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `seed` | Integer `[0, 4294967295]` | 必填，无默认值 |
| `scale` | Integer `[1, 128]` | `8` |
| `octaves` | Integer `[1, 6]` | `4` |
| `persistence` | Float `[0, 1]` | `0.5` |
| `basis` | Enum `value`、`cellular` | `value` |

在像素中心 UV `(x+0.5, y+0.5)/(width,height)` 采样，图像 v 向下。每个 octave 将整数格点周期翻倍、振幅乘 persistence，并派生独立 u32 种子。加权和除以总振幅后限制到 `[0,1]`。persistence 为零等价于单 octave。完整 u32 种子以整数上传，绝不经过 f32。

唯一 WGSL 使用模 2³² 溢出的 avalanche 哈希（乘数 `0x7feb352d`、`0x846ca68b`），取高 24 位乘 `2^-24`。格点身份按 octave 周期取模。value 基底使用五次平滑函数插值四个格点值。cellular 在每个格子的中央 60% 放一个采样点，X／Y 独立扰动，在有界 3×3 邻域取第二近与最近距离之差并限制范围。这是明确定义的局部 cellular 场，不是无限邻域最近点搜索，生成由暗沟分隔的颗粒内部。

两种基底都以一个 UV 单元重复。边界相邻像素来自不同像素中心，无需字节相等。cellular 梯度在细胞边界存在尖点。没有抗锯齿；高 scale／octaves 在小尺寸下可能欠采样。计算为 f32、存储为 f16；软件逐字节基准与显式硬件容差约束最终 RGBA8，而非保证所有驱动浮点逐位相等。WGSL 是唯一像素公式。

### gradient-map

[契约](../crates/mixture-core/src/nodes/gradient_map.rs)、[WGSL](../crates/mixture-wgpu/shaders/nodes/gradient-map.wgsl)、[夹具](../fixtures/nodes/gradient-map/README.zh-CN.md)。必填 `in: Scalar`，输出 `color: Color`。`colorA` 和 `colorB` 为 `[0,1]` 非预乘线性 RGBA，默认不透明黑和白。以 `clamp(in,0,1)` 插值全部四个分量，仅在输出编码时将颜色 RGB 转为 sRGB。这是双端点渐变，没有多色标或隐藏色彩转换。平铺随输入；使用 f32 插值、f16 存储。

### height-to-normal

[契约](../crates/mixture-core/src/nodes/height_to_normal.rs)、[WGSL](../crates/mixture-wgpu/shaders/nodes/height-to-normal.wgsl)、[夹具](../fixtures/nodes/height-to-normal/README.zh-CN.md)。必填 `in: Scalar`，输出 `normal: Normal`。`strength` 为 Float `[0,8]`，默认 `1`。

读取循环左／右／上／下邻居。`du = (right-left)*width/2`、`dv = (down-up)*height/2`，编码 `normalize(-strength*du, +strength*dv, 1)*0.5+0.5`，alpha 为 1。图像 u 向右、v 向下；切线 X 向右、Y 向上（OpenGL 约定）。导数以 UV 单位计算，使同一已充分采样表面在不同分辨率下具有可比强度，但不修复欠采样。零强度或常量输入产生中性法线；一／二像素轴的中心差分邻居相同，导数为零。循环邻居保留周期输入语义，包括合法的边界坡度。强烈／高频输入可能产生接近掠射方向的法线；下游测量须考虑接近中性分量的量化。

```bash
cargo xtask test-node fractal-noise
cargo xtask test-node gradient-map
cargo xtask test-node height-to-normal
```

法线测试将字面量半精度水平／垂直坡面直接送入生产着色器，以矩形尺寸验证循环、Y 符号、UV 缩放及零强度，不实现 CPU 法线渲染器。[皮革夹具](../fixtures/materials/leather/README.zh-CN.md)作为实际材质消费者使用全部三个新增节点。

## PR-010 标量重采样扩展

PR-010 添加 `transform-2d` 和 `warp`，使目录达到十一个节点，渲染器缓存上限达到九条管线。源 JSON 结构、文档／节点版本 1、原有节点契约以及已有像素／计划／哈希基准保持不变。这是增量目录扩展，现有文档无需迁移。[注册表测试](../crates/mixture-core/tests/registry.rs)随 `cargo xtask check` 执行，在 M3 验收前强制十二节点上限。

两个节点的输入和输出均为 `Scalar`。应先变换或扭曲高度，再派生颜色和切线法线；它们不隐式接受 `Color` 或 `Normal`。两者不引入随机运算，也无需额外种子。输入周期性来自源图，其中的随机节点仍须显式种子。

### 共用采样约定

输出纹素 `(x,y)` 使用中心 UV `(x+0.5,y+0.5)/(width,height)`，u 向右、v 向下。在所需源 UV 处计算 `p=fract(sampleUV)*inputDimensions-0.5`，读取 `floor(p)` 周围四个整数邻居；读取前将各坐标按对应输入维度取模环绕，然后按 `fract(p)` 做双线性插值。负邻居环绕到对侧边缘。明确使用四次 `textureLoad`，不隐含 sampler、mip 链或抗锯齿。

计算使用 f32，中间标量以 `[value,0,0,1]` 存入 f16；导出 RGBA8 时将红通道复制到 RGB，并使用不透明 alpha。单纹素轴有效且环绕到同一纹素。重复采样保持周期源的边界约定，但不会修复输入已有接缝。对侧边界像素是不同的中心采样，因此无缝平铺不要求它们的字节值相同。高整数缩放或快速变化的扭曲场可能欠采样输入细节。

### transform-2d

[契约](../crates/mixture-core/src/nodes/transform_2d.rs)、[WGSL](../crates/mixture-wgpu/shaders/nodes/transform-2d.wgsl)、[夹具](../fixtures/nodes/transform-2d/README.zh-CN.md)。必填输入 `in: Scalar`，输出 `value: Scalar`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `scaleX` | 整数 `[1,64]` | `1` |
| `scaleY` | 整数 `[1,64]` | `1` |
| `quarterTurns` | 整数 `[0,3]` | `0` |
| `offsetX` | 浮点 `[-1,1]` | `0` |
| `offsetY` | 浮点 `[-1,1]` | `0` |

令 `p=uv-0.5`。先逆向旋转坐标，再按源坐标轴缩放，最后平移：`sampleUV=rotated*vec2(scaleX,scaleY)+0.5+vec2(offsetX,offsetY)`。

| `quarterTurns` | 逆向旋转坐标 | 视觉图案旋转 |
| --- | --- | --- |
| `0` | `(p.x,p.y)` | 不旋转 |
| `1` | `(p.y,-p.x)` | 顺时针 90 度 |
| `2` | `(-p.x,-p.y)` | 180 度 |
| `3` | `(-p.y,p.x)` | 顺时针 270 度 |

枢轴是 UV 图块中心。旋转先于各轴缩放，因此也旋转各向异性纹理轴：`scaleX=8, scaleY=1` 在零次旋转时横向采样八个源周期，在一次旋转时纵向采样八个源周期。缩放表示采样周期数；增大缩放会重复／压缩源，而非放大它。旋转使用归一化 UV 坐标，矩形输出的宽高不交换。正偏移使源采样向右／下移动，使视觉图案向左／上移动。

整数缩放和四分之一圈旋转对所有允许设置都保持周期源的平铺性质，包括小数偏移。小数缩放和任意旋转角度明确不属于此 v1 契约，因为通常会破坏输出图块周期性。两个缩放均为一、零旋转且零偏移时，着色器直接读取对应输入纹素，精确保留标量像素。

### warp

[契约](../crates/mixture-core/src/nodes/warp.rs)、[WGSL](../crates/mixture-wgpu/shaders/nodes/warp.wgsl)、[夹具](../fixtures/nodes/warp/README.zh-CN.md)。必填输入 `in: Scalar` 和 `displacement: Scalar`，输出 `value: Scalar`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `strengthX` | 浮点 `[-1,1]` | `0.05` |
| `strengthY` | 浮点 `[-1,1]` | `0` |

在当前输出纹素读取位移标量，令 `d=2*clamp(field,0,1)-1`，再使用共用的重复双线性规则，在 `sampleUV=uv+d*vec2(strengthX,strengthY)` 采样 `in`。两个强度是独立的带符号 UV 位移。位移场 `0.5` 为中性，`1` 应用正强度，`0` 应用负强度。正 X／Y 位移向右／下采样，使视觉图案向左／上移动。位移场在输出坐标读取，而非位移后的输入坐标；钳制属于着色器语义，不放宽源参数验证。

中性场纹素或零强度向量直接读取输入，精确保留该标量像素。即使强度为零，也仍验证并编译必填位移场连接。周期源与周期位移输入保持平铺契约，因为跨越一个输出图块会增加整数源 UV 步长。大位移可能折叠采样图案；不承诺单调性或抗锯齿。

Warp 使用等价的局部纹素坐标：`delta=(2*clamp(field,0,1)-1)*strength*dimensions`，整数基址为环绕后的 `pixel+floor(delta)`，插值权重为 `fract(delta)`。同一渲染计划的全部纹理使用同一分辨率。避免将整数像素位置加入 f32 插值坐标，可在较大坐标保留微小位移。实数采样语义不变，但浮点像素可能改变；现有 golden 仍有约束力。

```bash
cargo test --locked -p mixture-core --test resampling --test registry
cargo xtask shader-check
cargo xtask test-node transform-2d
cargo xtask test-node warp
```

[重采样 API 测试](../crates/mixture-core/tests/resampling.rs)覆盖默认值、类型化输入顺序、重复绑定、精确种类、必填连接、依赖裁剪与参数／哈希行为。[字面量 GPU 探针](../crates/mixture-wgpu/tests/support/resampling_probe.rs)包含 17 个变换和 23 个扭曲用例，以手工指定的半精度输入和精确预期输出，经生产 WGSL 验证 X／Y 接缝插值、负偏移、全部旋转、先旋转后缩放、单纹素尺寸、位移极性／钳制及输出坐标场读取。节点图用例只读引用已有噪声恒等基准，并检查有意义的参数变化与热缓存重复性。节点证据保存于 `tmp/node-tests/<backend>/`，包括 `<node>-literal-probes.json`。

warp 插值在每行及两行之间使用显式 `fma(b-a,t,a)`，以减少已测后端的中间舍入。WGSL 允许非融合 fma，因此这不保证跨后端逐位一致。候选仍需满足冻结材质 golden，不修改容差或基线。字面值回归覆盖双轴不同权重、正负方向跨接缝、3x2 纹理的大位移及完整周期、细长奇数高度纹理和固定的 half 舍入临界点。
