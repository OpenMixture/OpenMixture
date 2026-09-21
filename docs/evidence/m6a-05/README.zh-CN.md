# M6A-05 综合验收

[English](./README.md) | 简体中文

M6A-05 在**有记录的 Linux 软件适配器矩阵**内验收 M6-A 资源增量。[验收回执](./acceptance.json) 绑定 PR #36 的 head `6e6af1872a7111263815a70ab54ad35009c9326a`、实际受测合并提交 `9dae2dff21958ac117504ab968a264cd900ae4bb`、六项通过的必需检查以及未发布的 0.3.0-alpha.0 精确归档。本次证据变更不把之后的提交冒充为这些像素的来源；堆叠实现 PR 的集成和发布仍是独立操作。

## 决定性结果

| 门禁 | 结果与留存证据 |
|---|---|
| 公开候选包 / 已发布基线 | [候选包](./candidate.json) 13 项测试，无跳过、抖动或失败；[registry](./registry.json) 对独立的已发布 0.2.0-alpha.0 身份执行九项测试 |
| 资源 Native/browser 一致性 | [资源对照](./resources.json)：四个冻结的 1K 权重 × 高度/法线，八组比较**最大差异均为 0**，计划哈希一致，销毁后输出仍归调用方所有；另见[浏览器测量](./browser-resources.json) |
| 上传与生命周期 | 每次 1K 渲染选择一个资源、上传 4,194,304 字节，结束后存活描述符字节为零；必需 GPU 检查还执行 Native 源码/归档资源探针和失败清理 |
| Scalar 回归 | [Scalar](./scalar.json)：四个权重 × 两个通道通过原有 ≤1 门槛 |
| 既有材质 | [材质比较](./materials.json)：釉面陶瓷、皮革、木材共 11 个案例 / 44 个通道通过原有语义、结构、数值和回归门禁；[外层产品验证器](./product.json) 绑定候选包及正常部署 |
| 平台与包检查 | 回执保留六项成功检查及作业 URL：Linux/macOS/Windows CPU、固定 Native SwiftShader GPU/包/材质/2K trace、WASM/npm 和 Chromium 矩阵 |

候选归档 SHA-256 为 `464540b5af7913629213f6d20df7f0b7f41acee2287334819bbf2b325c5e65e5`；下载后重新计算，与公开消费及[构建](./build.json)回执一致。Native 使用固定 SwiftShader 提交 `694585a05946e1ed49b6bd577ca6537cbb57f025`；Chromium 使用显式请求的软件适配器。macOS/Windows CPU 检查通过不代表这些平台的 GPU 验收。

## 图像审查与保留的失败

![1K 资源联系表](./contact.png)

列依次为权重 0、0.25、0.5、1，上排高度、下排法线。各 1K 输出按 `(4*x,4*y)` 取样为 256×256 面板，无颜色调整。Codex 已查看：中间权重仍能辨认导入高度的周期脊线，程序细节逐渐增多；高度与法线同步变化，两端均非空白。这是**代理图像审查**，不宣称维护者已批准。精确方向、字节坡度、端点、接缝和一致性由机器测试确立，不能只靠缩略图判断。回执绑定受审查图像和八张原始 PNG 的身份。

Windows NVIDIA GeForce GT 1030 Vulkan 对普通 Chrome 仍然**未通过验收**。[原始源码/归档来源](../m6a-04/windows-hardware-failure.json)和本次晋升留存的[完整 Native 失败报告](./windows-failure.json)保留八组比较及最差像素值。权重 0 / 0.25 / 0.5 / 1 的法线最大差异分别为 0 / 1 / 4 / 8。各运行时的权重 1 均与其直接程序噪声端点相同，因此即使导入高度不影响输出，该差异仍存在。这把验收阻碍定位到程序路径或后端运算，但尚未证明具体编译器/驱动原因。本次未修改着色器、黄金图、夹具或阈值来关闭 M6A-05。

[资源合同](../../m6a-resource-contract.zh-CN.md)现在明确把里程碑关闭限定于有记录的软件矩阵。每个宣称通过验收的环境仍必须满足 ≤1 门槛。扩展到该 Windows 硬件组合，必须独立审查数值修复/版本决策并重跑冻结资源、Scalar 和材质门禁；不得把现有失败重新标为通过。

## 复现与留存

检出受测合并提交，或构建新提交并保留其独立身份。按[浏览器材质工作流](../../../.github/workflows/browser-materials.yml)安装固定 Native 软件适配器、精确工具链并配置 Chromium 软件参数。运行 `cargo xtask check`、`cargo xtask test-node image-input`、`cargo xtask gpu-smoke`；最后一项包含源码/归档公开消费者和材质门禁。随后执行：

```sh
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/check-resources.mjs tmp/sdk-candidate tmp/sdk-resource-comparison
node scripts/browser-runtime/check-scalar.mjs tmp/sdk-candidate tmp/sdk-scalar-comparison
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

使用全新输出目录。链接工作流的后续步骤执行固定、临时产品消费者的材质及正常部署门禁；不修改 Studio。必需 CI 保留精确命令和完整运行结果。复现联系表时，解码 `tmp/sdk-resource-comparison` 中八张 `resource-<weight>-<channel>.png`，从左上角每四像素取样一次，按上述顺序排列。

浏览器产物为 `chromium-material-matrix`，ID `10637705492`，运行 [35596753957](https://github.com/OpenMixture/OpenMixture/actions/runs/35596753957)，于 **2026-10-21** 到期。解包后的候选归档已独立校验哈希；回执中 ZIP 摘要来自 GitHub。Git 保留选定原始报告、受审查联系表、源码/包身份以及用于限定验收范围的失败。完整 1K 原图、归档字节、常规日志只有有限期 CI 留存和临时本地副本，不构成永久完整运行归档。到期后仍可查看已保留测量与联系表，但哈希无法恢复原始归档或全分辨率像素。

范围外：硬件一致性修复、着色器语义、新节点、CLI 解码、容器打包、Studio 工作、Rust/npm 发布及合并依赖 PR。
