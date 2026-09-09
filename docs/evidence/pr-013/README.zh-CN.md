# PR-013 GPU 失败证据

[English](./README.md) | 简体中文

**本地验收，2026-09-09：**设备丢失与类型化执行 OOM 有独立结构化错误码；上下文自有丢失跟踪、缓存失效及受作用域保护的 readback 清理保留原始失败。实现版本为包含本记录的 `codex/pr-013-gpu-failures` 提交，父提交为 `8ce6c0a75eec7af14e5080519ad110802e11a5dd`。

[失败契约](../../gpu-failures.zh-CN.md)说明通知时机、公开访问器、分配证据与兼容性。未修改 shader、节点、文档／计划格式、依赖或锁文件。CLI 外层结构和退出码不变；严格诊断码解码器需要支持两个新增变体。

## 验证

| 检查 | 证据 |
|---|---|
| 修复前冷／热缓存丢失 | [修复前](./checks/device-loss-before.log)：两个预期回归均因返回通用执行错误而失败。 |
| 修复前清理缺陷 | [修复前](./checks/readback-loss-before.log)：对已销毁 staging buffer 执行 unmap 触发未捕获的验证 panic。[后续 doctor](./checks/doctor-after-readback-failure.json)验证新上下文。 |
| 类型化 OOM、首个错误与 source chain | [操作测试](./checks/operation-tests.log)；OOM 为合成错误，并非物理耗尽。 |
| 诊断词汇与工具守卫 | [core](./checks/core-diagnostics.log)、[消费者工具](./checks/consumer-tooling.log)。 |
| CPU 消费与 CLI 兼容性 | [消费者](./checks/test-consumer.log)、[CLI](./checks/cli-tests.log)。 |
| 修复后的销毁／readback 回归 | [设备丢失](./checks/device-loss-metal.log)、[readback](./checks/readback-metal.log)。 |
| Metal 完整 GPU smoke | [命令](./gpu-smoke/metal/command.log)、[GPU 测试](./gpu-smoke/metal/gpu-tests.stdout.log)、[doctor](./gpu-smoke/metal/doctor.json)、[消费者完成状态](./gpu-smoke/metal/consumer/status.json)。 |
| 固定软件完整 GPU smoke | [命令](./gpu-smoke/software/command.log)、[GPU 测试](./gpu-smoke/software/gpu-tests.stdout.log)、[doctor](./gpu-smoke/software/doctor.json)、[消费者完成状态](./gpu-smoke/software/consumer/status.json)。 |
| 最终仓库检查 | [check.log](./checks/check.log)。 |
| 双语可执行代码块 | [配对检查](./checks/doc-pairs.json)；正文另行审查。 |

两次完整 smoke 分别在 Apple M5／Metal 和 SwiftShader Device (LLVM 10.0.0)／Vulkan 上通过。软件源码在固定版本 `694585a05946e1ed49b6bd577ca6537cbb57f025` 上保持干净，loader 位于 `tmp/pr-004/swiftshader-build/bin`。每次运行包含既有 GPU 回归、公开 Rust 输出所有权、十个 CLI GPU 用例及独立设备丢失契约。本地 macOS 执行不满足暂缓的远端平台 CI 门槛。

独立消费者销毁冷／热缓存设备，每例验证两次相同重复失败、新分配为零、管线缓存为空，在 renderer drop 后保留诊断，并消费另一个存活上下文的正确像素。各完成状态的 `deviceLossEvidence` 指向新建双用例 JSON 回执。staging 销毁回归保留初始映射错误，将受保护 unmap 失败报告为次要证据；分配报告显示存活描述符字节为零。另一个后期 readback 回归验证已完成工作后的释放及随后健康复用。

[runs.json](./runs.json)定位选定的本次捕获。原始流、原始绝对路径、输入与 PNG 均保留。[capture-manifest.json](./capture-manifest.json)列出源码和归档字节；[smoke 前源码哈希](./checks/gpu-source-hashes.json)与最终源码集合一致。这些哈希记录本地捕获身份，并非嵌入式构建证明。

## 复现与限制

使用[失败指南](../../gpu-failures.zh-CN.md)和[独立消费者](../../../examples/native-consumer/README.zh-CN.md)中的确切 CPU 与适配器命令。运行 `cargo xtask check` 完成最终 CPU／文档门槛。按 [GPU 指南](../../gpu-context.zh-CN.md)准备固定 loader。

类型化合成 OOM 测试覆盖各操作阶段分类、序列化、次要错误作用域与 source chain。真实 GPU 测试使用显式设备销毁，不能证明自发驱动丢失或物理耗尽行为。若公开依赖未暴露类型化原因，获取错误仍为 `MIX_GPU_DEVICE_REQUEST_FAILED`。不通过错误字符串推断分类。

范围外：自动恢复、备用执行、资源池、取消／过期结果调度、打包消费、发布及远端 CI。PR-014 负责过期结果与有界输出保留。
