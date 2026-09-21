# 浏览器运行时 0.2.0-alpha.0 发布

[English](./README.md) | 简体中文

根据用户发布请求，经 npm 安全密钥认证后，于 **2026-09-21T05:03:13.279Z** 公开发布。安装命令：

```sh
npm install --save-exact @openmixture/runtime@0.2.0-alpha.0
```

本版交付 ENG-04 scalar-blend v1 和 ENG-03 独立 SDK 入口。API schema 1 与 .mix v1 不变。[ENG-04](../../eng-04-scalar-blend.zh-CN.md)定义语义、兼容范围及实现证据。Rust 0.2.0 仍未发布；Studio 升级、部署及产品验收不属于本次发行。

## 冻结身份与发布

从 main 成功的[浏览器材质 CI 35514729976](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729976)第 1 次运行下载精确归档，未经重建或重新打包直接发布。tarball 为 345816 字节，SHA-256 为 `6d7b24f613af9444cb6d854d192749f0e46db6d846d5636e460b16c8fed72cdf`，干净源码 `9cd62fe7f16fe51a43badd1d9f70267abd87f843`，build ID 为 `sha256:2418a0770ba86d7a583cdafcafddf935e3d91ba9d9f316f7835deb3ac5137183`。[构建元数据](./build.json)和[注册表元数据](./registry-metadata.json)保留工具链、完整性及发布时间。

执行 `npm publish <frozen-tarball> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/`。npm 首先返回成功并提示仍在处理，随后验证公开可下载。实测标签为 alpha → 0.2.0-alpha.0、latest → 0.1.0-alpha.0；请使用精确版本，此版本不是稳定版，标签可变。不可变归档内 README 仍描述此前公开 Alpha；本发行说明及当前源码文档替代该历史措辞。

## 资格验证与保留

发行源码的六项受保护检查均通过：[三平台 CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729970)、[固定 SwiftShader 原生 GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729967)、[WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729968)及上面的浏览器材质运行。[CI SDK 回执](./ci-sdk.json)绑定九项通过测试、无跳过／不稳定、精确安装文件和冻结归档。[Scalar 比较](./ci-scalar.json)绑定权重 0／0.25／0.5／1 的八张 1K 高度／法线输出；原生 Vulkan 与浏览器 SwiftShader 的最大分量差为零。同次运行通过固定的一次性 Studio 契约、三材质 v2 门槛、生命周期和部署检查；下载后核验其七份绑定证据的哈希。

发布后使用已提交的独立消费者及全新 npm 缓存复现：

```sh
node scripts/browser-runtime/consumer.mjs registry - tmp/release-registry
```

Windows 验证显式设置 MIXTURE_BROWSER_CHANNEL=chrome。注册表回执及复制的 lock 记录验证器源码哈希、适用时的脏源码状态、精确安装字节、浏览器身份及九项真实浏览器测试。交付文档提交不是软件包构建源码。不推断更广泛硬件／浏览器保证，已记录的 warp 精度限制保留。

Git 保留精简源码／归档／注册表／验收回执及注册表锁文件。已有 [ENG-04 视觉证据](../eng-04/README.zh-CN.md)仍绑定其原始审查源码，不替代本归档的执行。这些是重复资格验证，没有修改金图或新增视觉验收。完整 CI 日志、重复 PNG 和产品证据位于 chromium-material-matrix 工件，ID 10605779834，[元数据](./ci-artifact.json)，2026-10-20T14:01:51Z 到期；本地重复输出位于忽略目录 tmp/release-020。已重新下载注册表 tarball 并验证字节；注册表可用不代表承诺永久保留完整 CI 工件。

[发布回执](./publication.json)、[发布后注册表验收](./registry.json)及[精确注册表锁](./registry-lock.json)记录已完成的 Windows Chrome 执行：九项测试全部通过，无跳过或不稳定；下载 SHA-256 和所有安装文件均与 CI 候选一致。
