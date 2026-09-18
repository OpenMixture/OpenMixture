# 皮革与木材残余差异缩减 — 2026-09-17

[English](./README.md) | 简体中文

**已定位三个具体运算：cellular 站点偏移乘加、value noise 的 `mix`、warp 的 UV 乘加。** 本次接续[未获采用的局部坐标候选](../local-coordinate-candidate/README.zh-CN.md)，只做诊断和有界修复提案，未改生产代码、依赖、golden 或容差。

## 将探针绑定到实际失败

源版本为 `e41410859c457c13739198c580bc1e1e2af45ed9`。1024² 默认皮革、木材高度图分别有 16 和 37 个最终 PNG 差异点。[诊断管线](./pipeline.json)直接使用候选原样 WGSL、精度辅助函数、原始 uniform 字节、rgba16float 纹理、纹理读取和工作组大小。[原生程序](./texture_probe.rs)使用 registry wgpu、Release/DX12，[浏览器程序](./browser-textures.mjs)连接普通新配置 Chrome/Edge。这是 GPU 诊断调度，不是另一套产品渲染器，也不声称新的 WASM 认证。

每个标量节点都回读 half 纹理；最终两张高度纹理转 RGBA8 后，与候选原生/Chrome 的完整 PNG 零差异。Edge 的全部七张纹理与 Chrome 一致。[汇总](./summary.json)和[差异点 half 位模式](./half-witnesses.json)保留指标，大块完整回读只留临时目录及哈希。

随后在 Chrome 每次下游调度前上传同一份原生输入纹理，**不改该节点着色器**，区分传播和自身运算：

| 节点 | 原生/Chrome 不同 half 像素 | 固定为同一原生输入后 |
| --- | ---: | ---: |
| 皮革 grain | 223 | 223，无纹理输入 |
| 皮革 height/levels | 127 | 0 |
| 木材 grain | 54 | 54，无纹理输入 |
| 木材 stretch | 32 | 0 |
| 木材 distortion | 134 | 134，无纹理输入 |
| 木材 warp | 176 | 54 |
| 木材 height/levels | 176 | 0 |

本次 levels 和 stretch 差异来自上游传播，warp 另有自身差异。上述是 half 纹理计数，不是最终 RGBA8 计数。七张纹理均有限且在 [0,1]。不能据此声称所有 levels、transform 或参数组合都一致。

## 三个最小运算

[cellular](./min-cellular.wgsl)、[mix](./min-mix.wgsl)、[warp](./min-warp.wgsl)各只读取四个固定 f32 输入并输出一个标量。输入/结果缓冲及编译器模块记录均保留。Chrome/Edge 相同，原生差 1 ULP。[精确有理数对照](./summary.json)使用实际 f32 输入和常量，不替换材质参考。

1. **皮革 `(685,60)`：** 第 0 octave 最近站点的 jitter 完全相同，为 `0.22351199388504028`；X 偏移 `0 + 0.2 + 0.6*jitter - 0.84375` 得到原生 `-0.5096428394317627`、Chrome/Edge `-0.5096427798271179`。该站点参与最近距离结果。grain 最终舍入为 `0.2132568359375` / `0.213134765625`，height 为 `0.52392578125` / `0.5234375`，PNG 为 134 / 133。无需不同哈希、纹理坐标或 levels 自身错误即可复现。
2. **木材 distortion `(934,121)`：** 第 0 octave 的 lattice 值和 fade 相同，但 `mix(0.09506094455718994, 0.5717524290084839, 0.8831931352615356)` 得到 `0.516071617603302` / `0.5160715579986572`。后续 octave 累加另有舍入差异；纹理变为 `0.440185546875` / `0.43994140625`，影响位移，最终高度为 119 / 118。它证明 `mix` 是上游差异来源，不代表这一处插值解释所有木材像素。
3. **木材 warp `(831,109)`：** 即使输入纹理完全相同，`0.81201171875 + (2*0.2462158203125-1)*f32(0.018)` 仍得到 `0.8028754591941833` / `0.8028755187988281`。乘 1024、减 0.5 后为 `821.6444702148438` / `821.64453125`。同样的邻居 `0.625` 和 `0.7314453125` 插值得到 `0.6936008334159851` / `0.6936073303222656`，跨越 half 中点 `0.693603515625`。half 为 `0.693359375` / `0.69384765625`，最终高度为 170 / 171。

[噪声追踪](./noise-trace.wgsl)在两端对全部 53 个选定点复现了实际 grain/distortion half 值，但仍需承认插桩影响：同时输出所有中间值的 warp 追踪会**隐藏差异**，因为优化条件改变。分别只输出一个结果的 `warp-only-uv/p/t/mix/half.wgsl` 恢复了真实差异。不能把插桩后相同当作原算法一致。

复核的 [WGSL 草案](https://github.com/gpuweb/gpuweb/blob/cd910cf650d05481b60bad2b44476caff974962f/wgsl/index.bs)允许重结合/融合，不承诺跨后端逐位一致；这些微小差异不证明编译器违规。精确对照中 cellular 是 Chrome 更接近，mix 和 warp 是原生更接近。因此严格编译策略本身不是数值真值。未向上游发送 issue。

## 修复提案及边界

**优先单独评估 warp 的局部 texel 坐标。** 对等尺寸输入/输出纹理，用 `pixel + deltaTexels` 表达采样，其中 `deltaTexels = (2*field-1)*strength*size`；基址用整数 `pixel + floor(deltaTexels)` 再循环包裹，插值权重用 `fract(deltaTexels)`。这避免绝对 `uv + offset` 的舍入再被尺寸放大。必须保留零位移快速路径、负数循环、双轴行为和现有双线性/half 语义，并在实施前从执行器确认等尺寸前提。

[局部 texel 标量探针](./warp-local-texel.wgsl)使用同一组 37 个木材点，以精确算术确认邻居选择不变。相比原标量探针，最大插值误差从 `4.244320734869689e-6` 降到 `6.489608495030552e-8`，约 65 倍；两端原始差异从 4 降到 1，half 差异从 4 降到 0。[完整样本结果](./proposal-samples.json)已保留。这是按失败点选取的稀疏诊断，**不是完整着色器实现、普遍改善证明、性能测量或验收通过**。该提案仍会改变旧像素，必须独立面对冻结 golden 和三浏览器矩阵。

**噪声应单独决定数值契约。** 局部 cellular 消除了大坐标相消，却仍有敏感的仿射运算；value noise 则有插值和 octave 累加差异。不应盲目替换 `mix` 为 `fma`、再插入 half 量化或强制严格编译。以准确度为目标的仿射/插值改写必须对照固定输入精确计算，并评估旧像素兼容性。如果要求可移植的逐位一致，需要在现有 WGSL 路径中明确运算和舍入实现，而不是依赖未经证明的编译开关；较大语义范围应另行审查。本次不支持把噪声和 warp 一起重写。

## 复现与保留

从仓库根目录准备 `tmp/local-coordinate-engine` 候选检出和已安装依赖的固定消费者 `tmp/local-coordinate-product`，沿用前一记录。将保留脚本复制到新的 `tmp/path-reduction`，其浏览器依赖和输入路径有意指向这些忽略的诊断位置。运行 `generate.py` 生成七节点计划；将 `texture_probe.rs` 临时作为隔离 mixture-wgpu example，以独立 target、Release 和前述 DXC PATH 构建，运行 `texture_probe <pipeline.json> <native输出目录>`。用普通启动器启动浏览器，执行 `browser-textures.mjs <CDP端点> <输出目录>`，额外传 `native` 即固定原生输入重放。复制完源程序后恢复隔离检出。

`generate-traces.py` 生成 53×256 噪声和 37×32 warp 输入，使用[已有 buffer runner](../chromium-arithmetic-reduction/README.zh-CN.md)，计数分别为 13568、1184；最小探针计数为 4。`warp-only.py` 生成单输出探针，`local-proposal.py` 生成 texel 实验，`proposal-measure.py` 用精确有理数评估标量，不执行 CPU 纹理渲染。`verify.py` 可离线检查保留的小证据。

Git 保留小缓冲、源码、计划、汇总、适配器/启动记录和哈希；每节点 8 MiB 完整回读仍是临时内容，哈希不保留字节。候选 PNG 证据位于前一记录。未新增 golden、发布制品、认证浏览器、维护编译器分支或修改生产实现。
