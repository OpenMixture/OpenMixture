# 本地软件包消费

[English](./package-consumption.md) | 简体中文

M6B-03：[共享 CPU 资产 codec](./m6b-03-cpu-assets.zh-CN.md)已提供独立 `mixture-asset` 公共 API；源码与隔离归档消费者均覆盖它，Rust 0.5.0 未发布。

PR-015 实现 `cargo xtask package-check`：生成真实本地 Cargo 归档，在生产仓库外验证，从归档构建独立 Rust 消费者和 CLI，并执行 CPU 契约。显式 `gpu-smoke` 重复包验证，并添加真实软件包 GPU／CLI 消费。这完成本地 M4 包门槛，不发布 crate 或分发二进制。[验收证据](./evidence/pr-015/README.zh-CN.md)记录确切运行。

## 归档与依赖边界

M6A-03 还针对解包归档运行公开准备图像 GPU 消费测试。每次构建前从共享目标清理五个本地包，只保留外部依赖缓存，避免归档时间戳导致同版本旧运行时代码被误用。原资源合同见 [Native 资源实现](./m6a-03-native-resources.zh-CN.md)；当前资产合同见上文链接。

四个产品包（core、asset、wgpu、CLI）为 `0.5.0` 且 `publish = false`。工作区路径依赖现在同时精确要求 `=0.5.0`；Cargo 规范化归档清单保留版本，移除生产路径。仓库内消费者清单使用三个源码路径库依赖。

| 包内容 | 验证 |
|---|---|
| `Cargo.toml` 及 Cargo 的原始清单副本 | 通过解析元数据检查版本、禁用发布、target 路径和依赖要求。 |
| `src/**` | 解包文件哈希必须匹配生产包输入。core／asset／wgpu 单元测试和 Rustdoc 示例在隔离工作区构建。 |
| wgpu `shaders/**` | 携带全部十一个嵌入内核。删除已打包 `constant.wgsl` 必须实际编译失败；随后恢复字节并复核全部解包文件。 |
| 两种 README 语言及两个许可文本 | 归档必需条目；许可字节必须匹配仓库 MIT／Apache-2.0 文本。 |
| wgpu `src/testdata/*.mix` | 三个源码嵌入单元测试输入位于包内，与规范 constant-scalar／transform-2d／warp 夹具逐字节一致。 |

仓库集成测试、完整节点／材质夹具集合及历史证据不是包运行时依赖。源码内嵌单元测试会包含并运行。M6B-03 只增加可选 CPU 资产 crate，不增加外部依赖、shader 实现或像素算法。

检查器对四个包运行 `cargo package --locked --offline --no-verify --exclude-lockfile --allow-dirty`。Cargo 常规验证及包锁生成会尝试从注册表解析尚未发布的同组包，因此明确用下述本地验证器替代。`--allow-dirty` 捕获可审查的未提交实现；以源码／归档哈希识别内容，不声称 VCS 干净。Cargo 在 [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html) 中说明清单规范化及这些开关。

这些本地源码归档有意省略 `Cargo.lock`，不宣称可从注册表安装，也不作为 `cargo install --locked` 输入。验证器另外保存解析后的锁文件作为证据，两个已提交的仓库／消费者锁均不变。实际发布仍被禁用；未来分发需单独审查最终锁／注册表／安装策略。

## 隔离验证序列

清单文件身份及构建目标所属目录通过规范化文件系统路径验证，包括 Windows 特殊路径前缀的规范化。同一解包文件的不同路径写法可以通过；缺失文件或位于所属解包目录之外的构建目标会被拒绝。

1. 快照源码／构建输入，检查许可副本与三个单元夹具，仅获取已提交锁中的公共依赖。
2. 生成归档，检查成员路径／必需文件，解包到位于生产仓库**之外**的新建 OS 临时目录，并对照快照检查源码字节。归档本身从不重写。
3. 将独立应用自有源码、测试及 `.mix` 输入复制到该目录。仅临时清单变化：把三个生产路径替换为精确版本要求，加入一次性验证工作区。不改写源码或运行时行为。
4. 用 `[patch.crates-io]` 仅指向已解包的 core／asset／wgpu 目录。四个规范化包与应用组成隔离工作区。拒绝 Mixture 注册表替代或生产路径；解析包和 target 路径必须匹配解包树。
5. 用已提交工作区锁初始化临时锁。一次离线元数据解析可增加本地应用并裁剪未用包；外部**名称／版本／来源／校验和**身份必须仍是已提交固定项的子集。后续构建、测试、文档及进程契约测试全部使用 `--offline --locked`。
6. 构建和测试隔离工作区，包括源码内嵌单元测试、独立 CPU 测试及 Rustdoc 示例；构建 Rustdoc 并拒绝警告。`target/package-consumer` 保留编译产物缓存，不提供源码资源。这是文件系统／源码解析隔离，不是 OS 安全沙箱，也不声称无缓存构建。
7. 删除已打包常量 shader，运行真实失败检查，即使命令失败也恢复文件，并确保每个解包文件仍匹配原始哈希。缺失资源不能借助旧成功构建通过。
8. 在空的外部工作目录运行打包 Rust 可执行程序，并在新建外部目录通过 32 个 CPU 子进程用例运行打包 CLI。GPU 模式还在 renderer drop 后检查真实 Rust 自有像素，并运行十个 CLI GPU 用例，验证解码 PNG、适配器证据及部分写入。
9. 拒绝运行期间的源码漂移。保留归档、规范化清单、锁、哈希、原始流和回执；仅删除成功运行自有的临时解包树。失败保留未完成状态及临时路径供诊断。

本地 patch 是依赖解析机制，不是复制实现代码或备用执行器。包元数据证明所用解包 crate。内部应用的 `packagedCratesValidated: false` 保持不变：运行中的应用无法证明自身构建来源。外层验证器仅在检查来源及全部消费测试后设置 `packagedCratesValidated: true`，GPU 完成另外报告。

## 命令与证据

```bash
cargo xtask package-check
cargo xtask test-consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

按 [GPU 指南](./gpu-context.zh-CN.md)准备 loader。验证器要求仓库工具链的 Cargo／Rust，以及支持 `-tzf`／`-xzf`／`-C` 的 `tar` 命令；缺少工具会明确失败。`package-check` 不接受 GPU 标志，普通 `check` 不获取 GPU。`check` 现已包含源码消费者及包检查，既有 CPU CI 矩阵因此运行同一门槛。GPU CI 随 smoke／材质／trace 产物保留包证据；修改工作流不代表远端通过。

`tmp/package-check/latest.json` 引用新建 `cpu-*` 或 `gpu-*` 尝试。其 `status.json` 初始为未完成，仅验证后变为成功。成功状态包含 `packagedCratesValidated`、`gpuExecuted`、已删除的外部临时路径及 `verification` 对象，后者链接报告并记录归档／二进制哈希和锁策略。`phase.json` 与逐命令回执保留确切参数、工作目录和退出码。缺失 shader 日志有意记录非零退出。

`gpu-smoke` 在执行前令 `tmp/gpu-smoke/package-status.json` 失效，仅打包 GPU 检查通过后才链接成功的包尝试。CI 保留 `tmp/package-check/`，构建缓存位于忽略的 `target/`。原始路径描述原临时工作目录，成功后已不存在；CLI 证据和 PNG 在删除前复制。

范围外：发布、上传／下载 Mixture 注册表包、安装或二进制分发、WebAssembly、新绑定、材质改动及远端执行。[兼容性记录](./compatibility.zh-CN.md)和[发布清单](./release.zh-CN.md)区分本地验收与剩余发布门槛。
