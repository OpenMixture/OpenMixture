# 编织织物 — 阶段 A 公开材质矩阵

[English](./README.md) | 简体中文

材质计划已冻结，节点已实现；**materialAccepted=false**。首个提交 f3d0f3f 冻结[计划](./qualification-plan.json)与[配方](./graph-proposal.json)，然后加入[普通 .mix v1 材质](./material.mix)及公开调用方工具。原草案逐字保留在 [qualification-plan-draft.json](./qualification-plan-draft.json)、[graph-proposal-draft.json](./graph-proposal-draft.json)。[graph-design.json](./graph-design.json)及原收据保留修订 4 基线。

[分阶段计划](../../../docs/mat-03-woven-surfaces.zh-CN.md)：A 为本 PR 的公开 Native／浏览器矩阵；B raw-half／结构／周期／法线重放／压力探针，C 介电 PBR 与人工决定，D 证据保留与接受记录，均另开 PR。本矩阵不构成结构、PBR 或人工接受。

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
| underRatio | 0.5 | 0.25..0.75 | warpshare_underRatio → n06-weave-warp-share.underRatio; coverage_underRatio → n09-weave-coverage.underRatio; height_underRatio → n17-weave-height.underRatio |
| detailAmount | 0.08 | 0..0.1 | warpDark = warpColor.rgb*(1-4*d); weftDark = weftColor.rgb*(1-4*d); roughnessMin = yarnRoughness*(1-2*d) |
| warpSeed | 1729 | 0..4294967295（整数） | warpSeed → n03-warpNoise.seed |
| weftSeed | 65537 | 0..4294967295（整数） | weftSeed → n00-weftNoise.seed |
| warpColor | [0.22,0.08,0.035,1] | RGBA 0..1; alpha=1 | warpColor → n07-warpColor.colorB |
| weftColor | [0.38,0.23,0.1,1] | RGBA 0..1; alpha=1 | weftColor → n10-weftColor.colorB |
| backingColor | [0.015,0.012,0.01,1] | RGBA 0..1; alpha=1 | backingColor → n12-backingColor.value |
| yarnRoughness | 0.8 | 0..1 | yarnRoughness → n14-yarnRoughness.outputMax |
| backingRoughness | 0.95 | 0..1 | backingRoughness → n15-backingRoughness.value |
| normalStrength | 0.5 | 0..1 | normalStrength → n20-normal.strength |
| crown | 0.5 | 0..1 | warpshare_crown → n06-weave-warp-share.crown; coverage_crown → n09-weave-coverage.crown; height_crown → n17-weave-height.crown |

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
