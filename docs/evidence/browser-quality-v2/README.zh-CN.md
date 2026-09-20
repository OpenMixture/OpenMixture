# 浏览器质量 v2：独立控制案例

[English](./README.md) | 简体中文

2026-09-20。本文是 [v2 工程规则](../../browser-quality.zh-CN.md)应用于浏览器前的冻结证据，不是浏览器支持收据。控制案例在将 v2 应用于任何浏览器材质输出前生成。阈值依据量化和响应预算选择，不依据浏览器通过率。

[calibration.json](./controls/calibration.json)保留全部 40 项预先规定的正负标签及测量，全部符合预期。目录保留全部原图/扰动/差异对照图，[control-sha256.json](./control-sha256.json)绑定确切字节。Agent 检查了代表性的法线平衡噪声、高度局部偏移和高度细节抹除图；不代表人工材质批准。测试只消费生成的 64×64 数组，不执行 `.mix` 或 GPU。

从仓库根目录复现：

```sh
cargo test --locked -p xtask browser_quality
cargo xtask browser-quality-calibrate tmp/quality-controls-new
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/default-browser.test.mjs
```

输出目录必须全新。Rust 测试另外覆盖法线/响应解析值、标量/Alpha 编码、方向偏转、尖锐粗糙度敏感性及比较对称性。一项低粗糙度单级扰动在幅度/偏移通过时，按预期未通过响应门槛，防止把 v2 理解为无条件接受单级差异。

## 冻结后结果：2026-09-20

规则与指标实现在 `c85972e23ad684d267af59c9c607d1306a9ffed0` 冻结后才开始评估。[results.json](./results.json)将以下**全新普通配置**执行绑定到安装归档 `08cdce838500343b34d775a62058d6adfd5c6035f109d86e03e5ac1ff11f1ec2`、引擎 `ec571816026a945a706067769fef76e24c8398b0` 和同一记录的 Native 参照。这里有意用新规则测试已有包，不声称 Windows 执行安装了 PR #19 新归档。主机为 Windows 11 / NVIDIA GT 1030；确切主机、驱动和适配器信息保留在各收据中。

| 浏览器 | v2 通道 | 原稀疏像素通道 | 语义 / 结构 / 生命周期 |
|---|---:|---:|---|
| Chrome 153.0.8010.48 | 44/44 | 27/44 | 通过 |
| Edge 153.0.4234.32 | 44/44 | 27/44 | 通过 |
| Firefox 156.0 | 44/44 | 44/44 | 通过 |

Chrome/Edge 最大分量差仍为 **1**，局部有符号偏移最大 `0.01171875`（预算 `0.25`），法线角度 `0.5953195122435492°`（预算 `1°`），高度一阶差分误差 `1/255`（预算 `2/255`），粗糙度采样响应差 `0.00875639734842093`（预算 `0.05`）。Firefox 像素完全一致；报告中约 `0.0000017°` 法线角是测量浮点舍入，不是像素差异。看到结果后没有调整任何阈值。全部旧失败继续保留在 `legacyComparison`。

[Chrome 生产页](./production-chrome/receipt.json)及 [Edge 生产页](./production-edge/receipt.json)另通过了精确 65×3 checker 下载、正常静态部署、测试入口缺失及释放检查。未运行 Firefox 生产下载检查：该验证器仍只支持 Chromium。Agent 检查了[陶瓷](./visuals/ceramic-default.png)、[皮革](./visuals/leather-default.png)、[最大细节皮革](./visuals/leather-detail-max.png)、[木材](./visuals/wood-default.png)和[横向木材](./visuals/wood-horizontal.png)对照图，在保留的审查尺度上未观察到可见结构变化。这是 Agent 比较审查，不替代原生金图的人工批准或穷尽感知验证。

冻结提交的六项 CI 全部通过。独立 [Linux Chromium 资格](./ci/qualification.json)消费来自实际 PR 合并检出 `3f0939d1dc925a484b483fc2c5fca9817b089332` 的**新归档**（`0dfa42bf849fd0cfc7f91eab730edb6f92b167f74b0552f01212d0dd241f16ed`）。[运行 35485669447，第 1 次](https://github.com/OpenMixture/OpenMixture/actions/runs/35485669447)通过控制案例、安装/构建身份、28 项浏览器契约、11 案例/44 通道、12 次压力渲染及生产部署。受控 Chromium 参数不构成普通配置声明。[artifact 记录](./ci/artifact.json)说明提取与过期时间。最终文档/启动器后续修改的检查属于其各自 PR head，不属于本实验提交。

## 证据完整性、失败与重放

[attempts.json](./attempts.json)保留被拒绝的历史输入及工具运行。部分旧本地 Chrome/Edge PNG 文件头无效，旧 PR #16 Native PNG 未通过清单摘要检查。这些来源被排除，没有修补或用于拟合阈值；新运行补齐了证据。Edge 首次运行在渲染前失败，因为它创建了带 `--edge-skip-compat-layer-relaunch` 的直接子进程。启动器现记录实际操作系统父子进程/可执行文件，验证器将其绑定到 CDP 浏览器 PID 和完整命令行。启动器从不传入该标记，任意其他参数继续拒绝。启动器进程若在观察前退出，会明确记录，而不会伪造已观察命令。聚焦测试覆盖父进程/PID/可执行文件变化、缺失子进程证据，以及额外 GPU/无头/安全参数。本启动器修改不改变已冻结数值规则。

Git 保留**完整 176 张 1024² RGBA8 解码纹理**：44 张 Native，加每浏览器各 44 张。使用相对仓库现有软件金图的无损压缩 XOR 差分。[matrix/index.json](./matrix/index.json)绑定原 PNG 摘要、基准解码摘要、载荷及重建 RGBA 摘要；[matrix/sha256.json](./matrix/sha256.json)绑定保留的元数据和载荷。不重建原 PNG 压缩码流；原始解码像素可供全矩阵审计，不限于选定见证。原生金图字节始终不变。

```sh
python docs/evidence/browser-quality-v2/verify.py
python docs/evidence/browser-quality-v2/verify.py tmp/v2-replay-new
cargo xtask browser-material-check tmp/v2-replay-new/native tmp/v2-replay-new/chrome
cargo xtask browser-material-check tmp/v2-replay-new/native tmp/v2-replay-new/edge
cargo xtask browser-material-check tmp/v2-replay-new/native tmp/v2-replay-new/firefox
```

需要 NumPy/Pillow。新重放目录包含明确标识为**派生**的 PNG 编码/清单及 `DERIVED-REPLAY.json`，属于离线诊断，不是浏览器或包资格。本地重放验证了全部 176 张纹理哈希，并精确复现全部 132 个浏览器通道的所有指标、结构/因果及新旧判定。生产截图、选定材质对照图和机器收据留在 Git。Linux 完整常规像素/日志留在有期限 artifact 中，不承诺该独立 Linux 执行的永久全运行审计。

范围是已测纹理/响应预算及记录的包/环境。本结果不扩展为普遍设备支持、任意 BRDF/物理位移承诺，不接受 PR #15/#16，不改变原生基准或发布包。剩余 warp 准确性工作仍可独立审查，但不能继续仅用旧浏览器变化像素数量作为修复理由。
