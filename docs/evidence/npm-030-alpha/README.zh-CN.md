# 浏览器 Alpha 0.3.0 发布

[English](./README.md) | 简体中文

用户完成 npm 安全密钥验证后，已于 **2026-09-21T18:01:14.383Z** 发布。[发布回执](./publication.json)、[注册表元数据](./registry-metadata.json)、[13 项注册表验收](./registry.json)及[精确注册表锁](./registry-lock.json)确认下载归档和每个安装文件均与已验收 CI 候选相同。实测标签：alpha → 0.3.0-alpha.0；latest → 0.1.0-alpha.0。

```sh
npm install --save-exact @openmixture/runtime@0.3.0-alpha.0
```

本版新增 `image-input@1`、显式资源绑定与预算、同步不可变捕获，以及通过唯一 wgpu 渲染器执行的 Native/浏览器准备资源路径。SDK 使用严格 TypeScript 编写。`.mix` 保持 v1；公开浏览器 API schema 升为 2，全部计划/哈希升为 v2。消费者须处理 `resourceRef`、新增资源报告字段，并使旧计划缓存失效。图像须为同尺寸紧密排列的线性 RGBA8，R 提供高度，不隐式解码、转换 gamma 或缩放。参见[浏览器资源合同](../../m6a-04-browser-resources.zh-CN.md)。

## 源码、归档及资格验证

PR #35–37 已审查并集成 main；[集成记录](./integration.json)区分各自合并提交及审查范围。`acabc3a911c14c3b8a2775d184f8b9cd759209c9` 的六项 [main 检查](./main-checks.json)全部通过。精确 387,049 字节归档取自 main [浏览器材质运行 35624051146](https://github.com/OpenMixture/OpenMixture/actions/runs/35624051146)，不重新构建或打包：

- SHA-256：`af2ba690f0d56c347ab8336cfd5ffffac31b5098e4f3356a94a01958c01042d4`。
- 构建 ID：`sha256:721eed5458ce82d55c73fcae004a1edb497ec1a8aab9f470c12c034c1543d602`。
- [构建回执](./build.json)、[13 项公开候选测试](./ci-sdk.json)、[八组资源比较](./ci-resources.json)、[Scalar 回归](./ci-scalar.json)、[11 个材质案例 / 44 通道](./ci-materials.json)及[外层产品验证器](./ci-product.json)。

下载后已核对归档 SHA-256、构建身份及全部七项产品证据摘要。四个 1K 权重的高度/法线资源比较最大分量差均为 **0**，既有材质及 Scalar 门槛不变。

验收限定于记录的 Linux Native SwiftShader / Chromium 软件矩阵。Windows 硬件 Native/浏览器资源法线仍**未通过验收**：原 ≤1 门槛下，记录的最大差异为 8。[M6A-05](../m6a-05/README.zh-CN.md)保留受审查联系表及失败证据。历史 warp 精度限制仍在。普通 Chrome 接口测试不能证明跨运行时硬件一致性。不包含 Rust 发布、稳定版晋升、CLI 分发、Studio 升级或部署。

## 发布及注册表消费

仅通过 `npm publish <archive> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/` 发布冻结归档。用户已明确授权审查、合并与发布；npm 账户安全验证与该授权分别处理。保持原 `latest` 标签；Alpha 建议安装精确版本。

注册表可用后，将独立消费者的精确 manifest/lock 升级到本版，在本 Windows 宿主设置 `MIXTURE_BROWSER_CHANNEL=chrome`，运行 `node scripts/browser-runtime/consumer.mjs registry - <fresh-output>`。候选与注册表验收均要求全部 13 项测试，包括四项资源用例，不允许跳过或抖动。下载归档完整性和每个安装文件须与冻结 CI 候选逐一比较。交付文档/验证器提交不是包构建源码。

不可变归档 README 含此前发布状态；本发行记录及当前源码文档取代该历史措辞，发布后的字节绝不编辑。

Git 保留选定源码/归档/资格/注册表回执和精确注册表锁文件。完整重复 PNG、归档字节及常规日志位于 [CI 产物 10651552461](./ci-artifact.json)，到期日为 2026-10-21，本地副本为忽略的临时存储。既有绑定源码的 M6A-05 图像审查不重新标为新人工决策。本记录不承诺永久完整运行产物留存。
