# 原生 CLI 报告与退出码

[English](./cli-contract.md) | 简体中文

**M6A-02 更新：** [M6A-02 Core 实现](./m6a-02-core-resources.zh-CN.md)现提供资源引用、不可变准备请求及内容绑定计划 v2。Rust 源码为 0.3.0，未发布浏览器候选为 0.3.0-alpha.0／API schema 2；inspect 和图 render 报告 schema 为 2。原生上传及浏览器资源参数仍待 M6A-03／04；本次不接受新图像像素。以下历史版本说明须按此更新理解。

PR-012 通过[独立 Rust 测试程序](../examples/native-consumer/tests/cli_contract.rs)消费已构建的 `mixture` 可执行文件，使用自有输入及输出目录。它修复人类可读诊断上下文，记录现有 JSON 和退出行为，不引入 schema 版本、不改变像素语义，也不完成其余 [M4 工作](../M4_PRS.zh-CN.md)。

## 进程边界

使用参数数组启动已构建的可执行文件，选择 `--json`，分别捕获 stdout／stderr，等待进程完成，然后检查退出码并将 stdout 解析为一个 JSON 对象。渲染成功时，在进程退出后消费报告列出的文件。报告路径保留调用方提供的输入／输出路径；相对路径以子进程工作目录为基准解析。

| 退出码 | 含义及报告 |
|---|---|
| `0` | 请求命令完成。`validate` 检查源码；`inspect --plan` 编译计划；`doctor` 验证棋盘探针或显式跳过探针；`render` 完成每个请求 PNG 的写入。JSON 包含 `ok: true`。 |
| `2` | 调用、源码或编译请求错误。已完成参数解析的命令输出 `ok: false` 和结构化诊断。无效调用是下述例外。 |
| `1` | 操作错误：源文件 I/O、GPU 获取／执行／回读、编码、文件写入或报告输出。能够写出报告时，JSON 包含 `ok: false` 及可用证据。 |

**用法错误始终以文本写入 stderr，包括使用 `--json` 时；stdout 为空。** 例如缺少 `--plan`、缺少 `--out`、未知选项及选项语法错误。语法正确但公开值无效的 `--set 'repeat=0'` 会进入编译并返回 JSON 诊断。源文件缺失属于操作退出 `1`，文件内容格式错误属于源码退出 `2`。帮助／版本是退出 `0` 的文本，不是 JSON 报告命令。

已解析命令的诊断写入所选 stdout 报告。已测 CPU 路径保持 stderr 为空。GPU 后端可能自行向 stderr 输出消息；它们不是稳定的 Mixture 错误接口，不应合并到 JSON stdout。报告写入失败返回 `1`，并尝试在 stderr 说明原因；此时 stdout 可能不完整。调用方必须处理这一传输失败，不能假定所有退出 `1` 的输出都可解析为 JSON。

## 报告结构

下表描述当前传输契约。按字段名读取，不依赖 JSON 对象成员顺序。诊断数组、计划数组和完成输出的顺序遵循下文的确定性规则。消息、路径、适配器能力和耗时不构成跨平台字节快照。

| 命令 | 必需顶层字段 | 失败时的字段存在规则 |
|---|---|---|
| `validate` | `ok: boolean`、`diagnostics: array` | 保持两个字段；该原始结构**没有 `schemaVersion` 字段**。 |
| `inspect --plan` | `schemaVersion: 1`、`plan: object or null`、`ok`、`diagnostics` | `plan` 存在且为 null。 |
| `doctor` | `schemaVersion: 1`、`verdict: string`、`requested: object`、`adapter: object or null`、`device: object or null`、`computeProbe: string`、`readbackProbe: string`、`ok`、`diagnostics` | adapter／device 字段始终存在；设备请求失败时可保留已选适配器证据。`execution` 是**可选、缺省时省略**的棋盘报告，仅完成探针时存在。 |
| `render` | `schemaVersion: 1`、`input: string`、`outputDirectory: string`、`planHash: string or null`、`context: object or null`、`execution: object or null`、`outputs: array`、`ok`、`diagnostics` | `planHash`、`context`、`execution` **始终存在**，不可用时为 null。`outputs` 只包含已完成的文件写入。 |

`schemaVersion`、计划 `version`、源码 `documentVersion` 和节点版本描述不同边界。当前检查正文为 [RenderPlan v1](./render-plan.zh-CN.md)：`version`、`documentVersion`、`size: [width,height]`、`materialOutput`、`passes`、`outputs`、`estimates`、`hash`。pass／资源 ID 和字节估算均为非负整数；`hash` 是 `sha256:` 加 64 个小写十六进制字符。核心拥有计划序列化及哈希。调整请求通道顺序仍编译为相同规范计划；改变有实际意义的覆盖值会改变哈希，而裁剪未使用分支可以消除其工作量。

## 诊断数据与人类可读上下文

每项诊断包含字符串 `code`、`stage`、`severity`、`message`。可选字符串字段为 `documentPath`、`nodeId`、`portId`、`parameterId`、`suggestion`。缺失可选值会**省略**，不是 null。非空 `evidence` 是字符串键到布尔值、无符号 64 位整数或字符串的映射；空证据省略。原生 source-error 对象不序列化。显式 `sourceMessage` 证据保留相关 OS／解析器／驱动文本，但该文本不是稳定的分支判断契约。

例如，缺少 warp 位移输入时，`validate`、`inspect --plan`、`render` 都保留以下上下文：

```json
{
  "code": "MIX_PORT_REQUIRED_CONNECTION",
  "stage": "validation",
  "severity": "error",
  "message": "Required input has no valid connection.",
  "documentPath": "missing-warp.mix",
  "nodeId": "sample",
  "portId": "displacement",
  "suggestion": "Connect one compatible output to this required input."
}
```

验证、检查、图渲染、doctor 和内置棋盘的人类可读诊断现共用一个[小型格式化函数](../crates/mixture-cli/src/commands/human_diagnostics.rs)。它打印原有代码／消息、阶段／严重性、调用方文档路径、节点、端口、参数、证据和建议。人类文本中的阶段／严重性使用 Rust 枚举拼写（`Validation`、`Compile`、`Error`）；JSON 保留 camel-case 机器拼写。人类可读 `inspect`／`render` 现保留 `port: displacement`，非法渲染覆盖保留 `parameter: cellsX` 及公开 ID 证据。人类文本供阅读；消费者按 JSON 代码和上下文分支。

诊断依次按生命周期阶段、严重性、文档／节点／端口／参数 ID、代码、消息、证据和建议排序；缺失 ID 优先，证据键按字典序排列。重复诊断仍保留为观察项。[诊断契约](./diagnostics.zh-CN.md)定义完整顺序、词汇及精确整数类型。预算拒绝包含数值型 `configured`／`observed`；非法参数的 `observed` 则可能是**包含原始 JSON 值的字符串**，如 `"0"`。不要将证据值统一强制转换为数值。

## GPU 与完成文件证据

Doctor 的 `requested` 对象记录 `backend`、`powerPreference`、`softwareAdapter`、`effectiveBackends`、`requiredFeatures`、`requiredLimits`。实际适配器标识包括 `name`、`backend`、`deviceType`、数值 `vendor`／`device`、驱动字符串、支持的限制及已排序特性。设备证据包含实际获取的限制和启用特性。能力限制键暴露固定 wgpu 版本，见 [GPU 上下文](./gpu-context.zh-CN.md)及[依赖暴露决定](./native-sdk.zh-CN.md)。适配器名称、驱动细节及可选能力因主机而异。

| Doctor 状态 | `verdict`／`ok` | 探针与执行 |
|---|---|---|
| 棋盘探针完成 | `healthy`／true | `computeProbe`、`readbackProbe` 均为 `passed`；`execution` 存在。 |
| 显式 `--skip-probe` | `unverified`／true | 两个探针均为 `notRun`；省略 `execution`。 |
| 获取失败，包括 `--backend none` | `unhealthy`／false | 两个探针均为 `notRun`；省略 `execution`。显式禁用后端时没有 adapter／device。 |
| 获取后探针失败 | `unhealthy`／false | 保留所选上下文；失败／未运行探针状态及首个结构化诊断标明失败阶段。不虚构成功执行报告。 |

Doctor 的棋盘 `execution` 描述宽高、格式、dispatch、pass 数、传输／映射／RGBA 字节、估算 GPU 字节及墙钟耗时。图渲染顶层 `execution` 则包含 `planHash`、`adapter`、`size`、`passCount`、`pipelineCache`、`estimates`、`allocations`、`readbackBytes`、`mappedBytes`、`rgbaBytes`、`timings`。缓存／分配／字节计数为非负整数；`pipelineMs`、`executionMs`、`readbackMs`、`totalMs` 为有限非负 CPU 墙钟毫秒，不是 GPU 时间戳测量。排除项及生命周期限制见[分配计数](./development.zh-CN.md#2k-资源证据)。

渲染的 `context` 是获取快照，因此材质成功渲染后仍为 `unverified`、探针为 `notRun`；它不是第二个 doctor 判定。图执行报告记录实际适配器及完成工作。编码／写入失败后，`context.ok` 可能为 true，而外层渲染 `ok` 为 false。

每项已完成 `outputs` 包含：

| 字段 | 类型及含义 |
|---|---|
| `channel`、`kind` | 支持的通道名，以及 `color`、`scalar` 或 `normal` 解释。 |
| `source` | 计划中连接的端点或带版本的默认输入来源。 |
| `size` | 整数像素 `[width,height]`。 |
| `encoding` | 颜色 RGB 为 `rgba8-srgb`，alpha 线性且不预乘；标量／编码法线为 `rgba8-linear`，alpha 不透明。 |
| `path` | 按 `--out` 构造的调用方相对或绝对 PNG 路径。 |
| `writtenBytes` | 已完成压缩 PNG 的文件长度，不是原始像素缓冲区长度。 |

PNG 使用 8 位 RGBA。颜色文件携带 sRGB 元数据，线性标量／法线文件携带 gamma 1.0。标量复制到 RGB，编码法线保持线性 XYZ 字节。请求文件按材质契约排列：`baseColor`、`normal`、`roughness`、`metallic`、`height`、`ambientOcclusion`、`opacity`、`emissive`。仅输出所选通道。来源元数据及编码由公开计划／renderer 提供；CLI 添加路径、PNG 编码及写入完成信息。

## 失败边界与部分写入

| 失败位置 | 计划哈希 | Context | 图执行 | 完成输出 |
|---|---|---|---|---|
| 源码读取／解码／验证或请求编译 | null | null | null | 空 |
| GPU 获取 | 存在 | 失败上下文 | null | 空 |
| GPU 执行／回读 | 存在 | 已获取上下文 | null | 空 |
| 创建输出目录或首个 PNG 编码／写入 | 存在 | 已获取上下文 | 存在 | 空 |
| 后续 PNG 编码／写入 | 存在 | 已获取上下文 | 存在 | 按规范顺序已完成的前段文件 |

CLI 在获取 GPU 前编译，在创建／写入输出文件前完成 GPU 工作。输出目录在渲染后创建。它创建缺失目录、替换同名文件，保留无关文件，不提供目录原子替换或回滚。发生 I/O 失败的目标文件即使不在 `outputs` 中，也可能已被部分写入或截断；该列表只保证其中条目已完成写入。

独立测试预先创建一个名为 `normal.png` 的目录，然后请求四个通道。`baseColor.png` 完成后，写入 `normal.png` 在 `encoding` 阶段以 `MIX_ENCODING_FAILED` 失败；报告保留已完成 baseColor 条目、完整执行证据和失败的 `outputPath`。粗糙度／高度文件不写入。同一人类可读用例标明已完成文件及失败上下文。不更新任何像素基准。需要结果新旧判断或受控发布的消费者，将使用 PR-014 引入的按代次隔离目录。

## 复现与验收边界

```bash
cargo xtask test-consumer
cargo test --locked --all-features -p mixture-cli
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

`test-consumer` 构建真实 CLI 并显式调用独立 CPU 契约测试。32 次子进程调用覆盖验证／检查成功、确定性排序、覆盖／裁剪、人类／JSON 缺失 warp 输入、未知端口、格式错误输入、缺失源文件、非法覆盖、数值预算、禁用后端获取和用法错误。普通检查不获取 GPU。`gpu-smoke` 还在指定适配器上调用独立 GPU 契约测试，检查 doctor 状态、三种成功渲染请求、PNG 解码，以及两种报告模式的部分写入。

测试夹具使用 `serde_json`、标准进程／文件系统 API 和仅开发时使用的 PNG 解码器，不导入 Mixture Rust API 或 CLI 私有模块，仅使用自有嵌入源码夹具。应用仍是独立 Cargo 工作区。完成记录及原始子进程 stdout／stderr 保存在新目录；跳过或未完成的测试不能满足 `test-consumer`／`gpu-smoke`。[示例指南](../examples/native-consumer/README.zh-CN.md)提供独立调用细节，[PR-012 证据](./evidence/pr-012/README.zh-CN.md)记录本地结果。

PR-012 不改变 JSON 字段、null／省略规则、枚举拼写、源码／计划版本或退出码策略。测试固定相关语义字段，不快照易变的适配器／耗时。当前数据已在本地验证；软件包消费、过期结果调度、完整兼容性／发布策略及暂缓的远端平台 CI，仍属于后续 M4 验收。

PR-013 扩展诊断词汇，加入 `MIX_GPU_DEVICE_LOST` 与 `MIX_GPU_OUT_OF_MEMORY`；报告外层结构和退出码不变。失败诊断可携带适配器、已送达的丢失通知及分配释放证据，同时保留首个错误及其阶段。严格解码器兼容性与分类限制见 [GPU 失败契约](./gpu-failures.zh-CN.md)。

PR-014 添加使用新建代次目录的独立消费者测试。产品 CLI 仍顺序写入文件；消费者只选择最新完成目录，删除过期／部分自有目录并传播清理失败。见[调度契约](./stale-results.zh-CN.md)。

PR-015 对仅由本地 Cargo 归档及同组库归档构建的 CLI 运行同一独立 32 用例 CPU 和十用例 GPU 进程契约，工作目录位于生产仓库外。这添加[包证据](./package-consumption.zh-CN.md)，不改变产品报告外层结构或退出码。
