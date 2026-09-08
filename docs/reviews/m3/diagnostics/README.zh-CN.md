# M3 公开诊断证据

[English](./README.md) | 简体中文

这 16 个仅使用 CPU 的探针确认：在源版本 `e9dd03bb272a29de6243aec79a3450b47b90530e`（`e9dd03b`）上，结构化诊断提供了有效信息，退出码分类与文档一致。它们也复现了一个人类可读报告缺陷：**`inspect` 和 `render` 遗漏了 warp 缺失输入的 `portId`，而 JSON 和人类可读的 `validate` 输出均能指出 `displacement`。** 本证据不包含运行时代码修改或 GPU 执行。

## 记录与来源

- [原始 debug 记录](./original-debug/summary.json)：从 `tmp/m3-review/diagnostics/` 复制全部 49 个文件，没有改变任何字节。原始工具记录了二进制哈希、参数和退出码，但没有记录源版本或构建选项；此处不为该次运行追溯补造这些信息。
- [release 记录](./release/summary.json)：使用已构建的 release CLI 重新运行 16 个探针，从新建临时目录复制 50 个文件，未做规范化处理。记录包含工作区源版本、相关源码状态、fixture 哈希、helper 哈希、完整参数、声明的构建命令和 profile、运行前后二进制哈希、原始输出流哈希及退出码。相关源码没有未提交修改，二进制与 fixture 在运行期间保持不变。
- [复制完整性清单](./capture-files.json)：列出两次记录中每个复制文件的 SHA-256 和字节数。空 stderr 文件和结尾换行均被保留。
- [release 构建日志](../performance/release-build.log)：根任务在记录的源版本上执行了 `cargo build --release --locked --all-features -p mixture-cli`。helper 从不构建。其 profile／构建命令字段记录调用者声明；二进制哈希标识具体可执行文件，并非内嵌的源版本证明。

| 记录 | 二进制 profile | 可执行文件 SHA-256 |
| --- | --- | --- |
| `original-debug` | debug | `43e8e26cd69fba66ac492fe14122e73ff1d3f27a2c5a9ba73b075627e5603d16` |
| `release` | release，全部 features | `56ea817b5716e8db38dfe519e07f5b1f283708bcbcd3fa2eb0a8b21952c7136d` |

每个探针均包含 `<name>.json` 记录、完整的 `<name>.stdout` 和完整的 `<name>.stderr`。记录路径、不存在文件的绝对路径以及原生操作系统错误文本均被有意保留。在其他机器上复现时，路径或驱动／系统文本不必逐字节相同。

## 探针证明了什么

| 探针 | 预期退出码 | 观察结果 |
| --- | ---: | --- |
| `help` | 0 | 列出公开 CLI 入口。 |
| `unknown-port-json` | 2 | 指出节点、缺失的输出端口和下游必需输入。 |
| `unknown-port-human` | 2 | 人类可读的 `inspect` 输出遗漏两个端口标识。 |
| `missing-warp-json` | 2 | 返回 `MIX_PORT_REQUIRED_CONNECTION`、节点 `sample`、端口 `displacement`。 |
| `missing-warp-human` | 2 | 人类可读的 `inspect` 输出指出节点，但遗漏端口。 |
| `invalid-parameters` | 2 | 保留参数标识、预期类型／范围及观测值。 |
| `cycle` | 2 | 报告实际闭合路径 `a -> b -> a`。 |
| `duplicate-key` | 2 | 保留 parse 阶段代码、行号、列号及重复字段的原始错误信息。 |
| `invalid-exposed` | 2 | 指出公开绑定与无效的目标参数。 |
| `unknown-override` | 2 | compile 阶段错误指出未公开的参数 ID。 |
| `budget` | 2 | 两个轴均报告配置上限 `2048` 与观测值 `4096`。 |
| `doctor-none` | 1 | 返回适配器策略错误；没有 adapter／device，没有有效 backend，两项探针均为 `notRun`。 |
| `missing-file` | 1 | 保留文件 I/O 代码、传入路径和操作系统错误文本。 |
| `usage-json` | 2 | 缺少 `--plan` 时，按文档在 stderr 输出用法文本，stdout 为空。 |
| `missing-warp-render-human` | 2 | 人类可读的 `render` 输出遗漏端口；不创建输出目录。 |
| `missing-warp-validate-human` | 2 | 人类可读的 `validate` 输出保留 `port: displacement`。 |

release 的全部 16 个退出码均符合预期。这不表示每条人类可读诊断都已充分：release 汇总中，遗漏端口的两项观察值仍为 `false`。JSON 行为与[诊断契约](../../../diagnostics.zh-CN.md)、[格式指南](../../../file-format.zh-CN.md)和 [doctor 策略](../../../gpu-context.zh-CN.md)一致。

## 不申请 GPU 的复现方式

需要 Python 3.9 或更新版本、Git、本工作区和已构建的 CLI。在仓库根目录运行，选择一个尚不存在的输出路径：

```bash
python3 docs/reviews/m3/diagnostics/reproduce.py \
  --binary target/release/mixture \
  --profile release \
  --build-command 'cargo build --release --locked --all-features -p mixture-cli' \
  --out tmp/m3-review/diagnostics-release-rerun
```

可选的 `--build-command` 字符串仅作为来源信息保存，从不执行。helper 记录当前工作区版本；若要与这份历史记录比较，请使用 `e9dd03b` 的源码以及单独构建的对应二进制。helper 无法推断任意传入可执行文件实际使用了哪份源码。

[helper](./reproduce.py) 顺序运行探针，并按原始字节保存子进程输出流。它只调用传入的 CLI 与只读 Git 元数据命令。`doctor` 显式使用 `--backend none`；无效 `render` 探针也添加了 `--backend none`。相比原始 debug 记录，该 render 参数是有意增加的保护。其他探针只经过 CPU 验证、计划检查、帮助或用法错误路径。不存在文件的路径在新建输出目录内生成，但不会创建该文件。

已存在的输出路径（包括符号链接）会被拒绝。此持久证据目录内的所有输出路径即使尚不存在也会被拒绝。helper 不覆盖、不暂存、不提交证据。失败后会保留可用输出流和失败记录；重试时应选择另一个新目录。退出 `0` 表示记录完成、命令退出码符合预期、输入与二进制未变且显式禁用 backend 的检查通过；人类可读上下文观察值属于发现，不要求现有缺陷持续存在。

## 限制与最小后续改动

这是公开 CLI 失败路径的抽样证据，不是完整的 parser／graph 测试矩阵、独立原生 SDK 消费者测试或 GPU 健康结果。它没有实际测试适配器获取、设备／shader／pipeline 失败、执行、读回、设备丢失或远程 CI。不使用 GPU 的结论来自显式禁用 backend 的策略和已审查命令路径，并非硬件遥测。历史 debug 记录中的 `gpuInitialized: false` 也有相同的证据限制。

最小的 M4 修复是在 `inspect`／`render` 人类可读输出中保留 document、stage、node、port 和 parameter 上下文，使用 JSON 已能获取的同一组公开诊断。以缺少 warp 输入和未知端口 fixture 添加聚焦的 CLI 回归断言，保持现有 JSON 契约和退出码。这个问题不需要图修复、新诊断分类或新渲染器。
