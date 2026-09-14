# 浏览器运行时与首个独立消费者

[English](./browser-runtime.md) | 简体中文

**本地验收更新，2026-09-14：** M5-02／M5-03 初始浏览器执行与隔离软件包消费现已通过[记录的门槛](./evidence/m5-02-03/README.zh-CN.md)。未变更归档通过 13 项 Chromium 检查，包括可控真实设备丢失、映射清理及重复独立模块／设备。M5-04／M5-05 材质回归、完整 Player 流程与正式浏览器 CI 矩阵仍开放；软件包未发布。

M5-02／M5-03 现已提供轻量 WASM 绑定、完整本地 `@openmixture/runtime@0.1.0-alpha.0` tarball，以及带最小 Player 的独立 [Studio 产品仓库](https://github.com/OpenMixture/Studio)。软件包未发布到 npm。这是棋盘格执行／消费切片，不代表完整 M5 验收或节点编辑器。[M5 计划](../M5_PRS.zh-CN.md)和[交付契约](./browser-sdk.zh-CN.md)保留剩余范围。

## 引擎构建

使用仓库 Rust 工具链和精确匹配的 wasm-bindgen CLI。本检查点使用 Rust 1.98.1、wasm-bindgen 0.2.128、浏览器目标 `wasm32-unknown-unknown`、Node 24.20.0 和 npm 11.19.0。产品另行锁定 Vite／TypeScript／Playwright 版本。

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
cargo check --locked -p mixture-wasm --target wasm32-unknown-unknown
node --test packages/runtime/test/runtime.test.mjs
node scripts/browser-runtime/build.mjs
```

`WASM_BINDGEN` 可指定 CLI 可执行文件。构建拒绝版本不匹配。它编译锁定的 Rust 源，生成绑定，组装公开 JS／类型／WASM 与许可证，并在 `target/browser-runtime/` 下运行 `npm pack`。该目录包含归档、SHA-256 文件及标识源内容、构建 ID、工具的 `receipt.json`。消费者安装不编译 Rust。

生成的 no-modules wasm-bindgen 胶水被封装进 ESM 工厂。每次显式 `loadRuntime` 获得独立绑定／模块状态；导入公开入口不加载 WASM 或获取 GPU 状态。此打包细节不增加图或 shader 语义。手写公开 TypeScript 声明随同一次构建交付，并由独立消费者检查；不将其描述为自动生成的 Rust API 声明。

## 公开用法

```typescript
import { loadRuntime } from '@openmixture/runtime';

const runtime = await loadRuntime();
const inspection = runtime.inspect(source, {
  size: [65, 3], channels: ['baseColor'],
});
const gpu = await runtime.createGpu();
try {
  const result = await gpu.render(source, {
    size: [65, 3], channels: ['baseColor'],
  });
  const pixels = result.channels[0].pixels;
  // The consumer owns these RGBA8 bytes, including after gpu.destroy().
} finally {
  await gpu.destroy();
}
```

产品提供原始字符串／UTF-8 源；不得在 Rust 验证前解析并重新序列化用户文件。`getNodeCatalog`、`getBuildInfo`、`validate` 和 `inspect` 在显式 WASM 加载后运行，不请求 GPU。`validate` 对预期文档／请求失败返回诊断；`inspect` 通过结构化 SDK 错误拒绝无效输入。首版验证结果包含已编译请求及绑定元数据。

`loadRuntime({ wasm })` 接受 URL／字符串或自有字节输入；省略 `wasm` 使用包内资源 URL。每个 GPU 实例接受一个渲染；另一并发请求以 `MIX_BROWSER_RUNTIME_BUSY` 拒绝。`destroy` 停止接收新工作，等待已接受工作并显式释放设备，不使返回的 JS 像素失效。浏览器回调等待让出事件循环。原生阻塞等待保留原生行为；不承诺浏览器硬性期限、自动恢复或其他渲染器。

Core／GPU 诊断保留其代码和上下文。`MIX_BROWSER_BINDING_FAILED` 独立于无效参数报告意外绑定／trap 失败；浏览器加载、构建不匹配、不支持 WebGPU、busy 和 destroyed 错误继续区分。确切 JS 投影见[公开声明](../packages/runtime/src/index.d.ts)：u64／usize 数据使用 bigint，而流水线 hits／misses 等 u32 字段使用 number。空的可选诊断 evidence 可以缺失。

## 独立产品与验证

产品通过相对文件依赖和已提交锁文件安装 `vendor/` 下的真实归档。其来源回执标识已消费归档和引擎构建。首个 vendor 产物是为独立复现保留的未发布构建，不是 npm 发布。软件包升级替换真实归档并更新完整性／回执；生产方检出、软链接或开发者绝对路径不是运行时依赖。

产品 README 定义 `npm ci`、`npm run check`、`npm run dev` 和 `npm run test:browser`。浏览器命令构建测试消费者，以非根 `/player/` base 提供生产输出，使用锁定的完整 Chromium 浏览器及显式 WebGPU flags。GPU 不可用时测试失败，不予跳过。浏览器测试不会悄然并入原生 `cargo xtask gpu-smoke`。

首轮浏览器检查覆盖无需 GPU 的模块操作、无效原始源、加载失败、URL／字节资源加载、精确奇数尺寸棋盘格像素、请求输入捕获、busy 拒绝、重复渲染、销毁后结果保留，以及可见 Player 文件／渲染／销毁流程。这些补充原生棋盘格／回读／设备丢失回归，不认证全部 M5 材质案例或所有浏览器。

合并引擎变更前，依照现有必需 CI 政策运行 `cargo xtask check`、受影响的 shader 检查及显式原生 GPU 回归。[浏览器构建工作流](../.github/workflows/browser-runtime.yml)独立验证 WASM 编译、JS 包装测试和归档生成，不声称真实浏览器 GPU 执行。独立真实浏览器测试由产品负责。

## 剩余验收

- 完整三种材质的 1K 浏览器／原生比较，以及已评审的容差、平铺和参数因果性。
- 自发设备／驱动丢失与意外平台事件覆盖、更广生命周期压力验证，以及文档化的浏览器 CI 验收环境。可控真实设备销毁与映射清理已由 M5-02／M5-03 记录覆盖。
- 完整 Player 参数／通道／导出流程，包括独立解码的 PNG 元数据。
- 完整 M5 验收、稳定支持声明、Registry 发布、Studio 创作及 M6 资源打包。

按现有[证据政策](./evidence-policy.zh-CN.md)记录确切已测源、归档身份、浏览器／OS／适配器、命令和限制。原生运行、WASM 编译、归档构建或开发服务器成功均不能替代浏览器执行。
