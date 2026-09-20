# 最新请求与有界消费者状态

[English](./stale-results.md) | 简体中文

PR-014 在[独立原生消费者](../examples/native-consumer/README.zh-CN.md)中添加应用自有调度，提供可运行的 `latest` 验证模式。产品 core、renderer 和 CLI 行为不变。[本地验收记录](./evidence/pr-014/README.zh-CN.md)包含 CPU 状态转换测试、真实 Rust／CLI 输出和九内核缓存序列。

## 所有权与状态转换

[Latest](../examples/native-consumer/src/latest.rs) 是现有消费者包中的示例库 target，并非新增产品 crate 或公开 Mixture SDK API。宿主拥有一个 renderer 及 worker／事件循环。状态最多持有一个活跃代次、一个可替换待执行请求、一个展示输出和一个最新失败。它不启动 worker、不取消 future，也不克隆输出像素。

| 事件 | 效果 |
|---|---|
| `request(request)` | 检查溢出的单调代次递增，替换并释放先前待执行工作。已有展示立即过期，先前失败被清除。应在编译前登记，使失败的新请求也能取代旧工作。 |
| `start()` | 将待执行输入移交调用者并标为活跃。有活跃请求时返回 none，不启动并发渲染。 |
| 活跃完成，仍为最新 | 成功输出替换展示，返回旧输出供立即释放。失败保留旧展示并明确标为过期，记录最新失败。 |
| 活跃完成，已被取代 | 清除活跃槽，返回过期成功输出供释放；过期错误不会成为最新失败。 |
| 未知、待执行或重复完成 | 不清除活跃工作、不替换展示、不修改最新失败；如提供输出，则返回供释放。 |
| `displayed()` | 返回展示代次、借用输出及新鲜度布尔值。计划哈希是语义身份，不是请求新鲜度。 |

代次溢出返回错误且不修改既有状态。调用者必须立即 drop 返回的过期 CPU 输出，或显式清理其自有目录。`take_displayed()` 允许宿主释放保留的展示。额外复制或保存已完成结果历史的消费者不在此边界内。

`latest` 模式使用确定性事件交付：启动请求 2，在完成 2 前注入请求 3、4，丢弃 2，跳过被替换的请求 3，拒绝非法请求 4，然后发布请求 5。发布代次为 `[1,5]`，实际启动代次为 `[1,2,4,5]`。它在 renderer drop 后检查真实像素。这是调度状态的可复现驱动，不是交互服务或并行 GPU 执行演示。需要响应性的应用应提供显式管理的 worker，并向该状态交付事件；Mixture 原生 polling 可能阻塞该 worker。

## CLI 目录所有权

[latest_cli.rs](../examples/native-consumer/tests/latest_cli.rs) 使用预构建 CLI 和消费者自有输入验证同一状态。每个已启动代次独占新建 `generation-N` 目录，绝不接管或覆盖既有目录。发布是在消费者状态中选择当前完成目录，不将其重命名为共享输出目标，也不声称原子文件导出。

五次 CLI 调用覆盖初始成功、过期成功、较新非法覆盖（退出 `2`）、PNG 部分写入失败（退出 `1`）和最终成功。过期成功、失败／部分目录及被替换展示均显式删除。只保留代次 5，无关文件保持不变。被替换的待执行请求不分配文件。测试比较完成的 PNG、解码最终中性法线，验证旧展示在替换前保持不变，并拒绝较新失败后的过期发布。

`OutputDirectory::remove` 返回 I/O 错误，不通过析构函数静默隐藏清理失败。示例假设新建目录树由其独占。清理失败时，调用者必须报告错误，解决遗留输出前停止接收更多工作；忽略失败继续执行会破坏磁盘边界。此时测试工具失败并保留未完成回执。

## 内存与缓存边界

保留的 CPU 像素最多为展示输出，加上替换期间的当前完成输出。对于 `W × H` 尺寸和 `C` 个通道，紧密输出像素占用 `4 × W × H × C` 字节。不同请求的总量为 `B_display + B_completion`；65×3 四通道探针最多 6,240 字节，完成处理后回到 3,120 字节。报告的 `maximumCoexistingOutputs: 2` 是经 drop 计数测试检查的消费者结构边界，不是 OS 内存实测。

逐次 `SafetyLimits::transient_bytes` 和 `AllocationReport` 描述 GPU 描述符，不包含上述 CPU 聚合保留。宿主还需预算输入／计划、readback 转换、PNG 编码、分配器开销、驱动及管线状态。待执行输入按数量有界；若接受任意数据，宿主还必须限制其字节数。本示例只为有界自有输入排队整数覆盖值。只存在一个活跃 renderer 调用，没有输出历史、纹理池或隐藏全局缓存。

专项 [GPU 回归](../crates/mixture-wgpu/tests/nodes.rs) 对十二种节点契约各取前两个可用用例，以 33×3 渲染并重复验证像素一致和逐次计数，覆盖全部十种像素内核。缓存项等于已遇到的不同内核数，始终不超过十，在不同 renderer 间独立，并可显式清除或在观察到丢失后清除。drop 释放所有权；成功／失败后的描述符报告均为 `liveBytes == 0`，复用仍为零。这些是所有权／计数断言，不是物理 VRAM 立即回收的测量。既有后期 readback 失败测试继续纳入 smoke。

## 验证

```bash
cargo xtask test-consumer
cargo test --locked -p xtask consumer
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo test --locked --all-features \
  -p mixture-wgpu --test nodes \
  graph_gpu_cache_is_bounded_across_all_kernels_and_request_changes \
  -- --ignored --exact --nocapture
cargo run --locked --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- latest metal hardware 'Apple M5'
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

按 [GPU 指南](./gpu-context.zh-CN.md)准备固定 loader。CPU 检查运行六个状态／生命周期测试，编译忽略的 CLI 测试。显式 smoke 运行 Rust `latest` 及五次调用的 CLI 序列；`native-consumer/status.json` 的 `latestEvidence` 引用新建 CLI 回执，Rust 证据位于 `latest.stdout.log`。缺失／跳过／未完成回执均不能通过门槛。不需要 sleep 或时间竞争。

已经提交的 GPU 工作可能继续完成。drop 渲染 future 或达到逐次 poll 超时不承诺取消或整体 deadline。硬中断、运行时任务框架、daemon／IPC、UI 控件、资源池、包消费及暂缓的远端 CI 均在范围外。PR-015 现已提供[包验证](./package-consumption.zh-CN.md)及[兼容性／发布文档](./release.zh-CN.md)。
