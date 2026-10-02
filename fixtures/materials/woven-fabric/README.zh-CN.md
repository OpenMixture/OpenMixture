# 编织织物 — MAT-03a 设计草案

[English](./README.md) | 简体中文

[契约](../../../docs/mat-03-woven-surfaces.zh-CN.md)、[图设计](./graph-design.json)及[验收计划](./qualification-plan.json)均为 **draft，未冻结**，`runtimeImplemented: false`。本切片不提交可执行材质、请求构造器或编织测试命令。忽略的 `tmp/` 中现有临时解析文档及两次 Native 评审渲染；它们不是接受像素或完整材质实现。现有节点已实现，不代表此材质已实现。`$` 值是调用方占位符，不是 `.mix` 语法或运行时表达式语言。

## 构造与调用方映射

配方按依赖顺序列出全部 40 个节点实例、精确类型／版本、输入和五个输出别名，仅使用当前目录节点。契约解释直接 checker／brick／morphology 尝试的不足及既有组合如何解决；尚未证明渲染质量，也不足以接纳新节点。

将每个用例覆盖到默认值上，按契约表拒绝未知、非有限、越界控制，并要求轴数量为偶数。在未来构造普通 `.mix v1` 文档前解析：

| 占位符 | 调用方映射 |
|---|---|
| warpGap、weftGap | 1-warpWidth、1-weftWidth（各 0.1..0.45） |
| halfWarpCount | warpCount/2（整数） |
| crossingOffsetX | 0.5/warpCount |
| detailMin | 1-detailAmount |
| underHeight | relief*underRatio |
| 其他占位符 | 同名已验证控制；无随机变化的轮廓／选择器节点显式使用零种子 |

经线列数和纬线行数分别确定间距，gap 控制独立相对宽度，UV 宽度等于 width/count。每个轮廓通过饱和减法、均匀半权重混合及 levels 合成 max，组合砖块与沿轴半瓦片平移后的自身。Q 是平移后的错列 Scalar 砖块场，偶奇性为偶时经线上层。两份噪声均使用 value v2、scale 2、单 octave、整数方向变换 4×1，纬线旋转四分之一圈。所有最终生产像素必须只由 wgpu 执行。

图通过观察别名暴露 W/F/Q、两层独立高度及覆盖。两种颜色叠放顺序共用轮廓和 Q；粗糙度使用覆盖，金属度使用零，法线消费最终高度。每个中间节点均有 half 存储；别名和数学公式不是另一执行器。

## 草案矩阵与预算

八个基础用例 × 四尺寸 × 五通道，每个 Native／浏览器配对共 160 次通道比较，另加独立控制扫描和压力用例。扫描在 plain 默认值上每次只更改一个命名控制，颜色替换只更改对应颜色。独立 dense/thin 压力用例覆盖三个尺寸（另 15 次通道比较）。combined-high 仍必须满足有限值／结构／一致性门槛；只有 plain 与 varied 具有草案 4/255 降采样保证。压力用例不能替代任一质量用例。

全部用例要求精确重放、包等价及 <=1/255 跨运行时分量误差。零起伏必须产生零高度和中性法线；所有用例金属度为零。契约另外要求原始高度法线重放、周期平移探针、独立轴／种子因果性及 PBR／人工评审。计划中的冻结阻碍尚未解决；建议断言不是已实现工具。

应以复用后的实际 plan-v3 编译期峰值为准：**2K 的 570,426,320 字节超出 536,870,912（512 MiB）而失败**。此冻结阻碍必须通过配方变更或单独划定范围的实测 PERF-MAT 切片解决，绝不提高上限。历史全保留纹理算术 40×2048×2048×8 = 1,342,177,280 字节仅作背景；它不是调度后的描述符峰值，也不是冻结决定的依据。64-pass 目标和独立硬件／软件耗时目标仍为草案；本评审不验收耗时，也不实现优化。

## 已复现的评审观察与命令

2026-10-02 测量的干净分支为 `7555c8b25558fb7a2aedd2a14adc2474b6562006`，基于 main `5785068d8d3e49a503bfe30cb1d90d28f0bc548e`。[计划](./qualification-plan.json)的 `reviewMeasurements` 保留 release 二进制／输入哈希、修订前原始计划身份、解析文档哈希、精确 CLI 参数／退出码及输出哈希。本次修订不是被测源码。

无需提交解析器即可复现：使用该源码版本的图与计划，合并默认值／用例控制，并递归应用上表参数映射。输出 `{version:1,nodes,edges}`：复制各节点 id／type／version／已解析 parameters；把设计中 `inputs` 的 `source.port` 引用转换为 `from:{nodeId:source,portId:port}` 与 `to:{nodeId:target,portId:inputName}` 边；追加 ID 为 `material` 的 `material-output@1`，将每个设计输出别名接到相应材质通道。不要在 `.mix` 保留设计专用字段或占位符。把五个解析用例写入新建的忽略目录。以下 `mixture` 指 `target/native-consumer/release/mixture.exe`；stdout JSON 与 stderr 分开保存并检查退出码：

```text
cargo build --release --locked -p mixture-cli --target-dir target/native-consumer
mixture validate <case.mix> --json
mixture inspect <plain.mix> --plan --size 1024 --output baseColor,normal,roughness,metallic,height --json
mixture inspect <plain.mix> --plan --size 2048 --output baseColor,normal,roughness,metallic,height --json
mixture doctor --backend vulkan --json
mixture render <case.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json
```

plain、varied、flat、combined-low、combined-high 全部验证通过，各零诊断（退出 0）。plain 1K 检查编译出 **40 passes、16 个物理纹理、peakBytes 142,607,312、logicalTextureBytes 335,544,320**（退出 0）。plain 2K 检查返回 **MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED**，configured **536,870,912**、observed **570,426,320**（退出 2，未执行 GPU）。成功复现预期拒绝不等于通过材质预算。

plain／varied 的 1K 五通道渲染在 **NVIDIA GeForce GT 1030／Vulkan／NVIDIA 582.66** 上成功。代理观察原始 baseColor／normal 贴图，可见交替交叉及 varied 的 12×8 数量／不等宽度。横截面读作窄斜边的平顶木板；默认 detailAmount=0.03 只缩放 relief=0.025，未见清晰方向纹理。这些是决定 2 下尚未解决的质量风险，不证明缺少节点或完整独立控制因果性。本次不提新节点。

临时解析器、生成图、原始输入快照、原始 JSON／stderr、回执和十张 PNG 保留在忽略的 `tmp/mat03-review-amendment/`。这是本地评审保留，不是持久接受证据；哈希标识字节但不保证可获取。未执行 PBR／人工接受、Native／浏览器一致性、DX12／软件验收、耗时门槛或完整结构／奇数尺寸探针。

## 证据与下一次评审

后续冻结 PR 前先阅读契约中的五项维护者决定。有限 Native 评审现已记录 2K 预算拒绝及视觉风险；解决这些问题并完成剩余公开 Native／浏览器可行性工作，保留源码／请求／构建／适配器身份及失败结果。不能仅因配方较大便提出新节点。经证明的缺口需要单独的最小身份／ABI／版本评审。验收构建使用 `target/native-consumer`，普通输出使用忽略的 `tmp/` 或 CI artifact。按[证据政策](../../../docs/evidence-policy.zh-CN.md)保留接受图像和人工决定。本次不改变既有黄金图、清单、运行时或 Studio 内容。
