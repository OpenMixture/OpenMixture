# .mix v1 解码与验证

[English](./file-format.md) | 简体中文

**当前状态：** 实现、发布版本和硬件验收范围见[发布状态](./release.zh-CN.md)；本文带日期的早期记录仅描述当时结果。

PR-005 引入首个可执行的 `.mix` 源文件格式：UTF-8 JSON，文档版本为 `1`，节点版本独立且必须显式为 `1`。此前不存在已实现的格式或迁移。本次添加解码、六个节点契约、图验证和 CLI 验证器；PR-006 [图编译](./render-plan.zh-CN.md)现已实现，PR-007 [图执行](./graph-rendering.zh-CN.md)已实现。

## 使用验证器

```bash
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
cargo run --locked -p mixture-cli -- validate fixtures/format/valid/all-m2.mix
cargo xtask test-format
cargo xtask test-core
cargo xtask check
```

`validate <file.mix> [--json]` 读取一个文件，不修改文件、不获取 GPU。`--json` 向 stdout 写入共享的 `{ "ok": ..., "diagnostics": [...] }` 报告。文档有效返回 `0`；调用、解析、字段、版本、图、参数或预算无效返回 `2`；文件或报告 I/O 失败返回 `1`。即使启用 `--json`，用法错误仍写入 stderr。操作和输入诊断均包含传入的文件路径。文件名以 `-` 开头时，先传入 `--`；此时 `--json` 必须放在 `--` 之前。

CLI 最多读取 2 MiB 加一个超限检测字节。大文件的报告使用已读取前缀的实际大小，不伪造完整文件大小。验证器不读取嵌入资源，也不解析文档中的外部路径。

## 源文件结构

[棋盘格示例](../examples/checker.mix)是一个最小完整材质：

```json
{
  "version": 1,
  "nodes": [
    { "id": "checker", "type": "checker", "version": 1 },
    { "id": "out", "type": "material-output", "version": 1 }
  ],
  "edges": [
    {
      "from": { "nodeId": "checker", "portId": "color" },
      "to": { "nodeId": "out", "portId": "baseColor" }
    }
  ],
  "exposedParameters": [
    { "id": "frequency", "nodeId": "checker", "parameterId": "cellsX" }
  ]
}
```

| 对象 | 必填字段 | 可选字段 |
| --- | --- | --- |
| 文档 | `version`：32 位无符号整数；`nodes`：数组；`edges`：数组 | `exposedParameters`：数组，默认 `[]` |
| 节点 | `id`：字符串；`type`：字符串；`version`：32 位无符号整数 | `parameters`：对象，默认 `{}` |
| 边 | `from`：输出端点；`to`：输入端点 | 无 |
| 端点 | `nodeId`：字符串；`portId`：字符串 | 无 |
| 暴露参数 | `id`：公开字符串 ID；`nodeId`：目标节点；`parameterId`：可变参数 | 无 |

节点 ID 和公开参数 ID 必须匹配 `[A-Za-z][A-Za-z0-9_-]{0,63}`。节点 ID 与公开 ID 属于不同命名空间，各自在其命名空间内唯一。类型、端口和参数 ID 必须精确匹配[十一个节点契约](./node-contracts.zh-CN.md)，区分大小写。上述字段使用固定拼写。

每层对象均拒绝未知字段，包括布局、元数据、缩略图、预设、资源、子图、导出目标和任意 WGSL。即使值相同或键使用等价 JSON 转义，也拒绝重复 JSON 键；嵌套参数对象同样如此。无效参数形状只保留到语义验证，以便报告参数类型错误。不得使用后一个键覆盖前一个键的解释方式。

必填字段不能省略或为 null。数组必须使用数组类型。版本必须是整数记号；`1.0`、负数、字符串和超过 `u32::MAX` 的值均被拒绝。注释、尾逗号、尾随文档、无效 UTF-8、NaN、无穷大和数值溢出均被拒绝。JSON 默认递归限制保持启用，深度嵌套输入不能无限增长解析器调用栈。

## 限制与验证顺序

调用方显式传入 [SafetyLimits](./diagnostics.zh-CN.md)。默认仍为 2 MiB 输入字节、128 个节点、512 条边、64 个暴露参数。零上限保持为零。字节检查先于 UTF-8／JSON 解析。集合解码不使用不可信的长度提示预分配空间，遇到首个超限元素即停止，不构造该元素的类型化对象。`observed` 是首次超过上限的计数，不是未读取后缀的总数。

结构解码后，不支持的文档版本返回 `MIX_FORMAT_UNSUPPORTED_VERSION`。图验证覆盖所有节点和边，包括未使用的连通分量：

1. 对直接构造的 Rust 文档再次检查集合上限和文档版本；失败时停止后续图分析。
2. 检查节点 ID、已知类型、节点版本、参数名、类型、范围、枚举及参数间关系。不支持的节点版本不会套用版本 1 的默认值。
3. 边端点必须指向存在且无歧义的节点，并连接声明的输出端口与输入端口，数据种类严格一致。每个输入最多一条边。重复边身份与多个输入来源分别报告。
4. 采用按字典序遍历的迭代深度优先算法检测有向环，包括自环和不连通的环。证据给出真实闭合路径，不会将下游节点误标为环成员。报告的是遍历发现的回边环，不穷举所有可能的环。
5. 必须恰好有一个 `material-output`，其 `baseColor` 需要有效的 `Color` 连接；其他通道使用已记录的默认值。节点契约中的每个必填输入都需要有效连接，包括 `warp` 的两个 Scalar 输入。
6. 暴露参数的公开名称必须有效且唯一，每个名称绑定支持的节点版本中的真实可变参数，包括正在使用默认值的参数。目标也必须唯一：拒绝别名和重复绑定。节点 ID、节点版本、端口及结构字段不是可变参数。

独立诊断会尽量汇总，并按[共享顺序](./diagnostics.zh-CN.md)排序。节点无效或有歧义时，停止依赖它的端口解释，但继续其他安全的独立检查。验证不会添加节点／边、修复环、转换或截断值、插入类型转换，也不会改写显式参数。

输出尺寸、请求通道、GPU 临时内存估算、暴露参数覆盖值的应用、依赖裁剪和 RenderPlan 生成属于 [PR-006 编译请求](./render-plan.zh-CN.md)；它们不是文档字段，也不是 `validate` 已实现的能力。

## 公开 Rust API

```rust
use mixture_core::{DocumentError, MaterialDocument, SafetyLimits, ValidatedDocument};

fn validate_source(source: &[u8]) -> Result<ValidatedDocument, DocumentError> {
    let limits = SafetyLimits::default();
    MaterialDocument::decode(source, &limits)?.into_validated(&limits)
}
```

可执行的[消费方测试](../crates/mixture-core/tests/format.rs)与 crate 文档测试覆盖相同 API。[document.rs](../crates/mixture-core/src/document.rs)负责源数据与错误，[validation.rs](../crates/mixture-core/src/validation.rs)负责验证及已验证语义的只读访问，[registry.rs](../crates/mixture-core/src/registry.rs)与[节点模块](../crates/mixture-core/src/nodes/)负责契约。

`decode` 在结构、版本和解码限制检查后返回可编辑的 `MaterialDocument`，不保证图有效。`validate(&limits)` 返回不可变 `DiagnosticReport`。`into_validated(&limits)` 成功后返回 `ValidatedDocument`；包装类型没有公开构造器、反序列化器或可变访问方法。`DocumentError::report()` 保留全部返回诊断及原生解析器／UTF-8／限制错误来源。库不会推断文件系统路径。

`parameter` 解析显式值或版本化默认值，不把默认值插入源文档。`input_source` 区分真实连接与未连接输入的默认值。`material_channels` 按契约顺序报告八个通道的种类和来源。这些是语义描述，不是已计算的纹理。

`to_json` 为有效源文档生成确定性的格式化 JSON：节点按 ID 字典序排列，边按完整的源／目标端点元组排列，公开绑定按 ID 排列，参数对象内的键按字典序排列。输入省略的参数对象和暴露数组会显式写为 `{}` 和 `[]`。显式值、有意义的数组顺序及源语义得到保留，不插入参数默认值。这是结构序列化，不是规范化计划哈希。JSON 数值参数使用现有 `serde_json` 依赖的 `float_roundtrip` 特性，使有限 f64 值在序列化／解码后保持位值一致；原始十进制拼写与空白不保留。

## 新增诊断与验证证据

现有诊断码拼写及阶段顺序保持不变。PR-005 向[词汇快照](../crates/mixture-core/tests/snapshots/diagnostics-vocabulary.json)添加以下代码：

| 诊断码 | 阶段／含义 |
| --- | --- |
| `MIX_FORMAT_INVALID_DOCUMENT` | `parse`：字段结构、重复键或 JSON 结构类型错误 |
| `MIX_NODE_INVALID_ID`、`MIX_NODE_DUPLICATE_ID` | `validation`：节点身份无效或有歧义 |
| `MIX_GRAPH_UNKNOWN_NODE` | `validation`：边端点节点不存在 |
| `MIX_GRAPH_DUPLICATE_EDGE`、`MIX_GRAPH_MULTIPLE_INPUTS` | `validation`：连接身份／数量错误 |
| `MIX_GRAPH_MATERIAL_OUTPUT_COUNT` | `validation`：材质输出节点数量不为一 |
| `MIX_PORT_REQUIRED_CONNECTION` | `validation`：缺少有效的必填连接 |
| `MIX_PARAMETER_UNKNOWN` | `validation`：节点契约中不存在该参数名 |
| `MIX_EXPOSED_PARAMETER_INVALID` | `validation`：公开 ID 或目标绑定问题 |
| `MIX_IO_READ_FAILED` | `parse`：调用方读取源文件失败；CLI 退出码为 1 |

现有 UTF-8／JSON／版本／限制／节点／端口／参数／环代码处理对应错误。解析错误保留原生来源、解析器提供的行列位置以及选定的来源文本。语义诊断按情况标明节点、端口、参数、公开 ID、计数或环路径。

[格式夹具](../fixtures/format/README.zh-CN.md)包含双节点棋盘格、全部六个 M2 契约及针对性的无效文档。测试覆盖严格语法／字段、重复转义键、深度嵌套、字节／集合边界、显式限制、浮点往返、默认值、图输入顺序置换、精确环证据、CLI 退出码、有界读取及源文件不被修改。`cargo xtask test-format` 运行核心格式／验证／注册表测试与 CLI 验证测试；`cargo xtask test-core` 运行整个无需 GPU 的核心测试集。PR-005 时尚无远端跨平台 CI 结果；此后已通过已记录的[三平台 CPU 门槛](./evidence/remote-ci/README.zh-CN.md)。

## PR-009 增量目录扩展

文档 JSON 结构仍为 `.mix v1`，已有文档含义不变，无需迁移。三个版本 1 新节点见[节点契约](./node-contracts.zh-CN.md)。仅 `fractal-noise.seed` 没有参数默认值，源文档必须显式使用 0 至 4294967295 的无符号整数 token。缺失种子报告带参数证据的 `MIX_PARAMETER_INVALID_VALUE`，未使用节点同样检查。解码、确定性序列化及原源文件／计划夹具保持不变。

## PR-010 重采样节点增量扩展

`transform-2d` 和 `warp` 添加两个版本 1 节点类型，不添加字段，也不改变已有 `.mix v1` 含义。必填端口要求精确的 `Scalar` 连接。变换缩放与四分之一圈旋转次数要求有界无符号整数记号；偏移和扭曲强度接受有界有限 JSON 数值。默认值、参数边界、输入要求及显式公开绑定沿用已有目录的验证规则，包括未使用分支。参阅[节点契约](./node-contracts.zh-CN.md)和[公开重采样测试](../crates/mixture-core/tests/resampling.rs)。未引入迁移、源文件修复或隐式转换。
