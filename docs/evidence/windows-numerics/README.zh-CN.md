# Windows 数值调查 — 2026-09-22

[English](./README.md) | 简体中文

已复现 [M6A-05 硬件失败](../m6a-05/README.zh-CN.md)，并定位到 height-to-normal 之前的噪声输出。**这是调查，不是修复或扩大硬件验收。** 生产 shader、版本、golden 和资源 ≤1 门槛均未改变。

## 发现

[调查记录](./investigation.json)绑定 main `c2a08e14be9991d75df24e0f045d842d8db4b570`、新增诊断源码哈希、NVIDIA GT 1030 Native Vulkan/DX12，以及 Chrome 153.0.8010.48（NVIDIA/Pascal，非 fallback）。浏览器未暴露底层原生 API，不能根据操作系统推断。精确注册表包 `0.3.0-alpha.0` 的 13 项公共接口测试通过。公共资源比较在 Vulkan 和 DX12 上**均失败**：权重 0/0.25/0.5/1 的法线最大差值为 **0/1/4/8**。

直接捕获生产 shader 原始数据，排除了资源加载、Scalar 混合和 PNG 编码。在 1024²、value noise、seed 29、scale 32、四个 octave、persistence 0.5 下：

- Vulkan 与 Chrome 的 1,048,576 个标量 f16 像素只有 **9 个**不同，每个相差一个相邻的正 f16 值。
- 法线有 39 个 RGBA8 分量不同，最大差值 **8**，其中 11 个超过 1，与公共接口权重 1 的结果吻合。
- 将 Native 原始 f16 噪声纹理原样上传 Chrome，运行未改动的法线 shader，与 Native 相比，**原始 f16 分量差异为零**。测得的阻碍位于法线计算上游。
- [原始稀疏失败证据](./baseline.json)保留全部九处高度和所有超过门槛的法线分量。未使用 CPU 噪声或法线渲染器。

额外插桩仅替换噪声最终存储，将转换前 f32 位模式编码为四个可精确表示的 byte/256 通道。[测量](./prehalf.json)发现 90,132 个 f32 像素不同，最大 4 ULP。在原先九个 f16 差异点，值相差 1–2 个 f32 ULP，位于半精度舍入中点附近。例如 (590,190)，Native 为 `0.652099609375`（恰为中点），Chrome 为 `0.6520994901657104`；存储高度分别变为 `0.65234375` 和 `0.65185546875`。

这支持**微小噪声算术差异 → 不同半精度舍入结果 → 按 UV 缩放的导数放大**。1K 下，法线 shader 将邻接高度差乘以 512。f32 值已不同时，整数半精度舍入无法恢复相同输入。插桩可能影响优化，因此它支持这一机制，而不证明某条驱动指令有问题；相同输入重放是更强的隔离对照。

## 实验与决定

实验 shader 将嵌套 `mix` 改为 `ab=fma(b-a,t,a)`、`cd=fma(d-c,t,c)`，再执行 `fma(cd-ab,u,ab)`。[numerical-experiment.mjs](../../../scripts/browser-runtime/numerical-experiment.mjs)在生产源码外生成它。

| Shader / scale | Native 对 Chrome | 不同的 f16 高度 | 法线 RGBA8 最大差值 |
|---|---|---:|---:|
| 生产 / 32 | Vulkan | 9 | 8 |
| 生产 / 7 | Vulkan | 85 | 19 |
| FMA / 32 | Vulkan | 0 | 0 |
| FMA / 32 | DX12 | 0 | 0 |
| FMA / 8 | Vulkan | 0 | 0 |
| FMA / 7 | Vulkan | 66 | 19 |

各组使用 seed 29、四个 octave、persistence 0.5，相同输入重算法线的原始差异均为零。生产 scale-7 对照表明剩余失败并非仅由实验引入。这是小规模诊断样本，不是完整节点、种子或参数验收。

**不能将仅改 FMA 的方案作为修复交付。** 它解决原始 fixture，却在另一合法 scale 上失败。WGSL 允许浮点重结合、融合，且不保证 `fma` 跨实现正确舍入；参见 [WGSL 浮点规则](https://www.w3.org/TR/WGSL/#floating-point-evaluation)。本次未证明具体 Naga、Tint 或驱动缺陷。

下一项有界决定应定义稳定 value-noise 的数值/兼容契约，覆盖非二次幂 scale、seed、octave/persistence 边界及分辨率。评估显式可复现算术时应单独决定节点/版本，并保留 v1 行为。提高中间精度属于另一项 plan/资源预算变更。任何候选均需冻结的资源、Scalar、三材质、固定软件适配器节点和 Native/browser 消费门禁，以及前后视觉证据。Cellular noise 和 warp 需要独立证据。

## 复现

从仓库根目录开始，保存 `cargo run --locked -p mixture-cli -- doctor --json`，设置 `MIXTURE_BROWSER_CHANNEL=chrome`，使用新目录运行 `node scripts/browser-runtime/consumer.mjs registry - tmp/numerics-registry`。将 `MIXTURE_RESOURCE_BROWSER_DIR` 指向生成的包含 `resource-1-normal.png` 的目录，将 `MIXTURE_RESOURCE_EVIDENCE` 设为绝对输出文件。对各个显式后端运行：

```powershell
$env:MIXTURE_GPU_BACKEND = 'vulkan' # 再使用 dx12 重复。
cargo test --manifest-path examples/native-consumer/Cargo.toml --locked --all-features --target-dir target/native-consumer --test image_resources -- --ignored --nocapture
```

[Native 探针](../../../crates/mixture-wgpu/examples/numerical_probe.rs)通过 wgpu 使用生产 shader，仅作为显式示例运行，它不是公共渲染器。在引擎拥有的临时副本中安装已有 browser-consumer 开发依赖（`npm ci --ignore-scripts`），然后传入其 `node_modules/playwright/index.mjs`。本次使用已有的忽略目录 `tmp/m6a04-browser-host`。捕获目录必须是新的，绝对路径需适配自己的 checkout。

```powershell
$env:MIXTURE_GPU_BACKEND = 'vulkan'
$env:MIXTURE_NUMERICAL_OUTPUT = 'D:/Coding/OpenMixture/tmp/probe-native'
cargo run --locked -p mixture-wgpu --example numerical_probe
node scripts/browser-runtime/probe-numerics.mjs tmp/m6a04-browser-host/node_modules/playwright/index.mjs tmp/probe-browser tmp/probe-native/noise.rgba16
node scripts/browser-runtime/compare-numerics.mjs tmp/probe-native tmp/probe-browser tmp/probe-comparison.json
```

比较器核对噪声 shader、参数和精确重放输入身份，报告测量，**不作验收**。文件为 1024² 紧密排列的小端 RGBA f16。可选 `MIXTURE_NUMERICAL_PARAMETERS` 包含八个 u32 uniform 值；scale 7 对应 `[29,7,4,0,1056964608,0,0,0]`。两个探针都读取该变量。生产运行之前清除实验变量。

```powershell
node scripts/browser-runtime/numerical-experiment.mjs fma tmp/noise-fma.wgsl
$env:MIXTURE_NUMERICAL_SHADER = 'D:/Coding/OpenMixture/tmp/noise-fma.wgsl'
# 在新目录重复两个捕获，再比较。
```

若要捕获半精度转换前的值，用 `prehalf` 模式代替 `fma`。解码每个 f16 通道，乘以 256，将四个字节按小端组合为 f32 位模式。比较位模式和 `baseline.json` 中九个坐标。该模式的法线输出无材质意义，不得解释或验收。

## 检查与保留

后续修正：首次 PR GPU 检查通过现有 `--ignored` 全量执行带入了诊断探针，由于缺少手动输出变量而失败。探针现移至 `examples/numerical_probe.rs`，仅通过 `cargo run --example` 运行；没有削弱任何必需检查或测试过滤器。对照捕获与原始噪声字节一致。初始记录保留提交 `5f9fe45` 的历史测试路径及源码哈希。

Registry 测试：13 项通过。两项公共硬件一致性测试：如上失败。实验之前已有 `cargo xtask test-node fractal-noise` 通过。六组捕获/重放实验完成。首次 `cargo xtask check` 遇到 Rust 1.98.1 增量缓存 panic（`Invalid DepKind 488`），设置 `CARGO_INCREMENTAL=0` 后检查通过。这一环境失败与 GPU 一致性失败不同。PR 记录文档变更之后的最终检查。

Git 保留源码/输入身份、选定测量、全部原始稀疏失败值及复现工具。完整原始纹理、普通 PNG、doctor 输出和日志位于忽略目录 `tmp/windows-numerical`，没有永久完整归档承诺。探针源码哈希标识实际测试的新增内容，后续文档不冒充像素源码。未接受视觉变更。不在范围内：运行时修复、版本变更、golden 更新、更广硬件验收、发布、Studio 修改与部署。
