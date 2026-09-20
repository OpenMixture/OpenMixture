# ALPHA-04 准确候选与 Studio 升级 — 2026-09-20

[English](./README.md) | 简体中文

**记录范围内的候选与 Studio 升级门槛通过。**[发布说明与支持范围](../../browser-alpha-candidate.zh-CN.md)标识未发布的 `0.1.0-alpha.0` 归档。这是准确候选的新执行，不是旧 Studio 像素的离线重比较。注册表发布、托管部署、真人试用结果及 ALPHA-05 必需检查生效仍属独立工作。

## 绑定源码与执行

干净生产者为 `82b74707b2a8a998190e2f28b16f91fb9614486a`；干净 Studio 创作／Player 修订为 `6b2d53e3de16b21725b2a4359a2263f98671a6f9`，基于产品 `28d5e6a`。仅 vendor 归档、生产者回执与 runtime 锁条目改变。[生产者回执](./producer.json)、[候选替换](./candidate.json)、[安装](./installed.json)和[实际浏览器探针](./build-probe.json)绑定 SHA-256 `a9bcfe8d849f9fb9982a750dd99d026807fdf6453b5be491deeda90e99a2c6ae` 与构建 ID `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28`。混入历史 WASM 会被 `MIX_BROWSER_BUILD_MISMATCH` 拒绝。

准确归档通过引擎 [Linux CI 验收](https://github.com/OpenMixture/OpenMixture/actions/runs/35489430243)，attempt 1：28 项固定消费者契约、11 材质用例／44 通道、12 次生命周期渲染和生产部署。[验收回执](./ci-qualification.json)、[材质回执](./ci-material-receipt.json)和[比较](./ci-comparison.json)保留结果。该 CI 消费者为 `56c510a`；下述独立 Studio 升级测试 `6b2d53e`。生产者修订也通过三个原生 CPU 作业和固定原生 GPU 工作流。

| 新 Studio 验证 | 结果 |
|---|---|
| Windows 干净 npm 安装、公开类型、20 项单元测试、生产构建 | 通过 |
| Windows Chromium 153.0.8010.12 浏览器契约 | 52 项通过，无跳过／不稳定／失败 |
| 实际构建身份与历史 WASM 拒绝 | 通过 |
| 正常生产静态部署 | 通过；双入口、WASM MIME、无测试入口 |
| Windows 11 build 26100 普通 Chrome 153.0.8010.48 | 完整编辑／修复／历史／保存／Player／四通道 PNG／释放流程通过 |
| Windows Studio 保存文件／原生比较 | 七个 1024×1024 用例，28/28 通道通过 |
| Linux／WSL 文件系统隔离安装／check／浏览器／部署／Player | 通过；20 单元测试、52 浏览器测试、七保存文件用例 |
| 隔离 Linux Player／原生比较 | 七个 1024×1024 用例，28/28 通道通过 |

[普通浏览器回执](./ordinary/receipt.json)记录实际可执行文件／命令行、请求及可观测 GPU 身份。仅传入新 profile／CDP／about:blank 参数，没有 GPU 覆盖。独立注入的 GPU 不可用探针保持 CPU 图编辑可用；不是自然不支持硬件测试。Studio 与 Player PNG 字节及 sRGB／线性元数据符合棋盘预期。

原生参考使用干净匹配的引擎源码、标准 debug CLI、显式 DX12 与 NVIDIA GeForce GT 1030。完整上下文与驱动信息保存在各原生结果中。受控 Windows Chromium 使用记录中的 unsafe-WebGPU／blocklist 测试参数及 Playwright 默认值。Linux Chromium 在 WSL2 `6.18.33.2-microsoft-standard-WSL2` 中附加显式 SwiftShader 参数。浏览器适配器字段按实际暴露值记录，不能用主机硬件推断隐藏身份。Node 为 24.20.0，npm 11.19.0，Playwright 1.63.0。

## 保存文件、隔离与视觉证据

[创作下载](./authored.json)绑定真实 Studio 保存操作。[原生 manifest](./native-manifest.json)、[Windows Player](./windows-player.json)、[Linux Player](./linux-player.json)及其 [Windows](./windows-comparison.json)／[Linux](./linux-comparison.json) 比较绑定相同保存字节、全部及逐通道 plan、导出和候选。两套比较均通过冻结 v2 数值、结构、因果与关系规则，包括精确棋盘验证。各有 12 个通道字节精确；其余 16 个通道最大分量误差为 1，最大局部偏差为 0.0078125，低于冻结上限 0.25。本任务未修改 shader、基线、profile 或保存 fixture。

[隔离步骤](./isolation-recipe.sh)、[沙箱检查](./isolation-checks.sh)和[回执](./isolation.json)保留文件系统边界。两个源码检出及 Windows 挂载／home 均未挂载，直接读取失败，`cargo`／`rustc` 不存在。复制的消费者具有独立 Git 数据和已安装归档。网络仍可用于 npm。Windows／Linux 原始锁摘要因换行符不同而不同；完整解析锁图已验证相同，包括归档 SHA-512。

代理检查了全部七张 Windows 原生／Player／差异图及 Linux 创作皮革／木材图：棋盘交替、陶瓷平铺、皮革纹理与木纹方向在视觉上吻合。代表图：[棋盘](./checker.png)、[陶瓷](./glazed-ceramic.png)、[皮革](./leather.png)、[木材](./wood.png)。这是代理检查，不是新的人类 golden 验收。

## 保留与复现

[摘要](./summary.json)记录保留内容哈希。[数据包](./saved-file-bundles.tar.gz)及[索引](./evidence-index.json)保留全部 152 个源码／原生／Windows／Linux 文件，包括每张比较 PNG、plan、上下文、截图与联系图，并保留实际 runtime 归档。校验器回读所有保留文件，可恢复并逐一校验数据包成员：

```sh
node docs/evidence/alpha-04/verify.mjs tmp/alpha04-replay
cargo xtask studio-material-check tmp/alpha04-replay/native tmp/alpha04-replay/windows
cargo xtask studio-material-check tmp/alpha04-replay/native tmp/alpha04-replay/linux
```

重新执行时检出两个绑定修订，按记录锁在 Studio 安装保留归档，运行 `npm ci`、`npm run check`、`npm run test:browser`、`npm run test:deployment`、`npm run test:ordinary -- chrome <fresh-output>` 和 `npm run capture:studio -- <fresh-downloads>`。引擎运行 `MIXTURE_GPU_BACKEND=dx12 node scripts/browser-runtime/prepare-studio.mjs <fresh-native> 82b74707b2a8a998190e2f28b16f91fb9614486a <fresh-downloads>`；随后 Studio 运行 `npm run test:studio -- <fresh-native> <fresh-player>`，引擎运行 `cargo xtask studio-material-check <fresh-native> <fresh-player>`。PowerShell 须采用对应环境变量赋值语法。隔离步骤记录准确本地路径，重跑前须准备工具与路径。

首次本地 capture 因默认 Playwright 缓存的 Windows 并行配置错误无法启动，换用已有同版本 Chromium 后成功。首次仓库总检查到达打包 rustdoc 时以 OS `STATUS_IN_PAGE_ERROR` 退出；最终重跑结果单独记录于整合验证。两次失败均未当作通过。完整例行日志／编译产物及原始 55 MB CI 下载仍位于忽略的 `tmp/alpha04-*`。CI artifact `10598493452` 到期时间为 `2026-10-20T04:42:43Z`；保留的关键内容不依赖其期限，但完整 CI 像素／日志审计依赖该制品。Git 保留完整的新 Studio 比较，不保留整份重复 CI 数据包。

[本地最终检查](./local-check.json)：移走旧打包缓存后，完整 `cargo xtask check` 通过（含独立包消费、Rustdoc 和 185 份文档链接）。[缓存失败原文](./cached-rustdoc-failure.txt)保留 OS 错误，未修改源码规避检查。PR 的当前远端检查另行验证。
