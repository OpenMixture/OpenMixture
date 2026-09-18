# warp 显式插值候选 — 2026-09-18

[English](./README.md) | 简体中文

**未获准合并。** [PR16](https://github.com/OpenMixture/OpenMixture/pull/16)在 [PR15](https://github.com/OpenMixture/OpenMixture/pull/15)之上独立验证显式插值。已测 Chrome/Edge 的 half 临界点回归得到修复，原生 DX12 和固定 SwiftShader 均通过全部 23 个字面值用例；冻结软件木材 golden 仍失败。未修改基线或容差。

## 源码与测试

测试源码为 `1c65e8631c2e2bdba8fe3aaf8b7c5762c1c215f3`，父提交为 `91015e02255f20105586145db23bbd23b3c93546`。后续证据提交不是被测包源码。唯一生产 warp 着色器将三处插值改成 `fma(b-a,t,a)`，坐标、位移、环绕和 half 转换保持不变。WGSL 允许非融合 fma，因此不保证普遍逐位一致。

新增六个字面值用例覆盖 3×2 纹理的双轴不同权重、正负方向跨接缝、大幅反向位移、完整周期、细长奇数高度纹理及已缩减的 half 临界点。预期为精确字面值，不是 CPU 渲染器或重建 golden。原生改动前后均通过；Chrome 改动前仅临界点失败，改动后全部 23 项通过，Edge 改动后亦全部通过。[浏览器工具](./browser-literals.mjs)只转换纹理存储格式，像素计算执行生产 WGSL。

| 验证 | 结果 |
|---|---|
| 着色器验证、原生 warp 节点及 23 个字面值用例 | 通过 |
| 固定 SwiftShader GPU／节点／打包消费者及 23 个用例 | 通过 |
| 本地 `cargo xtask check` | 通过 |
| 三种材质的原生硬件 golden | 按现有硬件门槛通过 |
| 原生 release PNG 与 PR15 对比 | 44 个通道全部逐像素一致 |
| 普通 Chrome 的已安装候选包 | 冻结浏览器门槛通过 27/44，19 个完全一致 |
| 冻结 SwiftShader 陶瓷／皮革 golden | 通过 |
| 冻结 SwiftShader 木材 golden | 12/16 通道失败，最大分量差 1 |
| CI 2K trace | 金图失败后跳过 |

详见[摘要](./summary.json)、[CI 快照](./ci-checks.json)、[软件字面值结果](./software-literals.json)、[软件金图报告](./software-report.json)和 [Chrome 比较](./chrome-comparison.json)。本机 GT 1030 使用 DX12、驱动 32.0.15.8266 和 Chrome DXC，原生参照为 release 构建。浏览器版本为 Chrome 153.0.8010.48、Edge 153.0.4234.32。Edge 启动脚本未观察到短暂父进程，已记录仍存活的专用配置进程；本次 Edge 仅覆盖直接字面值诊断，不宣称普通配置下已安装运行时认证。没有新增 Firefox 测试。

## 像素影响与决策

相对 PR15 的固定软件输出，木材 7 个通道各变化 1–4 个像素，最大 1 字节：coarse-grain 的 baseColor/height/normal 分别变化 2/1/1；default 的 baseColor/height/normal/roughness 分别变化 2/3/4/1。straight-grain 和 horizontal-grain 不变。相对原始冻结 golden，default 的 baseColor/height/normal/roughness 分别有 244/254/380/61 个差异像素。结构、有限值／范围与材质关系检查通过，但精确软件门槛仍有效。

[全部新增软件差异像素](./software-witnesses.json)及[前后对比裁切图](./software-pr15-pr16-crops.png)保留了这些变化。裁切图三列依次为 PR15、PR16、绝对差 ×255，每个 32×32 区域放大 4 倍。[冻结金图接触表](./software-contact-default.png)另行比较旧验收基线与 PR16。完整 default height/normal PNG 保留在 `before/`、`after/`。视觉检查可见稀疏量化变化，这不能豁免数值门槛，也不构成人工验收。

该候选改善了已测插值见证，但没有证明软件像素兼容，也未完成完整材质认证。PR16 保持草稿。后续应明确真实局部坐标修正的兼容处理，并独立处理剩余噪声算术差异；挑选有利参照或放宽容差不能算修复。本次未增加编译策略、依赖分叉或上游 issue。

## 复现与保留

在被测提交运行 `cargo xtask shader-check`、`cargo xtask test-node warp`、`cargo xtask golden check` 和 `cargo xtask check`。原生策略为 `MIXTURE_GPU_BACKEND=dx12`、`MIXTURE_GPU_SOFTWARE=0`、`MIXTURE_GPU_EXPECT_ADAPTER=NVIDIA`，PATH 使用已记录 Chrome 目录。[GPU CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35316278778)按未修改的工作流构建 SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`，使用 Vulkan/software/SwiftShader 策略。已保留[下载产物身份和到期时间](./gpu-artifacts.json)。

复跑浏览器字面值时，将生产 `precision.wgsl` 和 `nodes/warp.wgsl` 拼接到临时文件，设置 `MIXTURE_PROBE_PRODUCT` 为已安装 Playwright 的锁定 Studio 目录，执行 `node browser-literals.mjs <CDP-endpoint> <native-literals.json> <combined.wgsl> <fresh-output.json>`。改动前使用父提交着色器，输入保持一致。普通 Chrome 材质测试使用仓库的 `candidate.mjs`、`prepare-materials.mjs`、`default-browser.mjs` 流程；[包身份](./package-candidate.json)、[原生清单](./native-manifest.json)及浏览器回执绑定归档、源码、输入和冻结标准。

Git 保留关键成功／失败内容和选定图像。完整常规日志、重复材质 PNG 与构建产物留在忽略的 `tmp/warp-fma-*`；完整 CI 产物按记录日期过期。未宣称外部永久归档、发布、视觉验收或完整材质认证成功。

在仓库根目录运行 `python docs/evidence/warp-explicit-interpolation/verify.py`，可无 GPU 核验保留的字面值和门槛结果。源码提交的 Linux、macOS、Windows 检查、WASM 打包及配置过的 Linux Chromium 矩阵均通过。
