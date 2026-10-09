# MAT-03 验收 — 在记录范围内接受

[English](./README.md) | 简体中文

本阶段 D 记录在下述范围内关闭编织织物配方修订 2。新的机器证据绑定 PR #86 合并后的干净 main **9dc8c1a4dec12a0566f5237e05dc6862056bc819**。人工决定绑定**干净 6eb52a6a4de18ae0f8081e6314cff0215e249783 生成的原六张阶段 C 图**，不是重生成图像或后续文档提交。Rust **0.9.0**／browser **0.9.0-alpha.0** 仍未发布；仅验收记录的适配器及范围。

## 维护者决定与视觉范围

[人工决定](./human-decision.json)按 **2026-10-06、Asia/Shanghai** 逐字记录：

- **"Accept with a narrower range"** — 已被取代：PR #87 测量表明只收窄 underRatio 不能消除 combined-high 的蝴蝶结形。PR #87 已关闭且未合并。
- **"Accept now, fix in @2 later (Recommended)"** — 最终决定：按下述质量边界接受；记录下一项 **weave-pattern@2** 可见性修复，让上层纱线在交叉处自己的整个宽度上胜出。它**尚未启动**，需要维护者另行启动及契约／目录／版本审查。

视觉接受预设：**plain、varied、warp-seed、weft-seed、combined-low**。**Combined-high 及类似高 underRatio／crown／宽度、低 bevel 且出现蝴蝶结形的组合，不在视觉质量保证内。** 数值比较通过不将保证扩展到全部合法控制空间。材质／节点 underRatio 范围仍为 0.25..0.75；默认仍为 underRatio=0.25、crown=0；所有冻结门槛、用例、尺寸和预算均不变。

[plain](./review/plain-pbr.png) · [varied](./review/varied-pbr.png) · [warp-seed](./review/warp-seed-pbr.png) · [weft-seed](./review/weft-seed-pbr.png) · [combined-low](./review/combined-low-pbr.png) · [combined-high — outside visual guarantee](./review/combined-high-pbr.png)

[preview.json](./review/preview.json)及其[索引](./review/index.html)保留决定之前的 false 标记与待定文字；后续决定写在本记录和 human-decision.json，不改写冻结计划或历史收据。预览使用固定介电 GGX 光照、正交平面／球体、1×／3× 平铺、4× 特写及五通道缩略图；高度只展示，不位移。roughness 0／1 的校验图不同，相同设置重复精确一致。代理检查不是人工决定。

## 干净 main 上的机器门槛

[机器摘要](./machine/summary.json)、[Vulkan 收据](./machine/native-vulkan.json)、[DX12 收据](./machine/native-dx12.json)、[浏览器矩阵](./machine/woven-browser.json)、[候选消费](./machine/browser-qualification.json)和[绑定清单](./binding.json)保留源码、请求、计划、包、适配器及文件身份。

- GT 1030 两个后端各通过 **51 行**（48 冻结材质行＋3 独立范围的压力行），对同一干净 Chrome **154.0.8037.98** 候选各完成 **255 次通道比较**。Vulkan／DX12 最大分量差均为 **1/255**，满足 ≤1/255。Vulkan 为 251 次精确、4 次差 1；DX12 为 244 次精确、11 次差 1。浏览器报告 BrowserWebGpu 且适配器名称为空，不能据此推定额外硬件身份。
- Chrome 候选消费 **23/23 测试通过**，无跳过、意外或不稳定测试。精确重复、散装／.mixpack 等价、切片高度、销毁后输出所有权、端点参考及物理分配计数均通过。
- 所有行均为 **21 passes／8 个物理纹理**。五通道 2048² 描述符峰值 **301,990,480 B**，低于 **536,870,912 B**；pass 门槛仍 ≤64，报告的存活字节归零。
- 两组默认／varied 下采样的最大受门槛约束分量平均误差，在 Native Vulkan、DX12、Chrome 均为 **0.335238457/255**，低于 **4/255**。压力单独标注，不扩展视觉或默认质量保证。
- 每个后端四个匹配适配器计时行全部通过。GT 1030 Vulkan 驱动为 NVIDIA **582.66**，DX12 驱动身份见收据。冻结硬件预算：1K 冷运行 ≤10,000 ms，1K 热运行中位数 ≤1,000 ms，2K ≤4,000 ms；软件预算仍为 60,000／20,000／80,000 ms。耗时含读回、不含适配器创建；不是 SLA。

| 后端 | 预设／尺寸 | 冷运行 ms | 五次热运行中位数 ms |
|---|---|---:|---:|
| vulkan | plain-1024x1024 | 257.082 | 215.408 |
| vulkan | plain-2048x2048 | 837.889 | 986.056 |
| vulkan | varied-1024x1024 | 231.922 | 241.751 |
| vulkan | varied-2048x2048 | 1564.909 | 1179.280 |
| dx12 | plain-1024x1024 | 1231.064 | 356.079 |
| dx12 | plain-2048x2048 | 2412.133 | 1033.852 |
| dx12 | varied-1024x1024 | 985.451 | 225.803 |
| dx12 | varied-2048x2048 | 1581.106 | 850.630 |

## 阶段 B 与节点门槛

GT 1030 Vulkan／DX12 的**七组探针**均通过其**受门槛约束规则**；[执行记录](./machine/probe-runs.json)保留实际起止时间。[SwiftShader 收据](./machine/swiftshader/)来自成功的合并后固定软件 GPU CI，不是由本地硬件推断。

| 探针 | GT 1030 Vulkan | GT 1030 DX12 | 固定 SwiftShader Vulkan |
|---|---|---|---|
| plain／varied 交叉结构 | 8 行／四尺寸 | 8 行／四尺寸 | 6 行／三尺寸 |
| flat 高度／法线 | 4 行／四尺寸 | 4 行／四尺寸 | 3 行／三尺寸 |
| 公开控制隔离 | 16 变体／257×129 | 16 变体／257×129 | 16 变体／257×129 |
| 最终高度法线循环重放 | 48 用例尺寸行／四尺寸 | 48 用例尺寸行／四尺寸 | 36 行／三尺寸 |
| 所选噪声周期性 | 48 行／四尺寸 | 48 行／四尺寸 | 36 行／三尺寸 |
| 织纹字段平移 | 672 门槛通过＋48 观察 | 672 门槛通过＋48 观察 | 492 门槛通过＋48 观察；180 行 notRunOnSoftware |
| dense-thin 压力结构 | 原冻结三尺寸 | 原冻结三尺寸 | 原冻结三尺寸 |

硬件完整矩阵尺寸为 256²、1024²、2048²、257×129；隔离本就仅用奇数尺寸，压力保留 256²／1024²／257×129。第四次修订对报告为 Cpu 的软件适配器省略 2048²，明确写为 **notRunOnSoftware**，不是通过；软件不验收硬件像素。Vulkan 上 **cargo xtask test-node weave-pattern** 通过，包含契约夹具、GPU 节点及字段平移探针；见[节点收据](./machine/weave-pattern.json)。

## 与接受一同保留的限制

- **蝴蝶结边界：**[原始源绑定扫描](./limitations/safe-region-sweep.json)及[派生摘要](./limitations/summary.json)保留被取代范围决定的依据。在后续 **0.75 报告阈值**下，**156/432** 网格点达到可见宽度指标；**crown=1 为 8/144**。原实验阈值 0.85 不变；0.75 摘要不是新门槛，也不证明一个连续安全区域。
- **默认残余收窄：**配方修订 2 的 plain 在 1K 两轴 min 均为 **0.875**；人工接受不宣称纱线处处为直边。
- **第三次修订：**257×129 平移 warp-share 与可见权重是**不设门槛的观察**，不是通过。每个记录的硬件后端在 combined-high 的原始 S 最大差 **27 half 步**；最大 |ΔP| 为 **0.00048828125**，约 **0.124512 个 8-bit 级**；相对被取代第二次修订不等式的最大比值为 **6.802104**。二次幂尺寸平移及零起点身份仍要求精确；奇数尺寸高度／覆盖仍须在一个 half 步内。这些跨 tile 的 f32 采样观察不改写生产语义。
- **第四次修订：**软件省略 2048²，该尺寸仅由硬件证据覆盖。不保证任意适配器、任意控制组合、布料模拟、纤维几何或通用抗锯齿。
- **未发布：**本记录不发布 Rust 或浏览器包；MAT-04 仍规划，weave-pattern@2 未实现、未启动。

## 合并后 CI 与验证

**9dc8c1a** 上六项必需检查通过：三平台 CPU、WASM/npm、固定 SwiftShader GPU／材质、Chromium WebGPU 材质：[CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415513), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415302), [pinned SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415383), [Chromium attempt 2](https://github.com/OpenMixture/OpenMixture/actions/runs/37348415333/attempts/2)。原 Chromium 尝试触及 50 分钟超时；[尝试 1](./ci/ci-browser-materials.json)仍记为取消，未改源码的[尝试 2 结果](./ci/ci-browser-retry.json)是独立证据，不覆盖取消记录。未更改工作流超时或冻结门槛。

本次文档／证据交付另外串行运行 cargo xtask evidence、cargo xtask links、cargo xtask check，结果写在交付 PR。它们不会将后续文档提交变成上述机器被测源码。

## 复现与保留

调用方控制及探针命令见[夹具指南](../../../fixtures/materials/woven-fabric/README.zh-CN.md)。在指定干净 main 上，将 Node 24.21.0 加入 PATH，设置 MIXTURE_BROWSER_CHANNEL=chrome，以 node scripts/browser-runtime/build.mjs 构建，再运行 consumer.mjs candidate target/browser-runtime 写入新目录。分别设置 MIXTURE_GPU_BACKEND=vulkan 或 dx12、MIXTURE_GPU_SOFTWARE=0、MIXTURE_GPU_EXPECT_ADAPTER="GT 1030"，以 check-woven.mjs 对同一输入运行。机器摘要保留精确聚焦命令和环境；后端串行执行，重日志保留于忽略的 tmp/woven-acceptance/。

**复制前**已逐项校验原 60 张 Native／浏览器输入 PNG、9 张截图（六评审图＋三粗糙度校验图）、全部预览输入收据、生成 HTML、评审计划及索引的 SHA-256；未重生成或重新编码图像。[路径映射](./review/path-mapping.json)将未改生产路径对应到保留路径。恢复的 runner 源码匹配原 hash，包含最后一行 CRLF；局部 Git 属性保留收据字节。

绑定清单：**126 文件／21,973,504 B**（不含 binding 及这两个 README），最大单文件 **961,337 B**。无文件超过 4 MiB；不新增原始日志、归档或保留例外。Git 保留被评审内容、关键测量、决定及所选收据；完整新矩阵 PNG、普通日志、候选归档、未选 CI 内容仍为临时输出，hash 不保留这些省略的字节。[CI 制品元数据](./ci/ci-artifacts.json)记录到期（所选 SwiftShader 来源制品为 2026-11-04），不承诺永久外部归档或完整原运行可审计性。OpenMixture 维护者负责保留。
