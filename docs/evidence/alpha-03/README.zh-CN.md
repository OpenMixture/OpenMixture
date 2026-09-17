# ALPHA-03 普通桌面浏览器验收 — 2026-09-17

[English](./README.md) | 简体中文

**结果：阻塞，未验收通过。** 两个普通浏览器均完成运行时生命周期和诊断探针，但均未通过冻结的完整材质比较。ALPHA-03 及依赖它的 ALPHA-04 发布门槛保持未完成。没有修改容差、着色器、原生 golden 或运行时实现，也不据此声明普遍支持 Windows/NVIDIA。

交付：[草稿 PR #12](https://github.com/OpenMixture/OpenMixture/pull/12)。11 项定向 Node 测试和完整本地 `cargo xtask check` 于 2026-09-17 通过；文档组装期间首次检查因记录链接尚不存在而失败，文件补齐后已重跑通过。记录撰写时远程 PR CI 仍待完成。本地工具检查通过不改变浏览器材质失败结论。

## 输入与环境

- 运行时：干净引擎 `7b1cec4ad1d42d6269ef6a9912c2e8ba3a2dfdd9`，未发布的 `0.1.0-alpha.0`；[生产回执](./package-receipt.json)、[保留归档](./openmixture-runtime-0.1.0-alpha.0.tgz)。SHA-256：`88f22ac295c3a1cc6bee2e995ed1e4683ca6669026167e4731ee10564f30d48c`。
- 独立 Studio：`56c510ab57daa1b68ef660525a648a582730a37e`；[候选及锁文件身份](./candidate.json)。同一归档通过了 [ALPHA-01 CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35112153338)，保留[验收回执](./ci-qualification.json)。受控 CI 通过不代表本次硬件环境通过。
- Windows 11 IoT Enterprise LTSC，10.0.26100，x64；NVIDIA GeForce GT 1030，驱动 32.0.15.8266。Chrome 153.0.8010.37、Edge 153.0.4234.32。全新临时桌面配置，仅 profile/CDP/about:blank 参数；各回执包含执行文件身份、实际命令行和主机／浏览器 GPU 报告。未添加 GPU、blocklist、headless 或 sandbox 覆盖参数。
- 浏览器请求 `powerPreference: high-performance`，实际返回适配器为 `nvidia`／`pascal`，不是 fallback。运行时后端为 BrowserWebGpu；适配器名称被隐藏，保持原样，不用主机清单冒充选中证据。
- 同机原生参照：显式 NVIDIA GT 1030/DX12，关闭软件选择；[清单](./native-manifest.json)。生成时验证工具提交为干净的 `7c2206203661da2c7b42dd1d4d10b9b263028394`，相对运行时 `7b1cec4…` 的源码漂移检查通过。这是新比较输出，不是替换 golden。

## 结果

所有比较使用现有 11 个案例、1024×1024、四通道和冻结容差；必须全部门槛通过，字节误差小不等于通过。

| 运行 | 像素通道通过 | 最大字节误差 | 最大变化像素比例 | 结论 |
|---|---:|---:|---:|---|
| Chrome 对历史 Linux SwiftShader 原生参照 | 12/44 | 2 | 0.21771526336669922 | 跨适配器实验失败 |
| Chrome 对同机 NVIDIA/DX12 | 33/44 | 1 | 0.0007572174072265625 | 失败 |
| Edge 对同机 NVIDIA/DX12 | 33/44 | 1 | 0.0007572174072265625 | 失败 |

两个同机浏览器均通过 44 项结构检查、实际归档／构建身份、加载／初始化／渲染、自有 RGBA 输出、回读后报告存活分配为零、后续渲染与销毁后仍保有像素、幂等销毁、已接收的在途工作完成、销毁后拒绝新工作。独立合成页面验证了无 WebGPU、无适配器和设备创建被拒绝的结构化错误，同时 CPU 验证仍可用。这不等于自然不支持环境的覆盖。

普通生产版 Chrome Player 单独通过初始化、65×3 棋盘格 PNG 精确像素与 sRGB 元数据、下载、销毁以及生产产物不包含测试入口：[回执](./production/receipt.json)、[PNG](./production/checker.png)、[截图](./production/player.png)。该有限通过不能覆盖材质失败，也不认证 Studio 打开／编辑／保存／重开流程。

查看 [Chrome 比较](./chrome/comparison.json)、[Edge 比较](./edge/comparison.json)及[跨适配器比较](./chrome-cross-adapter/comparison.json)。各目录 `receipt.json` 保留完整运行时观察与负向诊断，`ordinary.json` 保持 false，`failure.json` 保留首个验证失败。[皮革](./chrome/leather-default-comparison.png)和[木材](./chrome/wood-default-comparison.png)代表性对比图保留原生／浏览器／差异内容。检查皮革图未见明显布局或内容损坏，但这不是人工 golden 接受，数值失败仍是决定依据。编译器／驱动算术差异仅为假设，尚未确定根因。

## 复现与保留

遵循[普通浏览器流程](../../default-browser.zh-CN.md)，本次使用 Node 24.20.0。原始本地目录为 `tmp/alpha03-run`、`tmp/alpha03-paired-run`、`tmp/alpha03-edge-run-3`、`tmp/alpha03-native-windows` 和 `tmp/alpha03-production`。原生及首次 Chrome／生产运行使用工具提交 `7c220620…`；后续增加实际返回适配器观察，Edge 另规范化命令行末尾空白。这些工具修改没有改变运行时字节；PR 差异记录修改，不能把本次结果移用于新运行时候选。

Git 保留实际候选归档、含源文档的原生清单、生产／消费者／CI 回执、三次完整比较及运行时失败记录、代表性失败对比图、生产 PNG 和截图。其余逐案例 PNG、完整日志、浏览器配置与构建树为本地临时输出；CI 下载亦会过期，保留子集不承诺原始全量运行可永久审计。归档在安装及 2026-09-17 保留时完成完整性核对；没有声称存在外部永久归档。

下一门槛：用源码匹配的节点夹具隔离失败通道，判断需要正确的运行时修复还是另行约定更窄目标。任何语义修复都要生成新候选并重跑 ALPHA-01/03，不得靠重置基准或扩大容差接受本次运行。npm 发布、Studio 完整保存文件验收、其他设备／浏览器和治理变更不在本次范围内。
