# PR-012 CLI 契约证据

[English](./README.md) | 简体中文

**本地验收，2026-09-08：** 人类可读诊断保留文档、阶段／严重性、节点、端口及参数上下文。独立进程测试验证现有 CLI 报告、退出码、实际 PNG 和部分写入。实现版本为包含本记录的 PR-012 提交，父提交 `244b384aba257ab62ab225385ad6af1960775517`，分支 `codex/pr-012-cli-contract`。

[CLI 契约](../../cli-contract.zh-CN.md)记录已审查行为。产品改动仅涉及共享人类诊断格式化及五个调用方；参数解析、JSON 序列化、核心／GPU 语义、shader 及产品依赖不变。独立消费者添加仅开发时使用的 PNG 解码器，版本与产品锁文件已有版本一致，不添加像素执行器或生产绑定。

## 检查

| 验证 | 证据 |
|---|---|
| 修复前复现上下文遗漏 | [修复前日志](./checks/context-before.log)：四项回归按预期失败，包括遗漏 warp 端口及渲染覆盖参数；JSON 上下文当时已通过。 |
| 修复相同回归 | [修复后日志](./checks/context-after.log)：五项全部通过。 |
| 既有 CLI 测试 | [CLI 测试](./checks/cli-tests.log)通过；GPU 测试在此仍显式忽略。 |
| 工具保护检查 | [消费者工具测试](./checks/xtask-consumer-tests.log)拒绝跳过／未完成记录及错误退出码。 |
| 独立 CPU 消费 | [命令](./checks/test-consumer.log)、[测试输出](./checks/consumer/cli-tests.stdout.log)、[完成状态](./checks/consumer/status.json)：32 次子 CLI 调用通过，公开 Rust 的五项单元测试及三项进程测试也通过。 |
| 最终仓库检查 | [check.log](./checks/check.log)通过，包括 CPU 消费者检查及本地文档链接。 |
| 双语文档 | [配对检查](./checks/doc-pairs.json)：54 组英中配对，没有缺失译文或可执行代码块不一致。 |
| Metal GPU 冒烟 | [命令](./gpu-smoke/metal/command.log)、[CLI 测试输出](./gpu-smoke/metal/consumer/cli-tests.stdout.log)、[完成状态](./gpu-smoke/metal/consumer/status.json)：Apple M5、Metal；10 次独立 CLI 调用通过。 |
| 软件 GPU 冒烟 | [命令](./gpu-smoke/software/command.log)、[CLI 测试输出](./gpu-smoke/software/consumer/cli-tests.stdout.log)、[完成状态](./gpu-smoke/software/consumer/status.json)：SwiftShader Device (LLVM 10.0.0)、Vulkan；相同 10 次调用通过。 |

两种 GPU 运行还通过既有棋盘／图回归及 PR-011 公开 Rust 输出所有权探针。这里归档所选原始报告与测试日志；完整冒烟快照保留在 `tmp/pr-012/gpu-smoke-{metal,software}/`。软件源码在固定版本 `694585a05946e1ed49b6bd577ca6537cbb57f025` 上保持干净，使用 `tmp/pr-004/swiftshader-build/bin/` 下的已配置加载器。这是本地 macOS 证据，不是远端 Linux／Windows CI。

## 独立消费者检查内容

[测试程序](../../../examples/native-consumer/tests/cli_contract.rs)没有 Mixture Rust／私有导入。它嵌入自有源码，写入新用例目录，然后以该工作目录及显式路径／选项启动已构建 CLI。CPU 用例覆盖验证／检查成功、规范计划排序、覆盖／裁剪、人类及 JSON 缺失 warp 上下文、未知端口、格式错误输入、缺失文件、数值预算、非法覆盖、禁用后端获取及携带 `--json` 的用法错误。

每个 GPU 测试组检查 healthy／跳过探针的 doctor，再测试三种 65×3 渲染请求：`repeat=16` 的四通道、`repeat=4` 的四通道，以及仅粗糙度。它比较计划／适配器／来源／编码元数据，解码完成的 PNG，检查字面量颜色／不预乘 alpha／标量／法线值及覆盖改变后的像素，并验证执行／计数／耗时类型。不实现 CPU 像素算法。

部分写入测试使用 `normal.png` 目录阻止按规范顺序写入的第二个输出。JSON 和人类模式均返回 `1`，保留完成的 baseColor 文件并标明编码失败。JSON 仅列出该完成文件及字节长度，保留完整图执行证据。粗糙度／高度不存在，无关标记文件保留。这证明被测失败行为；文件导出不是原子的，失败的文件 I/O 可能留下部分目标文件。

[runs.json](./runs.json)定位所选 CLI 捕获目录。其状态包含准确参数／退出码及原始输出流文件名；自有输入和实际 PNG 保留在旁边。原始绝对工作目录及 stderr／OS 字符串保持不变。[capture-manifest.json](./capture-manifest.json)记录最终源码及归档哈希；时间戳和二进制哈希是采集证据，不是嵌入式构建证明。

## JSON 兼容性

使用未改动的 [M3 诊断脚本](../../reviews/m3/diagnostics/reproduce.py)对新 debug／all-features CLI 复跑。[复跑结果](./diagnostic-replay/summary.json)完成全部 16 项探针，退出码符合预期，二进制／夹具不变，所有缺失端口观察项现在均为 true。[比较结果](./json-compatibility.json)显示九份 JSON stdout 与历史 M3 release 捕获逐字节一致。第十份仅有刻意生成的缺失文件 `documentPath` 不同，其余数据一致。原始捕获未做规范化。

`validate` 保留原有无版本字段的 `{ok, diagnostics}` 结构。其余已审查命令保持 `schemaVersion: 1`；null／省略规则及退出码不变。独立 GPU 测试断言语义字段和有限耗时，不对易变报告做逐字节比较。

## 复现

```bash
cargo xtask test-consumer
cargo test --locked --all-features -p mixture-cli
cargo test --locked -p xtask consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

按照 [GPU 指南](../../gpu-context.zh-CN.md)准备／验证软件加载器。单独运行独立测试的命令见[消费者指南](../../../examples/native-consumer/README.zh-CN.md)。普通检查无需 GPU；完成保护防止跳过的测试表现为成功。

**排除范围：** JSON／版本／退出码策略重设计、事务性导出、设备丢失／OOM 分类（PR-013）、过期结果调度（PR-014）、软件包消费及发布策略（PR-015）、发布、远端推送／CI、绑定和服务。暂缓的远端验收保持开放。
