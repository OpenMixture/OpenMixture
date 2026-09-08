# M4 实施计划——稳定的原生消费路径

[English](./M4_PRS.md) | 简体中文

**状态：** 依据 `e9dd03b` 的 [M3 评审](./docs/m3-review.zh-CN.md)启动，日期 2026-09-08。PR-011、PR-012 已本地实现并验证；PR-013 至 PR-015 仍待实施。本计划具体落实 [M4](./ROADMAP.zh-CN.md#m4--稳定的原生-sdk)，不改变架构或扩展节点词汇。远端平台 CI 继续暂缓；M4 规划及本地工作不关闭这些验收项。

消费者需要现有的解码 → 验证 → 编译 → wgpu → 自有输出路径。Release 测量支持防抖预览，未显示值得引入原生绑定或守护进程的启动瓶颈。2K 描述符峰值无需池化即可满足当前预算。以下工作稳定可观察行为，并独立验证消费路径。

## 顺序与共同规则

```text
PR-011 Public Rust API + independent consumer
   -> PR-012 CLI JSON/exit-code contract + diagnostic context
      -> PR-013 Device loss / out-of-memory contract
         -> PR-014 Stale results + bounded consumer/cache lifetime
            -> PR-015 Packaged consumer + compatibility/release checks
```

每个 PR 保留为可独立评审的本地提交或 PR，包含成对中英文文档、定向测试和 `cargo xtask check`。保持 `mixture-core` 无 GPU 依赖、`mixture-wgpu` 为唯一像素执行器、CLI 轻薄、GPU 状态显式，并保持原始评审资料包不变。无需新增运行时 crate。独立消费者 Cargo 夹具属于实际依赖／编译边界，不能意外加入产品 workspace。

`cargo xtask test-consumer` 已由 PR-011 **实现**并纳入 `check`，保持纯 CPU。`cargo xtask package-check` **拟由** PR-015 引入，尚未实现。GPU 执行继续通过现有 `gpu-smoke` 策略显式进行，不给普通 `check` 新增 GPU 要求。增加测试时保持现有 CLI schema／退出行为；任何有意不兼容变更均须先明确兼容性决定。

## PR-011 — `feat(sdk): verify public Rust consumption end to end`

**本地完成，2026-09-08：** 提交 `244b384` 位于 `codex/pr-011-native-consumer`，父提交为 `f56bffe`。见[公开 API 契约](./docs/native-sdk.zh-CN.md)、[独立应用](./examples/native-consumer/README.zh-CN.md)及[检查、源码哈希与配对 release 证据](./docs/evidence/pr-011/README.zh-CN.md)。保留原始 wgpu getter，并文档化其逃生口限制。Metal 与固定 SwiftShader 均在 renderer 销毁后检查真实自有输出；产品运行时行为及依赖没有改变。软件包消费仍属于 PR-015。

### 结果与依据

独立 Rust 程序拥有自己的 `.mix`，使用公开导入，请求公开覆盖／通道，渲染并消费自有像素和报告。[M3 探针](./docs/reviews/m3/native-consumer/README.zh-CN.md)已证明 CPU 编译和 GPU 调用形态，但没有执行该 GPU 路径或消费打包 crate。

### 范围

- 针对该程序审查公开 decode／validate／compile／request／render／error／result 类型。优先改进文档和定向修正，不先增加门面或 builder。
- 在声称 API 稳定前，确定并记录原始 `GpuContext` wgpu getters 及重新导出的依赖类型的兼容性。不静默删除现有访问器；有依据时说明 escape hatch 的兼容性边界。
- 添加消费者自己的输入和独立 Cargo 清单，不使用私有导入或仓库相对运行时资源。修正六节点／未来编译器的过时 Rustdoc，替换依赖源码相对路径的输入示例。
- 在消费者中执行显式原生 GPU 获取／渲染。检查请求通道、颜色／标量／法线编码、connected／default 来源、尺寸／字节长度、适配器、计划哈希和度量，证明 renderer／context 被 drop 后返回像素仍可使用。
- 增加 `test-consumer`，覆盖 CPU 错误、稳定的覆盖／通道编译及完整 GPU 路径编译。将显式 GPU 消费者调用加入 `gpu-smoke`。
- 通过公开 Rust 测量相同 1K 材质，分别记录上下文创建、首次渲染和复用 renderer 的调用。比较耗时前先比较正确性。直接返回像素；示例无需编码 PNG 或实现像素语义。

### 验收与验证

运行 `cargo xtask test-consumer`、`cargo xtask test-plan`、`cargo xtask check`，然后在本地 Metal 和固定 SwiftShader 上显式运行 `cargo xtask gpu-smoke`。独立程序必须消费实际自有输出，以预期参数上下文拒绝非法公开覆盖，并报告所选适配器。性能记录须包含源码／构建／适配器／尺寸／通道及度量定义。不得将 path 依赖称为包消费者验收。

**不在范围内：** 没有消费者失败依据的新 API 抽象层；N-API、守护进程／IPC、浏览器／UI；打包／发布；着色器修改或性能优化。

## PR-012 — `fix(cli): preserve diagnostic context and verify consumer reports`

**本地完成，2026-09-08：** 包含本记录的 PR-012 提交位于 `codex/pr-012-cli-contract`，父提交为 `244b384`。共享 CLI 格式化函数保留文档／阶段／节点／端口／参数上下文，修复缺失位移端口及渲染覆盖参数遗漏。[CLI 契约](./docs/cli-contract.zh-CN.md)、独立 CPU／GPU 进程测试及[已记录证据](./docs/evidence/pr-012/README.zh-CN.md)验证现有报告结构、退出码、完成的 PNG 及部分写入。JSON 结构和版本不变；`validate` 保留无版本字段的诊断结构。GPU／核心语义及产品依赖没有改变。

### 结果与依据

独立程序能调用已构建 CLI，可靠区分成功、源／请求失败和操作失败。[M3 探针](./docs/reviews/m3/diagnostics/README.zh-CN.md)证明了具体缺陷：人类可读 `inspect` 和 `render` 遗漏 warp 缺失输入的 `displacement` 端口，而 JSON 与人类可读 `validate` 保留该信息。

### 范围

- 修复人类可读报告的上下文遗漏，仅在重复代码有依据时共享小型格式化辅助函数。语义和类型化错误继续归所属库所有。
- 记录并测试 `validate`、`inspect --plan`、`doctor`、`render` 当前版本化 JSON 报告：必填／可选字段、已承诺的确定性顺序、诊断及测量证据类型。
- 保持成功退出 `0`，调用／源／请求错误退出 `2`，操作错误退出 `1`。明确现有 usage 错误例外：`--json` 不会将 usage 错误转换为 stdout JSON。保持 JSON stdout 可解析并记录 stderr 行为。
- 在独立工作目录运行已构建可执行文件，使用消费者自己的输入／输出路径。覆盖公开覆盖、请求输出、适配器／计划标识、编码元数据及完成写入的文件。
- 覆盖 GPU 获取前失败和 PNG 部分写入失败，包括已经写完的文件列表。不承诺原子替换输出目录；调用者在 PR-014 使用逐请求目录。

### 验收与验证

增加 CLI 集成／契约断言，在人类可读和 JSON 模式下通过 validate／inspect／render 测试相同无效 warp 夹具，并覆盖格式错误、非法覆盖、缺失源文件、显式禁用后端获取、成功报告及部分写入。断言语义字段和退出码，不快照易变的耗时／适配器。运行 `cargo xtask test-consumer`、专项 CLI 测试和 `cargo xtask check`；通过 `cargo xtask gpu-smoke` 执行成功渲染／部分写入 GPU 路径。

**不在范围内：** 无实际需要的新文档格式或 JSON schema 版本；顺手更改 usage 错误策略；通用序列化框架；事务式导出；绑定或远端服务。

## PR-013 — `fix(gpu): classify device loss and out-of-memory failures`

### 结果与依据

库和 JSON 消费者无需解析驱动字符串，即可根据设备丢失和内存不足分支处理。当前错误作用域捕获 OOM，但仅按操作阶段映射；上下文没有运行时设备丢失记录。现有销毁设备测试证明会失败，未证明稳定原因／生命周期契约。

### 范围

- 为设备丢失和 OOM 定义类型化失败原因与稳定外部诊断，同时保留操作阶段、首个错误、source chain、所选适配器及可用计划证据。
- 每个显式 `GpuContext` 拥有自身丢失状态；定义其可观察时机，以及后续 render 是否立即失败。不重新获取设备，也不静默换处重试。
- 保留失败时资源／回读清理，避免将较早有用错误转换为通用包装。定义在另一操作失败期间记录到设备丢失时的优先级。
- 扩展现有销毁设备测试，覆盖冷缓存和已有管线缓存、不返回成功输出、逐次资源释放，以及独立上下文仍能运行。

### 验收与验证

确定性测试分类和序列化，不耗尽物理内存。通过显式 GPU 测试执行真实设备销毁／丢失通知；不得将合成 OOM 分类称为硬件耗尽测试。运行专项 context／operation／readback 测试、CLI 报告测试、`cargo xtask test-consumer`、`cargo xtask check`，并在两种本地策略上运行 `cargo xtask gpu-smoke`。保留驱动证据，区分分类覆盖与后端行为。

**不在范围内：** 自动恢复、备用执行器、进程全局错误状态、硬件内存耗尽、新资源池、通用重试框架。

## PR-014 — `feat(consumer): reject stale renders and bound retained state`

### 结果与依据

长渲染期间修改参数，不会让旧结果覆盖较新请求。M3 测得木材／皮革完整 CLI 输出中位耗时约 0.4–0.7 秒；现有逐次等待的 30 秒超时不等于取消或整体 deadline。M4 明确允许用过期结果处理替代取消。

### 范围

- 在独立消费者中添加单调递增请求代次。只发布最新请求代次；较新请求失败时，也不能将旧成功误标为当前结果。
- 示例中用一个活动渲染和一个可替换待处理请求约束工作量。请求新旧与调度留在消费者，renderer 所有权保持显式。
- 测试 CLI 输出时使用代次专属目录。仅提升／消费当前结果并清理自己拥有的过时输出；旧请求绝不能写入最新目录。
- 明确已经提交的 GPU 工作可能继续完成，drop async future 或 poll 超时不承诺 GPU 取消。没有实际需求不增加 abort API。
- 验证现有 renderer 管线缓存在参数／通道变化时不超过九种像素 kernel，支持显式 clear／drop，不泄漏逐次渲染描述符资源。除非新测量需要改变，资源复用保持为零。
- 解释单次与累计并发内存上限；替换期间最多保留当前显示的 CPU 结果和当前完成结果。

### 验收与验证

使用确定性消费者测试覆盖乱序完成、较新请求失败、替换待处理工作、重复完成、隔离输出路径和清理。使用有界 GPU 序列验证缓存增长／清空、独立 renderer、逐次计数重置，以及完成／失败后 `liveBytes == 0`。运行 `cargo xtask test-consumer`、专项 GPU／资源测试、`cargo xtask check` 和 `cargo xtask gpu-smoke`。新旧结果测试无需依赖 sleep 或计时竞争。

**不在范围内：** 强制 GPU 中断、core 内异步任务框架、守护进程／IPC、无界并行渲染、池化／最后消费者优化、UI 控件。

## PR-015 — `test(release): verify packaged native consumption and compatibility`

### 结果与依据

独立消费者可从预期包内容构建，并使用已构建 CLI，无需仓库内部资源。当前 workspace crate 为 `publish = false`，库依赖使用源码路径，文档引用外层示例文件。源码 path 探针不能证明这个退出验收项。

### 范围

- 引入 `cargo xtask package-check`，在隔离位置暂存并验证实际本地包内容，以本地包解析 core／wgpu 依赖，在不使用仓库资源路径的情况下构建独立消费者。记录准确的包解析方式；仅列出归档内容不够。
- 审查包包含项、许可证、Rustdoc／README 资源和依赖版本元数据。保持实际发布禁用，仅按验证需要调整本地验证元数据，避免无关锁文件修改。
- 将包验证和 CPU 消费者检查纳入适当仓库／CI 检查，GPU 消费仍显式执行。在隔离工作目录测试已构建 CLI，作为另一原生边界。
- 编写成对兼容性／发布文档，覆盖 `.mix`／节点版本、计划哈希变化、公开 Rust 类型及依赖暴露、CLI schema／退出码、输出编码、诊断演进、发布说明和可复现验证。
- 记录所有剩余外部平台验收及已测主机／后端矩阵。为最终实施序列重新运行三种材质 golden 检查和 2K 跟踪，不覆盖接受的像素。

### 验收与验证

运行 `cargo xtask package-check`、`cargo xtask test-consumer`、`cargo xtask check`、显式 `cargo xtask gpu-smoke`、`cargo xtask golden check` 和 `cargo xtask trace-2k`，GPU 部分使用预期本地适配器策略。包消费者必须在显式 GPU 检查中通过公开 API 渲染并消费输出。运行时资源缺失必须让包测试失败。仅当 PR-011–015 的证据、公开文档与行为一致时，M4 才在本地就绪；任何暂缓远端验收仍明确保持开放，阻止声称可发布。

**不在范围内：** 发布到 crates.io、远端 push／merge、二进制分发／安装器、WebAssembly、节点编辑器、引擎专属导出、第十三节点，或照搬旧仓库未来任务。

## 完成记录

每个 PR 落地时在对应章节记录实现版本及可复现证据；所属 PR 引入命令前，不将拟议命令标为已实现。使用 [PR 模板](./.github/pull_request_template.md)，说明有意排除的范围，并按依赖顺序保留独立提交。PR-015 之后先评估 M4 退出条件及未完成平台证据，再确定下一轮计划。
