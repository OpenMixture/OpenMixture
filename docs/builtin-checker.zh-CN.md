# 内置棋盘格计算与回读

[English](./builtin-checker.md) | 简体中文

PR-004 在图功能之前实现一次真实的无窗口计算 pass。它使用调用方的 `GpuContext`，写入离屏 `rgba16float` 纹理，复制到带行填充的回读缓冲区，返回由 CPU 持有的 RGBA8 像素。PNG 编码和文件 I/O 属于 CLI。固定探针不需要图模型、节点注册表、通用渲染器、资源池或第二套像素执行器。PR-005 单独添加[源文件验证与节点契约](./file-format.zh-CN.md)，不改变此探针。

## 运行与检查

```bash
mkdir -p tmp
cargo run --locked -p mixture-cli -- render-builtin checker --size 64 --out tmp/checker.png
cargo run --locked -p mixture-cli -- render-builtin checker --size 65 --out tmp/checker-65.png --json
cargo run --locked -p mixture-cli -- doctor --json
cargo run --locked -p mixture-cli -- doctor --skip-probe --json
```

唯一的内置名称是 `checker`。`--size` 默认 64，允许 1 至 2048。必须提供 `--out`，其父目录必须存在；已有输出文件会被替换。命令接受与 [doctor](./gpu-context.zh-CN.md) 相同的 `--backend`、`--power-preference` 和 `--software` 选择选项。未知名称／选项、重复选项或缺少值，会在访问 GPU 前失败。

人类可读输出包含实际适配器、尺寸、pass 数、回读字节数和 CPU 墙钟耗时。`--json` 报告包含 `schemaVersion: 1`、`request`、`output`、`writtenBytes`、获取时的 `context`、已完成的 `execution`，以及共享 `ok`／`diagnostics` 字段。PNG 写入成功之前，`writtenBytes` 为 null；后续 PNG 编码或 I/O 失败时，已经完成的 `execution` 仍保留。嵌套 context 是获取快照，保持 `unverified`；渲染报告通过独立的 `execution` 字段证明执行结果。

PNG 写入成功返回退出码 `0`；GPU／回读／编码／I/O 失败返回 `1`；调用、尺寸或预算无效返回 `2`。用法语法错误即使指定 `--json` 也写入 stderr；请求验证失败提供结构化 JSON。无效尺寸或 GPU 执行失败不会创建输出文件。PNG 先在内存中完整编码，再打开输出文件；随后发生的文件系统写入失败仍可能留下部分文件。

## 棋盘格 v1 契约

| 属性 | 契约 |
| --- | --- |
| 输入 | 正的输出宽高；CLI 使用正方形，Rust 允许矩形 |
| 图案 | 每轴八个格子；左上角为黑色，黑白交替且完全不透明 |
| 格子选择 | 整数 `floor(x * 8 / width)` 和 `floor(y * 8 / height)`；两者之和为奇数时为白色 |
| 采样 | 左上角像素原点，无过滤、无随机性；不能整除的尺寸会产生不等宽格子 |
| GPU 格式 | `rgba16float`，一个 mip、一个图层，存储写入与复制源用途 |
| 调度 | 工作组 `[8, 8, 1]`，组数向上取整，显式越界保护 |
| Uniform | PR-007 共享 ABI：`u32` 单元数／填充和两个 f32 RGBA 颜色，共 48 字节；尺寸来自纹理 |
| 输出 | 紧密排列的 RGBA8，按行存储，左上角原点；alpha 为 255 |
| 色彩空间 | 黑白 RGB 端点在线性空间和 sRGB 中相同；PNG 带有 sRGB 元数据 |
| 平铺 | 每轴八格形成周期图案；相对边缘像素可在预期棋盘边界上不同 |

唯一像素实现是 [checker.wgsl](../crates/mixture-wgpu/shaders/nodes/checker.wgsl)。小于八的尺寸可能对格子采样不足；1×1 为不透明黑色。没有 CPU 渲染器或通用颜色转换。后续彩色 kernel 必须定义自己的色彩空间转换契约。

## 公共 API 与限制

[CheckerRequest](../crates/mixture-wgpu/src/checker.rs) 包含宽和高。`validate(&SafetyLimits)` 在获取上下文前检查正尺寸、安全的整数格子运算、每轴输出上限、一个请求输出及估算的临时 GPU 字节数。默认每轴上限为 2048，临时字节上限为 512 MiB。渲染器还会在分配前检查已获取设备的纹理和缓冲区限制，不隐式提高任何请求上限。

```rust
use mixture_core::SafetyLimits;
use mixture_wgpu::{CheckerOutput, CheckerRequest, GpuContext, GpuContextOptions};

async fn checker() -> Result<CheckerOutput, Box<dyn std::error::Error>> {
    let request = CheckerRequest::default();
    let limits = SafetyLimits::default();
    request.validate(&limits)?;
    let mut context = GpuContext::request(GpuContextOptions::default()).await?;
    Ok(context.render_checker(request, &limits).await?)
}
```

`CheckerOutput::pixels()` 借用由 CPU 持有的图像；`report()` 提供实际适配器证据、尺寸、调度／pass 数、格式／编码、行步长、字节数、内存估算和阶段耗时。获取报告不可变。`GpuContext::probe_checker()` 执行并检查 64×64 棋盘格后返回新的 doctor 报告。

着色器与管线准备、执行、回读和总耗时均为 CPU 墙钟毫秒，不是 GPU 时间戳测量，也不参与像素比较。内存估算为逻辑纹理字节、带填充回读字节及 48 字节 uniform（PR-007 共享 ABI）之和，不含驱动分配和管线开销。

## 回读与错误行为

`rgba16float` 每像素占八字节。复制行步长为 `ceil(width * 8 / 256) * 256`，映射缓冲区大小为行步长 × 高度。回读检查字节数，去除行填充，解码半精度通道，拒绝 NaN／无穷大及 `[0, 1]` 之外的值，然后对每个通道 × 255 四舍五入。64×64 的原始纹理／回读为 32,768 字节，行步长 512，RGBA8 为 16,384 字节。65×3 的行步长为 768，映射缓冲区为 2,304 字节。

GPU 资源、着色器、管线、命令和映射请求使用验证、内部错误和内存不足错误作用域。等待结果前先弹出作用域，避免挂起时仍占用线程局部栈。着色器失败使用 `MIX_GPU_SHADER_VALIDATION_FAILED`／`gpuShader`；管线创建使用 `MIX_GPU_EXECUTION_FAILED`／`gpuPipeline`；分配、上传、调度或提交使用 `MIX_GPU_EXECUTION_FAILED`／`gpuExecution`；复制、映射、解码或探针像素失败使用 `MIX_READBACK_FAILED`／`readback`。PNG 和文件系统失败使用 `MIX_ENCODING_FAILED`／`encoding`。稳定错误码之下保留原生来源。

原生提交和映射等待各有 30 秒上限。成功和解码失败时，均先释放映射视图，再取消映射。成功调用显式销毁缓冲区／纹理；错误路径通过 Rust drop 释放持有的句柄。上下文可复用，返回的像素在上下文销毁后仍然有效。没有进程级全局资源或隐式恢复尝试。

## 基准、测试与证据

初始 [64×64 PNG](../fixtures/nodes/checker/checker-64.png) 和[原始 RGBA 基准](../fixtures/nodes/checker/checker-64.rgba)由 SwiftShader 提交 `694585a05946e1ed49b6bd577ca6537cbb57f025` 生成，经查看确认，并记录于[来源信息](../fixtures/nodes/checker/provenance.json)。它们包含 2048 个不透明黑色像素和 2048 个不透明白色像素。基准比较使用解码后的 RGBA 字节，不比较压缩 PNG 字节。

```bash
cargo test --locked -p mixture-wgpu checker
cargo test --locked -p mixture-wgpu readback
cargo test --locked -p mixture-cli
cargo xtask shader-check
cargo xtask check
cargo xtask gpu-smoke
```

普通测试无需 GPU，验证请求、布局算术、去除填充、半精度转换、通过 Naga 验证可移植 WGSL、PNG 往返和 CLI 失败。显式冒烟测试运行带探针及跳过探针的 doctor，渲染并解码真实 PNG，逐字节比较已审查基准，再运行默认忽略的 GPU 测试，覆盖非整齐／部分尺寸、重复渲染、上下文销毁、错误着色器／管线、已销毁设备、映射失败和输出 I/O 失败。失败不会转成跳过覆盖或新的基准。

[软件适配器准备与冒烟策略](./gpu-context.zh-CN.md)支持 Linux CI 和原生 macOS 复现。本地 [Apple M5／Metal](./evidence/pr-004-apple-m5.json) 与 [SwiftShader／Vulkan](./evidence/pr-004-swiftshader.json) 探针均通过，棋盘格 RGBA 字节完全一致。这只证明该夹具在已测试适配器上的结果，不代表浮点输出普遍一致。关闭里程碑前，仍需实际运行远端 Linux SwiftShader 和非 GPU 跨平台 CI 矩阵。PR-005 [严格 `.mix` 解码与验证](./file-format.zh-CN.md)已实现，PR-006 [计划编译](./render-plan.zh-CN.md)已实现，PR-007 [图执行](./graph-rendering.zh-CN.md)已实现，并共享固定探针分发路径，基准像素不变。
