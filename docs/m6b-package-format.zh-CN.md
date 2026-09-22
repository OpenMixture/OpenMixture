# M6B-02 — `.mixpack v1` 格式、所有权与预算

[English](./m6b-package-format.md) | 简体中文

**实施状态：** M6B-03–05 已实现本契约并完成有界验收，见[资产验收](./m6b-05-qualification.zh-CN.md)。下文版本选型过程属于 M6B-02 设计历史；当前源码与发布状态见[发布指南](./release.zh-CN.md)。

**M6B-03 实现更新：** [共享 Rust CPU codec](./m6b-03-cpu-assets.zh-CN.md)已落地。以下为 M6B-02 选定的字节/所有权契约；CLI/浏览器适配与综合验收仍待 M6B-04/05。设计 PR 未改代码，实现 PR 引入新 crate 和 0.5 版本。

## 选择与证据

采用**未压缩 POSIX USTAR 归档**，扩展名 `.mixpack`，包含可读 UTF-8 JSON manifest、精确 `.mix` 字节和紧密线性 RGBA8 字节。这是受限标准归档，不是自定义二进制图格式。现有 tar 工具可列举/读取条目，但任意 tar 文件不一定是合法资产。该规范刻意排除扩展及磁盘解包。参考：[标准 tar 布局](https://www.gnu.org/s/tar/manual/html_node/Standard.html)。

[可复现实验](../scripts/measure-package-formats.py)及[测量](./evidence/m6b-02/measurement.json)比较包括相同 manifest 的相同内容。最大资源用例为八张 2048×1024 图（64 MiB），符合现有数量/像素/单轴上限。源码大小与 M6B-01 不同，因为实验序列化了显式多资源文档。

| 表示 | 一张 1K 图 | 八张图 / 64 MiB | 判断 |
|---|---:|---:|---|
| 目录加 manifest | 4,195,771 字节 / 3 文件 | 67,112,543 字节 / 10 文件 | 最小松散基线，不是单个离线制品 |
| 条目字节采用 base64 的 JSON | 5,594,488 | 89,483,790 | 外层可读，但膨胀及解码字符串/复制成本明显 |
| ZIP stored | 4,196,103 | 67,113,631 | 紧凑标准选项；本地/中央元数据一致性及可选 ZIP 特性增加验证面 |
| **USTAR 子集** | **4,198,912** | **67,119,104** | 选定；仅比 ZIP stored 多 2,809 / 5,473 字节，顺序定长头及原始条目切片 |

ZIP 对照依据 [PKWARE 规范](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)，不声称有压缩优势。Python 标准库解码测量保留耗时及 `tracemalloc` 峰值；本机最大用例的 JSON 跟踪解码分配约 179 MB，两种归档约 67 MB。这是单次 Python 观测，不是 Rust/WASM 延迟或堆保证。不支持引入自定义二进制格式、压缩依赖或性能承诺。

## 精确归档子集

下列规定是 **v1 接受规则**，不是通用 tar 支持。在有界不可变字节切片上使用检查过的偏移，绝不解包到磁盘。每个 512 字节头必须与合法名称、长度对应的规范 USTAR 头完全一致：

- 名称为 ASCII，在 100 字节 name 字段以 NUL 补齐。顺序严格为 `manifest.json`、`material.mix`，然后按资源 ID 的 ASCII 升序对应 `images/0000.rgba` 至 `images/0007.rgba`。无目录条目。序号路径避免文件系统大小写别名，资源 ID 仍区分大小写。
- mode 为 `0000644\0`；uid/gid 为 `0000000\0`；size/mtime 为 11 位八进制加 NUL，mtime 全零。typeflag 为 ASCII `0`。linkname、uname、gname、设备号、prefix 及未用字节全部为零。magic 为 `ustar\0`，version 为 `00`。
- 头校验和为全部 512 字节的无符号和，计算时校验和字段视为八个 ASCII 空格；编码为六位八进制、NUL、空格。拒绝 base-256、负数、溢出或非规范数值字段。所有算术和宿主尺寸转换均检查。
- 头后跟条目字节，再以零补齐到 512 字节边界。末条目后必须**恰好两个零块（1024 字节）**及 EOF。拒绝额外记录补齐、拼接归档、缺少结束块、尾随数据或非零补齐。写入器不增加 tar 可选的大块记录补齐。
- 拒绝其他名称/顺序/类型/头：绝对/盘符/UNC 路径、反斜杠、`.`/`..`、prefix 别名、重复条目、链接、sparse/PAX/GNU 记录、设备、目录、时间戳、长名扩展、压缩和 ZIP 标记。先有界解析大小/名称，再重建预期规范头逐字节比较即可；不实现通用 tar 解析器。

可用检查过的切片及 Rust 标准算术实现，不选用 tar/ZIP 依赖。已评审的 [tar-rs manifest](https://raw.githubusercontent.com/alexcrichton/tar-rs/main/Cargo.toml)包含此严格字节子集无需的文件元数据依赖和通用归档能力；这不是说 tar-rs 不安全。M6B-03 须以独立标准工具交叉读取合法 fixture 并拒绝负例。若需要更广归档支持，应重开决策，不静默扩展解析器。

## Manifest 与身份

完整必填对象形状如下；仅摘要值在示例中缩写：

```json
{"format":"openmixture-asset","version":1,"document":{"path":"material.mix","byteLength":1060,"sha256":"<64 lowercase hex>"},"resources":[{"id":"heightSource","path":"images/0000.rgba","width":1024,"height":1024,"format":"rgba8-linear","bytesPerRow":4096,"byteLength":4194304,"contentDigest":"<64 lowercase hex>"}]}
```

拒绝各层重复键、未知/缺失字段、非 UTF-8/BOM、非整数数值 token（含指数/浮点形式）、负数、溢出、无效 ID 和非 64 位小写十六进制摘要。width/height 为正 u32，其他数值元数据为 u64，version 为 u32。manifest 可在字节上限内使用不同空白和对象键序；资源数组必须按唯一 ID 严格升序。写入器输出紧凑 JSON，键序如上，字符串 ASCII，整数为最短十进制，无末尾换行。源码字节绝不被写入器重新序列化或迁移。

`byteLength` 必须等于条目长度。资源仅支持 `rgba8-linear`，`bytesPerRow=4*width`，长度 `4*width*height`；无 PNG 解码、alpha/gamma 转换或重采样。文档摘要为精确源码字节的 SHA-256。资源 `contentDigest` 复用 M6-A 域 `mixture-image-rgba8-linear-v1\0`、小端 u32 尺寸和紧密 RGBA 字节；Core 须暴露/复用自身受检摘要/元数据辅助函数，不在 package crate 另写语义实现。包括未用分支在内的每份资源都验证。重算 manifest 哈希，不信任外部摘要；这些哈希检测损坏，不认证作者身份。

若报告包 SHA-256，计算整份归档并置于计划身份之外。改变 manifest 空白可改变包身份，但不改变源码/资源/计划身份。不存缓存可执行计划、请求、shader 二进制、编辑器状态、默认 override、runtime 版本固定值或引擎导出设置。

## 闭包与 override

包必须恰好包含已验证源码在文档默认值下所有图像节点引用的 ID 集合，含断开分支；不得缺失或多余。Core 提供已验证引用清单，codec 不重复遍历/目录语义。零图像文档使用空资源数组且恰有两个条目。所有图像尺寸必须相同；准备时调用方输出尺寸必须匹配，与松散 API 一致。无资源包保留普通尺寸自由。

包 v1 以 `MIX_PACKAGE_RESOURCE_OVERRIDE` 拒绝**所有指向 `resourceRef` 参数的显式 override**，包括等于默认值的 override。数字/枚举/颜色 override 保持 Core 行为。这一显式限制避免形成歧义的未激活资源库；需要替换资源 ID 的调用方继续使用现有松散 API，不限制这些 API。目标/类型由 Core 验证，未知/非法 override 保留原始 Core 诊断。不修复 override、不改写源码、不自动升级节点版本。

合法请求把完整已验证绑定表交给现有 `prepare`；Core 使用当前切片规则，验证已提供但未用的绑定，仅哈希/捕获所选资源，并保留缺失/未知/尺寸规则。包路径与松散源码加相同完整绑定表必须有相同计划哈希、分配和各后端像素。包闭包比松散可选绑定更严格，在通道切片前检查。

## Rust 所有权与公共边界

**M6B-03 已引入 `mixture-asset`**，见上述实现指南。这是可选的公共 CPU codec/API 依赖边界：`mixture-core` 使用者无需编译或信任归档解码器；Native 消费者和 `mixture-wasm` 需要同一 codec，不能把它放在 CLI、JS 或生成绑定中。依赖为 `mixture-asset -> mixture-core` 及已有 workspace serde/serde_json/sha2；M6B-04 计划增加 `mixture-cli`、`mixture-wasm -> mixture-asset`。Core/wgpu 不反向依赖 asset。此 crate 不拥有文件系统、浏览器、GPU、异步 worker 或进程全局状态。Core 只暴露可复用资源元数据/摘要/引用辅助函数；包错误与归档策略不进入 Core。

API 角色（Rust 已可调用）：不可变借用 `AssetView<'a>` 验证归档并暴露只读条目切片；自有资产保留一份移动或受检复制的缓冲区。检查操作无 GPU 地验证全部完整性/闭包。准备同步调用 Core，返回现有 `PreparedRender`，其所选快照独立于 view 生存。Rust 借用防止检查/准备期间修改，view 不得超出来源生命周期。Native writer 接受已验证源码/绑定，哈希输入，一次保留检查过的最终尺寸并确定性写入；不采用倍增缓冲区或文件系统遍历。writer/loader 共用 manifest 模型，不共写图执行。

浏览器适配在任何 await 前同步捕获已接受 Uint8Array 与选项，拒绝共享/可调整大小/已分离/accessor 输入、非法 offset/length，不做类型强转，然后转移到 Rust 自有字节。busy/closing 拒绝先于复制。保守地把 JS 与 Rust 两份包同时和 Core 所选快照计入预算，不依赖 JS 及时 GC。准备完成后、GPU 执行前释放传输包缓冲区；`PreparedRender`/输出生命周期和销毁遵循既有契约。浏览器创作 UI 和任意文件解包不在范围内。

## 显式限制与分配顺序

`PackageLimits` 是独立策略，浏览器 u64 字段用 bigint。省略使用下列默认值，零是合法较低上限，等值通过，不自动提高。这些 v1 最大值是格式子集的硬上限；策略可降低，不可超过。额外应用现有 `SafetyLimits` / `ResourceLimits`，取更严格者。提高 v1 子集上限须明确兼容性决策。

| 限制 | 默认值 / v1 上限 | 检查时点 |
|---|---:|---|
| `packageBytes` | 67 MiB = 70,254,592 | 任何包自有复制之前，从输入得知长度 |
| `manifestBytes` | 64 KiB = 65,536 | manifest 解析/复制前 |
| 源码字节 | 2 MiB = 2,097,152，另受 Core `decodedBytes` 约束 | 源码解析/复制前 |
| 资源数 / 归档条目数 | 8 / 10 | 构建条目/索引表前 |
| 图像单轴 / 累计像素 | 2048 / 16,777,216 | 乘法、哈希及资源捕获前 |
| 紧密资源字节 | 64 MiB = 67,108,864 | 任何载荷哈希/复制前 |
| `packageBufferBytes` | 202 MiB = 211,812,352 | 操作捕获/传输/输出缓冲区保留前 |

顺序：验证策略和输入长度；读取首头及有界 manifest；验证版本/字段/数量/元数据/预算；以受检偏移和零补齐规则遍历精确头/范围；验证哈希并交 Core 验证源码；检查闭包；验证请求/override 限制及操作缓冲预算；最后捕获/准备或写入。浏览器在复制前先按已知输入长度计费初始包快照；后续元数据仍可拒绝。不解压、不展开归档、不无限制索引条目。

设 `P` 为实际包字节，`D` 为源码字节，`M` 为 manifest 字节，`R` 为全部紧密资源字节，`S<=R` 为所选 Core 快照。即使切片省略复制，也保守预留 `D+M` 字节 scratch。Native 借用准备 ≤ `D+M+S`；自有准备/CLI writer ≤ `P+D+M+R`；浏览器已接受准备 ≤ **`2P+D+M+R`**。Native 从调用方借用输入写包，预留 `P+D+M`。累加所有同时自有资产/view，保留输入资产时另计 writer 输出。保留前拒绝超预算。每个浏览器 runtime 仍沿用一次 prepare 的 busy 规则；调用方可显式拥有多个独立资产，不承诺全局上限。

实测普通载荷的浏览器账本为 12,593,595 字节；最大资源 fixture 为 201,350,751 字节。在所有独立上限处，保守表达式为 209,780,736 字节（200.0625 MiB），低于 202 MiB。即使源码/manifest/资源均最大，归档边界开销仍容纳于 67 MiB。这些是**字节缓冲区界限**，不是进程 RSS：调用方输入、类型化图/manifest 对象、分配器开销、JS 垃圾及留存输出另计。类型化对象仍受 manifest/源码/图限制，允许可失败分配处须返回结构化分配失败。M6B-03/04 须对真实 Rust/WASM 复制路径计数并拒绝额外整包副本，不能以 Python 峰值替代验收。GPU 512 MiB transient 核算不变。

## 错误、版本与验收

`AssetError` 拥有包错误，含稳定 `code`、`stage="package"`、消息、可选条目/偏移/资源 ID、测量/配置证据及建议。保留 v1 错误码：`MIX_PACKAGE_INVALID`（边界/manifest/路径/闭包）、`MIX_PACKAGE_UNSUPPORTED_VERSION`、`MIX_PACKAGE_LIMIT_EXCEEDED`、`MIX_PACKAGE_CONTENT_MISMATCH`、`MIX_PACKAGE_RESOURCE_OVERRIDE`、`MIX_PACKAGE_ALLOCATION_FAILED`。委托失败时保留 Core 原始错误/stage 及 GPU 错误，不换成通用包包装。按上述顺序、条目顺序和 ID 字典序拒绝，保留首个决定性错误。CLI I/O 失败仍由适配层拥有，沿用 I/O 退出类别。

外层包版本 1；`.mix v1`、显式节点版本、计划 v2、API schema 2 不变。包检查有独立报告 schema 1。现有浏览器 API 保留，包 API 为新增。实现目标是高于 0.3 和独立 0.4 维护候选的下一未发布次版本：**Rust 0.5.0 / browser 0.5.0-alpha.0**。M6B-03 在改版本前须核对远端/源码版本占用，若冲突，以文档选择下一个 minor，不能重用版本/字节。本次不授权发布。

[已提交二进制 fixture](../fixtures/packages/mixpack-v1/cases.json)包含标准工具可读的合法资产及畸形/版本/哈希/路径/尺寸/重复负例。这些契约预期现由 M6B-03 Rust 回归测试执行，覆盖全部语料回归、源码/哈希不符、零/等值/超限、完整闭包、每个头字段、各边界溢出/截断、借用/自有生命周期、确定性与独立 tar 读取。M6B-04 覆盖浏览器复制预算、offset view、修改、busy/destroy/拒绝；M6B-05 比较松散/打包计划和像素、四权重、65×3、打包 Native/browser 消费者、已有材质及六项 CI。保留 v1/噪声历史，包验收不扩展数值支持范围。
