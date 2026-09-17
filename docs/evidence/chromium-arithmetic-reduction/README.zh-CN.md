# 算术最小复现与验收契约复核 — 2026-09-17

[English](./README.md) | 简体中文

**仅使用缓冲区的乘加表达式即可复现原生与 Chrome／Edge 的差异，两种结果均为 WGSL 所允许。仅凭这个复现，不能认定应该维护依赖分支或修改着色器。** 本记录取代[前次调查](../chromium-compile-policy/README.zh-CN.md)的维护路线建议，不改写其历史测量。Chrome／Edge 完整材质认证仍待完成；已接受候选、原生参照、golden 和容差均未改变。

## 最小保留案例

[minimal.wgsl](./minimal.wgsl)只计算 `inputs[0] * inputs[1] + inputs[2]`，从运行时 storage buffer 读取输入，并直接读回 f32。没有材质图、纹理、半精度转换、PNG 编码、三角函数、除法或 `mix`。四个相同输出用于排除单个读回值异常。[输入文件](./minimal-input.bin)包含 `0.6000000238418579`、`0.241208016872406`、`0.20000000298023224` 和一个未使用的零。第二个操作数来自整数哈希扫描；不声称这个确切种子就是完整材质中的失败像素。

| 执行环境 | f32 位模式 | 数值 |
|---|---|---|
| 原 registry wgpu，Release／DXC | `0x3eb07fc5` | 0.3447248041629791 |
| 隔离的 IEEE 严格诊断构建 | `0x3eb07fc6` | 0.3447248339653015 |
| 普通 Chrome 153.0.8010.48 | `0x3eb07fc6` | 0.3447248339653015 |
| 普通 Edge 153.0.4234.32 | `0x3eb07fc6` | 0.3447248339653015 |

[verify.mjs](./verify.mjs)用整数有效位和指数执行精确乘加，再按 binary32 最近偶数规则舍入。第一个结果对应只在最后舍入；第二个对应乘法之后舍入一次、加法之后再舍入一次。两者相距一个 ULP，即 `2^-25`。融合结果更接近精确实数结果，因此本例中“严格”不代表“更准确”。这是数值行为证据，没有采集 GPU 指令反汇编。

[WGSL 草案修订 cd910cf650d05481b60bad2b44476caff974962f](https://github.com/gpuweb/gpuweb/blob/cd910cf650d05481b60bad2b44476caff974962f/wgsl/index.bs)的 §§15.7.2–15.7.5 允许重排及满足精度条件的融合，也没有指定舍入模式。本例所有输入都是有限、正的正规数，两种结果均符合允许行为。这只确认缩减案例的合规性，不是对所有材质运算的完整规范审计。WGSL 的 `fma` 本身也可以展开为分开的乘法与加法，不能靠全部替换为 `fma` 建立通用逐位一致性。

## 与生产运算的联系

[八表达式扫描](./operations.wgsl)每个表达式使用 65,536 个样本。原生默认与严格构建之间，蜂窝坐标表达式有 62,584 个差异，标量乘加有 8,843 个，`mix` 有 9,318 个，levels 风格的输出映射有 4,796 个。两个浏览器的全部 524,288 个结果均与严格构建相同。本次扫描的 fade、显式 `fma`、除法和点积通道没有差异；这仅适用于受测输入，不证明这些运算永远一致。[结果文件](./results.json)保留数量、首个样本和完整输出摘要。

[cell-trace.wgsl](./cell-trace.wgsl)提取生产蜂窝噪声的 lattice 函数，计算皮革在 1024²、seed 271828、scale 64、persistence 0.35 时像素 (270, 0) 的三个 octave、各九个邻近点。lattice jitter 相同，但距离运算之前的坐标 delta 已不同。生产表达式 `neighbor + 0.2 + 0.6 * jitter - p` 先将小偏移加入较大的绝对坐标，再减去坐标，使重排和舍入产生影响。此处的距离差异包含 delta 差异的传播，不能独立认定点积存在缺陷。

[noise-grid 探针](./noise-grid.wgsl)复制生产蜂窝算法和半精度舍入辅助函数，将原始 f32 与半精度舍入值写到缓冲区。1,048,576 个像素中，两种原生编译策略有 1,032,165 个原始值不同，6,298 个舍入值不同。这是诊断中间值统计，不是冻结的 RGBA8 材质比较。[单点提取](./noise-point.wgsl)在 (270, 0) 得到：

| 执行环境 | 半精度舍入前 | 半精度舍入后 |
|---|---:|---:|
| 原生默认 | 0.10690304636955261 | 0.10687255859375 |
| 原生严格诊断 | 0.10690376162528992 | 0.10693359375 |
| Chrome／Edge | 0.1069038063287735 | 0.10693359375 |

舍入中点为 0.106903076171875。算术差异跨过中点；整数实现的半精度辅助函数对各自不同的输入执行一致的舍入规则。**严格原生与浏览器的原始 f32 仍有差异**，只是舍入后相同。因此，前次 44/44 PNG 相同不等于中间运算完全相同。提取及插入观测代码也可能改变优化；这些探针证明机制及传播路径，不把全部 17 个失败通道归因于一条指令。本次没有修改生产着色器。

## 逐像素门槛在保证什么

[冻结校准](../m5-05/calibration.zh-CN.md)基于已记录的 macOS 和 Linux 实现。[浏览器容差](../../browser-tolerances.json)是相对于声明的原生参照的经验性发布回归门槛，不是 WGSL 误差上界，也不是任意设备间的一致性承诺：

| 通道 | RGBA8 最大差 | RGBA8 平均差上限 | 变化像素比例上限 |
|---|---:|---:|---:|
| baseColor／height | 1 | 0.00001 | 0.00001 |
| normal | 1 | 0.00001 | 0.00002 |
| roughness | 1 | 0.001 | 0.001 |

任意分量有差异即计为变化像素。1024² 时，比例门槛分别最多允许 10、20、1,048 个变化像素，同时仍须满足平均差和最大差限制。失败表示此配置不满足冻结的比较契约；单凭失败，不能判定 WebGPU 不支持、材质视觉损坏或违反 WGSL。计划／语义一致、结构、接缝、因果关系、精确编码与棋盘格哨兵仍是独立的必过门槛；像素通过也不能代替这些检查。

## 此次缩减支持的决策

保留认证待完成及原始失败记录。现在不因为某个参照恰好匹配浏览器就更换参照，不放宽容差、重置 golden、开启浏览器开发者开关或维护 wgpu 分支。下一项有边界的实验应评估蜂窝噪声采用局部坐标是否减少数值消减，并对原有 golden 与完整材质案例保留前后证据。表达式重排可能改善稳定性，但不能预先假设它消除全部后端差异、保留现有像素或解决 gradient／levels 的差异；这尚不是获准采用的生产修复。

如果产品需要超出已记录矩阵的逐位一致性，应先界定运算、量化边界、配置和支持环境，再评估显式编译策略 API、性能与打包成本；严格编译参数已经不能让本噪声探针的中间 f32 完全相同。向上游报告时可以用最小复现讨论策略／接口需求，不应把规范允许的融合标为编译器错误。本次没有向上游发送 issue 或消息。

## 复现与来源

源码基线为 `2cc36863eb5cb4a0f419722b172fb5aceaec239f`，文档分支起点为 `ed07716f61bce9771f8492f1999cc737ecc891c3`。[results.json](./results.json)绑定探针源码、生产 shader／fixture／容差／lock 摘要、执行文件、原生适配器报告和实际加载的 DXC DLL 路径／摘要。原生为 GT 1030／DX12，驱动 32.0.15.8266。浏览器适配器信息被隐藏，主机 GPU／ANGLE 字段不能推断 WebGPU 后端。[Chrome](./chrome-launch.json)和 [Edge](./edge-launch.json)的新配置启动记录仅包含配置隔离和本地自动化开关。直接 WebGPU 探针用于算术诊断，不执行或认证已归档运行时包。

运行 `node docs/evidence/chromium-arithmetic-reduction/verify.mjs`，即可用精确标量计算检查保留的输入与结果。重新执行 GPU 探针：

1. 在源码基线建立两个隔离检出，把 [native-probe.rs](./native-probe.rs)复制到各自的 `crates/mixture-wgpu/examples/arithmetic_probe.rs`。一份保留原 registry 锁定依赖图；另一份按前次调查的说明，对独立复制的 wgpu-hal 30.0.1 应用[诊断补丁](../chromium-compile-policy/diagnostic-only.patch)，只在第二个检出添加本地 Cargo patch，保留诊断 lock 差异。分别使用独立 target 目录运行 `cargo build --locked --release -p mixture-wgpu --example arithmetic_probe`。
2. 仅给渲染子进程的 PATH 加入已记录的 Chrome DXC 目录。运行 `arithmetic_probe.exe <minimal.wgsl绝对路径> <output.bin> 4 <minimal-input.bin绝对路径>`。保留 stderr 的适配器／上下文及生成的 `<output.bin>.modules.json` 加载模块记录。在新结果目录将两个输出命名为 `minimal-regular.bin`、`minimal-strict.bin`，并复制输入文件。
3. 使用仓库的 `scripts/browser-runtime/launch-default-browser.ps1` 启动普通新配置浏览器。将 `MIXTURE_PROBE_PRODUCT` 设置为已有、包含锁定 Playwright 安装的已验证 Studio 产品目录。运行 `node docs/evidence/chromium-arithmetic-reduction/browser-probe.mjs <CDP地址> <minimal.wgsl绝对路径> <output.bin> 4 <minimal-input.bin绝对路径>`。输出命名为 `minimal-chrome.bin`、`minimal-edge.bin`，再执行 `verify.mjs <新结果目录>`。探针提供本地回环诊断页面，不使用浏览器功能覆盖。
4. 同一执行器也接受 `operations.wgsl`，计数 524288；`noise-grid.wgsl`，计数 2097152；`noise-point.wgsl`，计数 2；`cell-trace.wgsl`，计数 216。省略输入文件参数时输入全零。比较小端 f32 位模式；运算扫描每八项为一组，噪声网格每两项分别为原始值／半精度舍入值，cell-trace 每八项为一组字段。诊断结果不能作为正式材质通过回执。

Git 保留小型输入／输出缓冲区、可执行探针源码、上下文、精确标量校验器及测量摘要。大规模扫描读回、执行文件仍在忽略的临时 `tmp` 中；摘要不代表原数据永久可用。本次没有修改渲染输出或 golden，也没有作出新的视觉验收。
