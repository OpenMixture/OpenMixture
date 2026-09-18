# warp 双参考舍入审计 — 2026-09-18

[English](./README.md) | 简体中文

这是有界的 ALPHA-03 诊断，不是接受 PR #15／#16，也没有引入新的数值契约。生产着色器、旧金图、夹具参数、容差和节点版本保持不变。所有 GPU 像素执行仍使用 wgpu／WebGPU 和所记录的生产 WGSL。Python 有理数运算用于裁决标量见证，不是 CPU 材质渲染器。

**裁决：PR16 改写后的全部 41 个软件 warp 变化纹素都更接近精确参考 A，并符合分阶段参考 B。原有 14 个最终 PNG 差异全部可追溯到其中 11 个 warp 纹素的传播。** 候选在这组有界输入上有独立验证的数值收益，但仍改变旧像素，也不能对任意输入实现 A：独立双重舍入见证输出的是 B。

## 实际软件像素裁决

[软件裁决汇总](./software/verdict-summary.json)、[全部标量记录](./software/adjudication.json)和[最终像素依赖](./software/final-witness-links.json)保留完整变化集合。

| 变体 | warp half 变化纹素 | 改写前符合 A／B | 改写后符合 A／B | 原有最终 PNG 变化 |
|---|---:|---:|---:|---:|
| default | 26 | 0／0 | 26／26 | 四个通道共 10 个 |
| coarse-grain | 15 | 0／0 | 15／15 | 三个通道共 4 个 |
| horizontal-grain | 0 | 不适用 | 不适用 | 0 |
| straight-grain | 0 | 不适用 | 不适用 | 0 |

全部 41 个点的精确与分阶段邻居选择一致，实测 delta／权重符合 B。旧插值探针恰好落在两个实测 half 值的中点，符合 B_unfused；FMA 探针移到 A、B 独立选出的正确一侧。两个 half 探针都重现各自完整生产纹理。这把差异定位到插值舍入，而非输入、邻居、half 转换或回读变化；不等于证明实际执行了哪条机器指令。

例如原有 coarse-grain baseColor 见证 `(317,863)`：精确 A 为 `71772926317/137438953472`（`0.5222167697284021`），低于 half 中点 `0.522216796875`。旧插值落在中点并舍入为 `0.5224609375`；FMA 候选舍入为符合 A／B 的 `0.52197265625`。高度位模式从 `13940` 变为 `13937`，记录中的蓝色输出从 `48` 变为 `47`。参考选择不依赖 Native 输出。

[软件 PNG 绑定](./software/product-png-binding.json)与[原生绑定](./native/product-png-binding.json)各自完整重现旧 PR15／16 产品输出的 **32 个通道**，零像素差异，而不只是重现 14 个见证。法线关联包含相邻高度纹素。这证明传播关系，不是完整非线性材质／PNG 链路的数学真值。相对最初冻结金图的历史失败保持不变。

把[相同的 41 组软件输入重放](./software/cross-backend-replay.json)到原生 DX12 和普通 Chrome：三个已测后端的 delta／权重及 FMA f32／half 结果全部一致。Native 的旧 mix 已在全部 41 点符合 A；Chrome 与 SwiftShader 的旧 mix 均为零点符合。这是实际冻结输入的缓冲区重放，不是额外的浏览器完整材质验收。

## 软件实验前固定的参考

[oracle.py](./oracle.py) 定义独立的最近偶数舍入模型。定义在软件采集开始前已提交为 `8e7b3b0592368d7aad0fb3fa316b9d09fd6aa880`。其中任何一种都没有被宣称为 WGSL 强制要求的执行序列。

- **A：** 对实际 binary16 纹理值和 binary32 强度参数进行精确计算，包括位移、周期环绕的邻居选择及双线性插值，最后只做一次 binary16 舍入。
- **B：** 位移场归一化、强度和尺寸乘法、小数权重分别做 binary32 舍入；端点差做舍入，X／Y 插值融合计算；最后做 binary16 舍入。
- **B_unfused／B_weighted：** 使用相同分阶段坐标，分别采用独立的差值／乘法／加法，或分别舍入的加权和插值。
- **A_frozen_weight：** 冻结 B 的邻居和权重后进行精确插值。用于区分坐标误差与插值误差，不等于 A 的完整采样参考。

对 PR15、PR16 分别采集现有全部四个木材变体，保留**每一个发生变化的 warp 纹素**，再独立追溯原有 14 个 PNG 见证。没有删除不利样本，也没有根据 GPU 输出选择参考。单结果 delta、权重、插值和 half 探针与未改写的生产纹理输出对照；探针结果不能悄悄替代生产结果。

## 区分契约的字面案例

[check_literals.py](./check_literals.py) 独立计算固定的 23 个 PR16 字面输入：A、B、B_weighted 均符合全部 23 个夹具预期，B_unfused 符合 22 个。因此旧测试不能区分 A 与 B。这是新增的算术复算，不是重新运行这 23 个 GPU 案例。

新增双重舍入见证使用输入 `[0.5,1]`、尺寸 `[2,1]`、位移场 `1` 和强度位模式 `0x39800001`。局部权重恰为 f32 `0x3a000001`。A 得到 `0.50048828125`，B 得到 `0.5`。原生 DX12、普通 Chrome 和固定 SwiftShader 的生产 FMA 候选都输出 `0.5`。原生／Chrome 的单结果插值已经是 half 中点 `0.500244140625`，显式 half 转换与实际纹理一致。该差异不需要纹理复制错误或非融合 FMA 才能成立。软件字面案例覆盖生产纹理输出；首次软件运行不包含后来补充的字面缓冲区探针。

原有 half 边界见证仍然有效：A／B 要求 `0.343505859375`；原生得到该值，Chrome 的仅局部坐标候选得到 `0.34326171875`，Chrome 的 FMA 候选得到 A／B 值。这支持该案例中的改进，不支持通用精确舍入承诺。

在 1024×1 微小位移案例中，原生与 Chrome 都在像素 256 丢失旧 UV 路径的位移；局部坐标与 FMA 保留了位移。新增常规范围对照把交替输入 `[0,1]` 改为 `[2^-14,1]`，其他参数不变：旧输出为 `2^-14`，局部／FMA 为 `5*2^-16`，符合 A 和 B。原次正规数测试继续保留。这个对照帮助区分坐标消减和次正规数复制行为；[WebGPU 复制规则](https://www.w3.org/TR/webgpu/#texel-copies)允许将次正规数置零，但这种许可不意味着已测设备实际发生了置零。

## 复现

[源码身份](./source-identities.json)绑定材质、变体、锁文件、精度辅助函数、未改动的上下游着色器及三个 warp 源码。文本摘要统一使用 LF 换行。`generate.py` 拒绝输入漂移。保留的 Shader 副本是诊断证据，不是运行时的替代 kernel。

在当前检出中，把 `texture_probe.rs` 和 `buffer_probe.rs` 临时复制到 mixture-wgpu 的 examples 目录，分别命名为 `rounding_texture_probe.rs` 和 `rounding_buffer_probe.rs`。执行：

```sh
cargo build --locked --release -p mixture-wgpu --features software-vulkan --example rounding_texture_probe --example rounding_buffer_probe
python docs/evidence/warp-rounding-audit/run.py
python docs/evidence/warp-rounding-audit/check_literals.py
```

固定 SwiftShader 环境先运行 `.github/scripts/setup-swiftshader.sh`，显式选择生成的 ICD，设置 `MIXTURE_GPU_BACKEND=vulkan`。Windows 探针默认选择 DX12；复现已确认编译器的标量运行时，把记录的 Chrome DXC 目录加入 PATH。使用新的 `tmp/warp-rounding-audit` 输出目录。采集后移除临时 examples 文件，它们不是产品 API。首次软件采集使用记录的实验提交中的临时 CI 步骤。

普通 Chrome 使用仓库的 `scripts/browser-runtime/launch-default-browser.ps1` 启动新配置；只有配置隔离和本地调试开关。`MIXTURE_PROBE_PRODUCT` 指向已安装 Playwright 的独立消费者。`browser-literals.mjs <endpoint> <cases-json> <combined-precision-and-warp-wgsl> <report-json>` 执行字面纹理，不把竞争参考之一设为强制断言。`browser-probe.mjs <endpoint> <shader> <output> <float-count> <input-bin>` 执行保留的单结果缓冲区。这是浏览器诊断，不是安装包验收。

`compare_png.py <capture-dir> <PR15-PNG-root> <PR16-PNG-root>` 依赖 NumPy／Pillow，对 32 个已记录输出通道的**全部**像素进行比较；只复写既有回读编码。`link_witnesses.py <capture-dir>` 把原有 14 个 PNG 差异关联到 warp／height 变化，包括法线的相邻高度依赖。原材质 PNG 保持不变。

## 运行身份与保留范围

[Receipt](./receipt.json)绑定实验头提交 `8e7b3b0592368d7aad0fb3fa316b9d09fd6aa880`、实际 CI 合并检出 `a200b127defe66deec2dd76e70eeae6c8652023d`、[成功的软件运行 35321683099，attempt 1](https://github.com/OpenMixture/OpenMixture/actions/runs/35321683099)及 SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`。该 job 既有的冒烟、包消费、冻结材质金图和 2K trace 均通过，测试的是未改动的 main 运行时。这不等于接受单独分派的 PR15／16 Shader。后续文档提交不是这个已测试检出。

原生执行使用 GT 1030／DX12，驱动 `32.0.15.8266`；普通 Chrome 为 `153.0.8010.48`／NVIDIA Pascal。确认编译器的标量重放保留 Chrome DXC 模块路径。补充本地分析／重放源码与缓冲区按摘要保留。本地 `cargo xtask check`、最终诊断 Rust 编译、Python 参考检查及 JS 语法检查均通过。本次没有新增 Edge 或 Firefox GPU 运行。

Git 保留 24 份逻辑软件输入／warp／height 缓冲区的内容去重归档、全部 41 组标量输入／结果及参考、14 个最终依赖关联、字面纹理、源码计划和适配器 receipt。`texture-aliases.json` 还原字节完全一致的缓冲区，避免重复存储。[Artifact receipt](./software/gpu-artifacts.json)记录 artifact `10538370576`、大小 `113355138`、服务摘要和到期时间 `2026-10-18T08:20:33Z`；2026-09-18 完成取回和离线校验。完整例行采集及原始 32 通道 PNG 集合保留在本地／会过期的 CI artifact 中，其摘要绑定不承诺永久逐字节复核所有原 PNG。没有接受新视觉基线。

无需 GPU，运行 `python docs/evidence/warp-rounding-audit/verify.py`，即可检查保留内容摘要，从归档重建全部变化纹素，重新计算精确参考，核验标量输入／结果并追溯最终见证。文本使用 LF 换行，二进制缓冲区保留原始字节。这是代理生成的数值证据，不是独立的人类视觉批准。

由于较大的 Git／API 请求失败，软件纹理无损重打包为 4,917,964 字节的 tar.xz，并按 64 KiB 分片保存。`archive-parts.json` 记录顺序、完整归档大小和 SHA-256。校验器在内存中拼接、核验归档，并把每份解压缓冲区与原 SHA-256 比较。另有四份较大的 JSON 报告压缩空白格式。重打包和文本格式变化均不改变原始缓冲区、数值或实验结果。

## 范围限制

本次不选择 A 或 B 作为产品承诺，不证明任意合法输入的符合性，不认证普通 Chrome／Edge 完整材质，也不认定编译器违规。没有把所有剩余噪声／材质失败归类。软件回归、浏览器比较、结构／视觉门槛仍是独立义务。未来有意改变数值规则或兼容行为，需要单独审查；仅修改节点版本不能定义舍入语义。

下一步应依据已量化的收益和旧像素代价，提出窄范围的数值／兼容方案。这 14 个 PNG 差异已有来自更准确 warp 输入的证据，不应简单归为插值退化。决策之前，PR15／16 继续保持草稿，旧门槛继续有效。噪声路径诊断和普通浏览器完整验收仍是独立工作。
