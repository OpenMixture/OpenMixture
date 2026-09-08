# 公开原生 Rust 消费路径

[English](./native-sdk.md) | 简体中文

PR-011 使用[独立应用](../examples/native-consumer/README.zh-CN.md)验证现有公开 Rust 路径，不为产品 crate 增加 renderer 门面、运行时 crate、节点、着色器、文档版本或依赖。项目仍为 pre-alpha：CLI 稳定化、设备丢失／OOM 分类、过期结果处理及包消费是后续 [M4 步骤](../M4_PRS.zh-CN.md)。

## 已审查的 API 路径

| 消费者操作 | 公开 API 与所有权 |
|---|---|
| 使用显式限额解码源文件 | `MaterialDocument::decode(bytes, &limits)` 返回源文档或 `DocumentError`，不访问 GPU。 |
| 验证图 | `into_validated(&limits)` 返回不可变 `ValidatedDocument`；验证不修复或填充源文档。 |
| 选择参数、尺寸及通道 | `CompileRequest`、`compile(&validated, &request)` 返回不可变 `RenderPlan` 或 `CompileError`。覆盖影响编译，不修改源文档。 |
| 获取请求设备 | `GpuContext::request(options).await` 拥有自己的 instance／adapter／device／queue，并返回请求／实际上下文结构化证据。 |
| 渲染 | `Renderer::new(context)` 消费上下文；`render(&plan).await` 返回自有 `RenderOutput` 或 `GpuOperationError`。 |
| 消费数据及报告 | `channels()`、`pixels()`、`report()` 借用结果拥有的 CPU 数据，其生命周期可长于 renderer／context。 |

Core 也公开 `BUILT_INS`、`node_contract`、节点／端口／参数类型和版本化默认值。消费者无需第二套目录、解析器、编译器或像素实现。[Core Rustdoc](../crates/mixture-core/src/lib.rs)现使用完整内联源示例，替代仓库相对文件，并准确描述十一种契约及已实现的验证／编译。

独立示例拥有输入与 Cargo 清单，仅使用公开 crate 导入，独立编译，并在生产方工作目录之外运行。其 path 依赖证明源码级 API 消费，**不证明**未来发布归档包含全部必要资源；该验收归 PR-015。历史 [M3 仅编译探针](./reviews/m3/native-consumer/README.zh-CN.md)作为较早证据保持不变。

## 数据及生命周期契约

请求可按任意顺序选择支持的通道；返回通道遵循材质契约顺序。重复通道、非法公开覆盖、格式错误图及超限请求在 GPU 获取前返回类型化诊断。路径、适配器标识和墙钟时间不参与计划哈希。消费者检查确定性序列化／哈希，并验证仅请求 roughness 会裁剪未使用的 checker 分支。

`RenderedChannel` 包含 `channel`、`kind`、`source`、`size`、`encoding`，以及紧密排列、左上起点、按行优先的 RGBA8 字节。来源保留已连接端点或版本化默认值。颜色 RGB 为 sRGB，alpha 为 straight、线性；标量及已经编码的切线空间法线使用线性字节。标量复制到 RGB 并使用不透明 alpha。应读取 encoding 和 kind，不根据文件名猜测。

65×3 消费者夹具有意区分以下情况：

| 通道 | 来源 | renderer 被 drop 后检查的字面字节 |
|---|---|---|
| baseColor | 已连接棋盘 | `[137,188,225,255]` 与 `[255,0,0,128]`，含 straight alpha |
| normal | 默认 | `[128,128,255,255]` |
| roughness | 已连接标量 0.25 | `[64,64,64,255]` |
| height | 默认 | `[0,0,0,255]` |

这些是应用自有输入的字面测试预期，没有增加 CPU 颜色转换或棋盘算法。两次公开 `repeat` 覆盖必须产生不同 baseColor 字节和不同计划哈希。两个结果均在 renderer 及其上下文被 drop 后检查，证明实际所有权，而不只是类型签名。

`RenderReport` 提供实际适配器、计划哈希、pass 数、尺寸、管线缓存 hits／misses／entries、分配估算／计数、传输字节及墙钟耗时。描述符计数不包含驱动／管线／绑定组和 CPU 输出开销。完成渲染时 `live_bytes == 0` 表示逐次描述符已销毁，不承诺 OS 立即回收内存。保留两份 CPU 结果会增加主机内存，与单次 GPU 预算独立。

## 显式 GPU 状态与依赖类型暴露

普通消费者需要 `GpuContextOptions`、`ContextReport`、`Renderer` 和结果／错误类型，无需直接依赖 `wgpu` 或使用原始句柄访问器。获取成功在运行棋盘健康探针前为 `unverified`；完成材质渲染不会将获取快照改写成 doctor 判定。

现有公开 `instance()`、`adapter()`、`device()`、`queue()` getter **保留**为高级 escape hatch。其签名暴露本 crate 使用的 wgpu 主版本，当前为 30，并非独立于该依赖的门面。`RequestedPolicy`、`AdapterDiagnostics`、`DeviceDiagnostics` 中的 `wgpu::Limits` 字段具有相同耦合。公开 serde trait 及 core 的 `serde_json::Value` 参数也暴露各自依赖类型。升级依赖若改变公开签名或序列化 limits 形状，必须明确审查兼容性、测试并同步文档，不能隐藏在未改名的 Mixture 方法背后。

共享 wgpu 句柄引用不代表 GPU 状态不可变：调用者可通过它们提交工作、销毁设备或修改回调。这些干预由调用者负责，可能使后续 Mixture 操作失败。外部提交和分配不属于 Mixture 报告的耗时／计数。普通消费者验收不认证任意原始 wgpu 互操作。PR-011 不删除现有访问器，也不新增全面稳定性承诺；完整发布／兼容性策略归 PR-015。

虽然 `Renderer::render` 返回 future，且其原生类型满足 `Send`，原生 polling 可能阻塞执行线程。需要响应性的应用应在显式管理的 worker 上拥有 renderer。各次等待限制为 30 秒，并非整体渲染 deadline。drop future 不是 GPU 取消契约。本示例不引入线程、恢复、隐式备用执行或过期结果调度器。

## 诊断与验证

`DocumentError::report`、`CompileError::report`、`GpuContextError::report`／`diagnostic` 和 `GpuOperationError::diagnostic` 保留 Mixture 结构化错误。示例的渲染错误包装保留所选上下文和原始 source chain，不替换首个 GPU 错误。CPU 探针确认非法 `repeat=0` 标明 `pattern`、`cellsX` 和公开 ID `repeat`；进程测试覆盖不创建 wgpu 状态的禁用后端获取。

```bash
cargo xtask test-consumer
cargo xtask test-plan
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

`test-consumer` 仅使用 CPU，并纳入 `check`。`gpu-smoke` 在现有棋盘、图和 GPU 回归之外，显式构建并运行同一独立应用。其环境策略转换为显式消费者参数；独立消费者命令不隐式读取该策略。[示例指南](../examples/native-consumer/README.zh-CN.md)提供直接及固定软件命令。[PR-011 证据](./evidence/pr-011/README.zh-CN.md)记录本地结果与成对 release 测量；暂缓的远端 CI 仍开放。
