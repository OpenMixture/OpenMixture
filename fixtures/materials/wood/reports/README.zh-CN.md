# PR-010 证据

[English](./README.md) | 简体中文

**存储更新（2026-09-25）：** 普通 smoke 日志及逐次 CLI 输出已移至[完整历史归档](../../../../docs/evidence/archives/README.zh-CN.md)，使用 `early-runs` 组恢复；环境、测量、人工验收图像及失败／精度实验保持可查。下文描述原始运行，不声称全部附件仍在当前目录。

全部三种材质在本地固定 SwiftShader Vulkan 与 Apple M5 Metal 上通过 1024×1024 检查。软件解码 RGBA 精确一致；硬件使用各材质声明的容差。[木材人工评审](./human-review.json)已接受。远端 CI 仍暂缓；工作流已加入相同的三材质检查与 2K 追踪，但不宣称远端结果。

## 材质比较

| 材质 | 软件 | Metal | 案例／连接通道 |
| --- | --- | --- | --- |
| 釉面陶瓷 | [通过](./glazed-ceramic-software-regression.json) | [通过](./glazed-ceramic-metal-regression.json) | 3 组；可选 normal／height 为有意采用的默认值。 |
| 皮革 | [通过](./leather-software-regression.json) | [通过](./leather-metal-regression.json) | 4 组；四通道全部连接。 |
| 木材 | [通过](./software.json) | [通过](./metal.json) | 4 组；四通道全部连接。 |

陶瓷／皮革 expected 目录相对已接受的父提交保持不变。木材有十六张首版基准 PNG。[首次基准接受](./bootstrap-acceptance.json)记录独立 `--accept` 操作，没有渲染或 Git 操作。[初始软件测量](./software-initial.json)如实报告缺少旧 golden；`initial-contact-*.png` 显示 BEFORE MISSING。最终 `software-candidate.json` 与 `metal-candidate.json` 绑定当前运行时、着色器、工具、材质、契约、变体与产物哈希。

代理已检查[总览](./overview.png)、[2×2 重复平铺](./tiling.png)与[受控 PBR 对比](./pbr/comparison.png)。[观感报告](./pbr/review.json)绑定实际基准、两份场景脚本、四张 1000×1000 图及显式 Blender 4.5.13／Apple M5 Metal 设置。全部图像采用 512 样本、种子 8，以及相同相机、灯光、几何与 BRDF。导出法线只应用一次；高度为诊断用途。这些图像支持人工决定，但不代替该决定。

| 木材案例 | 高度横跨／沿纹理能量比（SwiftShader） | 声明的纹理轴 |
| --- | ---: | --- |
| 默认 | 209.03 | 纵向 |
| 粗纹 | 75.10 | 纵向 |
| 直纹 | 274.47 | 纵向 |
| 四分之一圈旋转 | 273.93 | 横向 |

最小能量比为 4，另有独立的跨度、标准差、邻域相关与接缝门槛。两个受测适配器的高度／法线符号一致率均为 100%。粗纹具有更低的归一化梯度能量；每个变体均改变全部四通道。完整测量与逐通道比较误差保留在链接报告中。

[初始粗纹法线测量](./metal-initial-measurement.json)保留建立基准前的阈值失败。之后的[首次硬件比较失败](./metal-initial-tolerance-failure.json)保留默认高度 222 像素、粗纹高度 2 像素相差两字节的记录。[中间导出与测量](./precision/measurements.json)显示 levels 前最多差一字节，重映射后最多差两字节。木材最终硬件范围为最大 2 字节、RGBA 均值 0.20，超过一字节的像素最多 0.025%（1K 下为 262 像素）。精确软件 PNG 与全部空间／因果门槛均保留。这是受测适配器的经验范围，不是对所有驱动的保证。

precision 目录包含实际诊断 `.mix`、PNG 与 CLI 报告。复现中间阶段时，使用现有 render 接口、`--size 1024 --output height` 与显式后端，例如：

```bash
cargo run --locked --all-features -p mixture-cli -- render fixtures/materials/wood/reports/precision/warp.mix --size 1024 --output height --out tmp/wood-warp-probe --backend metal --json
```

软件对照使用固定 loader 与 `--backend vulkan --software`。`height-linear.mix` 探针还将 gamma 改为 1。CPU 测量只比较导出字节，不执行节点公式。

## 最大案例 2K 追踪

[Metal 追踪](./2k/metal/trace.json)与[软件追踪](./2k/software/trace.json)各自以 2048×2048 编译全部十一组配置案例，并选取 `wood/default`。木材与陶瓷均有八个 pass，但木材 uniform 描述符使其峰值多出 48 字节。按估算峰值内存选取，不依据想象的视觉复杂度或耗时。各目录保留全部 inspect 结果、准确选取、doctor、渲染报告、四张全尺寸 PNG 及哈希。

| 测量 | 两个适配器 |
| --- | ---: |
| Pass／纹理／uniform 数量 | 8 / 8 / 8 |
| 纹理字节 | 268,435,456 |
| Uniform 字节 | 224 |
| Staging 分配次数／累计字节 | 4 / 134,217,728 |
| 峰值 staging 字节 | 33,554,432 |
| 累计估算／实测字节 | 402,653,408 / 402,653,408 |
| 峰值估算／实测字节 | 301,990,112 / 301,990,112（约 288 MiB） |
| 预算 | 536,870,912（512 MiB） |
| 已复用／最终存活字节 | 0 / 0 |
| 显式释放字节 | 402,653,408 |

| 适配器 | 管线 ms | 执行 ms | 回读 ms | 渲染器总 ms |
| --- | ---: | ---: | ---: | ---: |
| Apple M5 / Metal | 12.46 | 26.13 | 4091.53 | 4131.89 |
| SwiftShader Device (LLVM 10.0.0) / Vulkan Cpu | 111.45 | 381.42 | 4048.04 | 4541.48 |

这是开发构建中的单次验证运行，使用 CPU 墙钟测量。不是 GPU 时间戳查询或交互延迟基准。回读包含映射等待、半精度验证与转换；渲染器总时间不含 CLI PNG 文件编码。描述符字节不含驱动／管线／绑定组开销与 CPU 缓冲区。释放代表显式 `destroy`，不意味着操作系统立即归还物理内存。追踪验证 1K 基准未变，实际分配计数独立于核心估算。朴素生命周期符合预算，因此没有引入最后消费者释放、池化或兼容纹理复用。

## 验证与复现

`metal-gpu-smoke/` 与 `software-gpu-smoke/` 是完整冒烟运行的独立顺序捕获。节点报告包括十九个图案例与三十个字面重采样探针（17 个 transform、13 个 warp）。还覆盖奇数行宽下的分配计数、别名输出、逐次重置及失败释放。独立 transform／warp 套件已在固定适配器通过。`trace-tests.log`、`direction-tests.log`、`shader-check.log`、`clippy.log` 与 `repository-check.log` 保留专项及最终检查。CLI／诊断 JSON 按原始输出保存；文本日志仅可能移除结尾空行。

```bash
cargo xtask test-node transform-2d
cargo xtask test-node warp
cargo xtask test-material wood
cargo xtask golden check
cargo xtask trace-2k
cargo xtask gpu-smoke
cargo xtask check
```

GPU 命令使用[材质复现](../README.zh-CN.md)中的显式策略环境。冒烟策略须顺序执行，因为现有冒烟命令写入 `tmp/gpu-smoke/`；切换策略前复制证据。材质与 2K 任务创建唯一运行目录。[观感复现](../review/README.zh-CN.md)为可选外部工具。CI 不得更新基准；这些机器结果不暗示人工批准或 M4 实施。
