# 编织织物 — MAT-03a 配方修订 2（草案）

[English](./README.md) | 简体中文

[契约](../../../docs/mat-03-woven-surfaces.zh-CN.md)、[图设计](./graph-design.json)与[验收计划](./qualification-plan.json)仍为 **draft、frozen: false、runtimeImplemented: false**。决定 1 已于 2026-10-02 接受：不透明平纹加一个变化平纹，偶整数数量 4..32，宽度为间距的 0.55..0.9；无斜纹／其他组织。完整契约未冻结。维护者选择通过既有节点配方改进内存和视觉，不授权新节点或 PERF-MAT 引擎工作。

## 配方与调用方映射

修订 2 含 45 个既有节点实例和五个输出。节点 ID 的数字前缀为 Core 既有字典序就绪节点调度器建立合法顺序，仅重排 JSON 数组不会改变调度。此具体配方在增加纹理合成时将物理槽位从 16 降至 14，不是引擎优化器。复现测量时保留 ID 和引用。

将用例覆盖到默认值，按契约不变的控制范围验证，拒绝奇数数量、未知／非有限／越界控制。`$` 是调用方占位符，不是运行时表达式。解析：

| 占位符 | 映射 |
|---|---|
| warpGap、weftGap | 1-warpWidth、1-weftWidth |
| halfWarpCount、crossingOffsetX | warpCount/2、0.5/warpCount |
| profileBevel | 0.19+0.5*bevel；公开 bevel 0.02..0.12 映射为砖块 0.20..0.25 |
| detailMin、underHeight | 1-detailAmount、relief*underRatio |
| warpDark、weftDark | 相应线性颜色 RGB × (1-4*detailAmount)，alpha=1 |
| roughnessMin | yarnRoughness*(1-2*detailAmount) |
| 其他占位符 | 同名控制；无随机变化的轮廓／选择器仍显式使用固定零种子 |

两轴轮廓保留错相砖块／max 构造。更宽 bevel 产生更圆润肩部，但不是精确圆形，宽纱线仍有部分平顶。数量独立决定间距，宽度是间距比例。Q 保留交替上下选择。

默认 detailAmount 现为 0.08（原为 0.03），范围仍为 0..0.1。两份显式种子的 value-v2 噪声使用 scale 4、octaves 2、persistence 0.5，再进行 8×1 变换；纬线 quarterTurns=1。levels 将 0.3..0.7 映射为 0..1。各自纹理驱动既有高度衰减，以及从暗纱线颜色到完整颜色的 gradient-map。Q 选定纹理将纱线粗糙度在 roughnessMin 与 yarnRoughness 之间变化，再通过覆盖混合底布粗糙度。detailAmount=0 时颜色及纱线粗糙度不调制，所需种子仍保留。颜色／粗糙度控制继续隔离在对应通道，种子不移动 W/F/Q。完整公开因果性测试仍待完成。

观察别名包含 W/F/Q、max 前的两层高度、覆盖和两份纹理。法线使用最终存储高度，金属度保持零。所有生产像素仍由 wgpu 执行，每次 pass 使用 f32 计算及 half 存储。

## 绑定源码的前后评审

两次运行均使用干净引擎 main `e73e2b99c85987551d11db78eb90dfbf0564100a` 和构建到 `target/native-consumer` 的同一 release CLI。after 图／计划是临时候选输入，哈希绑定在 `recipeIteration`；随后附加记录的计划不是被测输入。当前图字节与 after 图一致。历史 `reviewMeasurements` 继续绑定修订 1 及旧源码，不指代修订 2。

前后八个计划用例均验证通过、零诊断。各用例完整五通道检查结果为：

| 配方／尺寸 | Passes | 物理纹理 | peakBytes | 结果 |
|---|---:|---:|---:|---|
| 修改前／1024² | 40 | 16 | 142,607,312 | 通过 |
| 修改前／2048² | — | — | observed 570,426,320 | MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED，退出 2；plan 为 null |
| 修改后／1024² | 45 | 14 | 125,830,240 | 通过 |
| 修改后／2048² | 45 | 14 | 503,317,600 | 通过 |

上限保持 **536,870,912 字节**，pass 上限仍为 64。这些请求的编译期 2K 阻碍已解除，未提高上限或修改运行时。修改后逻辑纹理字节为 377,487,360（1K）、1,509,949,440（2K）；全保留算术不决定峰值。基线 2K 编译被拒绝，因此不能渲染。

plain 与 varied 均在 **NVIDIA GeForce GT 1030／Vulkan／NVIDIA 582.66** 上以 1024² 和 2048² 五通道渲染，四次全部成功。代理观察到更宽的弯曲肩部和默认颜色／粗糙度中的纵向纹理。仍有部分平顶，较宽纬线尤为明显；尚未证明精确圆截面或最终织物质量。**维护者视觉接受仍待定。** 不主张 PBR 预览、浏览器一致性、DX12／软件矩阵、耗时验收或完整结构／压力门槛。

## 复现与图像

按上表解析选定修订的配方／默认值。输出 `{version:1,nodes,edges}`：复制 id／type／version／已解析 parameters，将 `inputs` 中 `source.port` 引用转换为 `from:{nodeId:source,portId:port}` → `to:{nodeId:target,portId:inputName}` 边；追加 ID 为 `material` 的 `material-output@1`，把五个输出别名连接到对应通道。不要在 `.mix` 保留设计专用字段或 `$` 占位符。修改前配方使用 main `e73e2b9` 及其原始映射。使用新建忽略目录，分别保留 stdout JSON、stderr 和退出码。

```text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <case.mix> --json
mixture inspect <case.mix> --plan --size <1024|2048> --output baseColor,normal,roughness,metallic,height --json
mixture render <plain|varied.mix> --size <1024|2048> --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
```

这里 `mixture` 指 `target/native-consumer/release/mixture.exe`。临时解析／搜索辅助脚本位于忽略的 `tmp/mat03-recipe-review/`，均未提交，也不是新仓库命令。评审原始通道 PNG：

- `tmp/mat03-recipe-review/after/plain-1024/{baseColor,normal,roughness,height}.png`
- `tmp/mat03-recipe-review/after/varied-1024/{baseColor,normal,roughness,height}.png`
- 对应 `plain-2048`／`varied-2048` 目录含四通道及 metallic；基线 1K 图像／报告位于 `baseline/`。

各运行目录保留输入快照、解析图、原始报告、回执和 PNG 哈希；较早被拒绝的候选调度也保留在忽略目录内。未修改预览工具，原始通道 PNG 即评审产物。这些是本地评审文件，不是持久人工接受证据；哈希不保证未来可获取。声明接受前须按[证据政策](../../../docs/evidence-policy.zh-CN.md)保留评审字节和真实决定。

## 剩余冻结门槛

计划仍覆盖八用例 × 四尺寸 × 五通道（每个 Native／浏览器配对 160 次比较）、独立控制扫描及三个尺寸的 dense/thin 压力用例。只有 plain／varied 具有拟议 4/255 降采样保证，不能以压力用例豁免失败。精确重放／包等价、包括法线在内的 <=1/255 跨运行时分量误差、零起伏／金属度、交叉／轴／种子因果性、原始高度法线重放、周期／奇数尺寸探针、降采样和压力仍待验收。后续冻结前完成耗时、材质回归、公开包消费、PBR／人工评审、六项检查及目录／版本评审。家族／控制域已接受，材质与契约尚未接受。
