# 独立 Rust 消费者

[English](./README.md) | 简体中文

本应用拥有自己的 Cargo workspace、锁文件和 [input.mix](./input.mix)，仅导入公开的 `mixture-core`、`mixture-wgpu`，以及用于报告的 `serde_json`、用于驱动原生 future 的 `pollster`。它没有直接 wgpu 依赖、私有导入或生产方拥有的运行时资源。两个 path 依赖定位公开源码 crate；打包 crate 消费仍属于 PR-015。

## CPU 检查

从仓库根目录运行：

```bash
cargo xtask test-consumer
```

该命令检查独立 workspace、格式、Clippy、五个单元测试、三个进程测试，并在无关工作目录执行实际 CPU 探针。完整 GPU 调用路径和原生 `Send` 约束会编译，但普通检查不获取 GPU。显式 `none` 获取测试执行 wgpu 初始化之前的公开提前失败路径。日志及完成状态写入 `tmp/consumer-check/`；每次运行前先将状态标为未完成。

等效应用调用为：

```bash
cargo run --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- check
```

[cpu.rs](./src/cpu.rs)演示解码、验证、公开覆盖、通道选择、确定性哈希、依赖裁剪和结构化诊断。`repeat=0` 标明 `pattern.cellsX` 及公开 ID `repeat`；重复请求通道会被拒绝，通道顺序则会规范化。源字节可确定性往返。所有夹具均属于本应用。

## 显式 GPU 检查

```bash
cargo run --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- gpu metal hardware 'Apple M5'
```

按目标主机替换后端／名称。后端支持 `auto`、`metal`、`vulkan`、`dx12` 及显式禁用的 `none`；策略为 `hardware` 或 `software`。`hardware` 表示普通 wgpu 适配器请求（`software_adapter=false`），不保证物理硬件。可选最后参数要求实际适配器名称包含该子串。应用不读取 `MIXTURE_GPU_*` 或 `WGPU_*` 适配器覆盖。

[gpu.rs](./src/gpu.rs)先以 `repeat=16`、再以 `repeat=4` 在 65×3 渲染，然后 drop 拥有唯一上下文的 renderer，再检查两个返回结果。检查消费四通道及元数据、字面预期的 sRGB／straight-alpha 颜色、已连接的线性粗糙度值、默认法线／高度像素、参数变化像素、计划／适配器／pass 报告，以及为零的逐次存活描述符字节。它不实现任何像素算法。

现有冒烟命令从无关工作目录调用同一可执行文件并验证报告：

```bash
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
```

使用固定 SwiftShader 时，先按 [GPU 指南](../../docs/gpu-context.zh-CN.md)准备 loader，然后运行：

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
cargo run --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer -- gpu vulkan software SwiftShader
```

## Release 测量

`measure` 接受调用者自己的 `.mix`，在 1024×1024 请求 `baseColor`、`normal`、`roughness`、`height`，执行首次及相同计划的复用 renderer 调用，分别报告编译、上下文获取和两次 render 调用墙钟时间。文件读写、输出检查及 JSON 编码不在这些计时内；不声称 GPU 时间戳或取消能力。两份自有 RGBA8 结果在 renderer 被 drop 后检查，并写入新目录以便独立比较。流程不经过 PNG 编码器。

```bash
cargo build --release --locked --all-features \
  --manifest-path examples/native-consumer/Cargo.toml \
  --target-dir target/native-consumer
target/native-consumer/release/mixture-native-consumer measure \
  examples/native-consumer/input.mix tmp/native-measure metal hardware 'Apple M5'
```

输出目录的父目录必须存在，输出目录自身必须不存在。测量输入最多读取默认字节限额加一个哨兵字节；实际限额诊断归 core 所有。不覆盖任何文件或基准。[PR-011 测量脚本](../../docs/evidence/pr-011/measure_native.py)将接受的材质与原始 Rust／解码 CLI 输出比较，所有哈希／比较都在计时后进行。

这是专项消费者示例，不是新的通用 CLI 契约。成功写入 JSON 报告并退出 `0`；操作或验证失败返回 `1`，在适用处保留原始 Mixture 诊断上下文／source chain；usage 失败退出 `2` 并写入 stderr。`RenderFailure` 将所选上下文与原始 GPU 操作错误一起保留。不提供备用执行或设备恢复。原生 `Renderer::render` 可能在 polling 期间阻塞调用线程；需要响应性的应用应管理自己的 worker。过期结果调度留给 PR-014。

所有权、依赖类型暴露及验收限制见[公开原生 API 指南](../../docs/native-sdk.zh-CN.md)。
