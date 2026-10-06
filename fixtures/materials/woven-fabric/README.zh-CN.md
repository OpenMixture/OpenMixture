# 编织织物 — 公开矩阵及阶段 B 结构探针

[English](./README.md) | 简体中文

MAT-03 配方修订 2 已在**记录范围内验收通过**；详见[阶段 D 证据及人工决定](../../../docs/evidence/mat-03/README.zh-CN.md)。视觉接受预设为 plain、varied、warp-seed、weft-seed、combined-low；combined-high 及类似蝴蝶结形组合不在视觉质量保证内。冻结计划、范围、节点语义和历史 false 接受标记均不变；materialAccepted=true 仅写入新的接受记录。weave-pattern@2 是下一项跟进（让上层纱线在交叉处自己的整个宽度上可见），尚未启动。未发布任何包。

阶段 A 公开矩阵、B 有界结构探针、C 精确图像的维护者决定及 D 保留证据共同支撑上述范围，不构成任意控制组合保证。原始草案、修订 1 文件、冻结配方修订 2 计划及下文源绑定历史观测均不改写。PR #87 已关闭且未合并；underRatio 材质范围仍为 0.25..0.75。

默认 underRatio=0.25、crown=0 保持不变；plain 收窄度量 min=0.875。维护者于 2026-10-06（Asia/Shanghai）的 "Accept with a narrower range" 被后续测量取代，最终决定为 "Accept now, fix in @2 later (Recommended)"。combined-high 蝴蝶结、奇数尺寸平移 share 的非门槛限制（最多 27 half 步，|ΔP|≈0.12 个 8-bit 级）、软件省略 2048² 及记录适配器范围见接受记录。原 preview.json 的 false 标记不追溯改写。

历史冻结：f3d0f3f 在实现公开调用方工具前冻结[计划](./qualification-plan.json)与[配方](./graph-proposal.json)。[草案计划](./qualification-plan-draft.json)、[草案配方](./graph-proposal-draft.json)、[原节点组合基线](./graph-design.json)及[修订 2 契约](../../../docs/mat-03-default-revision.zh-CN.md)继续保留。

## 阶段 A 绑定源码的本地观测

干净提交 **69fb6074477f3c156e775779cb0a3770cef5189a** 的公开矩阵通过；本结果提交只更新记录，不更改冻结配方、控制、用例或门槛。计划 reviewMeasurements 绑定实测计划／源图／构建器 SHA-256 及浏览器 buildId；后续元数据不能冒充该次执行。早期 Vulkan 独立运行绑定 9d0ddad，同一图及门槛；下表采用 69fb607 的最终对比运行。

51 行 CLI validate／inspect 均通过（48 材质 + 3 压力）；每行 21 passes、8 物理纹理。256²／1024²／2048²／257×129 的描述符峰值分别为 **4719184／75498064／301990480／2419600 B**。GT 1030 Vulkan（NVIDIA 582.66）与 DX12（32.0.15.8266）各通过 51 行，覆盖逐字重复、普通图／包等价、切片高度、销毁后输出所有权、常量参考、分配会计及四组匹配耗时预算。

Chrome **154.0.8037.98** 干净候选通过 **23** 项测试（woven 51 行）；浏览器回报 BrowserWebGpu，适配器名称为空，不据此断言浏览器的具体硬件。Vulkan 对 Chrome **255** 通道图比较最大分量差 **1/255**（门槛 ≤1/255）；plain／varied 两个下采样对中最坏分量均值误差 **0.365744/255**，浏览器 **0.365744/255**（门槛 ≤4/255）。密集细线压力仍保留单独质量范围，不据此扩展保证。

| 后端 | 行 | 冷渲染 ms | 五次暖中位 ms |
|---|---|---:|---:|
| Vulkan | plain-1024x1024 | 380.661 | 272.581 |
| Vulkan | plain-2048x2048 | 1138.523 | 1070.967 |
| Vulkan | varied-1024x1024 | 312.148 | 241.999 |
| Vulkan | varied-2048x2048 | 1209.119 | 1020.760 |
| Dx12 | plain-1024x1024 | 931.150 | 244.278 |
| Dx12 | plain-2048x2048 | 1770.509 | 944.424 |
| Dx12 | varied-1024x1024 | 1005.098 | 219.167 |
| Dx12 | varied-2048x2048 | 1748.931 | 969.360 |

冷 2K 仅记录，不设额外门槛；原冷 1K／暖 1K／暖 2K 预算不变。逐张查看了 plain 的 1K baseColor／normal 与 varied 的 baseColor，仅作结构显示自查，不能替代 B 结构证明或 C PBR／人工决定。原始日志、请求、图像与逐行收据在忽略目录 tmp/woven-matrix/（native-vulkan-2、native-dx12、browser、comparison）；CLI 命令与解析脚本在同目录，均不提交。节点／着色器、版本、其他材质及黄金未改变。固定 SwiftShader 未在本机运行；B–D 均待后续 PR，materialAccepted=false。

## 精确调用方控制

[请求构建器](../../../scripts/woven-fabric-requests.mjs)校验并写入每个公开覆盖值；三个 weave 实例共享七项几何控制，仅 mode 不同。无随机隐式种子。颜色 alpha 固定为 1；派生 dark 颜色 alpha 也是 1。warpColor／weftColor／yarnRoughness 变更同时重新计算派生端点。表中的公开 ID 映射由冻结计划 bindings 精确指定。

| 控制 | 默认 | 闭区间／类型 | 公开覆盖映射 |
|---|---|---|---|
| warpCount | 8 | 4..32（偶数，不舍入） | warpshare_warpCount → n06-weave-warp-share.warpCount; coverage_warpCount → n09-weave-coverage.warpCount; height_warpCount → n17-weave-height.warpCount |
| weftCount | 8 | 4..32（偶数，不舍入） | warpshare_weftCount → n06-weave-warp-share.weftCount; coverage_weftCount → n09-weave-coverage.weftCount; height_weftCount → n17-weave-height.weftCount |
| warpWidth | 0.7 | 0.55..0.9 | warpshare_warpWidth → n06-weave-warp-share.warpWidth; coverage_warpWidth → n09-weave-coverage.warpWidth; height_warpWidth → n17-weave-height.warpWidth |
| weftWidth | 0.7 | 0.55..0.9 | warpshare_weftWidth → n06-weave-warp-share.weftWidth; coverage_weftWidth → n09-weave-coverage.weftWidth; height_weftWidth → n17-weave-height.weftWidth |
| bevel | 0.08 | 0.02..0.12 | warpshare_bevel → n06-weave-warp-share.bevel; coverage_bevel → n09-weave-coverage.bevel; height_bevel → n17-weave-height.bevel |
| relief | 0.025 | 0..0.05 | relief → n18-surfaceHeight.outputMax |
| underRatio | 0.25 | 0.25..0.75 | warpshare_underRatio → n06-weave-warp-share.underRatio; coverage_underRatio → n09-weave-coverage.underRatio; height_underRatio → n17-weave-height.underRatio |
| detailAmount | 0.08 | 0..0.1 | warpDark = warpColor.rgb*(1-4*d); weftDark = weftColor.rgb*(1-4*d); roughnessMin = yarnRoughness*(1-2*d) |
| warpSeed | 1729 | 0..4294967295（整数） | warpSeed → n03-warpNoise.seed |
| weftSeed | 65537 | 0..4294967295（整数） | weftSeed → n00-weftNoise.seed |
| warpColor | [0.22,0.08,0.035,1] | RGBA 0..1; alpha=1 | warpColor → n07-warpColor.colorB |
| weftColor | [0.38,0.23,0.1,1] | RGBA 0..1; alpha=1 | weftColor → n10-weftColor.colorB |
| backingColor | [0.015,0.012,0.01,1] | RGBA 0..1; alpha=1 | backingColor → n12-backingColor.value |
| yarnRoughness | 0.8 | 0..1 | yarnRoughness → n14-yarnRoughness.outputMax |
| backingRoughness | 0.95 | 0..1 | backingRoughness → n15-backingRoughness.value |
| normalStrength | 0.5 | 0..1 | normalStrength → n20-normal.strength |
| crown | 0 | 0..1 | warpshare_crown → n06-weave-warp-share.crown; coverage_crown → n09-weave-coverage.crown; height_crown → n17-weave-height.crown |

## 重现及门槛

先提交实现以绑定干净源码；每次使用全新输出目录。48 行包含 12 用例 × 四尺寸；3 行压力记录为默认／varied 质量保证范围外，但保留重复／跨运行时门槛。Native 的 native.json 每行刷新，冻结门槛失败保留数值并立即停止。CLI 对每行使用 `inspect material.mix --plan --size WxH --output baseColor,normal,roughness,metallic,height --json` 并将 request.overrides 每项作为 `--set ID=JSON`；validate 使用将这些覆盖值代入普通 .mix 的临时副本。不得提交解析工具或重写原收据。

颜色／高度盒滤波门槛、全部通道跨运行时门槛及匹配适配器耗时预算见冻结计划。只有 pinned SwiftShader 且设置 MIXTURE_SWIFTSHADER_COMMIT 才能使用软件预算。输出在渲染器销毁后编码；常量参考仍由同一 wgpu 执行器生成。任何冻结门槛失败即停止，不调参或放宽。

```powershell
$env:PATH='C:\Users\krapnik\AppData\Roaming\fnm\node-versions\v24.21.0\installation;'+$env:PATH
node --test scripts/woven-fabric-requests.test.mjs
node scripts/woven-fabric-requests.mjs tmp/woven-matrix/requests
$env:MIXTURE_GPU_BACKEND='vulkan' # repeat separately with dx12
$env:MIXTURE_GPU_SOFTWARE='0'
$env:MIXTURE_GPU_EXPECT_ADAPTER='NVIDIA GeForce GT 1030'
$env:MIXTURE_WOVEN_ROOT=(Get-Location).Path
$env:MIXTURE_WOVEN_REQUESTS=(Join-Path (Get-Location).Path 'tmp/woven-matrix/requests')
$env:MIXTURE_WOVEN_EVIDENCE=(Join-Path (Get-Location).Path 'tmp/woven-matrix/native-vulkan')
cargo test --release --locked --all-features --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --test woven_material -- --ignored --nocapture
$env:MIXTURE_BROWSER_CHANNEL='chrome'
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/woven-matrix/browser
node scripts/browser-runtime/check-woven.mjs tmp/woven-matrix/browser tmp/woven-matrix/comparison
cargo xtask test-consumer
cargo xtask links
cargo xtask check
```

## 历史 PR #80 设计记录（保留当时状态）

## 区分基线和提案

- [graph-design.json](./graph-design.json) 原样保留修订 4 既有节点基线。[qualification-plan.json](./qualification-plan-draft.json) 的顶层默认值／用例／独立扫描及实测资源字段仍描述该基线。保留四份绑定源码的评审记录及输入身份；本设计变更不宣称新 GPU 运行。
- [graph-proposal.json](./graph-proposal-draft.json) 单独描述**提议的 weave-pattern@1**，当前目录尚未实现。不得当作已验收／接受材质运行；其推算不是已编译计划或渲染测量。
- 节点提案：无输入、一个 value: Scalar 输出、mode=height/coverage/warp-share；三个实例共用所有几何参数及尺寸。registry 元数据允许具名输出列表，但当前降低／ComputePass／资源查找仍是单输出。一个 mode kernel 避免多输出运行时重构。
- 提议结构将连续中心线起伏与横向占据分离，由同一可见性公式导出高度及覆盖加权的纱线选择。下层纱线即使高度较低仍保留完整占据；深度不再是底布混合权重。既有带种子噪声、颜色、粗糙度及 height-to-normal 节点继续负责这些通道。精确公式、参数、诊断、周期、2×2 加权采样和 48 字节 ABI 见契约。

| 范围 | Passes | 物理纹理 | 1K peakBytes | 2K peakBytes |
|---|---:|---:|---:|---:|
| 修订 4 实测；八个用例 | 52 | 10 | 92,276,064 | 369,100,128 |
| 提案图，仅静态推算 | 21 | 8 | 未测量 | 推算 301,990,480 |

推算假定数字 ID 顺序、plan-v3 精确描述符复用、592 uniform 字节及 2K 一个 33,554,432 字节 staging buffer：8*33,554,432+592+33,554,432。逻辑 704,643,072 字节不是峰值。保持 <=64 passes、<=536,870,912 字节，后续测量真实实现；该计算不构成耗时或像素结论。

## 基线复现

将基线用例覆盖到计划顶层默认值。按不变的已接受范围验证，拒绝奇数数量、未知／非有限／越界控制。解析 warpGap=1-warpWidth、weftGap=1-weftWidth、halfWarpCount=warpCount/2、crossingOffsetX=0.5/warpCount、profileBevel=0.19+0.5*bevel、warpDark/weftDark=对应颜色 RGB*(1-4*detailAmount) 且 alpha=1、roughnessMin=yarnRoughness*(1-2*detailAmount)。其余占位符直接使用同名控制，包括 underRatio。无随机变化的轮廓／选择器种子固定为零；两个噪声种子仍显式提供。这些是调用方表达式，不是 Core 表达式。

保留数字节点 ID。输出普通 .mix v1：复制节点 id/type/version/解析参数；把 inputs 的 source.port 转为 from/to 边；追加 id=material 的 material-output@1 并连接全部五输出。排除设计字段／占位符。修订 4 的 warpShape/weftShape 别名是归一化 A/B，coverage=C、surfaceOrder=D；历史公式见保留的修订 4 章节。该基线未获连续纱线外观接受。

~~~text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <baseline-case.mix> --json
mixture inspect <baseline-case.mix> --plan --size <1024|2048> --output baseColor,normal,roughness,metallic,height --json
mixture render <plain|varied.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
~~~

mixture 指 target/native-consumer/release/mixture.exe。此前修订 4 Native 渲染使用 NVIDIA GeForce GT 1030／Vulkan／NVIDIA 582.66。本地评审文件位于 tmp/mat03-recipe-review-4/after/{plain,varied}-1024/：baseColor、normal、roughness、height PNG；plain 还有相应 *-crossing-256.png 裁剪（x=64、y=64、256×256，无缩放／色调调整）。忽略目录还保留基线／探针输入及回执。中性 C、D*C 探针虽零差异，却未发现维护者指出的连续纱线失败。这些文件不是持久接受证据。

## 提案调用方映射及待定决定

仅对提案，按顺序合并顶层基线默认值、graph-proposal.proposedDefaults、各用例覆盖。这样新增 crown=0.5，并恢复 underRatio=0.5／relief=0.025；不修改基线默认值。三个 weave 实例的几何包 {warpCount,weftCount,warpWidth,weftWidth,bevel,crown,underRatio} 必须相同，仅 mode 不同。提议 bevel 直接控制横向占据羽化，不再使用砖块 profileBevel 换算。颜色／粗糙度暗端仍按上文解析，其他提案占位符直接使用同名控制。既有节点种子仍显式提供；确定性 weave 生成器无随机输出或种子。候选默认值和 crown 控制须批准。

待批准问题：有界节点理由；身份／Scalar-mode 设计或另行限定多输出支持；精确占据／起伏／轮廓／可见性／采样契约；crown／bevel／默认值；**17 类型／15 kernels → 18／16** 及未发布 **Rust 0.9.0／浏览器 0.9.0-alpha.0**；精确节点／材质／PBR／人工门槛；后续明确实现授权。本次不改清单；.mix／.mixpack v1 和 plan／API v3 不变。旧目录拒绝新类型，不回退、不迁移。

契约中的后续 PR 清单覆盖不变量 8 和节点流程：Core 契约／验证／降低、穷尽 kernel 消费者、一个 WGSL、夹具／文档／针对性测试、shader／软件／GPU／浏览器／节点／材质回归、包消费、不变预算及独立视觉接受。当前不能运行提案节点命令。保留完整结构、采样、raw-half／法线、周期／奇数尺寸、Native／浏览器 <=1/255、重放／包、耗时、PBR 和人工冻结阻碍。按[证据政策](../../../docs/evidence-policy.zh-CN.md)保留评审字节。本次不含新增 crate、运行时、shader、版本、黄金图或提交工具变更。

## 阶段 B 冻结结构探针

维护者于 2026-10-04 接受材质配方修订 2 默认值，不等于接受完整材质。[计划](./qualification-plan.json)新增 structuralProbes 节拥有精确探针。阶段 A 的字段、阈值、用例、尺寸、耗时预算、默认值和修订 1 收据全部不变。C 介电 PBR／人工评审与 D 保留证据／接受仍独立；materialAccepted=false。

### 交点结构与零起伏

通过真实图的高度输出别名观察原始 binary16 H/C/S，保留全部 21 pass。plain 与 varied 在 256²、1024²、2048²、257×129 遍历所有交点 UV ((i+0.5)/warpCount,(j+0.5)/weftCount) 和间隙 UV (i/warpCount,j/weftCount)。观察包含该 UV 的 floor(UV*size) 像素；高度预言使用其四个真实四分之一像素采样 UV。交点 C=1，偶奇校验对应 S=1／0；间隙 H=C=0、S=0.5 均精确。每个交点采样的上纱线必须高于下纱线，捕获 H 必须高于每个下纱线采样。稀疏四点高度预言绝对容差固定为 1/1024（小于 1 区间内两个 binary16 ULP 加 f32 运算误差）；不适用于精确端点、周期或重复检查。三实例几何参数及同模式原始场必须一致。flat 用例四尺寸所有像素最终高度精确 (0,0,0,1)，法线精确 (0.5,0.5,1,1)。这些探针不消除已接受的残余收窄，也不证明 PBR 外观。

### 公共控制隔离

257×129 的 plain 及计划中十六个单控制变体精确重复。列出的受影响交付通道必须改变，未列通道逐字节不变。三颜色仅改变 baseColor；两粗糙度端点仅改变 roughness；normalStrength 仅改变 normal；relief 仅改变 height／normal。每个 u32::MAX 纹理种子改变 baseColor／roughness，不改变几何高度／法线。数量、宽度、bevel、crown、underRatio 改变 baseColor／roughness／height／normal，metallic 不变。原始 H/C/S 有限归一化，非几何控制不改变它们，crown／underRatio 不改变 C。这只证明真实图的有界因果关系，不覆盖任意组合或浏览器原始半精度等价。

### 最终高度法线重放与周期边界

十二个计划用例 × 四尺寸将捕获的原始最终高度上传到既有生产 height-to-normal 内核。偏移 (0,0)、(1,0)、(0,1)、(floor(width/2),floor(height/2)) 的输出必须逐字节等于图法线的同样循环移位。CPU 仅重排字节，不计算法线。相对边缘像素不要求相等。这检查导数周期性，不等于视觉接缝接受或上游噪声周期性。

### 选定噪声输入与织纹平移

噪声为生产 value-noise v2，scale 4、octaves 2、persistence 0.5，种子 1729／65537／u32::MAX／0，覆盖四尺寸。未预先取模的输入原点 (width,0)、(0,height)、(width+3,height+5) 必须精确重现对应循环索引的原始半精度基线；基线必须非恒定、有限归一化。织纹平移覆盖十二用例 × 四尺寸的 H/C/S，整周期原点 (width,0)、(0,height)、(width,height) 与真实材质别名精确比较。它们仅覆盖选定输入，不证明任意图或浏览器周期性。评审者允许测试专用原点观测：只在测试副本中替换唯一匹配的采样原点表达式，基线的保留 uniform 字为零。生产公式、着色器文件、ABI、Core lowering 和公共 API 不变。按本次要求补充织纹周期加内部偏移 (width+3,height+5)，对照图场的循环索引；全部冻结用例、尺寸、整周期偏移和精确容差仍必须满足。

### 压力及失败边界

Dense-thin 使用原有 256²／1024²／257×129，检查 H/C/S 有限归一化、C=0 ⇒ H=0 且 S=0.5、相同几何及四偏移精确法线重放。仍不属于默认／变化质量保证；不新增欠采样交点质量承诺。任何冻结失败停止工作并记录用例、尺寸、场、像素、实际／期望值和适配器，不放宽容差。


### 阶段 B 定向命令

以下忽略测试由 `cargo xtask gpu-smoke` 的既有串行忽略测试选择自动运行。按上文设置 Vulkan 或 DX12 显式 GPU 环境。评审者已授权上述严格限定的采样原点观测。

```bash
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_crossing_structure -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_flat -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_control_isolation -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_normal_periodic -- --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib graph_gpu_woven_stress -- --ignored --nocapture
```

### 阶段 B 实现及本地结果

首提交 106546e 冻结探针契约，随后实现提交 ed6905fcbc18047e8ef639527923d67c07108f09。2026-10-04，GT 1030 Vulkan（NVIDIA 582.66）及 DX12（32.0.15.8266）均通过交点结构（每后端 640 交点及 640 间隙，含全部实例同模式相等）、flat（四尺寸）、控制隔离（十六变体）、法线重放（48 行 × 四偏移 = 192 次精确比较）、压力（三尺寸、十二次精确重放）。Vulkan／DX12 定向测试使用提交为 ed6905f 的实现字节（最初定向运行开始于该提交之前）；woven_tests.rs SHA-256 为 160dd5b0c600ee0926b9705ce99f4e9ff54a7333d3a3a0a7c7eeae66eebc2c1b。节点及完整 smoke 使用该实现提交；此后未改变运行时或配方。

四个构建器测试通过，包括冻结的新增阶段 B 节哈希及所有其余阶段 A 字段精确比较。Clippy、test-node weave-pattern、完整 Vulkan gpu-smoke 均通过（包括五个新增忽略探针、既有 painted 测试、Native／CLI 及打包 GPU 消费）。干净源码 release Native woven 矩阵在 Vulkan 上通过全部 51 行及四个耗时门槛。暖中位数：plain 1K／2K 为 183.82／791.17 ms；varied 为 228.61／901.08 ms。材质 SHA-256 仍为 95db023744a09224de3344613fe503c1e2187590b48667ef5ac199ae49959757；计划 SHA-256 为 392419a6fd210429d513fa0bf1a2e1300313c4299476d421b38b9cf834916523。普通日志及矩阵收据位于忽略的 tmp/woven-stage-b/，不是阶段 D 保留证据。

**dd6acf5 的历史状态**：阶段 B 当时尚未完成；选定噪声输入周期性及织纹整周期平移已冻结，但未实现／运行，等待观测范围澄清。既有节点周期测试不能替代这些材质专用探针。没有冻结探针失败，也未放宽容差。C／D 及材质接受仍待完成；本分支不宣称阶段 B 验收通过。

### 已授权的周期输入探针

评审者已解决观测范围问题。两个探针已有仅限测试的实现，结果全部通过后才能将阶段 B 标为完成。噪声探针复用 MAT-02 渲染／读回辅助代码，覆盖冻结的四参数集（包括 combined-low 的种子 0）。织纹探针先在零原点将各模式与真实图的原始场精确比较，再检查未预先取模的整周期及周期加内部偏移。非恒定基线和精确 raw-half 比较排除空泛通过。生产着色器字节、uniform 布局、lowering、配方默认值和计划全部字节不变。

```bash
cargo test --locked -p mixture-wgpu --test nodes node_fractal_noise_gpu_woven_periodic_inputs -- --exact --ignored --nocapture
cargo test --locked -p mixture-wgpu --lib node_weave_pattern_gpu_woven_periodic -- --ignored --nocapture
```

完整 gpu-smoke 自动选择两个忽略测试；test-node fractal-noise 与 test-node weave-pattern 显式包含对应材质探针。

### 冻结周期门槛失败 — 阶段 B 仍未完成

获批实现的测量绑定干净提交 47e2dd851303dccdee95b58f5da06cb391645978。GT 1030 的 Vulkan（NVIDIA 582.66）及 DX12 均通过 woven 噪声探针：每后端 48 次精确平移图像比较及 16 次零原点检查，覆盖冻结的四参数集和四尺寸。

织纹探针在 Vulkan **失败**：plain 用例、257×129、height 模式 H、未预先取模原点 (257,0)、像素 (59,0)。实际半精度字 [2149,0,0,15360] 与图基线 [2150,0,0,15360] 不同；标量为 0.0001341104507446289 与 0.0001342296600341797（差值 0.00000011920928955078125）。冻结容差为精确相等，因此判为失败，不能视作可容忍的视觉差异。失败前 plain 在 256²／1024²／2048² 的所有模式及原点共 45 次比较通过，奇数尺寸 H 的零原点比较亦通过。因此这是已验证零原点图身份后的整周期平移失败，不能作为完整材质周期性证据。

按停止规则，未运行其余织纹用例及 DX12 织纹探针，也未在此实现上重跑完整 gpu-smoke／check。运行探针之前，测试编译、clippy 和 xtask 编译通过。上文五探针 Vulkan／DX12 及完整 smoke／check 的通过属于前一实现，不能关闭本次失败。阶段 B 仍未完成；C／D 待完成，materialAccepted=false 不变，PR #84 保持 draft。

普通日志及源码／文件哈希收据位于忽略的 tmp/woven-stage-b/periodic/（vulkan-noise.log、dx12-noise.log、vulkan-weave.log、result.json）。生产着色器字节、Core lowering、ABI、材质默认值、计划字节和容差均与 dd6acf5 相同。已停止工作，未进行生产修复或放宽比较；后续处理需维护者评审这个失败探针。

### 维护者修订 — 2026-10-04：能精确处保持精确

维护者仅通过 structuralProbes.amendments（2026-10-04-exact-where-exact）修订织纹平移规则。原精确措辞及失败收据在上文和计划中完整保留：干净实现 47e2dd8、记录提交 8771d83，plain 257×129 H，原点 (257,0)、像素 (59,0)，half 2149 对 2150。维护者接受的分析是非二次幂分母下 fract(1+x) 的 f32 舍入，并非生产接缝；法线周期边界证据仍是独立门槛。该解释不授权任何更广泛的数值放宽。

- 所有尺寸的零原点身份仍逐半精度位精确。
- 两轴均为二次幂（256²／1024²／2048²）：每个整周期及周期加内部偏移仍逐半精度位精确。
- 仅 257×129：每分量最多相差一个相邻有限 binary16 步长。使用单调有符号排序（负数位取反，非负数位 XOR 0x8000）；-0／+0 相邻，超过一步的符号／零跨越失败。精确门槛仍比较位，包括有符号零。
- 按用例／尺寸／模式及原点报告不同分量／像素数量、最大半精度步长距离及最大绝对差。任何修订门槛失败即停止，明确记录未完成覆盖。

冻结计划其余内容全部不变。回归测试仅移除此命名修订条目后校验整个旧计划哈希，并保留阶段 A 和原始阶段 B 保护。修订探针通过之前，阶段 B 不算完成；C／D 待完成，materialAccepted=false 不变。

### 修订后探针结果 — 2026-10-04：超过一个 ULP 后停止

干净实现 8f84e7d7cd435a1760b5ed01531b66f07348af4e（先由 3aba3e6 冻结修订）在 NVIDIA GeForce GT 1030、Vulkan、NVIDIA 582.66 上失败。命令：`cargo test --locked -p mixture-wgpu --lib node_weave_pattern_gpu_woven_periodic -- --ignored --nocapture`。首个超限：plain 257×129 warp-share，原点 (257,0)，像素 (231,23)，实际 half [2492,0,0,15360]，期望 [2496,0,0,15360]，**相差 4 步，超过修订上限 1**。标量为 0.00017499923706054688 与 0.00017547607421875（绝对差 0.000000476837158203125）。未进一步放宽容差或修改生产实现。

plain 的全部 45 个二次幂尺寸比较（三尺寸 × 三模式 × 五原点）精确一致，差异分量数为零、最大 0 ULP。plain 奇数尺寸三模式的零原点恒等均精确。下表为奇数尺寸已观察统计；只有标量分量变化，所以差异分量数与像素数相同。跨原点计数为像素出现次数之和，并非去重后的瓦片位置数。

| plain 257×129 模式 | 原点 | 差异分量／像素数 | 最大 half 步数 |
|---|---|---:|---:|
| height | (257,0) | 93 | 1 |
| height | (0,129) | 87 | 1 |
| height | (257,129) | 162 | 1 |
| height | (260,134) | 177 | 1 |
| coverage | (257,0) | 153 | 1 |
| coverage | (0,129) | 299 | 1 |
| coverage | (257,129) | 445 | 1 |
| coverage | (260,134) | 443 | 1 |
| warp-share | (257,0)，失败图像 | 123 | **4** |

模式累计：height 519 处差异／最大 1 ULP；coverage 1340／最大 1 ULP；warp-share 仅截至首个平移图像为 123／最大 4 ULP。表内各平移图像的最大绝对差均为 0.00048828125；该最大值不一定与最大 ULP 位于同一像素。完整扫描已渲染的失败图像以保留统计后停止执行：56 次图像比较通过，第 57 次失败。其余 warp-share 原点、varied／combined 及所有其他用例，以及 DX12 织纹探针均未运行。

四个构建器测试（包括仅修订条目可变的完整计划保护）及比较器单元测试通过。按停止规则未重跑完整 Vulkan gpu-smoke 和 cargo xtask check。此前双后端 crossing、flat、control-isolation、normal-replay、stress 通过属于 ed6905f；双后端 noise-input 通过属于 47e2dd8。这些是未变更的历史证据，不代表本轮不完整运行通过。阶段 B 仍未完成，C／D 待完成，materialAccepted=false，PR #84 保持 draft。

普通输出位于忽略的 tmp/woven-stage-b/amendment/{builder.log,unit.log,weave-vulkan.log,result.json} 及 vulkan/woven-weave-periodic.json（逐图像及用例／尺寸／模式累计计数、适配器、首个失败）。这些是本地评审收据，不是阶段 D 保留证据。生产 WGSL、ABI、Core lowering、材质配方／默认值和其他冻结门槛均不变。

### 维护者第二次修订 — 2026-10-04：可见经线权重

structuralProbes.amendments 第二条保留原规则、第一次修订及干净实现 8f84e7d／记录 6991f46 的失败（plain 257×129 S，原点 (257,0)，像素 (231,23)，half 2492 对 2496）。任何重跑前，仅冻结奇数尺寸非零原点 warp-share 新门槛。所有尺寸零原点保持精确；二次幂尺寸的**三个原始模式（包括 S）**保持精确；奇数尺寸 H／C 保持一个 half 步长门槛。

S_i、C_i 是 share 与 coverage 实例相同像素、原点捕获的 half 值，P_i=f32(S_i*C_i)。u(x) 为非负存储值 x 两侧相邻 binary16 间距的较大者，u(0)=2^-24。对 i=1,2，定义 eSi=u(S_i)/2、eCi=u(C_i)/2，E_i=C_i*eSi+S_i*eCi+eSi*eCi。冻结：

**|f64(P_1)-f64(P_2)| ≤ B = E_1+E_2+min(1,max(S_1+eS1,S_2+eS2))*max(u(C_1),u(C_2))。**

E_i 由独立 half 存储舍入区间下的乘积展开得到；末项为已批准的一个 coverage 步长乘以区间内最大可能 share。系数严格为 1，没有根据四步差异拟合任何项。这是验收包络，不证明所有坐标扰动均满足它：coverage 误差本身不能约束几何 share 的独立变化，任何实测超限仍失败。非负 half 乘积可在 f32 精确表示（最多 22 位有效位，最小指数 -48）；将乘积转换为 f64 后比较并计算 B，不额外加入算术容差。操作数均须有限且位于 [0,1]。

奇数尺寸平移图像继续报告原始 S 差异，但不以它作为门槛。记录最大乘积差及差／上限比和相应像素，以及原始 S 差异数／最大步数。差与上限均零时比值为 0；正差配零上限失败。回归测试仅移除第二条修订并验证完整前一计划哈希，再保留原保护。用例、尺寸、配方默认值及其他门槛均不变；阶段 B 是否完成仍取决于结果。

### 第二次修订结果 — 2026-10-04：可见权重上限失败

干净实现 4af7e46b37c49baf73fc27948c571cd80cf7d104（先由 73bba64 冻结文档／计划），NVIDIA GeForce GT 1030 Vulkan、NVIDIA 582.66：相同织纹聚焦命令退出 101。plain 257×129 warp-share、原点 (257,0)、像素 (231,23) 的 S half 为 2492 对 2496，但**两侧 C half 均为 15360（1.0）**。因此该像素不支持上下文所说的微小 coverage 导致差异：乘以 coverage 后差异不变。小 S 不等于小 C。

P 差 = 4.76837158203125e-7；冻结 B = 4.618195816874504e-7；**差／B = 1.032518275775145**，也是完整扫描失败图像的最大比值，超过冻结包络。测量后没有修改任何项、系数或容差。

plain 的全部 45 次二次幂尺寸比较在所有模式下仍 raw-half 精确，奇数尺寸三个零原点恒等仍精确。奇数尺寸 H 四个平移累计 519 个差异像素出现次数、最大 1 half 步；C 为 1340、最大 1，逐原点计数与上文第一次修订表相同。首个奇数尺寸 S 平移图像有 123 个差异像素、最大 4 个原始 half 步。其最大绝对乘积差为像素 (231,20) 的 0.00048828125，此处 B=0.0016491413116455078，比值 0.29608211652450483；最大绝对差与最大比值不在同一像素。只测量了这一个 S 平移图像，其余原点／用例及 DX12 织纹均未运行。56 次比较通过，第 57 次失败。

四个构建器测试通过，包括仅第二条修订可变的完整计划保护及未变的第一次修订／原始保护。分析乘积比较器单元测试在修正手算期望常量后通过（冻结公式不变）；初始单元期望在 GPU 执行前失败。按停止规则未重跑完整 Vulkan gpu-smoke 和 cargo xtask check。其余五个结构探针保留此前 Vulkan／DX12 通过（ed6905f），噪声周期性保留此前双后端通过（47e2dd8），并非新重跑。阶段 B 未完成，PR #84 保持 draft，C／D 待完成，materialAccepted=false。

普通收据位于忽略的 tmp/woven-stage-b/amendment2/{builder.log,unit.log,weave-vulkan.log,result.json} 及 vulkan/woven-weave-periodic.json，后者记录每个已测用例／尺寸／模式／原点、原始计数、乘积统计及两侧 coverage。生产文件、ABI、Core、配方默认值及其他计划字段均不变。这是本地评审证据，不是阶段 D 接受记录。

### 维护者第三次修订 — 2026-10-04：奇数尺寸平移 share 仅作观察

重跑织纹探针前，维护者冻结 structuralProbes.amendments 第三条。原规则及修订 1／2 保留为不变历史，包括干净实现 4af7e46／记录 f9e0a51：plain 257×129 S，原点 (257,0)，像素 (231,23)，half 2492 对 2496，两侧 C=1，可见权重比值 1.032518275775145。此前微小 coverage 的解释有误。已接受的修正分析指出：陡峭的深度／可见性选择器 D（过渡宽度 2*bevel*(1-underRatio)）放大 fract(1+x) 的 f32 舍入误差。

所有尺寸／模式零原点恒等仍精确；二次幂尺寸所有平移在**三个原始模式（包括 S）**仍精确。奇数 257×129 H／C 仍不超过一个相邻 half 步长。仅奇数尺寸非零原点 S 与 P 改为**仅作观察：不在周期性质量保证内，绝不标为通过**。对每个用例／尺寸／模式／原点报告原始 S 差异数／最大步数、最大绝对 P 差及其相对未变的修订 2 包络的最大比值，附像素和值。有限归一化场及同原点 coverage 仍有门槛。此限制与材质接受分开，类似显式 stress 质量范围。

回归测试仅移除第三条修订后恢复完整前一计划哈希，再验证旧保护。全部用例、尺寸、配方修订 2 默认值、阈值及预算不变。阶段 B 仅可在其余受门槛约束规则通过后完成；每份结果摘要必须保留奇数尺寸 share 限制。

### 第三次修订结果及打包测试修复

阶段 B 受门槛约束的结构探针在记录的 GT 1030 Vulkan／DX12 范围内完成；**奇数尺寸平移 S／P 仍是未设门槛的实测限制，不是通过**。C PBR／人工评审、D 证据／接受记录仍另开 PR，materialAccepted=false。干净实现 30931b5（先于 69489d4 冻结第三次修订）以 `cargo test --release --locked -p mixture-wgpu --lib node_weave_pattern_gpu_woven_periodic -- --ignored --nocapture` 在双后端完成织纹矩阵：每后端 672 次受门槛约束比较通过、48 行仅作观察，覆盖十二用例、四尺寸及三个模式。二次幂平移及所有零原点恒等保持精确，奇数尺寸 H／C 不超过一个 half 步长。Vulkan 报告 NVIDIA 582.66；DX12 报告 GT 1030，driverInfo 为空。

其余五个结构探针双后端通过属于 ed6905f（交叉、零起伏、参数隔离、法线重放、stress）；噪声周期性双后端通过属于 47e2dd8。除共享测试读回代码迁移外，这些实现不变。PR 就绪前仍必须完成完整 gpu-smoke 及 check，最终运行结果记录于 PR。

下表为奇数 257×129 S／P 观察，每用例累计四个平移图像。计数是跨偏移的像素出现次数，不是去重瓦片位置；各最大值独立汇总，原始步数和乘积比值最大处不一定是同一像素。这些行**不属于周期性质量保证**。

| Case | Vulkan differing pixels | DX12 differing pixels | Max raw steps (both) | Max absolute ΔP (both) | Max ratio (both) |
|---|---:|---:|---:|---:|---:|
| plain | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| varied | 528 | 528 | 5 | 0.00048828125 | 1.5597867479055598 |
| no-detail | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| flat | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| warp-seed | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| weft-seed | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| combined-low | 181 | 180 | 5 | 0.00048828125 | 3.7911884487226954 |
| combined-high | 2964 | 2963 | 27 | 0.00048828125 | 6.802103515084418 |
| neutral-normal | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| constant-low | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| constant-high | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |
| max-normal | 556 | 553 | 4 | 0.00048828125 | 1.032518275775145 |

已复现 package-check 失败：隔离 lib 测试无法读取打包未包含的 tests/support/periodic_scalar_readback.rs。a7279b5 将同一辅助代码移至 src/periodic_scalar_readback.rs；executor 仅在 cfg(test) 下包含它，集成测试通过私有 path 模块复用。无新增公共 API 或重复实现。修复后第一次打包运行编译成功，但源码身份保护因并发源码编辑拒绝该次结果；随后稳定重跑通过打包 Rust 和 CLI CPU 消费。四个构建器测试通过，包括仅第三条修订可变的完整计划保护。

收据位于忽略的 tmp/woven-stage-b/amendment3/{package-before.log,package-after.log,package-stable.log,builder.log,weave-vulkan.log,weave-dx12.log,summary.json} 和 {vulkan,dx12}/woven-weave-periodic.json。每个观察行有 passed:null、显式质量范围、原始计数及带像素的 P 差／比值。生产 WGSL、ABI、Core、默认值、版本及 golden 不变。这些本地结果不是阶段 D 保留接受记录。

### 维护者第四次修订 — 2026-10-04：软件适配器尺寸范围

实现前冻结 structuralProbes.amendments 第四条，引用 671cf76 的 CI 运行 37185577047、制品 swiftshader-material-evidence / woven-weave-periodic.json。SwiftShader Device (LLVM 10.0.0) 在 constant-low 2048² 处取消前完成 582/720 行、零失败（07:23:46–08:07:53）。该部分收据**不是完整软件验收**。原规则及前三次修订保留为历史。

根据**实际报告的 adapter deviceType**选择：Cpu 使用 256²、1024²、257×129；其他类型保留 256²、1024²、2048²、257×129。不能仅凭环境变量缩减范围。保留原探针范围：交叉、零起伏、法线重放、所选噪声、织纹平移使用完整后端尺寸集；参数隔离仍仅奇数尺寸，stress 保留原三尺寸。较窄范围不属于新增软件省略。硬件完整矩阵探针必须在材质 2048² 缺失或未执行时失败。

每个探针收据列出冻结和已执行尺寸集；每个省略的 2048² 行标为 **notRunOnSoftware**、passed:null，附修订 id 2026-10-04-software-size-scope，绝不计为通过。硬件证据仍需 2048²；软件通过不能验收硬件像素。不改比较规则、容差、其他修订、阶段 A 门槛或工作流超时。

运行时间估算（非测量）：2048² 占四尺寸像素和的 78.522%。前 582 个平移行对应 788,155,143 个比较像素（九个完整用例，另加 256／1024／2048 的 15／15／12 行）；缩减后的完整矩阵为 206,507,700，比例 0.262014。将整个中断区间 44 分 07 秒计入此工作，得到约 11.56 分钟；加 main 所述 9–15 分钟，粗略**总作业目标为 20.56–26.56 分钟**，低于不变的 45 分钟限制。编译、调度、读回和其他探针不完全随像素数线性变化；软件时间与完成情况仅由 CI 验证。C／D 待完成、materialAccepted=false 不变。

硬件验证基于 6e95f1f：七项 release 定向探针在 NVIDIA GeForce GT 1030 Vulkan 与 DX12 上均通过受门槛约束的规则，预期名称子串为 `GT 1030`。回执确认交叉、平面、法线重放、噪声及织纹平移均执行四种尺寸；参数隔离保留 257×129，压力探针保留原三种尺寸。每后端织纹平移记录 672 项受门槛约束通过及 48 项仅观察行。三项尺寸策略单元测试和四项构建器测试通过，包含仅修订条目变化的计划保护。日志／回执位于忽略目录 `tmp/woven-stage-b/backend-sizes/`。SwiftShader 完整运行待 CI 验证，不声称本地软件结果。

## 阶段 C 冻结 PBR 评审计划

[评审计划](./pbr-review-plan.json)在工具实现和图像生成前冻结 plain、varied、combined-low、combined-high、warp-seed、weft-seed 六个 1024² 预设。每个预设包含平面／球体的 1×、3× 平铺及 4× 特写，以及五通道缩略图。复用 MAT-01 电介质 GGX：固定正交相机、600² 画布、位置缩放 2.5、视线 (0,0,1)，归一化主光 (-0.4,0.6,1) ×3、补光 (0.8,0.1,0.5) ×0.7、基色环境项 ×0.12。计划固定色调映射及 gamma 2.2；F0=0.04、metallic=0，粗糙度限于 [0.04,1]。高度仅展示，不置换几何。强制粗糙度 0/1/0 须改变明暗并精确重复，且不计入材质评审图。

输入必须是成功且源码干净的公开 Native／浏览器比较，源码、计划、构建器、修订、包及尺寸身份一致。回执绑定全部输入、截图及渲染器字节。生成器记录 humanAccepted／materialAccepted=false；只有独立且绑定源码的人工决定才能接受评审范围。代理检查不是人工决定。

### 生成绑定生产身份的织物视图

```sh
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/woven-review/browser
node scripts/browser-runtime/check-woven.mjs tmp/woven-review/browser tmp/woven-review/comparison
node scripts/woven-material-preview.mjs tmp/woven-review/comparison tmp/woven-review/pbr
node --test scripts/woven-material-preview.test.mjs scripts/woven-fabric-requests.test.mjs
```

先提交工具，构建、比较及预览均使用干净工作树；Native 比较使用 GT 1030 Vulkan，浏览器使用已安装 Chrome。各输出目录必须全新。织物工具通过唯一锚点变换复用未修改的砖材电介质 HTML（仅视图比例、粗糙度检查和布局）；MAT-01／MAT-02 源字节与历史回执不变。测试固定这些源码及评审计划。五个输入通道对照绑定生产身份的浏览器像素哈希和 Native／浏览器一致性校验；metallic 必须精确为零。六张 `<preset>-pbr.png`、三张独立粗糙度检查图、`index.html` 及 `preview.json` 绑定回执、渲染器、计划与图像哈希，并记录运行参数、适配器及浏览器身份。4× 视图放大固定投影而非改变材质频率。这是消费端可视化，不是另一图执行器、置换、布料模拟、新周期门槛或人工接受。

### 阶段 C 生成结果——保留原评审字节

评审设置在实现前由 44746d4 冻结。干净生产／工具修订 `6eb52a6a4de18ae0f8081e6314cff0215e249783` 构建候选并通过已安装 Chrome 154.0.8037.98 验证（23 项测试）。必须显式设置 `MIXTURE_BROWSER_CHANNEL=chrome`：首次调用使用默认 Chromium，已停止，其不完整输出不计入证据。`check-woven.mjs` 对 NVIDIA GeForce GT 1030 Vulkan 的 255 项比较全部通过（最大通道分量差 1/255）；冻结降采样及四项耗时门槛均通过。

生成文件位于忽略目录 `tmp/woven-review/pbr/{plain,varied,combined-low,combined-high,warp-seed,weft-seed}-pbr.png`、`index.html` 和 `preview.json`。比较输入为 `tmp/woven-review/comparison/`，浏览器验证为 `tmp/woven-review/browser/`。预览校验全部 30 对 Native／浏览器输入、精确零 metallic、强制粗糙度 0/1 的不同明暗，以及 0 的精确重复。回执记录生产／构建／渲染器身份、全部图像哈希，两个接受标记均为 false。它们原为阶段 C 临时输出；阶段 D 现保留原评审字节及所选收据，普通日志仍忽略。

代理检查发现视图和缩略图完整；combined-high 存在明显收腰轮廓，种子差异在评审图尺度下较细微。这些只是评审观察，不是人工接受决定或配方修改。后续维护者决定接受五个预设并排除 combined-high；见[阶段 D 记录](../../../docs/evidence/mat-03/README.zh-CN.md)。该决定未重新生成图像或配方。
