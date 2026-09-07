# M2 节点契约，版本 1

[English](./node-contracts.md) | 简体中文

PR-005 在 [mixture-core](../crates/mixture-core/src/registry.rs) 注册六个静态契约，在 PR-007 图执行之前定义 `.mix` 源文件验证和默认值。当前只有独立的 PR-004 固定棋盘格探针执行像素。本次契约不引入新着色器、CPU 像素实现、`KernelId` 或渲染器。

## 通用规则

六个节点类型都要求 `version: 1`。连接必须严格匹配 `Scalar`、`Color` 或 `Normal`；每个输入最多一条入边。没有默认值的输入为必填。省略的参数使用下表默认值；未知名称、类型错误及超范围值均为错误。下文所有参数均可变，允许通过唯一公开绑定暴露。没有随机节点，因此无需种子。

浮点参数接受有限 JSON 数值；整数参数要求无符号整数记号（`8` 有效，`8.0` 和 `8e0` 无效）。颜色必须是四个有限数值组成的数组，各分量在 `[0, 1]` 内，表示线性 RGBA，采用非预乘 alpha。浮点／颜色边界均包含端点。源模型保留 f64 JSON 数值，后续执行必须显式转换为 GPU 表示。参数验证不计算像素，也不转换颜色空间。

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

默认值与 [PR-004 固定棋盘格](./builtin-checker.zh-CN.md)一致。图契约允许固定探针命令未暴露的颜色和频率；PR-007 实现图执行时必须扩展现有的唯一 WGSL 路径。

## levels

[契约模块](../crates/mixture-core/src/nodes/levels.rs)。必填输入 `in: Scalar`，输出 `value: Scalar`。

| 参数 | 类型／范围 | 默认值 |
| --- | --- | --- |
| `inputMin` | 浮点 `[0, 1]` | `0.0` |
| `inputMax` | 浮点 `[0, 1]` | `1.0` |
| `gamma` | 浮点 `[0.01, 100]` | `1.0` |
| `outputMin` | 浮点 `[0, 1]` | `0.0` |
| `outputMax` | 浮点 `[0, 1]` | `1.0` |

解析默认值后必须满足 `inputMin < inputMax`。声明的运算为 `t = clamp((in - inputMin) / (inputMax - inputMin), 0, 1)`，再计算 `outputMin + pow(t, 1 / gamma) * (outputMax - outputMin)`。允许反转输出上下界以实现反相。正 gamma 和不同输入边界可避免未定义的除法。这是后续着色器的语义公式，Rust 仅验证参数。

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

该编码法线对应切线空间 +Z；后续 RGBA 存储的填充分量与此逻辑 XYZ 值分开定义。M2 没有 `Normal` 生成节点，因此有效 M2 文档使用该默认值。颜色输出不能充当法线。`ValidatedDocument::material_channels()` 提供连接／默认状态，不生成纹理。

## 夹具与检查

[双节点棋盘格](../fixtures/format/valid/checker.mix)覆盖默认参数及对默认频率的暴露。[all-m2.mix](../fixtures/format/valid/all-m2.mix)连接全部六个契约，将 levels 应用于标量混合遮罩，并显式提供粗糙度。[无效文档](../fixtures/format/README.zh-CN.md)与公开 API 测试覆盖参数边界、枚举、端口方向和种类、必填／默认输入、图错误及公开绑定。

```bash
cargo test --locked -p mixture-core --test registry
cargo test --locked -p mixture-core --test validation
cargo xtask test-format
```

这些测试无需 GPU，仅验证契约。现有固定棋盘格像素基准保持原样。契约存在不代表其图像素执行器或节点专属基准已经实现。
