# 浏览器材质比较——M5-05

[English](./browser-materials.md) | 简体中文

**2026-09-20 门槛重设计：** 新运行时比较采用 [v2 规则](./browser-quality.zh-CN.md)：有界幅度、局部偏移和逐通道响应。原稀疏像素判定保留为诊断；历史接受、原生金图及 Studio 保存文件门槛不变。新浏览器支持仍需绑定源码的资格证据。

原生参考生产者通过公开 CLI 执行全部 11 个既有验收用例，尺寸为 1024 × 1024，请求 baseColor／normal／roughness／height，保留原始源码字节和公开覆盖。准备步骤要求显式后端及新目录；相对归档生产者修订存在运行时实现漂移时拒绝继续。源码、夹具和构建身份与计划哈希分开记录。

```bash
MIXTURE_GPU_BACKEND=metal node scripts/browser-runtime/prepare-materials.mjs tmp/browser-native 4b914feb9f3365d292b27ea60c5e0b6004f745e8
# In the independent product checkout:
npm run test:materials -- /absolute/native-reference /absolute/new-browser-output
# Back in the engine checkout:
cargo xtask browser-material-measure tmp/browser-native /absolute/new-browser-output
cargo xtask browser-material-check tmp/browser-native /absolute/new-browser-output
```

Linux 使用已有锁定 SwiftShader 设置及显式 Vulkan／软件策略。产品只消费独立参考清单和已安装 tarball。生产资源在 `/player/` 下静态提供；缺失 WASM 或不可用 WebGPU 均失败。引擎比较解码浏览器 PNG，复用既有材质结构、接缝、非退化、因果性和高度／法线关系检查，不改变原生 golden。

`browser-material-measure` 记录差异并检查结构／语义门槛，但明确不接受像素容差。`browser-material-check` 执行 [v2 规则](./browser-quality.zh-CN.md)，原逐通道容差判定保留为诊断。两者在浏览器输出目录写入 `comparison.json` 及逐用例的原生／浏览器／差异接触表。计划比较保持整数精确，规范化 f32 JSON 投影，并要求语义哈希一致。比较前检查原始清单及 PNG 摘要。

## 当前候选验收——ALPHA-01

上方命令示例复现历史归档。[材质工作流](../.github/workflows/browser-materials.yml)现已在同一 job 中从检出的引擎版本（包括 PR 合并版本）构建并消费新归档，不下载未绑定版本的最近 artifact。这是已实现的验证工具，只有该版本运行通过后才构成新的验收矩阵。

消费者仍固定为 Studio `56c510ab57daa1b68ef660525a648a582730a37e`。[候选验证器](../scripts/browser-runtime/candidate.mjs)要求干净生产者元数据、预期引擎与消费者版本及归档 SHA-256。只替换临时消费者的 vendor 归档、构建回执和运行时 lock 条目；新条目使用归档 SHA-512，其他依赖全部保持锁定。随后 `npm ci` 安装，并将每个已安装包文件与候选归档比较。产品源码与已提交 vendor 历史不变。未来包版本变化需要显式更新消费者契约。

安装后工作流执行 `npm run check`、固定消费者的全部 28 项浏览器契约、实际浏览器构建身份探针、11 用例／44 通道材质矩阵及 12 次生命周期渲染、冻结比较和正常生产部署。探针还把历史 WASM 字节交给当前 JS，要求返回 `MIX_BROWSER_BUILD_MISMATCH`。独立消费者覆盖公开类型和真实 bigint／自有输出行为。两个浏览器 job 都运行定向拒绝测试：

```bash
node --test packages/runtime/test/runtime.test.mjs scripts/browser-runtime/candidate.test.mjs
```

验证新候选时，先提交引擎改动并用 `node scripts/browser-runtime/build.mjs` 构建。用该确切完整引擎 SHA 准备原生参考，保留既有源码漂移守卫。准备固定消费者的新检出后执行：

```bash
# From the engine root; use fresh output directories for every run.
node scripts/browser-runtime/candidate.mjs stage target/browser-runtime /absolute/product /absolute/candidate-evidence <full-engine-sha> 56c510ab57daa1b68ef660525a648a582730a37e
# In /absolute/product: npm ci; npx playwright install chromium; npm run check; npm run test:browser
node /absolute/engine/scripts/browser-runtime/candidate.mjs installed /absolute/product /absolute/candidate-evidence
# Keep the product as the working directory for probe (it serves the browser-test build).
node /absolute/engine/scripts/browser-runtime/candidate.mjs probe /absolute/product /absolute/candidate-evidence
# Run test:materials, then the engine's browser-material-check as above.
# In the product, run npm run test:deployment after material comparison.
node /absolute/engine/scripts/browser-runtime/candidate.mjs verify /absolute/product /absolute/candidate-evidence /absolute/native-reference /absolute/browser-output
```

Linux 浏览器命令须取消 `VK_ICD_FILENAMES` 和 `VK_DRIVER_FILES`，并使用工作流中的显式 Chromium SwiftShader 参数。`probe` 必须先于部署步骤执行，后者会重建不带测试入口的资源。`verify` 在检查前撤销旧验收成功状态，绑定实测构建信息、归档／lock／原生身份、浏览器／部署结果及比较摘要，拒绝跳过或缺失的门槛。`qualification.json` 为最终回执；`candidate.json` 保留原始及替换后的 lock／归档身份、产品版本和 CI run／attempt。工作流上传这些记录、候选归档、原生／浏览器像素、测试报告和部署证据，保留 30 天。已接受结果仍须遵守[证据保留政策](./evidence-policy.zh-CN.md)。

此 job 使用独立软件包消费者，但不宣称通过 OS 沙箱禁止读取引擎检出，也不代表默认浏览器支持。Chromium 仍使用受控测试参数；普通用户配置、npm 发布与 Studio 完整保存文件升级验收仍为独立 Alpha 门槛。原生 golden 和浏览器容差不变。

## 冻结准则

[校准评审](./evidence/m5-05/calibration.zh-CN.md)记录本地与 Linux 测量、未通过的候选比较及冻结后的逐通道门槛。正式验收必须在此次冻结后运行。

[正式验收与完整像素](./evidence/m5-05/README.zh-CN.md)保留冻结后的两端结果和复核命令。
