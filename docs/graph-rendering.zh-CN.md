# 十一节点材质图渲染

[English](./graph-rendering.md) | 简体中文

本地 `.mix → validate → RenderPlan → wgpu → PNG` 路径执行十一个内置契约。PR-007 建立六个 M2 契约，PR-009 添加噪声／颜色／法线 kernel，PR-010 添加标量变换／扭曲。[Renderer](../crates/mixture-wgpu/src/executor.rs)只接受编译器生成的不可变计划，持有一个显式获取的上下文及小型管线缓存。GPU crate 不解析文档、不解析节点默认值、不解释参数覆盖。[CLI](../crates/mixture-cli/src/commands/render.rs)负责文件 I/O、参数、PNG 编码和报告。

## 运行示例

```bash
cargo run --locked -p mixture-cli -- render examples/checker.mix --size 256 --out ./tmp/checker
cargo run --locked -p mixture-cli -- render examples/levels.mix --size 256 \
  --output baseColor,roughness --set 'gamma=2' --out ./tmp/levels --json
cargo run --locked -p mixture-cli -- render examples/blend.mix --size 256 \
  --output baseColor,roughness,normal --set 'frequency=16' --out ./tmp/blend --json
cargo run --locked -p mixture-cli -- render examples/blend.mix \
  --output roughness --out ./tmp/roughness --backend metal --json
```

[checker.mix](../examples/checker.mix)输出棋盘格底色。[levels.mix](../examples/levels.mix)将常量标量重映射为粗糙度，并提供白色底色。[blend.mix](../examples/blend.mix)连接全部六个契约：棋盘格与色调通过 levels 调整的蒙版相乘，标量同时输出粗糙度。这些示例用于展示执行能力，不是真实感黄金材质。

`render <file.mix> --out <directory>` 默认使用 64×64 和仅 `baseColor`。`--size`、`--output` 及可重复的 `--set` 与 [inspect](./render-plan.zh-CN.md)共享语法和核心语义。覆盖必须使用暴露的公开 ID，在获取 GPU 前检查 JSON 类型及跨参数约束。`--backend`、`--power-preference` 和 `--software` 使用显式[上下文策略](./gpu-context.zh-CN.md)。省略 GPU 选项时不读取冒烟测试环境变量。适配器不可用／禁用时失败，不切换执行后端。

请求文件按材质契约顺序精确命名为 `<channel>.png`。CLI 在渲染成功后创建目录，替换同名文件，保留其他文件。以 `-` 开头的路径可放在 `--` 后。源文件不被修改。输出按顺序写入，不是多文件事务：后续写入失败时，报告列出已完成的文件；失败写入可能留下部分文件。

退出码 `0` 表示全部请求 PNG 写入完成。用法／源文档／编译请求无效返回 `2`；输入 I/O、GPU 获取、执行／回读、PNG 及输出／报告 I/O 失败返回 `1`。用法错误写入 stderr，语义和运行失败使用所选人类可读／JSON 报告模式。无效源文件／请求不初始化 GPU，也不创建输出目录。

## 公共 Rust 使用方

```rust
use mixture_core::{CompileRequest, MaterialDocument, compile};
use mixture_wgpu::{GpuContext, GpuContextOptions, Renderer};

let request = CompileRequest::default();
let source = MaterialDocument::decode(source_bytes, &request.limits)?
    .into_validated(&request.limits)?;
let plan = compile(&source, &request)?;
let context = GpuContext::request(GpuContextOptions::default()).await?;
let mut renderer = Renderer::new(context);
let output = renderer.render(&plan).await?;
assert_eq!(&output.report().plan_hash, plan.hash());
for channel in output.channels() {
    let rgba8: &[u8] = channel.pixels();
}
renderer.clear_pipeline_cache();
```

[crate 文档测试](../crates/mixture-wgpu/src/lib.rs)编译异步公共使用示例，[GPU 集成测试](../crates/mixture-wgpu/tests/nodes.rs)实际执行。`RenderOutput` 持有 CPU 缓冲区，渲染器／上下文释放后输出仍有效。每个 `RenderedChannel` 包含通道 ID、逻辑类型、尺寸、传递编码及从 `PlanOutput` 复制的连接端点或显式默认值。Alpha 为直通形式，不做预乘。

## 唯一 kernel 路径

[穷尽映射](../crates/mixture-wgpu/src/kernels.rs)将每个 `KernelId` 配对到唯一的 WGSL 入口。Uniform 上传使用显式小端值与填充，不需要 unsafe 转换或生成式着色器语言。

| Kernel | WGSL 与绑定 | Uniform |
| --- | --- | --- |
| `constant` | [constant.wgsl](../crates/mixture-wgpu/shaders/nodes/constant.wgsl)；uniform 0，存储输出 1 | 打包 RGBA，16 字节 |
| `checker` | [checker.wgsl](../crates/mixture-wgpu/shaders/nodes/checker.wgsl)；uniform 0，存储输出 1 | 单元数及两种颜色，48 字节 |
| `levels` | [levels.wgsl](../crates/mixture-wgpu/shaders/nodes/levels.wgsl)；uniform 0，输出 1，标量输入 2 | 五个 f32 值及填充，32 字节 |
| `blend` | [blend.wgsl](../crates/mixture-wgpu/shaders/nodes/blend.wgsl)；uniform 0，输出 1，a／b／mask 输入 2／3／4 | 模式码、不透明度及填充，16 字节 |
| `fractalNoise` | [fractal-noise.wgsl](../crates/mixture-wgpu/shaders/nodes/fractal-noise.wgsl)；uniform 0，输出 1 | u32 种子／scale／octaves／basis，f32 persistence 及填充，32 字节 |
| `gradientMap` | [gradient-map.wgsl](../crates/mixture-wgpu/shaders/nodes/gradient-map.wgsl)；uniform 0，输出 1，标量输入 2 | 两个线性 RGBA 端点，32 字节 |
| `heightToNormal` | [height-to-normal.wgsl](../crates/mixture-wgpu/shaders/nodes/height-to-normal.wgsl)；uniform 0，输出 1，标量输入 2 | f32 强度及填充，16 字节 |
| `transform2d` | [transform-2d.wgsl](../crates/mixture-wgpu/shaders/nodes/transform-2d.wgsl)；uniform 0，输出 1，标量输入 2 | 两个 u32 缩放、u32 旋转次数、两个 f32 偏移及填充，32 字节 |
| `warp` | [warp.wgsl](../crates/mixture-wgpu/shaders/nodes/warp.wgsl)；uniform 0，输出 1，标量输入 2，标量位移场 3 | 两个 f32 强度及填充，16 字节 |

常量同时服务标量／颜色节点及编译器生成的可选默认值。`material-output` 回读映射资源，不增加着色器。逐点 kernel 在相同像素坐标以 `textureLoad` 读取输入；高度转法线读取环绕邻居，变换／扭曲执行四次读取的周期双线性采样。输出使用 `rgba16float` 存储。全部 kernel 使用 8×8 工作组，对不完整工作组进行边界检查。棋盘格尺寸来自输出纹理，频率／颜色来自类型化调用；噪声接收完整的显式 u32 种子。重采样 kernel 不引入额外随机运算。

[节点契约公式](./node-contracts.zh-CN.md)保持不变。Levels 在除法／pow 前处理输入端点，保持钳制语义，并避免在上下限之间没有可表示半精度输入的极小区间中计算未定义结果。Blend 使用 `opacity × mask` 执行 normal／multiply／screen RGB 插值，并独立插值 alpha。没有 source-over 合成、隐式预乘或 CPU 图求值器。

固定 `render-builtin checker` 和 `doctor` 探针现在调用与图渲染相同的分配／分发／回读引擎和棋盘格着色器。其 8×8 不透明黑白像素及基准不变。共享棋盘格 uniform 从 16 增至 48 字节，因此固定探针的逻辑分配估算与精确预算边界增加 32 字节（64×64 时为 65,584 字节）。PR-004 证据保留为历史记录。固定 API 使用调用内的临时管线缓存；持久缓存由 `Renderer` 持有。

## 缓存、生命周期与失败

一个渲染器最多保留九条管线，以 `KernelId` 为键。在同一渲染器内，设备、着色器 ABI／版本、局部工作组尺寸及存储格式固定，参数值和输出尺寸无需增加缓存键。只有管线创建成功后才填入缓存。`cached_pipeline_count()` 提供数量；`clear_pipeline_cache()` 及渲染器释放会释放保留句柄。每次渲染报告按 pass 统计命中／未命中，包括本次调用中较早 pass 建立的缓存复用。

[每次调用的资源](../crates/mixture-wgpu/src/resources.rs)实现 [PR-006 生命周期模型](./render-plan.zh-CN.md)：全部 pass 纹理及含填充的 uniform 保留至执行和回读结束。每个请求通道使用一个 staging 缓冲区，在下一通道前完成映射、解包、解除映射与销毁。别名通道共享生产者纹理，但仍独立回读。成功或失败都会释放调用内的全部缓冲区／纹理。没有纹理池、最后使用者优化、pass 融合或磁盘缓存。

分配前，执行器检查实际设备的纹理尺寸、最大缓冲区及工作组数量。图／请求安全上限已由编译器检查。GPU 着色器、管线、执行与回读使用配平的错误作用域和共享类型化诊断，保留原生错误来源。计划失败包含计划哈希，管线／分配失败在可用时标明 pass 和源节点。每次原生完成／映射等待限时 30 秒。耗时是 CPU 墙钟时间；峰值／累计字节是逻辑估算，不包含驱动／管线开销及 CPU／PNG 缓冲区。

## 输出编码与精度

源文档保留 f64，编译调用记录 f32，每个 pass 在 `rgba16float` 中存储 f16 分量。回读拒绝非有限或越界分量，去除 256 字节行填充。返回数据为紧密排列、左上角原点的 RGBA8。

| 逻辑类型 | RGBA8 转换 | PNG 元数据 |
| --- | --- | --- |
| Color（`baseColor`、`emissive`） | 线性 RGB → sRGB；alpha 保持线性；乘 255 后舍入 | sRGB intent |
| Scalar | 红分量复制到 RGB，alpha 不透明；乘 255 后舍入 | gAMA = 1.0，无 sRGB 块 |
| Normal | 编码 XYZ 保留于 RGB，alpha 不透明；乘 255 后舍入 | gAMA = 1.0，无 sRGB 块 |

回读传递函数在 `c ≤ 0.0031308` 时为 `12.92 × c`，否则为 `1.055 × c^(1/2.4) − 0.055`。这是输出编码，不是第二套材质执行器。仅 CLI 写入 PNG。法线 `[0.5, 0.5, 1]` 变为 `[128, 128, 255, 255]`；标量 `0.5` 变为 `[128, 128, 128, 255]`；线性颜色 `[0.25, 0.5, 0.75, 0.25]` 约为 `[137, 188, 225, 64]`。

针对浮点像素的哨点在 f32／f16／传递函数舍入后最多允许一个 RGBA8 码值误差。端点／默认法线／别名用例按指定使用零容差。现有黑白棋盘格对全部 16,384 字节进行精确比较。这些检查不承诺不同 GPU 的浮点输出普遍逐字节相同。

## 报告

JSON schema 版本 1 包含 `input`、`outputDirectory`、`planHash`、`context`、`execution`、`outputs`、`ok` 和 `diagnostics`。Context 记录实际选择并保留仅获取状态的 `unverified`；成功的 `execution` 报告才是渲染证据。源／编译失败时上下文与执行为 null；GPU 失败保留已获取上下文，编码／写入失败保留完成的执行及输出条目。

`execution` 包含实际适配器、计划哈希、尺寸、已执行 pass 数、管线缓存查找、分配估算、紧密原始回读字节、累计填充映射字节、返回 RGBA8 字节及分阶段耗时。`outputs` 只列出成功写入文件的通道／类型／来源／尺寸／编码／路径／字节数。相同语义请求的计划哈希与 `inspect` 一致，适配器名称、路径和耗时均不进入哈希。

## 验证与证据

```bash
cargo xtask shader-check
cargo xtask test-node constant-scalar
cargo xtask test-node constant-color
cargo xtask test-node checker
cargo xtask test-node levels
cargo xtask test-node blend
cargo xtask test-node material-output
cargo xtask gpu-smoke
cargo xtask test-plan
cargo xtask check
```

`shader-check` 无需 GPU 即可验证全部 WGSL 入口／工作组尺寸和 uniform 结构大小。`test-node` 先在无 GPU 环境验证[全部节点夹具](../fixtures/nodes/README.zh-CN.md)，包括无效覆盖／源文件，再精确运行指定节点的 GPU 用例。它使用与冒烟相同的 `MIXTURE_GPU_BACKEND`、`MIXTURE_GPU_SOFTWARE` 及可选预期适配器策略。报告位于 `tmp/node-tests/<backend>/<node>.json`。未知节点和无效策略在 Cargo／GPU 工作开始前失败。

`gpu-smoke` 保留固定棋盘格／doctor 基准验收，渲染三个图示例，将图棋盘格与同一基准比较，并运行所有忽略的库／CLI GPU 回归。覆盖全部十一个节点、非对齐尺寸、非平凡颜色／alpha、全部混合模式、levels 极值、默认值／别名、执行裁剪、缓存复用／清理、设备拒绝／恢复、无效着色器／管线／映射／设备路径、PNG 元数据、文件名及部分输出失败报告。证据保存在 `tmp/gpu-smoke/`，包括各节点用例报告。普通 `check`／工作区测试不初始化 GPU。

本地 [Apple M5／Metal](./evidence/pr-007-apple-m5.json) 和固定 [SwiftShader／Vulkan](./evidence/pr-007-swiftshader.json)通过验证。已查看三个 256×256 示例。固定棋盘格与受保护基准逐字节一致，未覆盖任何像素基准。没有变更依赖或锁文件版本。

已查看的 256×256 预览：[棋盘格](./evidence/pr-007-checker.png)、[levels 粗糙度](./evidence/pr-007-levels.png)和 [blend 底色](./evidence/pr-007-blend.png)。它们是证据图，不是新增测试基准。

[远端 GPU 任务](../.github/workflows/gpu-smoke.yml)现已覆盖图示例和全部节点，但远端 Linux／macOS／Windows 结果仍待运行。M2 已实现并经本地验证，不宣称关闭仍开放的远端里程碑验收。PR-008 现已添加[受保护基准工具与釉面陶瓷机器验收](./material-goldens.zh-CN.md)，陶瓷观感已接受，皮革观感已获用户接受，不为这些 M2 示例添加真实感质量宣称。

## PR-009 噪声到法线切片

三个新增穷尽 `KernelId` 映射分别使用 32 字节噪声 uniform、32 字节渐变映射 uniform 和 16 字节高度转法线 uniform。节点默认值仍属于核心。新 kernel 复用相同分发、纹理分配、回读和错误路径。[皮革材质](../fixtures/materials/leather/README.zh-CN.md)以五个 pass 渲染四个连接通道，1K 估算峰值存活字节为 50,331,792。未添加资源池或 2K 优化。三个新节点测试和完整冒烟路径均在显式 Metal 与固定软件 Vulkan 适配器运行。

## PR-010 周期标量重采样

`Transform2d` 和 `Warp` 是类型化计划调用，使用已有分发／资源／回读路径；[核心降级](../crates/mixture-core/src/compiler/lower.rs)拥有所有参数默认值与必填输入解析。`Transform2d` 消费一个 Scalar 资源；`Warp` 按源、位移场的绑定顺序消费两个 Scalar 资源。两者填充后的 uniform 布局分别为 32 和 16 字节，[ABI 测试](../crates/mixture-wgpu/src/kernels.rs)根据编译计划验证上传字节的精确值。

[节点契约](./node-contracts.zh-CN.md)定义 v 向下的像素中心 UV、先逆向旋转坐标实现顺时针视觉旋转再按源轴整数缩放、采样偏移的符号，以及环绕邻居的重复双线性插值。Warp 在输出纹素读取并钳制位移场，以 `0.5` 为中心，应用各轴独立带符号 UV 强度。默认变换和零强度／中性场扭曲通过直接读取实现精确恒等。整数缩放与四分之一圈旋转保持源周期；周期位移场同样保持该周期。未引入通用仿射矩阵、sampler／滤波选项、mip 链或额外像素执行器。

```bash
cargo xtask test-node transform-2d
cargo xtask test-node warp
```

两个节点测试集在固定 SwiftShader 通过，包含 17 个变换和 13 个扭曲[字面量 GPU 探针](../crates/mixture-wgpu/tests/support/resampling_probe.rs)、既有噪声的精确恒等比较、边界／无效输入夹具以及完整图因果／缓存检查。字面量探针验证 X／Y 环绕插值、旋转顺序、输出坐标位移场读取及 f16 恒等路径。节点测试继续使用显式适配器策略；报告和原始输出保存于 `tmp/node-tests/<backend>/`，并另存 `<node>-literal-probes.json` 证据。这验证节点语义；材质观感和里程碑验收使用独立的[材质验收](./material-goldens.zh-CN.md)。

PR-010 添加 `execution.allocations`，公开 API 为 `RenderReport.allocations: AllocationReport`。它记录成功创建的描述符：纹理／uniform 数量与字节数，staging 数量、累计字节和峰值存活字节，以及总累计、峰值、存活、已释放和已复用字节。当前成功渲染最终满足 `liveBytes = 0`、`releasedBytes = cumulativeBytes` 和 `reusedBytes = 0`。释放记录显式 `destroy` 调用，不代表驱动立即归还物理内存。驱动分配粒度、管线／绑定组开销及 CPU 像素／PNG 缓冲区不计入。这些观测不进入 `RenderPlan` 或其哈希。

[PR-012 CLI 契约](./cli-contract.zh-CN.md)统一记录报告字段类型、null／省略规则、退出码和部分文件写入行为，并提供独立 PNG／元数据／覆盖检查。像素执行及编码语义不变。

PR-013 在 GPU 工作前与返回输出前检查已送达的设备丢失。观察到丢失会清空 renderer 管线缓存；重复调用返回结构化失败，不重新获取设备。错误路径释放逐次描述符并附带分配证据，readback unmap 在错误作用域内运行，使清理失败不会替换原始错误。见 [GPU 失败契约](./gpu-failures.zh-CN.md)。

PR-014 通过全部九内核、变化请求、重复、clear／drop 及独立 renderer 验证既有缓存边界；renderer 实现不变。[消费者保留](./stale-results.zh-CN.md)另外限制 CPU 输出并拒绝过期结果。
