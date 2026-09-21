# 确定性 RenderPlan 编译

[English](./render-plan.md) | 简体中文

**M6A-02 更新：** [M6A-02 Core 实现](./m6a-02-core-resources.zh-CN.md)现提供资源引用、不可变准备请求及内容绑定计划 v2。Rust 源码为 0.3.0，未发布浏览器候选为 0.3.0-alpha.0／API schema 2；inspect 和图 render 报告 schema 为 2。[M6A-03 Native 路径](./m6a-03-native-resources.zh-CN.md)现可执行准备后的图像；浏览器资源参数及跨平台图像资格仍待 M6A-04／05。以下历史版本说明须按此更新理解。

PR-006 实现纯 CPU 编译和 `inspect --plan`。[编译器](../crates/mixture-core/src/compiler.rs)负责参数覆盖语义、依赖裁剪、排序、类型化降级、分配估算及哈希；[RenderPlan](../crates/mixture-core/src/plan.rs)定义与后端无关的类型。PR-007 [图执行](./graph-rendering.zh-CN.md)实现穷尽 WGSL 映射。固定棋盘格像素保持不变，共享的 48 字节 uniform 在该指南中说明。

## 运行与检查

```bash
cargo xtask test-plan
cargo run --locked -p mixture-cli -- inspect examples/checker.mix --plan --json
cargo run --locked -p mixture-cli -- inspect examples/checker.mix --plan \
  --size 65x3 --output roughness,baseColor --set 'frequency=16'
cargo run --locked -p mixture-cli -- inspect fixtures/format/valid/all-m2.mix \
  --plan --json --output roughness
cargo xtask test-core
cargo xtask check
```

必须指定 `--plan`。默认使用 64×64、仅 `baseColor`、无覆盖及标准 `SafetyLimits`。`--size N` 指定正方形，`--size WxH` 指定矩形。`--output` 接受任意顺序、逗号分隔且不重复的通道名：`baseColor`、`normal`、`roughness`、`metallic`、`height`、`ambientOcclusion`、`opacity`、`emissive`。名称区分大小写，空通道或重复通道无效。`--set` 可为不同的已声明暴露 ID 重复指定，值必须是精确 JSON。暴露枚举需保留 JSON 字符串引号，例如 `--set 'mode="screen"'`。公开名称必须在当前文档中存在；未暴露的节点参数不能直接寻址。

`inspect` 与验证命令共享有界源文件加载器：默认策略最多读取 2 MiB 加一个检测字节。编译前验证整个源文档，包括未请求的分支。命令不获取适配器、不写入输入文件。选项可位于路径之前；以 `-` 开头的路径前使用 `--`。

JSON 模式输出一份含 `schemaVersion: 1`、`plan`、`ok` 和 `diagnostics` 的报告。成功时包含计划及空诊断数组；失败时 `plan: null`，并包含共享的有序诊断及传入路径。人类可读模式显示 pass 来源、kernel、资源映射、连接／默认输出来源、哈希及估算。退出码 `0` 表示编译成功，`2` 表示调用／源文档／请求无效，`1` 表示文件／报告 I/O 失败。CLI 语法错误、重复选项／覆盖 ID 或无效覆盖 JSON 向 stderr 输出用法；语义失败使用所选报告模式。

## 公共 API 与规范化源文档

```rust
use mixture_core::{CompileRequest, MaterialDocument, OutputChannel, compile, normalize};

let mut request = CompileRequest::default();
request.size = [65, 3];
request.outputs = vec![OutputChannel::BaseColor, OutputChannel::Roughness];
request.overrides.insert("frequency".into(), serde_json::json!(16));
let document = MaterialDocument::decode(source_bytes, &request.limits)?
    .into_validated(&request.limits)?;
let normalized = normalize(&document, &request)?;
let plan = compile(&document, &request)?;
assert_eq!(plan.passes().len(), 2);
assert_eq!(plan.estimates().peak_bytes, 5488);
```

[核心文档测试](../crates/mixture-core/src/lib.rs)覆盖可执行版本。`CompileRequest` 显式持有尺寸、类型化通道、公开 ID 到 JSON 的覆盖映射以及限制。`normalize` 和 `compile` 接受 `&ValidatedDocument`，按传入策略复查图集合限制，拒绝零尺寸，检查各轴及请求输出／覆盖数量，且不提高限制。源字节限制在解码时执行；已验证文档不保留原始字节长度。调用方要求更严格的解码策略时，也必须在该边界传入。

覆盖目标必须已声明，每个值必须满足现有参数契约。整数参数要求整数 JSON token；浮点／颜色必须有限且在范围内；枚举必须精确匹配。所有覆盖一起应用于克隆文档，再对最终状态验证跨参数约束。被裁剪分支上的覆盖也需验证。编译不修复或钳制值。

`NormalizedDocument` 仅提供完整源视图的共享引用：默认值显式化，浮点／颜色使用 f64 JSON 表示，负零规范为正零，节点／边／公开绑定按稳定字典序排列。未使用节点仍保留在该视图中。规范化与编译均不修改原始 `ValidatedDocument`。完整源规范化与计划裁剪分别进行。

## 排序、kernel 与资源

编译器从请求的材质输入沿连接反向遍历依赖。字典序 Kahn 排序在每个节点处理后选取就绪节点中最小的 ID。同一生产者的重复绑定只形成一条排序依赖。请求按材质契约中的通道顺序输出，与 CLI／API 中的请求顺序无关。

每个选中的像素节点生成一个 pass。未连接的可选输入按契约输入顺序，在使用它的节点之前立即生成常量 pass。请求的未连接材质输入在图 pass 之后按通道顺序生成常量。`material-output` 仅生成输出映射，不生成像素 pass。不同默认输入不去重；多个绑定／通道共享的生产者只编译一次，通道可映射到同一资源。

`PassId` 和 `ResourceId` 按生成顺序从零连续编号。每个 pass 生成一张所请求尺寸的逻辑二维、单 mip、单层 `rgba16float` 纹理；`TextureDesc` 同时携带 `Scalar`、`Color` 或 `Normal`。标量存储为 `[value, 0, 0, 1]`；法线存储为 `[x, y, z, 1]`，使用契约的编码切线空间默认值。局部工作组为 `[8, 8, 1]`，dispatch 为 `[ceil(width/8), ceil(height/8), 1]`。

| KernelId／调用 | 来源与类型化参数 | 计划 uniform 字节数 |
| --- | --- | --- |
| `constant` | `constant-scalar`、`constant-color` 或生成的输入／通道默认值；`[f32; 4]` | 16 |
| `checker` | `[u32; 2]` 单元数和两个 `[f32; 4]` 颜色 | 48 |
| `levels` | 标量输入 `ResourceId`，五个 f32 参数 | 32 |
| `blend` | 颜色 `a`／`b` 与标量 `mask` 资源 ID、类型化 `BlendMode`、f32 不透明度 | 16 |
| `fractalNoise`／`FractalNoise` | u32 `seed`、`scale`、`octaves`；f32 `persistence`；类型化 `NoiseBasis` | 32 |
| `gradientMap`／`GradientMap` | 标量 `input: ResourceId`，两个线性 RGBA `[f32; 4]` 端点 | 32 |
| `heightToNormal`／`HeightToNormal` | 标量 `input: ResourceId`，f32 `strength` | 16 |
| `transform2d`／`Transform2d` | 源节点 `transform-2d`；`input: ResourceId`、`scale: [u32; 2]`、`quarter_turns: u32`、`offset: [f32; 2]` | 32 |
| `warp`／`Warp` | 源节点 `warp`；`input: ResourceId`、`displacement: ResourceId`、`strength: [f32; 2]` | 16 |

`KernelInvocation::id()` 穷尽匹配；`inputs()` 按绑定顺序提供类型化输入。计划没有任意参数 JSON，也没有第二份无类型输入列表。`PassOrigin` 保留节点 ID／类型／版本，或生成默认值的所属节点与端口。`PlanOutput` 保留通道类型、连接端点或显式默认值及真实逻辑资源。`RenderPlan` 的公共 API 不可变，不允许反序列化或通过公开构造函数伪造引用／哈希。

源 f64 值在降级时舍入为 f32，计划记录实际参数。不同十进制值若舍入为同一 f32，生成相同计划。`levels` 输入上下限转换后仍须严格递增，重合则在编译阶段失败。返回计划前检查棋盘格坐标乘法及分配算术。设备限制、着色器绑定布局、实际 GPU 分配、f16 纹理精度和像素验证仍由 PR-007 执行器负责。

`KernelInvocation::Transform2d` 将源参数 `scaleX`／`scaleY` 降级为 `scale`，`quarterTurns` 降级为 `quarter_turns`，`offsetX`／`offsetY` 降级为 `offset`。scale 分量保留为 `[1, 64]` 范围内的精确 u32 整数，quarter turns 保留为 `[0, 3]` 范围内的精确 u32 整数，两者均不经过浮点转换。默认值为 `scale: [1, 1]`、`quarter_turns: 0` 和 `offset: [0.0, 0.0]`。偏移从 `[-1, 1]` 范围内的有限源 f64 值降级为 f32。该调用先围绕纹理块中心进行逆向四分之一圈旋转，再沿源坐标轴缩放，最后叠加源 UV 偏移；`quarter_turns` 描述以左上角为原点的图像坐标中可见的顺时针旋转。

`KernelInvocation::Warp` 将 `strengthX`／`strengthY` 降级为 `strength: [f32; 2]`，默认值为 `[0.05, 0.0]`。每个分量从 `[-1, 1]` 范围内的有限源 f64 值降级而来。两个调用都要求源端口 `in` 连接 `Scalar` 输入，并生成 `Scalar` 资源。Warp 还要求 `Scalar` 类型的 `displacement` 连接，位移场值 `0.5` 表示无位移。`inputs()` 对 transform 返回 `[input]`，对 warp 返回 `[input, displacement]`。即使 warp 的两个绑定引用同一个共享生产者 pass，列表仍保留重复绑定；必填连接不会生成默认值 pass。重复寻址的双线性采样及坐标语义见[节点契约](./node-contracts.zh-CN.md)。

执行器在绑定 `0` 放置 uniform，在 `1` 放置输出纹理，在 `2` 放置 `input`；warp 在 `3` 增加 `displacement`。Transform 的 32 字节 uniform 先包含四个 u32 字 `[scale[0], scale[1], quarter_turns, 0]`，再包含四个 f32 字 `[offset[0], offset[1], 0.0, 0.0]`。Warp 的 16 字节 uniform 包含四个 f32 字 `[strength[0], strength[1], 0.0, 0.0]`。计划的 `uniform_bytes()` 包含这些填充字节；资源 ID 是类型化绑定，不是 uniform 载荷中的值。

## 分配估算契约

计划版本 1 假定简单的执行顺序：所有 pass 纹理和 uniform 保留至执行／回读结束；按请求通道顺序逐个分配、释放一个 staging 缓冲区。没有提前释放或资源池。估算的是逻辑 GPU 分配字节，不是驱动测量值，也不包含 CPU 内存、PNG 分配、着色器／管线内存或设备分配粒度。

令 `W`／`H` 为尺寸，`P` 为 pass 数，`O` 为请求通道数，`U` 为含填充的 uniform 总字节数，`T = W × H × 8`，`R = ceil(W × 8 / 256) × 256 × H`：

| 报告字段 | 定义 |
| --- | --- |
| `textureBytes` | `P × T`，也是纹理峰值驻留字节数 |
| `uniformBytes` | `U`，上表 uniform 大小之和 |
| `paddedBytesPerRow` | `ceil(W × 8 / 256) × 256` |
| `readbackBufferBytes` | `R`，一个驻留 staging 缓冲区 |
| `readbackBytes` | `O × T`，紧密原始 rgba16float 输出字节数 |
| `cumulativeReadbackBytes` | `O × R`，各次回读的累计 staging 分配 |
| `cumulativeBytes` | `P × T + U + O × R` |
| `peakBytes` | `P × T + U + R`，与 `limits.transient_bytes` 比较 |

别名仍按每个请求通道回读一次。算术使用经检查的 u64 运算，staging 行跨度通过经检查的转换收窄为 u32。限制等于 `peakBytes` 时允许通过。65×3 的棋盘格加默认 roughness 有 2 个 pass、768 字节 staging 行、5488 峰值字节和 7792 累计字节。最后使用者释放、别名回读去重、资源池及驱动专属开销不属于本版本；修改该模型须明确决定计划版本。

## 稳定哈希契约

`plan.version` 为 `1`。`hash` 是 `sha256:` 加 64 位小写十六进制。哈希输入精确为 `mixture-render-plan-v1` 的字节、一个 NUL 字节，再接不含 `hash` 的计划主体紧凑 UTF-8 JSON。`RenderPlan::hash_input()` 为使用方返回这些字节。根字段顺序为 `version`、`documentVersion`、`size`、`materialOutput`、`passes`、`outputs`、`estimates`；嵌套顺序由类型化序列化器和[已检入快照](../crates/mixture-core/tests/snapshots/)固定。这是版本化序列化契约，不是通用的键字典序 JSON。修改键顺序、数值格式、降级或内存语义时，需要对照该版本审查。

主体包含源文档版本、选中节点／所属节点的 ID、类型与版本、有效类型化参数（含覆盖结果）、输出来源、请求通道、尺寸、pass／资源标识、描述、dispatch 及确定性估算。节点 ID 为诊断／排序而保留；节点重命名不属于图同构规范化。

空白、源键／节点／边／公开绑定顺序、请求通道顺序、省略与显式默认值、负零及无效应覆盖均不改变计划／哈希。未请求分支、仅影响这些分支的有效覆盖、未使用的暴露元数据，以及允许相同计划通过的安全策略上限均不进入主体。路径、时间戳、适配器、日志和耗时不在其中。实际计划相同的两个源文档共享哈希。完整源文档仍是事实来源；此哈希标识一次编译请求，而非 `.mix` 文档的每个字节。

`sha2` 是唯一新增的核心直接依赖，用于 SHA-256，避免使用依赖进程的标准哈希或自写密码学实现。有意识地将其依赖闭包加入 `Cargo.lock`，保留先前已解析包的版本。见[依赖策略](./development.zh-CN.md)。

## 验证与下一步

[核心计划测试](../crates/mixture-core/tests/plan.rs)覆盖四份计划／哈希快照、置换／默认值／无效应覆盖等价性、有效覆盖、完整源不可变性、未请求分支、共享生产者、可选默认值、类型化绑定、非对齐尺寸、精确预算、大图裁剪、算术溢出和 f32 上下限重合。[CLI 集成测试](../crates/mixture-cli/tests/inspect.rs)覆盖真实进程报告、路径独立性、无 GPU／源修改、用法及语义失败、有界读取和退出码。`cargo xtask test-plan` 运行两套测试；`test-core` 和 `check` 也包含公共 API 文档测试。

四份新快照分别是 checker/baseColor、checker/全部默认通道、all-M2/baseColor+roughness、all-M2/仅 roughness。使用 Python hashlib 从紧凑主体独立复现了 SHA-256。all-M2 请求 baseColor+roughness 时有 5 个 pass，仅请求 roughness 时只剩一个 `mask` 常量 pass。现有 GPU 像素基准未修改。

PR-006 在固定工具链上的本地定向检查与工作区检查通过，当时尚无远端跨平台 CI 结果。现已关闭已记录的[远端 CI 门槛](./evidence/remote-ci/README.zh-CN.md)。PR-007 添加了通过唯一 `wgpu` 图渲染器[执行这些类型化调用](./graph-rendering.zh-CN.md)的能力，并提供全部六个 M2 节点的像素夹具。PR-008 添加了[材质基准工具](./material-goldens.zh-CN.md)；已完成的 [M3 评审](./m3-review.zh-CN.md)记录了三种材质的人工接受。

PR-009 添加类型化 `FractalNoise`、`GradientMap` 和 `HeightToNormal` 调用，不改变计划版本 1 或原计划／哈希快照。噪声上传全部 u32 种子位，哈希包含种子、基底、scale、octave 及 persistence 语义；编译器保留 Scalar／Color／Normal 类型化连接并正常裁剪新分支。公共 [M3 API 测试](../crates/mixture-core/tests/m3_nodes.rs)验证默认值、必填种子、降级、分支裁剪及哈希敏感性。

PR-010 添加类型化 `Transform2d` 和 `Warp` 调用，作为 v1 源节点目录的兼容新增扩展。文档版本和计划版本仍为 `1`；现有哈希前缀、原计划／哈希快照以及现有材质像素基准保持不变。新调用沿用相同的类型化序列化、有效参数哈希及依赖裁剪规则。公共[重采样 API 测试](../crates/mixture-core/tests/resampling.rs)覆盖降级后的默认值、整数字段、有序及重复的标量绑定、uniform 大小、未请求分支裁剪、参数哈希敏感性、源顺序等价性，以及缺失连接或连接类型错误的拒绝行为。

## ENG-04 兼容性与未发布版本

源码 Rust 包升级到 0.2.0，因为公开且穷尽的 KernelId／KernelInvocation 枚举新增 ScalarBlend 可能破坏下游穷尽匹配。不顺带增加 non_exhaustive 或重设计 API。浏览器候选升级到 0.2.0-alpha.0；API schema 1、.mix v1 及计划版本／哈希域保持不变。已有变体序列化及旧计划哈希快照不变。注册表消费者仍固定公开 npm 0.1.0-alpha.0，并须以 MIX_NODE_UNKNOWN_TYPE 拒绝 scalar-blend。候选安装仅调整暂存 runtime 归档／版本／完整性，工具依赖及固定的一次性 Studio 源码保持不变。Rust 包及新浏览器候选均未发布，本项工作不授权发布。
