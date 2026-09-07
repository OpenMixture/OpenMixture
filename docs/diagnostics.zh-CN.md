# 诊断与安全限制

[English](./diagnostics.md) | 简体中文

PR-002 在 `mixture-core` 中建立公共诊断和预算术语。本项工作不解析 `.mix`、验证图、获取适配器或实现运行时 CLI 命令。

## 公共 API

crate 根模块重新导出 `DiagnosticCode`、`Stage`、`Severity`、`EvidenceValue`、`Diagnostic`、`DiagnosticReport`、`LimitKind`、`SafetyLimits` 和 `LimitExceeded`。实现位于 [error.rs](../crates/mixture-core/src/error.rs) 和 [limits.rs](../crates/mixture-core/src/limits.rs)。

```rust
use mixture_core::{DiagnosticReport, LimitKind, SafetyLimits, Stage};

let limits = SafetyLimits::default();
let diagnostics = limits
    .check(LimitKind::Nodes, 129)
    .err()
    .map(|violation| violation.diagnostic(Stage::Validation));
let report = DiagnosticReport::new(diagnostics);
assert!(!report.is_ok());
```

将超限错误转换为诊断时，由调用方显式指定阶段，也可以附加文档、节点、端口或参数上下文。库不会推断路径、初始化设备、修复输入或修改配置上限。

运行[公共 API 示例](../crates/mixture-core/examples/diagnostics.rs)：

```bash
cargo run --locked -p mixture-core --example diagnostics
```

示例有意检查两个超限数值，并打印 `ok: false` 的报告。演示完成时，示例进程成功退出；它不是 CLI 验证操作，也不是渲染健康探针。

## JSON 契约

`Diagnostic` 实现 Serde 序列化和反序列化。`DiagnosticReport` 是可序列化的输出封装，只包含 `ok` 和 `diagnostics`。列表在构造后保持私有且不可变。`ok` 根据是否存在错误级诊断推导；空报告也为真，但这不表示已经执行渲染或 GPU 探测。只有警告和信息级诊断时，报告不会被判定为失败。

每条诊断包含：

| 字段 | 契约 |
| --- | --- |
| `code` | 来自 `DiagnosticCode` 的稳定 `MIX_*` 字符串 |
| `stage` | 显式生命周期阶段 |
| `severity` | `error`、`warning` 或 `info` |
| `message` | 面向人类的文本；调用方应根据错误码进行分支处理 |
| `documentPath` | 可选，由调用方提供的字符串 |
| `nodeId`、`portId`、`parameterId` | 可选，文档内的局部标识符 |
| `evidence` | 可选映射，字符串键按字典序排列 |
| `suggestion` | 可选的可操作建议 |

未提供的可选值和空证据映射会被省略。诊断解码拒绝未知字段、错误码、阶段和严重程度。`DiagnosticReport` 仅作为输出，不提供可接受伪造 `ok` 标志或未排序列表的反序列化入口。

证据值初期支持布尔值、精确的无符号 64 位整数和字符串，序列化为普通 JSON 标量。不支持负数、浮点数、空值、数组或嵌套对象。这样可保留精确计数，并避免非有限测量值静默转换为 JSON null。未来新增数值或结构化证据需要明确扩展契约。读取超出所用语言安全整数范围的数值时，使用方必须保留整数精度。

完整示例见[报告快照](../crates/mixture-core/tests/snapshots/diagnostics-report.json)和[超限错误快照](../crates/mixture-core/tests/snapshots/limits-exceeded.json)。快照变化必须对应有说明的契约变更；不提供自动更新基准的命令。

## 稳定排序

`DiagnosticReport::new` 按以下键依次排序诊断：

1. 阶段：`parse`、`validation`、`compile`、`gpuAdapter`、`gpuDevice`、`gpuShader`、`gpuPipeline`、`gpuExecution`、`readback`、`encoding`。
2. 严重程度：`error`、`warning`、`info`。
3. 文档路径、节点 ID、端口 ID、参数 ID；缺失值先于有值项，字符串采用 Rust 字典序。
4. 稳定错误码字符串、消息、证据和建议。

证据键存储在 `BTreeMap` 中。映射按有序键值对比较；不同证据值类型按布尔、无符号整数、文本排序，同类型再按值排序。每个序列化诊断字段都参与排序。重复诊断会保留。原生源错误不影响顺序，因此排序键相同的诊断仍具有相同 JSON。此保证针对相同诊断数据，不覆盖因环境不同而变化的消息、路径或证据。

## 源错误

`Diagnostic` 实现 `std::error::Error` 和 `Display`。`with_source` 通过自己持有的 `Arc` 保留真实的 `Send + Sync + 'static` 错误，包括嵌套来源和向具体类型转换的能力。克隆诊断会保留同一来源。`Display` 输出稳定错误码和简洁消息；原生调用方可单独遍历 `Error::source()` 来生成面向人类的报告。

源错误对象及消息不会自动序列化，反序列化后的诊断没有原生来源。JSON 中传入的 `source` 字段会被拒绝。调用方可显式选择适合公开的驱动细节放入证据，同时为原生调试保留原始强类型来源。这样可以将机器稳定数据与平台错误分离，同时不吞掉底层失败。

## 预留错误码族

本 PR 只有限制检查会产生失败。其他错误码为后续工作预留共同术语；它们的存在不代表解析、图语义、渲染或恢复已经实现。

| 错误码族 | 初始错误码 |
| --- | --- |
| `MIX_PARSE_*` | `MIX_PARSE_INVALID_UTF8`、`MIX_PARSE_INVALID_JSON` |
| `MIX_FORMAT_*` | `MIX_FORMAT_UNSUPPORTED_VERSION` |
| `MIX_LIMIT_*` | 下表列出的七个 `MIX_LIMIT_*_EXCEEDED` 错误码 |
| `MIX_NODE_*` | `MIX_NODE_UNKNOWN_TYPE`、`MIX_NODE_UNSUPPORTED_VERSION` |
| `MIX_PORT_*` | `MIX_PORT_UNKNOWN`、`MIX_PORT_TYPE_MISMATCH` |
| `MIX_GRAPH_*` | `MIX_GRAPH_CYCLE` |
| `MIX_PARAMETER_*` | `MIX_PARAMETER_INVALID_VALUE` |
| `MIX_COMPILE_*` | `MIX_COMPILE_INVALID_REQUEST` |
| `MIX_GPU_ADAPTER_*` | `MIX_GPU_ADAPTER_UNAVAILABLE` |
| `MIX_GPU_DEVICE_*` | `MIX_GPU_DEVICE_REQUEST_FAILED` |
| `MIX_GPU_SHADER_*` | `MIX_GPU_SHADER_VALIDATION_FAILED` |
| `MIX_GPU_EXECUTION_*` | `MIX_GPU_EXECUTION_FAILED` |
| `MIX_READBACK_*` | `MIX_READBACK_FAILED` |
| `MIX_ENCODING_*` | `MIX_ENCODING_FAILED` |

`DiagnosticCode` 和 `Stage` 是非穷尽 Rust 枚举。使用方匹配已知情况时需保留兜底分支；已有传输字符串和相对顺序必须保持稳定。[术语快照](../crates/mixture-core/tests/snapshots/diagnostics-vocabulary.json)固定初始字符串。退出码属于 CLI 职责，不作为核心诊断的方法提供。

## 安全限制

所有字段使用 `u64`，在各目标上使用相同单位。`SafetyLimits::default()` 是默认策略的显式来源：

| 字段／限制类型 | 默认上限 | 失败错误码 |
| --- | ---: | --- |
| `decodedBytes` / `DecodedBytes` | 2 MiB = 2097152 字节 | `MIX_LIMIT_DECODED_BYTES_EXCEEDED` |
| `nodes` / `Nodes` | 128 | `MIX_LIMIT_NODES_EXCEEDED` |
| `edges` / `Edges` | 512 | `MIX_LIMIT_EDGES_EXCEEDED` |
| `exposedParameters` / `ExposedParameters` | 64 | `MIX_LIMIT_EXPOSED_PARAMETERS_EXCEEDED` |
| `outputDimension` / `OutputDimension` | 每轴 2048 | `MIX_LIMIT_OUTPUT_DIMENSION_EXCEEDED` |
| `requestedOutputs` / `RequestedOutputs` | 8 | `MIX_LIMIT_REQUESTED_OUTPUTS_EXCEEDED` |
| `transientBytes` / `TransientBytes` | 512 MiB = 536870912 字节 | `MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED` |

`check(kind, observed)` 接受等于上限的值；超限时返回包含 `kind`、`configured` 和 `observed` 的 `LimitExceeded`。它只比较数值，不进行加法、窄化转换、资源分配估算或策略修改。转换后的诊断包含 `configured`、`limit` 和 `observed` 证据、建议，以及作为原生来源的强类型超限错误。

调用方可以显式构造更严格或更大的策略。零始终是零，不会被视为缺失或替换成默认值。JSON 解码要求提供全部七个字段，拒绝未知、缺失、负数、小数和溢出值。见[默认值快照](../crates/mixture-core/tests/snapshots/limits-defaults.json)。

这些是上限检查基础能力，不是文档或请求验证。尺寸为零的图像可能满足上限，但仍必须在后续请求验证中失败。宽和高应分别检查。调用方必须先安全计算实际计数和估算字节数，再执行检查；编译器资源估算器尚未实现。v1 仍不支持内嵌资源，其预算为零；PR-002 不添加资源字段或加载 API。

## CLI 退出码策略

下表适用于 PR-004 doctor／内置渲染及未来运行时命令，不代表全部命令已经实现：

| 退出码 | 含义 |
| --- | --- |
| `0` | 命令成功完成；真实结果可以伴随仅包含警告／信息的报告 |
| `1` | 运行故障：文件 I/O、适配器／设备／着色器／执行、回读或编码失败 |
| `2` | 调用或输入无效：用法、解析、格式、节点／端口／图／参数验证、预算违规或无效编译请求 |

CLI 负责映射及 `ok` 一致性。聚合失败时，只要存在运行故障就选择退出码 `1`；否则存在输入错误时选择 `2`；其余情况选择 `0`。结果不依赖发现顺序。运行时命令实现必须为此策略添加集成测试。缺少必需适配器是失败，不能作为成功回退。未来显式跳过 doctor 执行探针时，应保持 `unverified`，不能报告 `healthy`。

PR-004 扩展 [doctor](./gpu-context.zh-CN.md)：棋盘格探针验证通过返回 `0`、`ok: true` 和 `healthy`；显式跳过返回 `unverified`，探针为 `notRun`。获取／探针失败返回 `1`、`ok: false` 和 `unhealthy`。[内置棋盘格渲染](./builtin-checker.zh-CN.md)完成 PNG 输出后返回 `0`，GPU／回读／编码／I/O 失败返回 `1`，请求限制无效返回 `2`。用法错误即使在 JSON 模式也返回 `2` 并写入 stderr；输出 I/O 失败返回 `1`。帮助／版本保持 `0`，缺少／未知／未来命令保持 `2`。

## 验证与范围

```bash
cargo test --locked -p mixture-core diagnostics
cargo test --locked -p mixture-core limits
cargo test --locked -p xtask
cargo run --locked -p mixture-core --example diagnostics
cargo xtask check
```

测试覆盖公共导入、JSON 快照与拒绝输入、输入排列变化时的排序、可选上下文、精确整数证据、原生源错误链、全部默认边界、显式覆盖、零上限和极端计数。公共示例与 crate 文档测试提供使用方层面的 API 验证。

核心唯一的运行时依赖是 `serde`；`serde_json` 是用于快照和示例的开发依赖。[依赖策略](./development.zh-CN.md)强制区分两者。PR-002 未引入 `.mix` 格式字段、图实现、GPU 依赖、着色器、CLI 运行时命令或自动修复。GPU 获取、棋盘格执行与 doctor 另有专门文档。M0 远端 CI 验收仍待完成；PR-002 不宣称关闭该验收项。
