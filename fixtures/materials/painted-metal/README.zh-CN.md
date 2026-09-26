# 涂漆金属 — MAT-02a 冻结设计

[English](./README.md) | 简体中文

[图设计](./graph-design.json)与[资格计划](./qualification-plan.json)保留 MAT-02a 的冻结契约。两个节点现已实现，完整材质验收仍未完成。计划中的 `runtimeImplemented: false` 与 `planned-mat-02a-frozen` 是冻结时的历史状态，不代表当前实现状态。含 `$` 的配方不是运行时表达式语言，也不是 `.mix` 文档。

## 已实现的请求准备

[夹具请求构建器](../../../scripts/painted-metal-requests.mjs)严格校验下述控制值，生成七种预设 × 四种尺寸 × 五个通道的请求（每组 Native/browser 配对共 140 项通道比较）。它复制[修订 2 图](./material.mix)的精确字节，保持拓扑及节点版本，以普通公开请求覆盖参数。Native 与浏览器验收工具可共用这些 JSON 请求；图语义仍归 Core 所有。

```bash
node --test scripts/painted-metal-requests.test.mjs
node scripts/painted-metal-requests.mjs tmp/mat02-requests
```

输出目录必须不存在。`requests.json` 绑定图字节、冻结计划、构建器、源码版本和工作区状态；`material.mix` 保留原始图字节。CPU 测试覆盖矩阵完整性、非法控制值、端点、独立轴半径与快照隔离，并在两个既有浏览器 CI 工作流中执行。准备请求不渲染像素，也不代表材质通过验收。完整 Native/browser 对照、原始半精度关系及参数因果、接缝、包与重复渲染、release 性能、金属 PBR 和人工验收仍是必要条件。


## 公开 GPU 矩阵候选

Native 的 `painted_material` 集成测试与浏览器的 `painted.spec.mjs` 共用 28 项请求，检查全部五通道、精确重复、普通图／包等价、高度切片、销毁后的输出所有权和物理内存计量。Native 在 release 模式测量冻结的默认 1K/2K 冷／热预算，拒绝没有对应预算的适配器。这些矩阵工具仍是候选，代码存在不代表运行已通过。

Native 运行需设置 `MIXTURE_PAINTED_ROOT` 为仓库路径、`MIXTURE_PAINTED_REQUESTS` 为生成的请求目录、`MIXTURE_PAINTED_EVIDENCE` 为新的输出目录。显式设置 `MIXTURE_GPU_BACKEND=vulkan|dx12` 与 `MIXTURE_GPU_SOFTWARE=0|1`，可用 `MIXTURE_GPU_EXPECT_ADAPTER` 强制匹配记录的设备。

```bash
cargo test --release --locked --all-features --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --test painted_material -- --ignored --nocapture
node scripts/browser-runtime/check-painted.mjs tmp/sdk-candidate tmp/sdk-painted-comparison
```

第二条命令消费通过的独立 SDK 候选运行：候选模式现在必须执行 21 项浏览器测试，含完整涂漆金属矩阵。工具校验图／请求／包／构建身份，并重跑 Native 矩阵，对全部浏览器通道维持 <=1/255 门槛。既有 Chromium 工作流调用该对照并保留输出。registry 模式仍为 13 项测试；已发布包不支持此材质。原始半精度参数因果、周期接缝、压力及 PBR 视图、人工决定仍是独立待完成门槛。`materialAccepted` 保持 false。


## 端点与分辨率质量门槛

两个公开执行端均为完整涂层、裸露底材、完全锈蚀底材渲染独立常量参考图。在四种冻结尺寸下，全部五通道的每个像素都必须与常量参考一致，包括平坦法线。对照要求全部 60 项端点通道断言通过。参考采用普通 `.mix` 节点经唯一 wgpu 执行器求值，不复用分层材质拓扑，也不引入 CPU 渲染器。

每端还将默认 256² 的高度／底色与自身 1024² 交付 RGB 字节的精确 4×4 均值比较。均值不先舍入为整数字节，而是直接计算绝对误差；各 RGB 分量均须符合冻结的 4/255 上限，对照拒绝缺失或失败的测量。alpha 固定不透明，不纳入 RGB 测量。Native 判定测试覆盖小数均值、独立分量、矩形索引和截断输入。这是执行既有契约，不改变着色器、格式、门槛或 golden。完整材质验收仍需原始半精度参数因果、周期接缝、压力及 PBR／人工评审。

## 公开参数隔离

Native 与浏览器矩阵在冻结的 257×129 尺寸上，对完整默认材质逐项修改十一组控制：三种颜色、三种粗糙度端点、0/0.5/1 法线强度，以及分别递增的宏观与细节种子。每个变体渲染两次。颜色只能影响 baseColor，粗糙度只能影响 roughness，法线强度只能影响 normal。修改的参数必须实际改变至少一个交付像素；保持原值的法线强度用例则必须逐通道完全一致。种子变体必须改变像素并精确重复。两端记录一致的参数／影响结果，对照拒绝缺失用例。

这些是实际图的交付字节因果检查，不能证明内部半精度遮罩不等式或从存储高度精确重建法线。原始 half 的 W/I/B/R 关系、裸露／宽度单调性、种子对 W 的隔离、接缝、压力以及 PBR／人工验收仍必须完成。请求清单绑定冻结的因果输入；不修改着色器、运行时 API 或材质契约。

## 原始 half 遮罩验收

忽略执行的 Rust 测试 `graph_gpu_painted_raw_mask_relations_and_control_causality` 从源码目录读取实测材质和冻结计划。它保留全部 23 个像素计算，只改变 height 输出连接以观察 W、I、B、R、S 或 D。Core 编译每个别名，并通过普通分配计划保留其输出。唯一执行器运行生产内核及原有拷贝／映射／清理路径。私有 `cfg(test)` 回读格式在 RGBA8 转换前返回紧密排列的 half 字节；不增加公开原始输出 API、着色器变体、全局捕获缓冲或第二执行器。发布构建仍只有普通 RGBA8 回读。

在 257×129 上，十九组用例逐像素检查有限归一化值、I≤W、B=half(max(W−I,0))、B≤S≤W、R≤S≤W、细节范围、分数 W 下零宽 B 精确为零、边带／裸露／锈蚀控制单调性、填充精确端点、锈蚀控制对 W/I/B 的隔离、细节种子对 W/I/B 的隔离及种子精确重复。不采用字节容差。CPU 回读探针验证去除行填充后仍保留 1/4096 的差异。`cargo xtask gpu-smoke` 包含此测试；以下专项命令使用既有文档的显式 GPU 环境。普通打包单元测试编译不依赖仓库夹具，执行此忽略的验收测试则需要源码目录。

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_painted_raw_mask -- --ignored --nocapture
```

此门槛不验收浏览器原始 half 字段、完整金属度／粗糙度／高度合成、法线重建、周期接缝、压力或 PBR／人工评审。独立公开矩阵及其余材质门槛仍必须完成。

## 标量合成与法线重放

忽略执行的测试 `graph_gpu_painted_composition_and_final_height_normal_replay` 在 257×129 上覆盖全部七个冻结预设。它通过同一原始 half 测试设施观察真实 W/R、涂层／最终高度、涂层／最终粗糙度和金属度。独立标量断言计入各节点间的 half 存储舍入，包括常量锈蚀高度／粗糙度，逐像素检查合成关系，并将最终高度限制在允许的底材与涂层端点之间。这些有限逐像素断言是测试参考，不是 CPU 渲染 API。

原始五输出图还回读最终高度与法线的原始 half 字节。测试把这些精确高度字节上传到独立探针纹理，经既有流水线工厂调用生产 height-to-normal WGSL。在 0/0.5/1 法线强度下，图的存储高度必须不变，重放法线的每个字节必须与图输出一致。不经 RGBA8 重建高度，也不采用 CPU 法线算法。此设施仅用于 GPU 测试，使用同一着色器及回读／清理辅助函数，不改变公开 API 或运行时行为。`cargo xtask gpu-smoke` 包含此测试，专项命令为：

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_painted_composition -- --ignored --nocapture
```

这些检查不替代浏览器原始 half 证据、其他分辨率、周期接缝／压力测量或金属 PBR／人工评审。公开多分辨率 Native／浏览器矩阵仍必须完成。

## 精确调用方控制

夹具请求构造器拒绝未知控制、非有限值和超出下列范围的值。这不是新增 Core API。先将预设控制覆盖默认值，再执行映射。宏观／细节种子均为显式 u32，即使关闭细节也保留两者。所有颜色采用 [0,1] 内线性 RGBA，alpha 固定为 1。

| 公开控制 | 范围 | 映射 |
|---|---|---|
| `exposureAmount` | 浮点 [0,1] | 0<a<1 时，exposure levels 的 inputMin=0.8*(1-a)、inputMax=inputMin+0.2、outputMin=0、outputMax=1。a=0 或 1 时采用 inputMin=0、inputMax=1，两个输出端点均为 a。gamma 始终为 1。精确端点不依赖噪声极值。 |
| `exposureScale` | 整数 [1,64] | 宏观噪声 scale；固定三倍频层与 persistence 0.5。默认质量只覆盖默认尺度，不保证所有高频输入。 |
| `macroSeed`、`detailSeed` | u32 | 对应噪声种子。细节噪声固定 scale 32、两倍频层、persistence 0.5；两者均显式使用 v2 `value`。 |
| `edgeWidth` | 整数 [0,8] | 1024 下的参考像素宽度；按计划独立映射每轴半径。本验收工具拒绝轴长 >2048。 |
| `rustAmount`、`rustFill`、`detailAmount` | 浮点 [0,1] | 分别映射锈迹遮罩 opacity、边带到裸露区域的插值权重、细节 levels 的 outputMin=1-detailAmount；outputMax=1。 |
| `paintColor`、`substrateColor`、`rustColor` | 线性 RGBA | 对应 constant-color 值，alpha 须为 1。 |
| `paintRoughness`、`substrateRoughness`、`rustRoughness` | 浮点 [0,1] | 前两者为 coatingRoughness 输出端点，第三者为 rustRoughness 常量。levels 反向输出端点是有意使用的既有能力。 |
| `paintThickness` | 浮点 [0,0.5] | coatingHeight 的 outputMin=paintThickness、outputMax=0。 |
| `rustRelief` | 浮点 [0,paintThickness] | rustHeight 常量=rustRelief；拒绝超过涂层厚度的值，不静默夹取。 |
| `normalStrength` | 浮点 [0,8] | 既有 height-to-normal 的 strength。 |

生成真实 `.mix` 时，输出名映射普通 material-output 端口。检查别名仅为验收请求暴露中间遮罩，不增加材质通道类型或公开表达式语法。所有渲染（包括端点预设）从相同图拓扑开始，不能靠替换图隐藏控制间的不当依赖。请求输出的依赖裁剪仍由 Core 负责。

概念公式描述结构；实际参考须计入既有节点逐级半精度存储舍入。遮罩不等式检查原始 half 值，不能从 RGBA8 推断精确值。端点像素通过唯一 wgpu 执行器与独立构造的常量参考图比较，包括现有法线编码／转换。不能用仅 RGBA8 的减法探针掩盖半精度差异。

`detailAmount` 调制锈迹覆盖，从而影响下游起伏／粗糙度／颜色，但不移动 W。`rustFill=0` 将锈迹限制在侵蚀边带，`rustFill=1` 允许覆盖全部裸露区域。边宽为零强制 B=0；rustFill>0 时不强制 R=0。涂层高度不低于允许的锈迹／底材高度端点，但不声称每个过渡像素等于某个端点。

## 历史资源决定与冻结证据

显式图包含 23 个像素节点及未来一个结构性 material-output 节点。保留 24 pass 上限。用 levels 实现两组涂层端点插值，保留公开控制的同时避免四个冗余常量／插值 pass；不改变任何节点实现。

冻结设计时的执行器保留全部 rgba16float 中间纹理。2048² 下仅 23 张纹理就需要 **771,751,936 字节**，尚未计入 uniform 与顺序回读缓冲区，已超过不变的 **536,870,912 字节**描述符上限。这是静态设计估算，不是编译计划、GPU 实测分配或已接受性能结果。Core 默认瞬时字节限制也是 512 MiB，因此当前 2K 图预计在执行之前就会被拒绝。节点具备后，先保留真实的 2K 预算结构化失败及有效的 1K 编译计划／渲染基线，再启动独立 PERF-MAT 生命周期／复用切面。Core 估算须与执行器生命周期一致，不能提高全局限制来收集通过的 2K 结果。不得提高上限或引入 pass 融合／全局缓存掩盖失败；优化期间图与质量目标保持固定。

[留存既有输入测量](../../../docs/evidence/mat-02-input-feasibility/README.zh-CN.md)支持保持 4/255 目标：记录的 GT 1030 Vulkan／DX12 上，baseColor 最大平均误差为 0.158619/255，height 为 0.101737/255。JSON 冻结精确探针场、有限集合解析判定与 ABI。契约选定两个新身份，并在首次实现推进 0.7 候选。既有节点可行性不验收尚未实现的形态处理或完整材质。准入、实现、验收及发布继续分别记录。


## 最终高度法线的周期边界

`graph_gpu_painted_normal_periodic_boundaries` 在四种冻结尺寸下捕获全部七个预设的最终高度和法线原始 half 字节。分别沿每个轴循环平移一个像素，以及同时沿两个轴平移半幅图，再调用现有生产法线内核。输出的每个字节必须等于图法线的相同循环平移。将边界像素移入内部并反向移动，可检查周期导数采样，而不会错误地要求相对两侧边缘像素相等。CPU 辅助函数只重排字节；非对称矩形测试验证其索引。

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_painted_normal_periodic_boundaries -- --ignored --nocapture
```

28 个用例在每个适配器上包含 84 次精确平移比较。测试纳入 `cargo xtask gpu-smoke`，输出适配器身份及用例／尺寸／偏移记录。它仅覆盖最终高度的法线导数，不能证明上游噪声／遮罩合成周期性、浏览器原始 half 一致性、没有明显视觉接缝、压力场景质量或人工验收。未修改着色器、运行时 API、节点版本或黄金基线。

## 配方修订 2

当前图／计划使用[零基底相对高度](../../../docs/mat-02-relative-height.zh-CN.md)。[原计划](./qualification-plan-v1.json)、[原设计](./graph-design-v1.json)及历史性能图保持不变。此前通过记录不能用于验收修订后的高度／法线像素。


## 所选噪声输入的周期性

`node_fractal_noise_gpu_periodic_material_inputs` 测试生产 `fractal-noise@2` 着色器：宏观噪声 scale=8、octaves=3、seed=1729 或 u32::MAX；细节噪声 scale=32、octaves=2、seed=65537 或 u32::MAX；压力噪声 scale=64、octaves=3、seed=1729，persistence 均为 0.5。覆盖全部四种材质尺寸。测试只修改 WGSL 测试副本中的采样原点调用，借用未使用的 uniform 填充字段；生产代码和 ABI 不变。探针有意传入一个周期之外的坐标，不预先取模。沿 x/y 移动完整周期，以及完整周期加内部偏移，必须精确重现原图对应位置的 half 像素。断言源码替换锚点唯一，非恒定基线避免恒定输出误通过。60 次整图比较覆盖有限值／范围及资源清理。

```bash
cargo test --locked -p mixture-wgpu --test nodes node_fractal_noise_gpu_periodic_material_inputs -- --ignored --nocapture
```

`cargo xtask test-node fractal-noise` 和完整 GPU smoke 也会选择该测试。设置 `MIXTURE_NODE_EVIDENCE_DIR` 可写入 `value-noise-periodic.json`；输出记录尺寸、种子、参数、偏移及适配器。这只证明所选材质输入的周期采样，不代表 v1／cellular 噪声、任意图、视觉接缝验收或浏览器原始 half 一致性。形态学／合成和材质视觉门槛仍需单独验证。
