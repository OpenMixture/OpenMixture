# 编织织物 — MAT-03a 发现与节点提案（草案）

[English](./README.md) | 简体中文

维护者于 2026-10-03 评审 ca2e98b 的配方修订 4 后改变方向。四轮既有节点尝试从平顶／不可见纹理、交叉接缝、通道不匹配及骨头／绗缝形状，变成对齐但看似断开的胶囊。[契约](../../../docs/mat-03-woven-surfaces.zh-CN.md)记录有界发现及解析节点提案；这不证明不可能，也不授权实现。继续保持 **draft、frozen: false、runtimeImplemented: false、materialAccepted: false**。

## 区分基线和提案

- [graph-design.json](./graph-design.json) 原样保留修订 4 既有节点基线。[qualification-plan.json](./qualification-plan.json) 的顶层默认值／用例／独立扫描及实测资源字段仍描述该基线。保留四份绑定源码的评审记录及输入身份；本设计变更不宣称新 GPU 运行。
- [graph-proposal.json](./graph-proposal.json) 单独描述**提议的 weave-pattern@1**，当前目录尚未实现。不得当作已验收／接受材质运行；其推算不是已编译计划或渲染测量。
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
