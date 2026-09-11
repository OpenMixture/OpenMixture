# PR-014 过期结果与保留边界证据

[English](./README.md) | 简体中文

**本地验收，2026-09-11：**独立消费者只发布最新代次，将工作限制为一个活跃／一个可替换待执行请求，最多保留展示 CPU 输出加正在处理的完成输出。实现版本为包含本记录的 `codex/pr-014-stale-results` 分支 PR-014 提交，父提交为 `6246647`。

[契约](../../stale-results.zh-CN.md)说明状态转换、确定性驱动、目录清理及内存限制。产品运行时代码、shader、节点／格式／计划语义、依赖和锁文件不变。现有独立示例内的库 target 在可执行程序与测试之间共享应用状态，并非新增产品包。

## 检查与观察

| 验证 | 证据 |
|---|---|
| CPU 状态与 CLI 兼容性 | [消费者命令](./checks/test-consumer.log)；最终[消费者测试输出](./checks/consumer/tests.stdout.log)包含六个新增状态／生命周期测试及既有检查。 |
| 拒绝跳过／未完成证据 | [工具测试](./checks/tooling-tests.log)拒绝错误发布代次、泄漏输出和缺失 CLI 用例。 |
| 专项缓存序列 | [Metal 日志](./checks/cache-metal.log)：全部九内核、22 个变化用例且各重复一次，显式 clear、设备销毁及独立 renderer。 |
| 真实 Rust 消费者 | [Metal](./gpu-smoke/metal/consumer/latest.stdout.log)、[软件](./gpu-smoke/software/consumer/latest.stdout.log)：启动 `[1,2,4,5]`，发布 `[1,5]`，请求 3 被替换，请求 4 编译失败。 |
| Metal 完整 smoke | [命令](./gpu-smoke/metal/command.log)、[doctor](./gpu-smoke/metal/doctor.json)、[GPU 测试](./gpu-smoke/metal/gpu-tests.stdout.log)、[缓存报告](./gpu-smoke/metal/cache-bound.json)、[完成状态](./gpu-smoke/metal/consumer/status.json)。 |
| 固定软件完整 smoke | [命令](./gpu-smoke/software/command.log)、[doctor](./gpu-smoke/software/doctor.json)、[GPU 测试](./gpu-smoke/software/gpu-tests.stdout.log)、[缓存报告](./gpu-smoke/software/cache-bound.json)、[完成状态](./gpu-smoke/software/consumer/status.json)。 |
| 最终仓库检查 | [check.log](./checks/check.log)：格式、依赖边界、严格 Clippy、测试、独立消费者、Rustdoc 和本地链接。 |
| 双语文档 | [配对检查](./checks/doc-pairs.json)；可执行／配置代码块一致，正文另行审查。 |

两次完整 smoke 分别在 Apple M5／Metal 和 SwiftShader Device (LLVM 10.0.0)／Vulkan 上通过。软件源码在固定版本 `694585a05946e1ed49b6bd577ca6537cbb57f025` 上保持干净，loader 位于 `tmp/pr-004/swiftshader-build/bin`。既有 checker／图回归、PR-011 所有权、PR-012 十次 CLI 调用检查及 PR-013 销毁／清理检查继续纳入。这是本地 macOS 证据；远端平台 CI 仍暂缓。

新增 Rust 驱动在 renderer drop 后检查字面像素与元数据。过期成功被释放，被替换的待执行覆盖从未编译／渲染；非法最新覆盖使初始展示保持明确过期，直到代次 5 成功。最终四通道 65×3 输出保留 3,120 像素字节；双输出边界是通过 drop 计数测试验证的结构边界，不是 OS／VRAM 测量。报告保留有界序列的固定规模证据，不保留历史像素缓冲区。

每个新增 CLI 序列执行五次调用：初始成功、过期成功、非法覆盖（退出 `2`）、部分写入（退出 `1`）及最终成功。它验证代次隔离、旧展示不变、最终解码法线字节、变化的棋盘 PNG、代次 1–4 清理及无关数据保留。最终只保留 `generation-5`。已目视检查最终 65×3 baseColor；未修改 shader 或基准像素。

[runs.json](./runs.json)通过 `latestEvidence` 定位各次实际捕获。回执包含原始子进程报告，最终 PNG 和自有输入保留在旁。过期／部分 PNG 被测试中的清理操作有意删除，因此原始报告路径不再存在。原始绝对路径和 stdout／stderr 均保留，同时归档既有十用例 CLI 与设备丢失回执。

缓存序列恰好达到九项，每份报告的存活／复用描述符字节均为零。重复请求的分配计数和像素一致，没有新管线 miss。clear 后项数为零，随后一次渲染 miss 一次。销毁该 renderer 的设备后，下次调用返回零分配失败并清空缓存。另行显式获取的 renderer 仍产生一致像素；第一个 renderer drop 后输出数据仍可访问。既有后期 readback 失败回归验证分配后的清理，补充本次前置丢失用例。

## 复现与限制

使用[确切命令](../../stale-results.zh-CN.md)与[独立示例指南](../../../examples/native-consumer/README.zh-CN.md)。`cargo xtask test-consumer` 运行确定性 CPU 转换，显式 `gpu-smoke` 运行两个真实消费者及 GPU 序列，最后 `cargo xtask check` 验证仓库门槛。新增测试不依赖 sleep 或时间竞争。

最终仓库检查后重新核对了 [smoke 前源码哈希](./checks/gpu-source-hashes.json)。[capture-manifest.json](./capture-manifest.json)记录源码／构建身份与归档哈希，自身除外。这些是捕获证据，不是嵌入式构建证明。

已提交的 GPU 工作可能完成。不包含硬中断、后台运行时、全局状态、资源池、最后使用者优化器、UI、daemon、包验证或远端发布操作。CPU 保留、GPU 描述符预算与磁盘清理是不同边界；忽略清理错误或宿主额外持有副本会破坏示例边界。PR-015 仍是下一步，负责软件包消费及兼容性／发布文档；不声称已可发布。
