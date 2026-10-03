# 编织织物 — MAT-03a 配方修订 4（草案）

[English](./README.md) | 简体中文

[契约](../../../docs/mat-03-woven-surfaces.zh-CN.md)、[图设计](./graph-design.json)和[计划](./qualification-plan.json)共同定义本草案。决定 1 的家族／控制域已接受，完整契约未冻结。历史修订观察保留在契约和计划中。

## 配方修订 4 — 共用结构与减轻收腰（2026-10-03）

维护者确认修订 3（`793f73e96c5c7be6b1ff280d9a23b6d442198043`）的交叉接缝线和平顶已修复，但**未接受材质**。修订 4 仍为 draft、frozen: false、runtimeImplemented: false，仅使用既有节点。已接受的规则／变化平纹家族、数量／宽度含义及所有控制范围均不变。

P/R 表示带冠部的经／纬轮廓。Q 保留 mortarX=0.45、bevel=0.25 的平滑砖块选择器。归一化分层形状为 A=P*mix(underRatio,1,Q)、B=R*mix(1,underRatio,Q)，共用表面覆盖 C=max(A,B)，共用可见纱线选择 D=clamp((A-B+0.1)/0.2,0,1)。D 不需要有符号减法：levels 反转 B，scalar-blend 得到 (A+1-B)/2，再以 levels 将 [0.45,0.55] 映射至 [0,1]。max 继续由现有 subtract/blend/levels 合成。颜色=mix(backing,mix(weftColor,warpColor,D),C)；D 同样选择方向纹理生成纱线粗糙度，再以 C 混合纱线／底布粗糙度。高度=relief*C，法线由同一存储高度导出。因此高度、颜色边缘和粗糙度使用同一表面形状，取代独立的矩形颜色合成和覆盖并集。金属度保持零。C 是归一化表面权重，不是二值几何掩码或 alpha 透明度。

默认 underRatio 从 **0.5 → 0.25**，relief 从 **0.025 → 0.0125**，均处于原范围内。数量、宽度及其他默认值不变。窄冠部正下限从 0.4 → 0.65，上限仍为 1，保留横向 mortar 0.45、bevel 0.25 和沿轴平移半个 tile 的 max 构造。这减小肩部与冠部差异；W/F 仍决定外部支撑域和间距。降低下层纱线可减少其对上层肩部的切入，减小 relief 可降低鼓包感。解析交叉中心 P=R=1、Q=0/1，A/B 交替为 1/underRatio，D=0/1；relief>0 时上层严格高于下层。混合采样足迹和 half 量化仍须单独验收。纹理仍仅作用于颜色／粗糙度。relief=0 时保留织物颜色／粗糙度，而高度为零、法线中性。由于 C/D 共用层序，underRatio 现影响所有非金属度通道；relief 仍仅影响高度／法线。

### 绑定源码的评审与对齐观察

修改前使用干净分支 `793f73e`；修改后使用相同引擎及修改的草案图／默认值。二者使用从引擎 main `e73e2b99c85987551d11db78eb90dfbf0564100a` 重新构建的同一 release CLI。计划的 `reviewMeasurementsRevision4` 记录图／计划／二进制／解析输入哈希、命令、适配器及观察。附加记录后的计划晚于被测快照；图字节一致。八个用例均验证无诊断，并在两个尺寸以完整五通道编译通过。plain／varied 的 1K 渲染在 **NVIDIA GeForce GT 1030／Vulkan／NVIDIA 582.66** 上成功，显式使用 `--backend vulkan`。

| 完整五通道；plain、varied、combined-high（其他五用例亦相同） | Passes | 物理纹理 | peakBytes |
|---|---:|---:|---:|
| 修改前修订 3，1024² | 53 | 14 | 125,830,464 |
| 修改前修订 3，2048² | 53 | 14 | 503,317,824 |
| 修改后修订 4，1024² | 52 | 10 | 92,276,064 |
| 修改后修订 4，2048² | 52 | 10 | 369,100,128 |

上限仍为 **64 passes／536,870,912 字节**，2K 编译阻碍继续解除。逻辑字节为 436,207,616／1,744,830,464，不是调度峰值。减少重复合成并使用显式数字节点 ID 顺序，降低中间量存活数量；未改调度器／分配器。

通过同一 release Vulkan 执行器运行两个 plain-1K 中性探针检查对齐。C 探针将两种纱线颜色设为白色、底布黑色，纱线粗糙度 1／底布 0，并通过 height 输出 C。D*C 探针将经线白色／纬线黑色，warpGrain=1／weftGrain=0，粗糙度渐变 0..1／底布 0，并输出 scalar-mask-blend(0,D,C)。每个生产 baseColor 与相同标量参考的独立 gradient-map 比较（保留 sRGB 编码），roughness 与线性标量参考比较。**两个探针的 RGB 最大字节差均为 0，差异像素均为零**，覆盖完整 1024² 和 x=64、y=64、256×256 裁剪。该检查验证真实共用覆盖／选择边缘；法线是导数，不是相同掩码。不代表任意颜色、raw-half 精度、浏览器一致性或完整控制矩阵验收。

另以归一化线性 RGBA8 渲染独立纱线形状，进行有界收腰观察。在 x=[144,160,176,192,208,224,240] 上统计 y=0..127 中 weft>warp 且 weft>13/255 的像素：修改前宽度 [76,72,50,46,50,72,76]，修改后 [80,78,66,64,66,78,80]。中心／近端比例由 46/76（约 0.605）升至 64/80（0.8），符合收腰减轻的观察；这不是冻结的几何宽度容差。对应实际裁剪显示颜色／粗糙度跟随圆润交叉形状，高度鼓包感降低。圆端及风格化织物外观仍有评审风险，尤其是非默认控制。**维护者视觉接受继续待定**，不提新节点。更窄／多采样选择器和高度平移尝试因改进不足而舍弃；多采样版本引入条带。

评审 `tmp/mat03-recipe-review-4/after/{plain,varied}-1024/{baseColor,normal,roughness,height}.png`。plain 另含 `{baseColor,normal,roughness,height}-crossing-256.png`，原生 x=64、y=64、width=256、height=256，无缩放或色调调整。对应修改前图／裁剪、输入快照、回执、探针和解析器保留在忽略目录，未提交像素或工具。高度因原始线性范围较小而有意保持偏暗。这些是本地评审观察，不是持久人工接受、PBR／耗时验收、新的 2K 渲染或完整结构／采样／一致性证据。

## 调用方映射与复现

将用例控制覆盖到默认值，并按契约范围验证；拒绝奇数数量、未知、非有限或越界控制。`$` 是调用方占位符，不是 Core 表达式。解析 warpGap=1-warpWidth、weftGap=1-weftWidth、halfWarpCount=warpCount/2、crossingOffsetX=0.5/warpCount、profileBevel=0.19+0.5*bevel、warpDark/weftDark=相应颜色 RGB*(1-4*detailAmount) 且 alpha=1、roughnessMin=yarnRoughness*(1-2*detailAmount)。其他占位符直接使用同名控制（包括 underRatio）；固定种子为零。旧 detailMin/underHeight 已不再使用。

保留数字节点 ID；数组重排不改变 Core 字典序调度。输出普通 `.mix v1`：复制节点 id/type/version/解析后的 parameters，将 inputs 的 source.port 转成 from/to 边，追加 material-output@1（id=material），将 outputs 接入对应通道。排除设计字段和占位符。观察别名 warpShape/weftShape 指归一化 A/B，不是已乘 relief 的高度；coverage=C、surfaceOrder=D。历史基线使用 793f73e 的图／默认值／映射。

```text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <case.mix> --json
mixture inspect <case.mix> --plan --size <1024|2048> --output baseColor,normal,roughness,metallic,height --json
mixture render <plain|varied.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
```

这里 mixture 为 `target/native-consumer/release/mixture.exe`。使用新的忽略目录，分别保留 JSON、stderr、退出码和输入快照。两个中性探针按上文覆盖解析节点常量／输出；详细命令和输入哈希在计划中。无需新增仓库工具。

## 剩余冻结门槛

仍须完成全部结构／独立控制／周期及奇数尺寸探针、raw-half 法线重放、下采样／压力、Native／浏览器 <=1/255、精确重放／包等价、耗时、材质回归、公开包消费、PBR／人工接受、六项检查及目录／版本评审。按[证据政策](../../../docs/evidence-policy.zh-CN.md)保留人工评审字节后方可宣称接受；本地探针不能替代这些门槛。
