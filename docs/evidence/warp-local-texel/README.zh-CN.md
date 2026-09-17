# 独立 warp 局部纹素修正 — 2026-09-17

[English](./README.md) | 简体中文

**已实现在草稿 [PR #15](https://github.com/OpenMixture/OpenMixture/pull/15)，但原有 golden 门槛下尚不可合并。** 候选 `91015e02255f20105586145db23bbd23b3c93546` 直接基于 `2cc36863eb5cb4a0f419722b172fb5aceaec239f`，不包含 PR #14 的 cellular 改动。它修复微小 warp 位移丢失，不代表消除全部跨后端算术差异。未改 golden、容差、依赖或噪声。

## 改动与回归证明

执行器为全部资源传递同一渲染尺寸，因此 warp 可用等价的局部纹素位移替代绝对 UV 换算：`delta=(2*field-1)*strength*size`，整数邻居基址 `pixel+floor(delta)`，权重 `fract(delta)`。像素坐标不再参与浮点权重计算。strength 在 [-1,1]、field 截断到 [0,1]，所以 delta 在 [-size,size]，邻居取模前加 size 可处理负数。中性/零强度快速路径、双轴、四次纹理读取、插值和显式 half 舍入均保留。

修改前原有 warp 与木材检查通过。新增字面值测试使用交替的 1024×1 输入和正 `2^-26` UV 位移：每个低值纹素应保留精确 `2^-16`，高值纹素在 binary16 中舍入为 1。[旧实现从第 256 个像素失败](./red-node-fixed.log)，输出零；[新实现通过](./node.log)。另加三例覆盖奇数宽/高、负位移及负整周期环绕。预期来自解析字面值，不是 CPU 重采样器或基线更新。[字面值结果](./literal-probes.json)保留全部 17 个 warp 探针。

本地 shader 校验、warp 夹具、四个硬件木材用例、`cargo xtask check` 均通过，固定独立消费者的类型/单元测试/生产构建也通过。CI 的三平台检查、WASM 打包、配置 Chromium 候选矩阵、固定软件节点/GPU/打包消费通过。但必需的软件材质检查失败，后续 2K trace 因而跳过。

## 候选身份与兼容性

[候选记录](./candidate.json)绑定干净源码、固定 Studio `56c510ab57daa1b68ef660525a648a582730a37e`、包 SHA-256 `cbb944c7b886044070f958c2f9b3e25e51c4cf493f0715dae0a884df4def769e` 和 build ID `sha256:83ce87cf204ef7f78a86f06c90ffd38eeb0c034e6a249f08318abe49f2c2d8cf`。[原生参考](./native-manifest.json)来自同一候选、registry 依赖和 Release/DXC/DX12。三种普通新配置浏览器运行同一包的全部 11 个用例/44 通道，以及生命周期/压力和模拟失败诊断。

| 比较 | 结果 |
| --- | --- |
| 普通 Chrome 153.0.8010.48 | 27/44 达到原门槛，19 个完全相同 |
| 普通 Edge 153.0.4234.32 | 各通道指标与 Chrome 相同 |
| 普通 Firefox 156.0 | 44/44 完全相同 |
| 固定 SwiftShader 旧 golden | 陶瓷/皮革通过；木材 12/16 通道失败 |
| 软件木材结构/关系检查 | 通过 |
| 原生原版/候选兼容性 | 陶瓷/皮革完全相同；木材 12 通道改变，最大字节差 1 |

浏览器总通过数与原始候选相同：陶瓷 12/12、皮革 4/16、木材 11/16。不能把它与 PR #14 的 33/44 直接视为回归，因为那个候选额外改了 cellular。Chrome/Edge 生产 Player 的下载、编码、释放和无测试入口检查通过；当前验证器不覆盖 Firefox 生产下载。

软件默认木材相对精确旧基线改变 baseColor 242、height 253、normal 380、roughness 62 个像素；straight-grain 仍完全相同。[完整指标](./summary.json)、[软件报告](./software/wood.json)、[软件接触表](./software/contact-default.png)和[原生前后对照](./contact.png)保留失败。原生默认计数分别为 244/256/384/62，单独记录。代理在接触表尺度未见明显结构破坏；这不代表人工接受，也不能覆盖像素比较。

## 全图诊断边界

上轮[37 点提案](../residual-path-reduction/README.zh-CN.md)并未承诺普遍一致。将新生产 warp 着色器在相同原生输入纹理上重放 1024² 像素，仍有 **26 个 half 差异**，比原先 54 个减少。三个上游纹理在修改前后字节相同；保留[诊断输入](./diagnostic/pipeline.json)、输出哈希和全部 26 个差异点。这是直接 WebGPU 诊断，与安装包矩阵分开，既不证明新的精度上界，也不保证跨端逐位一致。仍有仿射/插值变化，本次没有悄悄改写它们。

## 交付决定与复现

保持 PR #15 为草稿，保留必需 GPU 检查的失败，不合并、不更新 golden。微小位移丢失已修复且可审查，但旧像素兼容性未解决。改变已接受像素的数值改动需要另行明确兼容性决策，本次不授权该接受。不提议维护编译器分支、放宽容差、发布或改噪声。

在候选完整 SHA 上执行 `cargo xtask shader-check`、显式适配器下的 `cargo xtask test-node warp`、`cargo xtask test-material wood` 和 `cargo xtask check`。要复现失败测试，可在另一隔离检出中保留新测试，仅将 warp shader 恢复为父版本。固定软件使用已有 `.github/scripts/setup-swiftshader.sh` 和 GPU 工作流，不更新 golden。浏览器准备沿用[前一候选流程](../local-coordinate-candidate/README.zh-CN.md)，替换候选身份、包和新目录。保留的诊断浏览器脚本在忽略目录 `tmp/warp-local-evaluation/diagnostic` 使用，配合之前的独立纹理 runner；先在该目录重新生成完整原生节点文件，再进行同输入重放。

[CI 状态](./ci-state.json)绑定 PR 合并版本 `01074654cf98e1b432bf1108c7ae00440fcf1962`，本地测试使用候选 head。GPU 运行 [35207157131](https://github.com/OpenMixture/OpenMixture/actions/runs/35207157131)第 1 次尝试在木材失败；[制品元数据](./gpu-artifacts.json)为 `10490403918`，于 2026-10-17T09:55:26Z 过期。源码/测试在 PR #15，本记录在 PR #13，以保持被测候选身份固定。Git 保留报告、选定完整分辨率图和接触表；完整包、全套 PNG、8 MiB 节点回读和常规日志留在忽略的本地目录或会过期的 CI 输出中。哈希不保留省略字节，未接受新基线或宣称普遍平台支持。
