# 局部坐标候选验收 — 2026-09-17

[English](./README.md) | 简体中文

**不能将这项改动单独作为正式修复。** 草稿 [PR #14](https://github.com/OpenMixture/OpenMixture/pull/14) 的候选 `e41410859c457c13739198c580bc1e1e2af45ed9` 改善了数值稳定性，但仍未通过原有软件材质基线和 Windows 普通 Chromium 门槛。本次没有修改基线、容差、依赖策略或证据分支的生产着色器。这完成了[稳定性研究](../shader-stability/README.zh-CN.md)之后的采用评估，不代表 Chrome/Edge 认证完成。

## 受控候选

候选基于 `2cc36863eb5cb4a0f419722b172fb5aceaec239f`，仅应用已有[一行补丁](../shader-stability/diagnostic-local.patch)，将 cellular 距离计算改为局部坐标。上轮已在修改前记录原始节点和材质检查；`git diff 2cc3686 909de3e -- crates fixtures Cargo.lock` 为空。未改 UV、插值、levels、平方根或编译策略。独立构建目录避免上轮出现的共享缓存问题。

[候选记录](./candidate.json)绑定干净源码、新包 SHA-256 `8e4ed1dc82e1a34bbd269bf04fdfe7a0719e78f2b1c49415835cddc4441047b2` 和 build ID `sha256:1b2ae86ef1e2893ade8b2babb3c0102362e9336ee4235af5ec220109e8a619b5`。固定 Studio `56c510ab57daa1b68ef660525a648a582730a37e` 安装此包，通过类型、单元测试和生产构建。包生产记录为 Node 24.21.0，消费者检查使用其要求的 24.20.0。原生参考由同一候选、原 registry 依赖、Release/DXC、显式 DX12 和固定 11 个请求生成。它用于衡量跨端一致性，旧 golden 则独立约束兼容性。

## 决定性结果

| 检查 | 结果 |
| --- | --- |
| Windows / GT 1030 主机，普通 Chrome 153.0.8010.48 | 33/44 通道通过，19 个完全相同 |
| 同主机普通 Edge 153.0.4234.32 | 33/44 通过，19 个完全相同；各通道指标与 Chrome 相同 |
| 同主机普通 Firefox 156.0 | 44/44 完全相同 |
| 固定 SwiftShader 节点、GPU 和打包消费者检查 | 通过 |
| 固定 SwiftShader 1K 材质 golden | 陶瓷、木材通过；皮革 16 个通道全部失败 |
| 皮革软件结构与关系检查 | 通过；失败来自精确像素基线 |
| Linux 配置软件适配器的 Chromium 候选 CI | 通过；这是独立的 CI 构建包和环境，不是 Windows 普通浏览器认证 |

[机器汇总](./summary.json)保留所有通道计数。Chrome/Edge 的失败包括皮革 default、detail-min、coarse-grain 的 height/normal 共 6 项，以及原有木材路径的 5 项。最大字节差仍为 1。默认皮革相对候选原生的 height 有 16 个像素变化，normal 有 37 个，原比率门槛分别只允许最多 10 和 20 个。相比旧候选，失败通道从 17 降到 11，并未通过。

软件皮革相对旧 golden 各通道变化 64–1,739 个像素，最大字节差 1。默认材质分别为 baseColor 137、height 472、normal 1,110、roughness 129。[软件报告](./software/leather.json)、[默认接触表](./software/contact-default.png)和完整分辨率[高度图](./software/default/height.png)/[法线图](./software/default/normal.png)保留该回归；另保留四个用例接触表及平铺证据。代理在接触表尺度未见明显结构破坏，这不能覆盖精确比较，也不代表人工接受。

三种普通浏览器都完成了 11 个用例、生命周期/压力检查及模拟的 unsupported、adapter、device 失败诊断，然后才进行像素比较。Chrome/Edge 的生产 Player 检查分别[通过](./chrome/production.json)[通过](./edge/production.json)，涵盖棋盘 PNG 下载、编码、释放和测试入口不存在。当前验证器不支持 Firefox 生产下载，因此不声称通过。Firefox 最初两次连接发生在调试端点就绪前，第三次完成。Chromium 最初以 Vite preview 服务生产页面，404 检查失败；换成消费者实际静态服务器后通过。没有把这些准备失败记作产品通过。

## 决定与下一步边界

数值稳定性、跨端一致性、视觉/结构质量、旧版兼容性回答不同问题。上轮固定输入研究支持大周期下的稳定性改善；本次结构检查通过，但软件旧像素不兼容，普通 Chromium 一致性也仍失败。因此保持 PR #14 为草稿，不合并、不更新 golden，保留必需 GPU CI 的失败。该结果不支持维护依赖分支。

下一步从实际候选管线缩减一个剩余皮革 height 差异点，并独立缩减未改动的木材路径。通过固定输入回读区分坐标生成、噪声、插值、levels 和 half 存储，再提出下一项改动。未来提案仍需明确兼容性决策并沿用门槛；不能靠挑选编译器或参照把结果变绿。

## 复现与保留

1. 在独立目录检出候选完整 SHA，执行 `cargo xtask shader-check`、显式适配器下的 `cargo xtask test-node fractal-noise` 和 `cargo xtask check`，不共享 target。
2. Linux 使用 `.github/scripts/setup-swiftshader.sh` 及工作流的 Vulkan/software/expected-adapter 环境，执行 `cargo xtask gpu-smoke` 和 `cargo xtask golden check`。golden 失败导致本次 CI 未运行后续 2K trace。
3. 用 `node scripts/browser-runtime/build.mjs` 构建，通过 `candidate.mjs stage`、`installed` 安装至固定 Studio。以候选完整 SHA、`MIXTURE_NATIVE_PROFILE=release`、`MIXTURE_GPU_BACKEND=dx12` 以及[原生记录](./native/manifest.json)的 `MIXTURE_DX12_COMPILER_DIRECTORY` 生成新的 `prepare-materials.mjs` 参考。
4. Studio 以 browser-test 模式构建并在 loopback 4173 提供静态资源。使用 `launch-default-browser.ps1` 启动各浏览器新配置，仅带配置隔离和调试传输开关，等待端点就绪。向 `default-browser.mjs run` 提供 product、candidate、native、launch 和新输出目录，保留失败。Chromium 生产检查需正常重新构建，使用 `scripts/static-server.mjs` 服务并执行 `default-browser.mjs production`。

[CI 状态](./ci-state.json)和[制品元数据](./gpu-artifacts.json)绑定远端结果。GPU 运行 [35201070526](https://github.com/OpenMixture/OpenMixture/actions/runs/35201070526)第 1 次尝试在皮革失败；制品 `10487694047` 于 2026-10-17T08:48:48Z 过期。CI 使用 PR 合并版本，Windows 使用候选 head。[浏览器 CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35201070610)与本地包分别记录。报告、选定完整分辨率失败图、接触表、包/消费者身份及哈希保留在 Git；完整包、全套 PNG、二进制、日志、初次准备失败和详细测试输出留在忽略的本地目录或会过期的 CI 制品中。不声称永久保留完整运行，也未接受新视觉基线。
