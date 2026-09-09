# GPU 失败原因与上下文生命周期

[English](./gpu-failures.md) | 简体中文

PR-013 通过公开 Rust API 和现有诊断 JSON，使设备丢失与类型化 GPU 显存不足可被识别，并修复设备销毁后复现出的回读清理 panic。[独立消费者](../examples/native-consumer/tests/device_loss.rs)使用真实设备销毁验证公开契约。[本地证据](./evidence/pr-013/README.zh-CN.md)区分这些 GPU 运行与合成 OOM 分类测试。

## 公开契约

| API | 含义 |
|---|---|
| `GpuContext::device_loss()` | 借用首个已送达的 `DeviceLoss`；尚未记录通知时返回 `None`。该读取方法不轮询或阻塞。 |
| `DeviceLoss::reason` | 类型化的 `DeviceLossReason::Unknown` 或 `Destroyed`，直接从原生回调转换。 |
| `DeviceLoss::message` | 原始回调文本，包括显式销毁时的空消息。 |
| `GpuOperationError::reason()` | 首要原因：`GpuFailureReason::DeviceLost`、`OutOfMemory` 或 `Other`。 |
| `GpuOperationError::device_loss()` | 失败对象拥有的丢失证据，也包含与其他首要错误同时观察到的丢失；渲染器／上下文释放后仍可使用。 |
| `GpuOperationError::adapter()` | 错误来自执行器时，拥有所选适配器证据。 |
| `GpuOperationError::allocations()` | 清理后的逐次成功描述符分配计数；进入执行器前的失败不提供此报告。 |
| `GpuOperationError::diagnostic()` | 既有结构化错误码、阶段、上下文、标量证据及原始来源链。 |

原因枚举为非穷尽枚举。`Other` 包括验证／内部错误、超时、主机像素分配失败，以及没有首要类型化 OOM／丢失原因的其他操作错误。驱动字符串包含“out of memory”或“device lost”不会改变分类。

`reason()` 描述**首要失败**。考虑复用上下文前，还应检查 `device_loss()`：OOM 或映射错误可能仍为首要原因，而附带的丢失记录表明该上下文已经不可用。反过来，没有通知也不保证设备健康。

## 设备丢失生命周期

每个显式上下文拥有自己的 `Arc<OnceLock<DeviceLoss>>`。注册的回调仅持有该记录，不持有 GPU 句柄或指向上下文的反向引用。首个通知保留且不可清除。不添加全局状态、后台工作线程、自动重新获取、重试或替代执行器。

获取时的 `ContextReport` 仍为不可变快照。记录丢失不会改变其原始 `unverified` 判定；当前已送达的丢失证据由 `device_loss()` 提供。同样，先前 healthy 的 doctor 报告不保证设备未来健康。

渲染请求通过既有尺寸／设备限制检查后，执行器先检查已记录丢失，执行一次非阻塞原生 `Poll`，再在缓存查找或分配前检查一次。这会送达已有销毁回调，包括已测试的冷／热缓存场景。若丢失已经记录，则跳过原生轮询及新的 GPU 工作。输入／请求验证仍先于 GPU 工作。

执行器还在分配、提交后及回读边界，以及返回完成像素前检查丢失。已送达的丢失会阻止成功。观察到丢失时，执行器清除保留的流水线，并且不返回部分 `RenderOutput`。同一上下文上后续合法调用失败，新增描述符分配为零。既有 CPU 自有输出仍可使用。

原生通知的送达由后端驱动，可能需要轮询。在检查点之后到达的回调会在后续检查点或调用被观察到；读取方法不会同步查询驱动健康状态。保留的原始 `device()` 访问入口仍允许替换回调，这会禁用 Mixture 的丢失跟踪，不在本契约范围内。正常消费者应保留已注册的回调。参见固定版本的 [wgpu 回调 API](https://docs.rs/wgpu/30.0.1/wgpu/struct.Device.html#method.set_device_lost_callback)及[依赖暴露策略](./native-sdk.zh-CN.md)。

## 失败选择与证据保留

| 观察结果 | 首要诊断 |
|---|---|
| 继续工作前已记录丢失，且没有更早的操作错误 | `MIX_GPU_DEVICE_LOST`，阶段为该检查点的操作阶段。被阻止的新渲染使用 `gpuExecution`。 |
| 作用域捕获 `wgpu::Error::OutOfMemory` | `MIX_GPU_OUT_OF_MEMORY`，保留 shader／流水线／执行／回读阶段及操作消息。 |
| 附加丢失信息之前，其他操作已经失败 | 保留该操作的错误码、阶段、消息及来源；单独附加丢失记录。 |
| 映射／访问／转换已经失败，随后 unmap 清理也失败 | 保留首个失败；附加清理错误码／消息／驱动证据。 |
| 回读其他步骤成功，仅清理失败 | 返回清理错误，不返回成功像素。 |

三个原生错误作用域都会在等待结果之前弹出。错误作用域不提供跨过滤器的时间顺序。同一作用域操作产生多种错误时，PR-013 按 **OOM、验证、内部错误**顺序选择，保留其他捕获的驱动消息。没有 OOM 时，既有“验证先于内部错误”的顺序不变。这避免只报告次生的资源无效错误而遗漏分配失败原因。操作已经选定的错误，不会仅因后来出现丢失或清理失败而被替换。

[类型化 wgpu 错误](https://docs.rs/wgpu/30.0.1/wgpu/enum.Error.html)及其嵌套原生来源仍可通过 `Error::source()` 访问。所选作用域还记录 `driverMessage`，以及存在时的 `driverCause`。首要丢失错误以拥有的 `DeviceLoss` 为来源。原生来源对象不参与序列化。

新增 JSON 证据沿用仅支持标量的映射：

| 键 | 值 |
|---|---|
| `failureReason` | 对应首要错误码的 `deviceLost` 或 `outOfMemory`。 |
| `deviceLost` | 附加了已送达丢失时为布尔值 `true`，即使首要错误码是其他类型。 |
| `deviceLostReason`、`deviceLostMessage` | `unknown`／`destroyed` 及原始回调文本。 |
| `adapterName`、`backend` | 所选适配器名称及原生后端。 |
| `planHash` | 图渲染失败时的准确编译计划哈希。既有 pass／节点上下文在可用时保留。 |
| `allocationCumulativeBytes`、`allocationPeakBytes`、`allocationReleasedBytes`、`allocationLiveBytes` | 清理后的精确无符号描述符字节计数。 |
| `allocationTextureCount`、`allocationUniformCount`、`allocationStagingCount` | 成功记录的分配次数，使用精确无符号整数。 |
| `secondaryValidationMessage`、`secondaryInternalMessage` | 同一作用域操作捕获的其他错误，存在时提供。 |
| `cleanupCode`、`cleanupMessage`、`cleanupDriverMessage` | 次生清理失败证据，存在时提供。 |

因此，OOM 与丢失通知同时出现时，仍保留 `MIX_GPU_OUT_OF_MEMORY`／`failureReason: outOfMemory`，另外提供 `deviceLost: true` 及其独立原因／消息。映射错误随后伴随丢失时，保留 `MIX_READBACK_FAILED` 并附加相同的丢失字段。消费者无需解析驱动文字来做这两种判断。

## 资源与回读清理

成功和错误路径都会显式完成同一个逐次渲染资源守卫。纹理／uniform 被销毁，staging 守卫释放缓冲区，错误附带最终分配报告。一次回读已经完成后再失败，仍返回错误；资源释放，不返回部分像素结果。

回读释放所有映射视图后，总是在原生错误作用域内尝试 unmap，包括映射／轮询／访问／转换失败的路径。回归测试复现了在作用域之外 unmap 设备已销毁的缓冲区时发生的原生验证 panic；修复后返回原始映射错误，并把 unmap 失败记录为次生证据。清理不依赖上下文仍然可用。

计数覆盖 Mixture 保留的成功描述符批次。不完整失败批次的临时句柄会释放，不计为完成的分配。计数排除驱动分配粒度、流水线／绑定组开销及 CPU 像素／PNG 内存。`live_bytes == 0` 表示记录的资源已销毁，不表示物理内存立即归还。保留全部中间资源的调度方式及无资源池状态不变。

## JSON 与兼容性边界

两个诊断码是对非穷尽词汇表的有意扩展。已知的类型化 OOM／丢失现在使用这些具体错误码；普通操作错误保留既有错误码。不改变 `.mix`、节点或计划版本。CLI 外层报告结构、可选／null 规则及退出码不变：GPU 错误返回 `1`，不提供完成的图执行或 PNG 输出。已有获取上下文及计划证据保留。参见 [CLI 契约](./cli-contract.zh-CN.md)。

核心诊断解码器会拒绝未知错误码字符串。因此，严格的旧解码器需要更新词汇表才能消费这些新失败；这不承诺 pre-alpha 诊断词汇表可以互换。完整的包／发布兼容性仍属于 PR-015。

分类基于公开的类型化运行时信号。固定版本 wgpu 的 `RequestDeviceError` 不公开其类别；获取失败继续使用 `MIX_GPU_DEVICE_REQUEST_FAILED` 并保留原生证据。主机分配失败不会被标记为 GPU OOM。不添加私有 wgpu-core 依赖或驱动消息解析器。

## 验证与范围

```bash
cargo test --locked -p mixture-core --test diagnostics
cargo test --locked --all-features -p mixture-wgpu operation
cargo test --locked --all-features -p mixture-wgpu context
cargo test --locked --all-features -p mixture-cli
cargo xtask test-consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

按 [GPU 指南](./gpu-context.zh-CN.md)准备软件加载器。默认检查编译独立丢失测试，但不运行 GPU。显式冒烟使用新证据文件运行该测试，由 `native-consumer/status.json` 中的 `deviceLossEvidence` 引用；缺失、跳过或未完成的测试不能通过该门槛。

CPU 测试构造类型化的合成 OOM／内部／验证错误，在不耗尽内存的情况下检查分类、优先级、序列化及来源链。真实 GPU 测试销毁具有冷／热缓存的设备、拒绝重复渲染、保持另一上下文可用、在销毁后释放 staging，并在后期回读失败后释放全部已记录描述符。独立消费者仅使用公开 Rust 导入及自有源码输入。这些测试覆盖通知／生命周期行为，不覆盖自发驱动丢失或真实物理内存耗尽。

**范围之外：** 自动恢复、回退、新资源池、shader／像素改动、取消／过期结果调度、真实内存耗尽、打包消费、发布及已延期的远端平台 CI。
