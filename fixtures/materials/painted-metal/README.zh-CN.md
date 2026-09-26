# 涂漆金属 — MAT-02a 冻结设计

[English](./README.md) | 简体中文

[图设计](./graph-design.json)与[资格计划](./qualification-plan.json)保留 MAT-02a 的冻结契约。两个节点现已实现，完整材质验收仍未完成。计划中的 `runtimeImplemented: false` 与 `planned-mat-02a-frozen` 是冻结时的历史状态，不代表当前实现状态。含 `$` 的配方不是运行时表达式语言，也不是 `.mix` 文档。

## 已实现的请求准备

[夹具请求构建器](../../../scripts/painted-metal-requests.mjs)严格校验下述控制值，生成七种预设 × 四种尺寸 × 五个通道的请求（每组 Native/browser 配对共 140 项通道比较）。它复制[实测基线](../../../docs/evidence/perf-mat-before/material.mix)的可执行图字节，保持拓扑及节点版本，以普通公开请求覆盖参数。Native 与浏览器验收工具可共用这些 JSON 请求；图语义仍归 Core 所有。

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
| `paintThickness` | 浮点 [0,0.5] | coatingHeight 的 outputMin=0.2+paintThickness、outputMax=0.2。 |
| `rustRelief` | 浮点 [0,paintThickness] | rustHeight 常量=0.2+rustRelief；拒绝超过涂层厚度的值，不静默夹取。 |
| `normalStrength` | 浮点 [0,8] | 既有 height-to-normal 的 strength。 |

生成真实 `.mix` 时，输出名映射普通 material-output 端口。检查别名仅为验收请求暴露中间遮罩，不增加材质通道类型或公开表达式语法。所有渲染（包括端点预设）从相同图拓扑开始，不能靠替换图隐藏控制间的不当依赖。请求输出的依赖裁剪仍由 Core 负责。

概念公式描述结构；实际参考须计入既有节点逐级半精度存储舍入。遮罩不等式检查原始 half 值，不能从 RGBA8 推断精确值。端点像素通过唯一 wgpu 执行器与独立构造的常量参考图比较，包括现有法线编码／转换。不能用仅 RGBA8 的减法探针掩盖半精度差异。

`detailAmount` 调制锈迹覆盖，从而影响下游起伏／粗糙度／颜色，但不移动 W。`rustFill=0` 将锈迹限制在侵蚀边带，`rustFill=1` 允许覆盖全部裸露区域。边宽为零强制 B=0；rustFill>0 时不强制 R=0。涂层高度不低于允许的锈迹／底材高度端点，但不声称每个过渡像素等于某个端点。

## 历史资源决定与冻结证据

显式图包含 23 个像素节点及未来一个结构性 material-output 节点。保留 24 pass 上限。用 levels 实现两组涂层端点插值，保留公开控制的同时避免四个冗余常量／插值 pass；不改变任何节点实现。

冻结设计时的执行器保留全部 rgba16float 中间纹理。2048² 下仅 23 张纹理就需要 **771,751,936 字节**，尚未计入 uniform 与顺序回读缓冲区，已超过不变的 **536,870,912 字节**描述符上限。这是静态设计估算，不是编译计划、GPU 实测分配或已接受性能结果。Core 默认瞬时字节限制也是 512 MiB，因此当前 2K 图预计在执行之前就会被拒绝。节点具备后，先保留真实的 2K 预算结构化失败及有效的 1K 编译计划／渲染基线，再启动独立 PERF-MAT 生命周期／复用切面。Core 估算须与执行器生命周期一致，不能提高全局限制来收集通过的 2K 结果。不得提高上限或引入 pass 融合／全局缓存掩盖失败；优化期间图与质量目标保持固定。

[留存既有输入测量](../../../docs/evidence/mat-02-input-feasibility/README.zh-CN.md)支持保持 4/255 目标：记录的 GT 1030 Vulkan／DX12 上，baseColor 最大平均误差为 0.158619/255，height 为 0.101737/255。JSON 冻结精确探针场、有限集合解析判定与 ABI。契约选定两个新身份，并在首次实现推进 0.7 候选。既有节点可行性不验收尚未实现的形态处理或完整材质。准入、实现、验收及发布继续分别记录。
