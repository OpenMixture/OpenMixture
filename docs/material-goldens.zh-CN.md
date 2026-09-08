# 材质基准与审查

[English](./material-goldens.md) | 简体中文

PR-008 实现受保护的材质比较与[釉面陶瓷夹具](../fixtures/materials/glazed-ceramic/README.zh-CN.md)。工具通过现有 CLI 验证、编译并使用 `wgpu` 渲染全部用例，再测量输出 PNG。没有新增节点、着色器、运行时 API 或像素执行器。本任务明确暂缓远端 CI 验收；PR-008 和本地通过均不关闭 M0／M1／M2 或 M3。

## 命令

```bash
# 显式访问 GPU，默认采用本机硬件适配器策略。
cargo xtask test-material glazed-ceramic
cargo xtask golden check

# 独立的接受操作，消费最近一次完整的软件适配器候选。
# 不执行渲染，不暂存或提交 Git 文件。
cargo xtask golden update glazed-ceramic --accept
```

`test-material` 检查指定材质；`golden check` 按字典序检查所有含 `acceptance.json` 的材质目录，并报告全部失败材质。只有全部机器验收、计划身份和像素比较通过时，检查才返回成功。缺少基准会返回失败并留下审查产物，不会隐式接受。普通 `check`／`test` 运行夹具、度量和保护测试，不获取 GPU，不修改基准。

[GPU 冒烟](./gpu-context.zh-CN.md)的适配器变量也适用于此：`MIXTURE_GPU_BACKEND=auto|metal|vulkan|dx12`、`MIXTURE_GPU_SOFTWARE=0|1`，以及可选的 `MIXTURE_GPU_EXPECT_ADAPTER`。软件运行要求显式 `vulkan`、`Cpu` 和 `SwiftShader`；硬件运行拒绝 CPU 适配器。每次运行保留请求策略、实际适配器、doctor 和逐用例渲染报告；失败时保留原始诊断。不进行回退。

使用[现有准备脚本](../.github/scripts/setup-swiftshader.sh)构建固定驱动。macOS 示例：

```bash
bash .github/scripts/setup-swiftshader.sh
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
  MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask golden check
```

Linux 使用 GPU 指南中脚本对应的 `VK_DRIVER_FILES`／`VK_ICD_FILENAMES` 设置，以及上述三个 `MIXTURE_GPU_*` 值。`MIXTURE_SWIFTSHADER_SOURCE` 可指向已有检出，替代 `tmp/swiftshader/source`。工具核对实际 Git HEAD 与夹具固定版本，并拒绝已跟踪源码的改动；源码路径、系统／架构及加载器环境与实际适配器分别记录。这是源码与适配器证据，不是封闭构建的二进制认证。干净源码检出本身不能证明任意加载器配置使用了由它构建的二进制。

## 两步基准工作流

1. 运行软件检查。新的 `tmp/golden/<id>/software-<run>/` 目录保留全部用例／通道 PNG、doctor／渲染报告、`report.json`、`candidate.json` 和审查图。硬件比较写入独立 `hardware-<run>/`。`latest-software.json`／`latest-hardware.json` 标识对应运行。
2. 查看完整分辨率 PNG、全部 `contact-<case>.png`、`overview.png` 和 `tiling.png`。接受语义改动前先解释原因。新材质显示 **BEFORE MISSING**，缺失图用斜线占位；缺失数据不会伪装成零误差。差异 RGB 是绝对分量误差放大四倍，alpha 误差也计入三个差异分量；报告使用未放大的数值。
3. 仅在接受已审查的软件候选时运行 `golden update <id> --accept`。它拒绝存在 `CI` 或 `GITHUB_ACTIONS` 变量（包括 `CI=false`）、不完整／失败候选、过期源码或基准、被修改的产物、不安全路径，以及重复接受同一候选。替换前重新检查 PNG 元数据、结构和因果关系。不调用渲染器或 Git 暂存／提交。
4. 更新先准备完整新目录，将旧 `expected/` 保存在审查目录下，再安装新 `expected/`。若移动旧目录后安装失败，会尝试恢复旧目录并报告错误。`update.json` 记录新旧哈希、已审查报告摘要和显式接受标志。审查目录继续保留；不会自动提交验收证据。
5. 再次运行软件检查和硬件比较，将有用的报告保存在夹具 `reports/` 中。真实人工审查单独记录。`--accept` 接受像素文件，不能宣称人类已完成视觉审查。

候选通过 SHA-256 绑定运行时源码、着色器、工具源码、清单／锁文件、工具链、源材质、验收契约、参数变体和产物字节。清单顺序确定。渲染期间源码变化会使候选失效。这避免意外接受过期结果；本地审查文件并非数字签名，也不防御故意同时重写清单和哈希的行为。已有基准损坏会在渲染前失败，应调查原因，不能为通过测试而覆盖。

## 夹具与 schema

[JSON Schema](../fixtures/materials/acceptance.schema.json)描述仓库验收 v1。权威的类型化解码与跨字段检查位于 [model.rs](../xtask/src/golden/model.rs)，与 `.mix` 源格式独立。

```text
fixtures/materials/<id>/
  material.mix                  版本化源图
  acceptance.json               机器验收契约
  README.md / README.zh-CN.md    意图、参数、限制、复现步骤
  variants/<case>.json           暴露参数覆盖
  expected/manifest.json         固定驱动、用例计划哈希、PNG 哈希
  expected/<case>/<channel>.png  软件适配器参考输出
  reports/                      保留的比较、对照图及审查记录
```

V1 要求以 1024×1024 请求 `baseColor`、`normal`、`roughness` 和 `height`；首个用例为 `default`，随后至少两个参数变体，总数不超过十六个。ID 由小写 ASCII 字母、数字和连字符组成，以字母开头。每个非默认用例引用同名 `variants/<id>.json`，包含非空 `overrides` 对象。覆盖值传入公共 CLI `--set` 选项，其语义仍由编译器负责。

每个通道声明编码及连接／默认来源。每个用例为四个通道提供结构检查。默认用例没有因果规则；每个变体均为四个通道指定规则，包括保持不变的通道，且至少声明一个有意义的变化。未知字段、非法阈值、缺少通道、重复 ID、奇数／欠采样／不能整除的棋盘格数量、错误驱动版本均在渲染前失败。此 1K schema 用于 PR-008／009；代表性 2K 证据属于 PR-010。

## 度量含义

| 验收项 | 定义 |
| --- | --- |
| PNG 契约 | 尺寸精确、RGBA8、无动画；颜色为 sRGB，标量／法线为线性元数据；成功 CLI 输出含连接／默认来源。 |
| 非有限值／范围 | 现有 `mixture-wgpu` 回读在编码前检查每个 f16 分量，拒绝 NaN、无穷大和 `[0,1]` 之外的值。完成渲染因此证明没有被拒绝的分量；不会用 PNG 整数反推隐藏浮点值。该边界测试保留于 `readback.rs`。 |
| 通道统计 | 各 RGBA 分量的最小值、最大值和均值，使用编码后的 0–255 单位，不是线性光颜色误差。 |
| 基准比较 | 分量及整体最大／平均绝对误差；任意分量变化的像素比例；超过 `pixelThreshold` 的像素比例。所有阈值均须通过，边界包含。软件要求解码 RGBA 精确一致，硬件使用夹具容差。 |
| 均匀结构 | 每个分量处于指定哨兵值容差内。中性法线和零高度是有意的常量，不是细节生成失败。 |
| 交替结构 | 恰好两种 RGBA 颜色、占比平衡、最小 RGB 对比、每行／列含环绕的预期跳变数量，以及移动两个格子后周期误差为零。 |
| 接缝 | 比较各轴环绕跳变与内部格线跳变。棋盘格在环绕处有意换色；零额外跳变和正确周期共同证明其接缝契约，不宣称通用连续材质接缝度量。 |
| 参数因果关系 | 未变化通道精确一致；最低变化像素比例；或标量红通道归一化至 `[0,1]` 后的最小均值增幅。仅看直方图不能证明频率变化。 |

对照图包含标签，采用最近邻缩略图。线性标量和法线字节在 sRGB 对照图中作为诊断色块展示，原始线性 PNG 及元数据仍为权威输出。2×2 平铺预览由已渲染像素拼接，不是第二种语义执行器或三维着色预览；不能证明光照质量、生成表面细节或真实感。

陶瓷夹具还提供可选的[受控 Blender 观感评审](../fixtures/materials/glazed-ceramic/review/README.zh-CN.md)：它在固定的外部场景中消费经过验证的导出 PNG，并记录输入、脚本及输出哈希。[对照图](../fixtures/materials/glazed-ceramic/reports/pbr/comparison.png)展示相同灯光下的亮面／哑光响应与图案密度。此评审属于夹具工具，不进入 Mixture 运行时，不执行 `.mix`，也不替代贴图基准或人工决定。

## 验证与验收状态

```bash
cargo test --locked -p xtask golden
cargo test --locked -p mixture-wgpu readback
cargo xtask test-material glazed-ceramic
cargo xtask golden check
cargo xtask check
```

[材质记录](../fixtures/materials/glazed-ceramic/README.zh-CN.md)区分机器比较、智能体视觉检查和人工批准。工具不会伪造人工验收。PR-009 的噪声／法线、资源生命周期优化、2K、Web 查看、新节点及远端 CI 关闭均不在 PR-008 范围内。

## PR-009 空间与法线验收

[皮革夹具](../fixtures/materials/leather/README.zh-CN.md)使用新增的三种节点，以及下列增量机器检查。原陶瓷 schema／像素不变。验收 schema 版本仍为 1，`relationships` 为可选字段，严格 Rust 验证继续检查跨字段范围约束。

| 检查 | 定义 |
| --- | --- |
| `spatial` | 红分量跨度、标准差、两轴循环相邻 Pearson 相关，及重复边界均值跳变／内部邻接均值跳变比；分母至少一字节以处理量化。所有像素 alpha 不透明。 |
| `normal` | 解码 XYZ 后的最大单位长度误差、平均 `1-nz` 倾斜、正 Z、不透明 alpha 与上述接缝比。 |
| `normalizedGradientEnergy` | 两轴相邻红分量平方差均值（含循环）除以红方差，再取变体／默认值比；区间必须排除 1，另要求最小变化像素比。常量默认值不能通过。归一化防止纯对比度变化冒充频率变化。 |
| `heightNormalDirection` | 比较导出高度的循环中心差分符号与法线 X／Y 符号，仅计高度差至少四字节的轴样本，中性法线分量不算一致；同时要求最小覆盖率和高于偶然的一致率。图像 v 向下、切线 Y 向上。不重建预期法线像素。 |

皮革的 detail 最小／默认／最大值及更粗 grainScale 必须同时通过逐图基准、空间、法线与因果约束。受控 PBR 图只消费实际 PNG，以同一法线表达高度坡度，不叠加第二次 bump。详见[皮革观感评审](../fixtures/materials/leather/review/README.zh-CN.md)。陶瓷已获用户接受；皮革人工评审已获用户接受。远端 CI 和 PR-010 的方向木纹／2K 工作保持开放。
