# M3 评审证据

[English](./README.md) | 简体中文

本目录支持[六问 M3 评审](../../m3-review.zh-CN.md)及 [M4 计划](../../../M4_PRS.zh-CN.md)，记录 PR-010 源码 `e9dd03bb272a29de6243aec79a3450b47b90530e` 的本地证据。本目录不实现 M4，也不关闭暂缓的远端 CI 验收。

| 证据 | 范围 |
|---|---|
| [acceptance.json](./acceptance.json) | 只读验证当前人工接受绑定、留存 1K 门禁及 2K 跟踪。 |
| [诊断探针](./diagnostics/README.zh-CN.md) | 十六个 CPU 公开 CLI 探针，原始 debug 和 release 采集；包含人类可读端口上下文缺陷。 |
| [独立 Cargo 探针](./native-consumer/README.zh-CN.md) | 执行 CPU 路径、编译公开 GPU 调用形态；GPU 执行及包消费仍未验证。 |
| [Metal 性能](./performance/metal/summary.json) | 五种工作负载各三次顺序 release 渲染，含输出裁剪和公开参数覆盖。 |
| [软件性能](./performance/software/summary.json) | 固定 SwiftShader 三次顺序 release 木材渲染。 |
| [Release 构建](./performance/release-build.log)、[工具链](./performance/toolchain.txt) | 实际构建输出、Rust 工具链及核对的 SwiftShader 源码版本。 |

## 性能方法与保留文件

[measure_cli.py](./measure_cli.py)在新建工作目录中调用已构建 CLI，使用复制的 `.mix` 源文件。它计时到每个进程退出，采集完整 stdout／stderr，检查所选适配器，然后使用 Pillow／NumPy 解码 PNG，与接受的基准比较。这些 Python 库只检查渲染输出，不执行图像素。脚本不修改任何基准或验收文件。

每种策略的 summary 记录二进制／辅助脚本／运行时源码／输入／契约／清单哈希、主机和库版本、准确命令／cwd、逐样本耗时、renderer 耗时、pass／缓存／分配度量及逐通道解码比较。引用的 `.stdout`／`.stderr` 原样保留在旁边。生成 PNG 留在忽略的 `tmp/` 中，保留其哈希和比较；接受的基准 PNG 仍位于 `fixtures/materials/`。可通过脚本重新生成候选以检查。本次评审未引入需要新观感决定的视觉变化。

留存可执行文件由未改变的 PR-010 运行时源码通过下列命令构建。脚本记录调用者声明的 release profile；任意二进制的来源／profile 无法从哈希推断。历史复现时应构建记录的源码版本，并核对保存的运行时哈希。没有丢弃较慢样本或预热。Mixture 缓存逐进程为空，OS／驱动缓存未受控。单机三个样本是描述性证据，不是统计性能门禁。`--version` 是最小调用参考，不是隔离的渲染启动时间。

这些计时 summary 的 `comparisons[].changedPixelRatio` 表示最大 RGBA 字节差异**大于材质 `pixelThreshold`** 的像素占比，即超过阈值的比例。它不是独立 golden 报告中同名字段表示的全部变化像素比例。如果所有差异均不超过阈值，`meanAbsolute` 为正而该比例为零是有效结果。原始采集与辅助脚本 schema 一起保留。

## 复现

使用具备 Pillow 和 NumPy 的 Python 3、仓库 Rust 工具链及显式适配器。从仓库根目录运行，选择尚不存在的输出路径：

```bash
cargo build --release --locked --all-features -p mixture-cli
python3 docs/reviews/m3/measure_cli.py \
  --repo . --binary target/release/mixture \
  --out tmp/m3-review/performance-metal-rerun \
  --backend metal --expect-adapter 'Apple M5'
```

在其他硬件主机上按实际情况修改预期适配器字符串及显式后端；脚本目前接受 Metal 或 Vulkan。预期字符串在 doctor 和 render 报告中均会核验。这些命令执行 GPU，不属于普通 `cargo xtask check`。

按照 [GPU 配置指南](../../gpu-context.zh-CN.md)准备固定版本 macOS loader，然后运行：

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/swiftshader/source" \
python3 docs/reviews/m3/measure_cli.py \
  --repo . --binary target/release/mixture \
  --out tmp/m3-review/performance-software-rerun \
  --backend vulkan --software --expect-adapter SwiftShader --wood-only
```

留存运行使用较早本地 loader `tmp/pr-004/swiftshader-build/bin`；环境保留在软件 summary 中。配置指南创建上例用于全新检出的路径。将软件结果视为同一基准策略前，须固定并核对文档中的 SwiftShader 版本；本脚本记录传入的源路径，不自行证明所加载驱动的构建来源。软件像素必须精确匹配，硬件像素须满足各材质未改变的声明容差。完整结构性材质门禁由 `acceptance.json` 单独审计，计时脚本不重新计算这些门禁。

脚本拒绝已有输出目录，在 CLI／适配器／元数据／像素不匹配时失败，并保留已有回执供检查。失败运行不算成功证据；重试应选择新目录。实测中位数及解释见[评审](../../m3-review.zh-CN.md#4-1k-性能是否支持交互式消费者)。

## 评审验证

[首次仓库检查](./checks/initial-repository-check.log)发现测试专用临时目录冲突：并行 golden 测试可能得到相同进程号加墙钟时间戳。本次在现有测试辅助代码中增加原子递增后缀，不改变产品代码、着色器、基准或分配行为。修正后的 [golden 专项测试](./checks/golden-tests.log)和[最终仓库检查](./checks/repository-check.log)均通过。原始日志和子进程输出流保留原有空白。

仓库检查覆盖格式、依赖边界、Clippy、CPU 测试、Rustdoc 和 105 份 Markdown 文件的链接；显式 GPU 证据来自上述 release 采集及单独审计的 PR-010 材质报告。独立消费者的 locked／offline CPU 运行和 49 对活跃语言文档另行检查。验收审计中的历史源码绑定指向测试辅助修正之前的 `e9dd03b`。
