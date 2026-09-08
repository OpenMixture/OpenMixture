# PR-011 原生消费者证据

[English](./README.md) | 简体中文

**本地验收，2026-09-08：** 独立 [Rust 应用](../../../examples/native-consumer/README.zh-CN.md)通过公开 API 编译，在 Apple M5 Metal 和固定 SwiftShader Vulkan 上执行真实 GPU 工作。renderer／context 销毁后，两份返回结果仍可使用。产品运行时行为、shader、已接受基准及产品锁文件不变。本次验证源码 path 消费；软件包内容、PR-012–015 和暂缓的远端 CI 仍未完成。

## 源码与检查

实现版本为包含本记录的 PR-011 提交，位于 `codex/pr-011-native-consumer`，父提交为 `f56bffe732ae9364c0402ec04ab5e84ac3e1990e`。测量在最终文档更新前，从其未提交实现采集。两份性能汇总保留父版本、未提交源码路径、准确源码／输入／manifest 哈希、脚本及二进制哈希、主机、库版本、适配器选项、命令、工作目录、退出码及原始输出流哈希。[capture-manifest.json](./capture-manifest.json)核对测量时的源码哈希与最终源码一致，并列出归档证据。

| 检查 | 证据与结果 |
|---|---|
| 独立 CPU 消费者 | [命令日志](./checks/test-consumer.log)、[状态](./checks/consumer/status.json)、[CPU 报告](./checks/consumer/cpu.stdout.log)：五项单元测试、三项进程测试、独立锁定构建／Clippy／格式化，以及从生产方目录外执行真实调用，均通过。 |
| 编译器／请求回归 | [test-plan.log](./checks/test-plan.log)通过。 |
| 仓库命令回归 | [xtask-consumer-tests.log](./checks/xtask-consumer-tests.log)通过。 |
| 最终仓库检查 | [check.log](./checks/check.log)通过，包括 CPU 消费者；普通检查不获取 GPU。 |
| Metal 显式 GPU 冒烟 | [命令](./gpu-smoke/metal/command.log)、[消费者报告](./gpu-smoke/metal/native-consumer/gpu.stdout.log)、[状态](./gpu-smoke/metal/native-consumer/status.json)通过。 |
| 固定软件显式 GPU 冒烟 | [命令](./gpu-smoke/software/command.log)、[消费者报告](./gpu-smoke/software/native-consumer/gpu.stdout.log)、[状态](./gpu-smoke/software/native-consumer/status.json)通过。 |

CPU 消费者断言稳定计划哈希、源码往返、输出排序、重复通道拒绝、仅粗糙度的依赖裁剪，以及格式错误／缺失端口／限制失败的类型化诊断。非法公开 `repeat=0` 报告 `MIX_PARAMETER_INVALID_VALUE`、compile 阶段、节点 `pattern`、参数 `cellsX` 和公开 ID `repeat`。禁用后端的进程测试验证结构化早期获取失败，不初始化 wgpu。

两种 GPU 运行都在应用自有 65×3 输入上使用 `repeat=16` 和 `repeat=4`。检查四个请求通道、连接／默认来源、编码、尺寸、字节长度、字面量 sRGB／直通 alpha 与线性标量／法线值、覆盖后的不同像素、计划标识、pass 数、所选适配器及逐次调用描述符释放。全部像素检查在唯一 renderer／context 销毁后执行。[原生 API 指南](../../native-sdk.zh-CN.md)说明已审查的生命周期、依赖暴露及原生轮询契约。

## 配对 1K release 测量

[测量脚本](./measure_native.py)在 Metal 上对每种已接受默认材质各运行三轮，在 SwiftShader 上对木材运行三轮。每轮先启动一个原生进程，再启动一个 CLI 进程。原生进程获取一个 context，连续渲染相同计划两次；不丢弃预热轮。全部请求为 1024×1024，包含 `baseColor`、`normal`、`roughness`、`height`。[原生](./checks/consumer-release-build.log)与 [CLI](./checks/cli-release-build.log)构建使用 `--release --locked --all-features` 和 Rust 1.98.1。主机为 macOS 26.5.1 arm64；软件源码固定版本为 `694585a05946e1ed49b6bd577ca6537cbb57f025`。

48 项通道比较均检查首次 Rust 字节、renderer 复用字节及解码后的 CLI 像素在**同一适配器上完全一致**，随后比较已接受 1K 基准，全部通过。软件木材逐字节一致；Metal 相对软件基准的最大字节差异为陶瓷 0、皮革 1、木材 2，均在未修改的材质容差内。脚本验证基准文件哈希、预期计划哈希及 PNG 编码元数据，不更新或合成像素。默认材质比较不替代 M3 已记录的完整材质变体验收矩阵。

三轮中位耗时，单位毫秒：

| 适配器／材质 | 解码＋验证＋编译 | Context 获取 | 首次渲染调用 | Renderer 复用调用 | CLI 进程＋PNG＋报告 |
|---|---:|---:|---:|---:|---:|
| Metal / glazed-ceramic | 0.094 | 8.345 | 27.229 | 21.730 | 54.627 |
| Metal / leather | 0.089 | 8.087 | 32.614 | 23.641 | 683.056 |
| Metal / wood | 0.109 | 7.912 | 32.878 | 27.747 | 385.638 |
| SwiftShader / wood | 0.113 | 31.567 | 217.801 | 99.588 | 582.201 |

完整逐轮报告、逐通道比较指标及最小／中位／最大值见 [Metal 汇总](./performance/metal/summary.json)和[软件汇总](./performance/software/summary.json)；原始 stdout／stderr 文件位于各汇总旁。逐轮渲染调用包含管线准备、dispatch、回读及 RGBA8 转换；原生源码读取、原始文件写入、检查及 JSON 编码不计入。外部 CLI 计时包含进程生命周期、源码／图处理、GPU 获取／渲染、PNG 编码／文件写入及 JSON。这是两种有用但成本不同的消费流程，不是隔离的绑定速度比较。不声称使用 GPU 时间戳或单独测量 PNG 编码器耗时。

Metal 首次调用范围为陶瓷 26.1–214.4 ms、皮革 32.2–212.4 ms、木材 32.5–227.5 ms。各材质首轮约花费 180–195 ms 准备管线；这些样本保留在记录中。OS／驱动缓存未受控，因此新 renderer 不代表驱动无缓存。复用调用范围分别为 21.7–22.1、22.7–24.2、26.8–29.4 ms；软件木材为 98.1–100.4 ms。每次复用调用均记录零管线 miss，三种材质分别保留 4／4／6 个缓存项。小规模本地样本支持显式复用 renderer，不确立帧率承诺、UI deadline、取消能力或发布性能 SLO。

## 复现

从仓库根目录执行：

```bash
cargo xtask test-consumer
cargo xtask test-plan
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

对于本次记录的软件检出及加载器路径（按照 [GPU 指南](../../gpu-context.zh-CN.md)准备并验证固定版本）：

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

先构建二进制，再测量。脚本需要带 Pillow 和 NumPy 的 Python，仅用于输出检查；采集版本位于各汇总中。每个输出目录必须是新目录。

```bash
cargo build --release --locked --all-features -p mixture-cli
cargo build --release --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer
python3 docs/evidence/pr-011/measure_native.py \
  --repo "$PWD" \
  --native-binary "$PWD/target/native-consumer/release/mixture-native-consumer" \
  --cli-binary "$PWD/target/release/mixture" \
  --out "$PWD/tmp/pr-011-repro-metal" \
  --backend metal --expect-adapter 'Apple M5' --trials 3
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
python3 docs/evidence/pr-011/measure_native.py \
  --repo "$PWD" \
  --native-binary "$PWD/target/native-consumer/release/mixture-native-consumer" \
  --cli-binary "$PWD/target/release/mixture" \
  --out "$PWD/tmp/pr-011-repro-software" \
  --backend vulkan --software --expect-adapter SwiftShader \
  --trials 3 --materials wood
```

脚本将调用方输入复制到新输出目录，并从该目录调用两个二进制。原始 RGBA 文件、生成的 PNG 及输入副本保留在 `tmp/pr-011/performance-{metal,software}/`；持久归档保存原始报告流、哈希及比较结果，不重复存储这些 1K 图像。两种策略分别保存 GPU 冒烟快照，因为普通冒烟命令复用 `tmp/gpu-smoke/`。

**有意排除：** 软件包／发布验收、PR-012 CLI 变更、PR-013 设备丢失／OOM 变更、PR-014 调度、绑定、daemon／IPC、UI、新节点／shader、资源池，以及远端推送／CI。不声称完整 M4 或发布就绪。
