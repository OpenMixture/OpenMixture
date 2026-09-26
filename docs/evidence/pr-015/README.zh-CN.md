# PR-015 本地验收证据

[English](./README.md) | 简体中文

**存储更新（2026-09-25）：** 原始执行与验收结论不变。完整历史附件按[归档取回说明](../archives/README.zh-CN.md)获取；机器回执、原始哈希和捕获清单保持不变，须在恢复的完整快照中检查其原路径。当前目录保留关键记录与评审图像。

2026-09-11 在 macOS/aarch64 完成本地验收：真实 Cargo 包通过独立 CPU 与 GPU 消费验证。至此完成 M4 本地实现评估；远程平台门槛与发布仍未关闭。参见[包验证](../../package-consumption.zh-CN.md)、[兼容性](../../compatibility.zh-CN.md)和[发布评估](../../release.zh-CN.md)。

实现提交为首次引入本目录的提交，基于 `30a190a41b8d425c40976a5cde8924e1e2ba9cd4`，分支为 `codex/pr-015-package-consumer`。运行验证的是提交前的工作区实现。[环境](./environment.json)、[源文件标识](./checks/verified-source-hashes.json)、[基线标识](./checks/baseline-hashes.json)和[归档清单](./capture-manifest.json)记录输入与归档输出，不声称二进制内含提交来源证明。

## 结果

| 门槛 | 证据 |
|---|---|
| 最终仓库检查，包含 package-check | [完整日志](./checks/check.log) |
| 定向工具测试与严格 clippy | [测试](./checks/tooling-tests.log)、[clippy](./checks/tooling-clippy.log) |
| 独立源码消费者 CPU 契约 | [日志](./checks/test-consumer.log) |
| 真实包 CPU 验证 | [日志](./checks/package-check.log)、[状态](./packages/cpu/status.json) |
| Apple M5 / Metal GPU smoke 与包消费 | [Smoke 日志](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/pr-015/gpu-smoke/metal/command.log)、[包状态](./packages/metal/status.json) |
| 固定 SwiftShader / Vulkan GPU smoke 与包消费 | [Smoke 日志](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/pr-015/gpu-smoke/software/command.log)、[包状态](./packages/software/status.json) |
| 两种策略下三种材质的 1K 验证 | [运行索引](./runs.json)链接六份成功报告及产物。 |
| 排序选出的最大 2K 工作负载 | [Metal trace](./trace/metal/trace.json)、[软件 trace](./trace/software/trace.json) |

每次包验证保留三个 `.crate` 文件、规范化清单、解析元数据、隔离锁文件、文件/二进制哈希、命令参数、原始输出与消费者回执。验证器确认仅含版本要求的依赖全部解析到生产仓库之外的解包目录；执行包内单元测试/Rustdoc、32 个 CLI CPU 用例，并在每种 GPU 策略下检查 Rust 实际拥有的输出与十个 CLI GPU 用例。删除内嵌 constant shader 后必须编译失败；对应非零退出日志是预期的反例证据，之后恢复文件并检查哈希。成功运行的外部暂存目录已删除，记录中的原始路径不要求继续存在。

外层包验证器的 `packagedCratesValidated: true` 表示上述检查通过。应用自身继续保留 `false`，因为应用输出本身无法证明构建来源。源码消费者 smoke 还重跑设备丢失、过期结果与保留量用例，完整日志保留结果计数。设备丢失使用真实销毁的设备；OOM 分类仍由合成类型错误覆盖，不代表耗尽了物理内存。

陶瓷、皮革与木材在两种策略下均通过机器检查与 golden 比较。已查看默认接触表的外观与差异；已接受基线和人工评审记录保持不变。排序选出的最大工作负载为 wood/default、2048×2048、八个 pass，满足现有 512 MiB 描述符预算。Trace 反映描述符与生命周期，不是物理显存或 GPU 时间戳基准。产物副本保持原始字节，原始日志可能保留末尾空行。

## 复现与范围

执行[发布检查表](../../release.zh-CN.md#可复现验证)中的命令；[GPU 指南](../../gpu-context.zh-CN.md)说明本地加载器准备方式。CPU `cargo xtask check` 现包含包验证，不获取适配器。两种 GPU 策略均显式指定，不存在执行器回退。

范围包括包/源码资源验证、精确同伴版本、README/许可证入包及配套中英文兼容性/发布文档。版本仍为 `0.1.0` 且禁止发布；归档有意不含锁文件，使用单独的固定验证锁。没有更新锁文件、新增节点、修改 shader 语义或格式，也未引入产品 crate、运行时依赖、WebAssembly 或编辑器。原始文档与材质基线保持不变。不声称远程推送、CI 结果、合并、标签或分发已经完成。

完整历史捕获（含逐次运行目录与共享 2K 图片）可恢复为：

```bash
python scripts/evidence/restore.py early-runs tmp/retained-early
```
