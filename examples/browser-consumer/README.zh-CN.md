# 独立浏览器 SDK 消费者

[English](./README.md) | 简体中文

**M6B-04（2026-09-22）：** [CLI/浏览器资产适配](../../docs/m6b-04-adapters.zh-CN.md)已实现显式文件流程和有界 `inspectPackage` / `renderPackage` 公共 API；源码版本未发布，完整验收属 M6B-05。下方较早状态保留为历史。

ENG-03 提供引擎自有的小型 Vite／TypeScript 示例，仅使用 `@openmixture/runtime` 公开入口。它加载自带 `.mix`、覆盖公开参数、选择通道、通过 WebGPU 渲染，并在 `finally` 中销毁 GPU 实例。显示像素在销毁后仍由调用方持有。不依赖 Studio，不导入引擎源码，不需要 Rust 编译步骤，不添加其他渲染器或产品编辑器。

## 运行已发布包

在本目录使用 Node 24 和 npm 11：

```sh
npm ci --ignore-scripts
npm run check
npm run build
npm run preview
```

打开输出的本地地址下 `/consumer/`。点击 **Render material**，页面加载 `public/input.mix` 并为本次调用显式初始化 WASM／GPU。提交的锁文件从注册表安装精确 `@openmixture/runtime@0.3.0-alpha.0`，不使用 `latest` 或源码检出。需要安全上下文（localhost 或 HTTPS）及支持 WebGPU 的浏览器。GPU 获取失败时显示 SDK 结构化诊断，不自动回退。

夹具暴露 `frequency` 和 `roughness`。示例有意使用这些已知公开 ID，不复制节点目录或构建通用编辑器。`src/consumer.ts` 是简短的公开 SDK 流程。报告显示包构建身份、计划哈希、公开参数有效值和所选适配器。bigint 转字符串属于宿主显示逻辑，不是新的 Mixture 报告结构。Scalar 画布只是字节预览，不声称提供颜色管理材质预览或 PNG 导出。

## 浏览器检查

```sh
npx playwright install chromium
npm run check
npm run build
npm run test:browser
```

Linux CI 使用 `npx playwright install --with-deps chromium` 安装浏览器系统依赖。自动化默认使用固定 Playwright Chromium，并显式传入 `--enable-unsafe-webgpu --ignore-gpu-blocklist`。`MIXTURE_BROWSER_ARGS` 可指定额外启动参数的 JSON 数组；CI 使用既有显式 Chromium SwiftShader 策略。明确选择本地 Chrome 时设置 `MIXTURE_BROWSER_CHANNEL=chrome`（PowerShell：`$env:MIXTURE_BROWSER_CHANNEL = 'chrome'`）。不静默重试或替换浏览器通道。这些是自动化测试配置，不是普通浏览器或所有硬件资格认证。

全部 13 项测试（包括下述资源用例）要求通过，不允许跳过／重试：惰性导入／无 GPU API 与打包身份、无效输入结构化诊断、精确 65×3 checker／Scalar 像素、多次渲染和销毁后的自有输出、busy／closing 生命周期、明确的 GPU 不可用错误、非根路径静态 UI 渲染，以及正常构建不含测试宿主。`test:browser` 单独生成包含验收宿主的 `test-dist`，普通 `dist` 只有示例。checker／常量的解析预期是测试判据，不是第二个材质执行器。测试使用真实 WASM 和 WebGPU，不使用模拟像素后端。

## 候选与注册表验收

从引擎根目录运行以下生产者命令，每次尝试使用全新输出目录：

```sh
node --test scripts/browser-runtime/consumer.test.mjs
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

候选必须来自当前引擎提交的干净源码构建。验证器将示例复制到仓库外的 OS 临时目录，只将复制后的 manifest／lock 中的运行时依赖改为指定 tarball。保持冻结的工具依赖图，验证归档 SHA-256／SHA-512 及每个已安装包文件，再比较浏览器打包代码的 `getBuildInfo()` 与安装元数据。过期、脏源码或身份不匹配的候选会失败。不执行发布。

注册表模式不改变提交的 manifest／lock，通过全新 npm 缓存下载精确版本，对照锁完整性验证归档，独立记录已发布构建身份。不要求已发布包具有当前候选的提交或 build ID。注册表字节不能继承新候选验收，反之亦然。两种模式均执行类型检查、生产构建及全部浏览器测试，消费者不调用 Rust。临时宿主位于检出目录之外，但不是 OS 文件系统沙箱。

指定输出目录记录 `qualification.json`、安装文件哈希、消费者源码哈希、复制的锁、命令／日志、Playwright 报告、渲染器／适配器证据及截图。失败尝试保持失败并保留临时路径供诊断；成功后删除临时宿主。普通日志／截图放在被忽略的本地目录或有限保留期 CI artifact；来源明确的验收摘要遵循仓库证据政策。

## 覆盖与发行边界

| 覆盖 | 本消费者 | 保留的已有验收 |
|---|---|---|
| 精确安装／构建身份及公开类型 | 候选和注册表分别验证 | 保留固定 Studio 候选／归档／锁检查 |
| 基本 SDK 生命周期、错误、通道、覆盖及所有权 | 九项直接测试，微型 checker／Scalar 负载 | 保留已有更广泛的浏览器契约和生命周期测试 |
| 像素／材质质量 | 精确 checker／常量预期 | 已有流程继续验证三种材质、11 用例、44 通道、v2 门槛、压力和原生比较 |
| 部署 | `/consumer/` 下静态 Vite 资源；正常构建不含测试宿主 | 保留固定产品的部署／导出检查 |
| 产品 UX／升级／试用 | 不覆盖 | 归 Studio，不分配给本引擎任务 |

已有 `Chromium WebGPU material matrix` job 在固定 Studio 验收**之外新增**两种独立模式。不移除任何已有检查、材质用例或必需检查名称。这是 SDK 验收入口，不替代完整支持材质／浏览器矩阵的证据。

npm Alpha 已在记录范围内发布。公开 Rust API 可通过源码／本地 Cargo 归档消费，Rust crate 仍未发布；本示例不引入 CLI 二进制分发。未来浏览器发行需要新版本和归档身份、引擎资格验证、明确发布，再进行干净环境的精确注册表消费。Studio 自行决定升级／部署节奏。版本变化时，须在经审查的变更中更新本夹具的精确包／锁及兼容预期。

ENG-04 增加第九项测试：候选及精确注册表 0.3.0-alpha.0 均在 1K 下渲染四种权重的双 Scalar 夹具。0.1.0-alpha.0 拒绝新类型的历史测试证据保留在 ENG-04 记录中。候选安装仅修改暂存 runtime 版本／归档／完整性。见 [ENG-04](../../docs/eng-04-scalar-blend.zh-CN.md)及[新发行记录](../../docs/evidence/npm-030-alpha/README.zh-CN.md)。

M6A-04 候选模式要求全部 13 项测试：原九项加四项资源测试，覆盖同步快照、偏移视图、非法缓冲区、生命周期及冻结 M6A-03 的 1K 图像／噪声组合。注册表 0.3.0-alpha.0 模式同样要求全部 13 项，不跳过用例来凑通过。通过 `check-resources.mjs` 独立对照 Native，固定最大分量差 ≤1；[范围和已知硬件失败](../../docs/m6a-04-browser-resources.zh-CN.md)不能被接口通过替代。

MAT-02b 候选模式现必需 18 项测试，包括 `morphology.spec.mjs` 的 192 组周期形态处理用例及精确重复检查。仅候选验收通过 `MIXTURE_MORPHOLOGY_TESTS=1` 显式包含该测试；精确已发布注册表消费保持 13 项。[形态处理夹具指南](../../fixtures/nodes/scalar-morphology/README.zh-CN.md)定义独立 Native 对照命令。本次增加引擎验收，不是 Studio 升级或材质接受。
