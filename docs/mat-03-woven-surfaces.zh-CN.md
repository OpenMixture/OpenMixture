# MAT-03a — 有界编织表面（草案）

[English](./mat-03-woven-surfaces.md) | 简体中文

状态：**draft，未冻结；runtimeImplemented: false；materialAccepted: false**。维护者在 MAT-02 验收后选定 MAT-03，目前只启动 MAT-03a 契约设计。本切片遵循 MAT-02a PR #53 与 #55 的准备／评审分离方式，不选定新节点、不实现材质，也不授权实现。[路线图](../ROADMAP.zh-CN.md)负责范围；[夹具指南](../fixtures/materials/woven-fabric/README.zh-CN.md)、[图设计](../fixtures/materials/woven-fabric/graph-design.json)和[验收计划](../fixtures/materials/woven-fabric/qualification-plan.json)均为待评审草案。

## 材质简述与坐标

一个不透明介电织物家族：规则平纹，以及具有不同轴密度／宽度、两种纱线颜色和显式种子纵向纹理的变化平纹。“变化”仍使用同一种交替组织，不是斜纹或第二套编织目录。经线沿图像 v（纵向），纬线沿 u（横向）；原点左上，u 向右、v 向下。一个归一化 UV 瓦片沿两轴重复。经纬数量均为偶数，以保持交叉相位周期。数量决定中心间距（1/count），宽度另以对应轴间距的比例控制。不承诺物理毫米或网格密度。

交点 (i,j) 在 (i+j) 为偶数时经线上、纬线下，奇数时相反。更改宽度、间距、颜色或种子不能交换此顺序。高度是零基准表面起伏，不是双面几何。输出 baseColor、roughness、metallic=0、height，以及仅从最终存储高度生成的 normal。间隙显示不透明暗色底布；排除 opacity、AO 和 displacement 输出。方向纹理体现在贴图中；当前法线／粗糙度通道不提供各向异性 BRDF。

## 先检查既有节点可行性

审阅基线为 main `5785068`、[注册表](../crates/mixture-core/src/registry.rs)、[实际节点契约](./node-contracts.zh-CN.md)、[MAT-01 轮廓公式](./mat-01-structured-materials.zh-CN.md)和生产[砖块着色器](../crates/mixture-wgpu/shaders/nodes/brick-pattern.wgsl)。下表为原始源码／契约推导；下方绑定源码的 Native 评审补充有限渲染观察，不构成材质验收。MAT-02 的[输入测量](./evidence/mat-02-input-feasibility/README.zh-CN.md)提供后续绑定源码测量的先例，不构成织物证据。

| 尝试／实际契约 | 具体局限 | 既有节点构造／剩余风险 |
|---|---|---|
| `checker@1` 提供格子索引和的奇偶性。 | 输出为 Color，目录没有 Color 转 Scalar；不能连接 Scalar 高度／遮罩端口。整数像素索引也不同于轮廓子采样。 | 不连接错误端口，不增加隐式转换；使用错列 Scalar 砖块场选择交叉顺序。 |
| `brick-pattern@1` 使用 x/y 内向距离的最小值、单个 bevel、格子振幅和四个四分之一像素样本。 | 单个长砖不是连续圆滑纱线：即使 rows=1、mortarY=0，bevel>0 仍使 v=0 处降至零；bevel=0 又会移除横截面平滑。variation 按砖块变化，不是连续纵向纹理。 | 将沿纱线方向错开半瓦片的两份轮廓取 max，可消除底层连续场的轴向端部轮廓。bevel<=0.25 时，总有一份的轴向内向距离>=0.25。但四样本均值、双线性移相及 half 存储使此代数不能直接等同于实测像素保证。 |
| 错列 `brick-pattern@1`：columns=warpCount/2、rows=weftCount、rowOffset=0.5。 | 普通砖块高度自身不能表达两层纱线交叉。 | 源 X 平移 0.5/warpCount，在交点中心得到交替 Q=1/0；mortarX=0.45、mortarY=0、bevel=0.025、variation=0。它固定交替选择，不提供任意组织。行边缘 bevel 提供过渡，其平滑性和奇数尺寸重采样仍需验收。 |
| `scalar-mask-blend@1` 是钳制的逐点插值，`scalar-subtract@1` 是饱和减法。 | 二者都不生成空间顺序；单独 `mix(A,B,Q)` 会压低不重叠的可见纱线，也不是 max(A,B)。 | 先构造轮廓。max(A,B) 合成为 D=subtract(A,B)，M=scalar-blend(B,D,weight=0.5)，再 levels(M,inputMin=0,inputMax=0.5,gamma=1)。实数下等于 max，但额外两次 half 舍入必须测量，尤其在生成法线前。 |
| `scalar-morphology@1` 是整数半径 0..16 的周期轴向 min/max。 | 腐蚀／膨胀以像素改变支撑域和宽度，可能移除细线，不能恢复丢失的亚像素形状或生成交叉奇偶性。 | 本配方不需要；不能将其视为连续宽度轮廓或抗锯齿替代品。 |
| `fractal-noise@2` value 基函数只有一个各向同性整数 scale、显式种子、周期格点，无抗锯齿。 | 单独使用不能产生独立轴纹理；本设计不覆盖 cellular/v1。 | `transform-2d@1` 的 scaleX=4、scaleY=1 使纹理沿 v 拉长，quarterTurns=1 转为纬线方向；两个显式种子独立控制纹理。整数缩放保持周期，但双线性重采样不是预过滤。 |
| `levels`、常量、颜色 `blend`、Scalar 混合和 `height-to-normal@1`。 | 逐点操作不修复上游接缝／混叠；法线微分可能放大 f16 台阶。 | 提供染色、有界起伏、粗糙度及法线合成，不新增通道或执行器。测试真实存储高度，而非 RGBA8 重建。无需 `warp@1`，其折叠／精度风险也不能解决这些缺口；`image-input@1` 只会导入调用方制作的结构，不能证明程序化可行性。 |

结论：现有目录在契约／代数层面可以构造所需布局、交点中心的交替顺序和方向细节。上述直接单节点尝试确有失败，但**不足以证明必须新增节点**。本切片不提出候选身份／版本或 ABI。下方 Native 评审暴露了编译期 2K 预算失败及未解决的视觉风险。连续覆盖、完整交叉区域、f16 法线质量、采样及完整验收仍未证明。冻结前必须解决这些失败，保留后续测量及通过结果。若合理的既有节点组合仍有有界失败，后续评审才可提出最小节点，明确端口、范围、ABI 和目录／版本影响。不得预先接纳编织生成器，也不得将缺少测量当作能力缺口。

### 绑定源码的评审复现

2026-10-02 在本次修订前，测试了干净分支版本 `7555c8b25558fb7a2aedd2a14adc2474b6562006`，对应 main `5785068d8d3e49a503bfe30cb1d90d28f0bc548e`。通过 `cargo build --release --locked -p mixture-cli --target-dir target/native-consumer` 新构建 release CLI。计划的 `reviewMeasurements` 记录二进制、锁文件、原始图／计划及已解析文档 SHA-256、精确 CLI 参数数组和退出码；修订后的计划不是被测输入。按夹具映射合并默认值与用例，替换所有 `$` 值，把节点输入引用转为普通 `edges`，并将五个输出连接到 `material-output@1`，生成 `.mix v1` 文档。临时解析器不是仓库工具。

- `mixture validate <case.mix> --json` 对 plain、varied、flat、combined-low、combined-high 全部通过，无诊断。
- `mixture inspect <plain.mix> --plan --size 1024 --output baseColor,normal,roughness,metallic,height --json` 编译 plan v3：**40 passes、16 个物理纹理、peakBytes 142,607,312、logicalTextureBytes 335,544,320**。
- 同一请求改为 `--size 2048`，退出码 2，诊断 **MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED**，configured **536,870,912**，observed **570,426,320**。这是已应用 plan-v3 复用后的编译期描述符预算失败，未发生 2K GPU 分配／渲染。当前配方**不满足草案 <=512 MiB @2048² 预算，阻止冻结**。只能通过配方变更或单独划定范围、以测量为依据的 PERF-MAT 切片解决，**绝不能提高上限**；CLI 关于调用方上限的通用建议不授权修改本材质门槛。
- `mixture render <case.mix> --size 1024 --output baseColor,normal,roughness,metallic,height --backend vulkan --out <fresh-directory> --json` 对 plain 与 varied 均成功，实际适配器为 **NVIDIA GeForce GT 1030／Vulkan，NVIDIA 驱动 582.66**。观察原始 baseColor／normal PNG 可见上下交替及 varied 的 12×8 数量／不等宽度。这是布局视觉证据，不是完整独立控制因果性测试。横截面读作窄斜边的平顶木板，而非圆润纱线；默认纹理未见清晰方向性。细节只衰减起伏：默认 detailAmount=0.03 作用于 relief=0.025，不改变纱线颜色。这些是决定 2 下尚未解决的质量风险，不证明需要新节点。

原始报告、解析器、五份解析图、回执和十张 PNG 位于忽略的 `tmp/mat03-review-amendment/`，属于本地评审输出，不是持久接受证据。保留的计划摘要及[夹具复现步骤](../fixtures/materials/woven-fabric/README.zh-CN.md)将观察绑定到被测源码。不主张浏览器／DX12／软件对照、耗时验收、完整结构探针、PBR 或人工接受。

## 草案控制、合成与精度

夹具指南负责精确调用方映射，JSON 负责默认值／用例。这些是普通调用方准备，不是 Core 执行的表达式或 MAT-04 可复用图 API。

| 控制 | 建议闭区间／默认值 |
|---|---|
| warpCount、weftCount | 偶整数 4..32／8、8；间距分别为 1/count |
| warpWidth、weftWidth | 比例 0.55..0.9／0.7、0.7；UV 宽度为 width/count |
| bevel | 格子比例 0.02..0.12／0.08，始终小于最小宽度的一半 |
| relief、underRatio | 0..0.05／0.025；0.25..0.75／0.5 |
| detailAmount | 0..0.1／0.03；纹理只衰减高度，不移动中心 |
| warpSeed、weftSeed | 必需 u32 0..4294967295／夹具值 1729、65537；detailAmount=0 时也保留 |
| warpColor、weftColor、backingColor | 线性 RGBA [0,1]，alpha=1；默认值见计划 |
| yarnRoughness、backingRoughness | 0..1／0.8、0.95 |
| normalStrength | 0..1／0.5 |

W/F 为经纬连续轮廓近似，Q 为交替选择器。每个轮廓乘以独立的 [1-detailAmount,1] 纹理。经线高度系数随 Q 从 underRatio 插值至 1，纬线从 1 至 underRatio，再乘 relief 并合成最大值。由于 underRatio<=0.75、detailAmount<=0.1，relief 非零时，解析轮廓中心选定上层仍高于下层（最小 0.9 对最大 0.75）。这不证明所有交叉输出像素；观察必须排除混合采样足迹或明确计入。relief=0 不要求严格高低，但必须得到零高度和中性法线。

颜色在底布上分别按两种纱线顺序合成，再按 Q 插值。连续覆盖并集为 mix(W,1,F)，同一覆盖控制纱线／底布粗糙度。金属度固定为零。轮廓和选择器的种子显式为 0、variation 为 0，只有两个纹理种子随机化像素。改变纹理种子不得改变 W/F/Q；颜色仅改变 baseColor，粗糙度仅改变 roughness，法线强度仅改变 normal。宽度可以改变覆盖、高度和法线；经线间距不得改变独立纬线轮廓，反之亦然。

生产计算保持 f32，每次 pass 存储 rgba16float。max 合成在存储后不是精确实数运算；不允许隐式更高精度、CPU 渲染器或重置黄金图。输出保留现有 sRGB 颜色、线性 Scalar 字节和 OpenGL 切线法线约定。细节与形状在各尺寸共享归一化 UV，不暗中按分辨率改变拓扑。

## 建议验收门槛（未冻结）

- 所有计划用例、五通道均覆盖 256²、1024²、2048²、257×129。验证默认、独立端点、组合极值及两个独立替代种子。奇数数量、缺少种子、非有限／越界值及未知控制必须在请求构造前失败；现有 Core 节点验证继续权威。
- 经唯一执行器独立观察 W/F/Q、max 前的两层高度和最终高度。检查交点中心奇偶性、relief>0 的严格高低、纱线中心连续无底布孔洞、独立宽度／数量、有限归一化值及精确种子重放。除像素观察外，在选定 UV 做解析中心探针；奇数矩形通常不包含精确交点中心。实现前冻结坐标／容差。
- 比较 UV+(1,0)、+(0,1)、+(1,1) 平移求值、周期邻域及从原始存储高度进行的生产法线重放。两侧边缘像素无需相同。原始 half 的 max 合成容差暂拟 1/1024；重放及法线重放必须精确。这是建议的测试专用观察路径，不是已实现命令或公开 raw-half API。
- 同适配器重放和松散 `.mix`／`.mixpack` 等价必须逐字节相同。每个 Native／浏览器 RGBA8 分量（包括法线和压力用例）差异必须 **<=1/255**。覆盖固定 SwiftShader，以及分别标识的 GT 1030 Vulkan/DX12 对 Chrome；不得继承织物验收或推广硬件保证。
- 默认及变化材质的 256² baseColor／height 对 1024² 字节的未舍入 4×4 均值：各分量平均绝对误差 <=4/255；也测量 1024² 对 2048² 的 2×2 均值。法线角度／输出字节误差、交叉丢失率和方向纹理对比需记录指标，额外验收阈值留待冻结决定。32×32 纱线、宽度 0.55 的压力用例独立记录混叠，不得用于豁免默认／变化失败。257×129 时每纬线间距约 4 像素，不提供通用抗锯齿保证。砖块固定 2×2 采样及噪声／双线性重采样不是 mip 链。
- 草案上限为 <=64 passes、2048² 描述符峰值 <=512 MiB。实测 40-pass 配方在 plan-v3 复用后，2K 编译以 570,426,320 字节超出不变的 536,870,912 字节上限而失败。此描述符测量是冻结决定的依据，优先于全保留算术估计。保留全部 40 张 rgba16float 图像的 2K 成本为 1,342,177,280 字节，但这一历史纹理估计不是调度峰值或分配测量。通过配方变更或实测 PERF-MAT 工作解决，不得提高上限。GT 1030 目标：冷 1K <=10 秒，五次热运行中位数 1K <=1 秒、2K <=4 秒；固定 SwiftShader 对应 <=60／20／80 秒。记录解析／编译、冷／热渲染、读回／转换、保留输出、描述符／物理峰值及实际适配器。冻结前按负载评审目标；冻结后失败触发实测 PERF-MAT 工作，不能追溯放宽。
- 保留绑定生产身份的通道图和固定相机／灯光的介电 PBR 平面／球体、1×／3× 平铺和 4× 特写，覆盖规则与变化用例。人工独立评审交叉可读性、纱线方向、接缝、阶梯法线及织物外观。数值一致不等于 PBR／人工接受；按[证据政策](./evidence-policy.zh-CN.md)保留评审字节和维护者真实决定。
- 保留陶瓷／皮革／木材、MAT-01 和 MAT-02 回归、独立 Native／浏览器包消费及六项必需检查。记录精确源码、图／请求／计划、包、着色器／构建和适配器身份。普通输出放入忽略的 `tmp/` 或 CI artifact；有限的本地评审回执不构成接受像素或完整验收。

## 后续冻结 PR 前的决定

1. 接受不透明规则／变化平纹家族及受限宽度／数量，还是需要其他有界用例？
2. 解决已复现的视觉风险：窄斜边的平顶木板式轮廓，以及 detailAmount=0.03／relief=0.025 时默认方向纹理不清晰。1K 评审显示交替交叉和变化布局，不代表圆润纱线质量或完整因果性。评审配方改进，并继续测试奇数尺寸的错相轮廓 max／Q 过渡。这些观察不证明必须新增节点；本次不提新节点。
3. 可行性检查后、实现／冻结前，批准精确交叉／接缝／法线及方向细节探针、容差、压力标签和质量范围。
4. 通过配方变更或单独划定范围的实测 PERF-MAT 切片解决 2K 编译期预算失败，保持 512 MiB 上限不变。按负载及具名环境确认 64-pass 和耗时目标；本次修订不实现优化。
5. 若组合通过，确认目录不变。仅现有节点夹具工作无需新节点身份或格式。任何未来经证明的节点新增必须明确评审目录和 Rust／浏览器版本（包括穷尽 KernelId 消费者）；现在不选发布版本。`.mix`／`.mixpack` v1 和 plan／API v3 不变。

本次 MAT-03a 只交付可评审草案。后续 MAT-03a PR 解决可行性并冻结契约后，才可安排最小实现边界、材质构造及验收。不包含布料模拟、纤维几何、任意组织目录、通用抗锯齿、各向异性着色 API、子图系统、新 crate、运行时／着色器变更、版本清单变更、黄金图变更、Studio 工作或发布。
