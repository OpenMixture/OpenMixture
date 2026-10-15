# 浏览器 Alpha 0.11.0 发布

[English](./README.md) | 简体中文

**2026-10-15T17:26:31.644Z** 由用户以其本人的 npm 认证发布。[发布回执](./publication.json)、[注册表元数据](./registry-metadata.json)、[13 项注册表验收](./registry.json)及[精确注册表锁文件](./registry-lock.json)确认：注册表上的包与冻结的 CI 候选逐字节相同，安装后的每个文件都与其一致。发布后的标签：alpha → 0.11.0-alpha.0；latest → 0.1.0-alpha.0（未变）。

```sh
npm install --save-exact @openmixture/runtime@0.11.0-alpha.0
```

## 相对 0.3.0-alpha.0 的变化

API schema 2 → **3**，计划 v2 → **v3**；使用方必须使已保存的计划和哈希失效，并处理新增的报告和估算字段。`.mix` v1 文档含义不变，新版本均需显式选用。自 0.3 起新增：

- 稳定值噪声 `fractal-noise@2`（[NUM-01](../stable-noise/README.zh-CN.md)）。
- 材质节点 `brick-pattern`、`scalar-mask-blend`（[MAT-01](../mat-01/README.zh-CN.md)）、`scalar-morphology`、`scalar-subtract`（[MAT-02](../mat-02/README.zh-CN.md)）以及 `weave-pattern@1`／`@2`（[MAT-03 r3](../mat-03-r3/README.zh-CN.md)），共 18 种类型、20 个支持身份。
- 单次渲染内的纹理复用（PERF-MAT，计划 v3）。
- 通过 `inspectPackage`／`renderPackage` 使用 `.mixpack` 资产（[M6-B](../m6b-05/README.zh-CN.md)）。
- 通过 `recipes` 请求字段为 `.mix` v2 配方父图提供显式 `.mixrecipe` 闭包（[MAT-04](../mat-04/README.zh-CN.md)）。

## 源码、归档与验收

发布源码为干净的 main **`ffb10f71ab68689eb472b21fdb89cafb0b941322`**，其上全部 11 个[检查运行](./main-checks.json)均成功，包括六项必需检查和三个 Chromium 分区。准确的 510,006 字节归档取自 main [浏览器材质运行 38345148032](https://github.com/OpenMixture/OpenMixture/actions/runs/38345148032) 的 artifact `chromium-material-matrix-core`（[元数据](./ci-artifact.json)），未重新构建或重新打包：

- SHA-256：`5265cda8931bd9eee7788c8d3cbb58f1449ab053f40dee17586d449540842dfc`。
- 构建 ID：`sha256:3b0fb8f74959c4b5a4eae7900c82c3443e4aa1f0f3a0facd58b1be892d123843`（[构建回执](./build.json)）；`engineDirty` 为 false，固定 Rust 1.98.1／wasm-bindgen 0.2.128。
- 同一 artifact 中的[候选验收](./ci-candidate.json)（`ok: true`）在同一源码上验收了这一 SHA-256，并运行了 core 分区的全部 spec（含 `recipes.spec.mjs`）；woven 和 painted 分区在同一工作流中运行。

发布验收范围是各已接受记录的并集：记录的 Linux SwiftShader／Chromium 软件矩阵，以及各材质记录中注明的 GT 1030 Vulkan/DX12 和 Chrome。软件通过不验收硬件像素，也不作通用硬件保证。已记录的限制仍然有效，包括 [MAT-04](../mat-04/README.zh-CN.md) 机织物配方父图更高的 2048² 描述符峰值、warp 精度限制，以及 Windows 上 v1 资源法线的失败（显式 `fractal-noise@2` 输入已验收）。不包括 Rust crate 发布、稳定版晋升、CLI 分发、Studio 升级或部署。

## 发布与注册表消费

用户在这台 Windows 主机上运行了 `npm publish <冻结归档> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/`。发布前，代理核对了归档身份，并确认注册表上尚无 0.11.0-alpha.0。原有的 `latest` 标签保持不变；Alpha 建议精确版本安装。

发布后，从注册表下载的包与冻结归档逐字节相同（npm 的 `shasum` 与 `integrity` 一致）。随后把独立消费者固定到注册表上的精确版本和完整性值，并提交了这一改动（`ea3a4e1`）。`MIXTURE_BROWSER_CHANNEL=chrome node scripts/browser-runtime/consumer.mjs registry - tmp/registry-0.11` 在这台 Windows／GT 1030 主机上通过，消费者工作树干净（Node v24.21.0、npm 11.19.0、Playwright 1.63.0）。共运行 **13 项测试：13 项通过，0 项跳过，0 项不稳定**（`sdk.spec.mjs` 和 `resources.spec.mjs`）。注册表模式按设计排除仅候选使用的 spec。

归档内的 README 描述的就是本次发布。已发布的字节在发布后绝不修改。

## 保留

Git 保留上述构建、候选、注册表、锁文件、元数据、dist-tag、检查和 artifact 回执。归档字节、CI 日志和 PNG 保留在 [CI artifact 11788986576](./ci-artifact.json) 中，于 2026-11-12 过期；被忽略的 `tmp/registry-0.11/` 下的本地注册表运行输出是临时的。本记录不承诺永久保留完整运行 artifact。
