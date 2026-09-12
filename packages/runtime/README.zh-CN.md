# @openmixture/runtime

[English](./README.md) | 简体中文

这是由 `mixture-core` 和唯一像素执行器 `mixture-wgpu` 构建的未发布浏览器 ESM 运行时。初版包实现 M5 浏览器链路；完整 M5 材质、部署和发布验收另行进行。没有 TypeScript 渲染器、隐藏设备、worker 或降级执行器。

```ts
import { loadRuntime } from '@openmixture/runtime';

const module = await loadRuntime();
const validation = module.validate(source);
if (!validation.ok) throw validation.diagnostics;
const gpu = await module.createGpu({ powerPreference: 'high-performance' });
try {
  const result = await gpu.render(source, { size: [65, 3], channels: ['baseColor'] });
  const { pixels, encoding } = result.channels[0];
  // pixels 是独立 Uint8Array RGBA8；encoding 描述其传递函数。
  console.log(result.plan.hash, result.report.allocations.liveBytes, pixels, encoding);
} finally {
  await gpu.destroy();
}
```

`import` 仅求值无副作用的 facade 常量。`loadRuntime()` 导入包内胶水代码，获取一个相对包路径的 `wasm/mixture_wasm_bg.wasm` 资源，然后初始化独立 WASM 模块。Vite 在生产构建中重写静态资源 URL。可用 `{ wasm: new URL('/assets/runtime.wasm', location.href) }` 或 `{ wasm: bytes }` 覆盖位置；字节输入不触发 loader fetch。URL 字符串相对页面 URL 解析。加载器缓冲获取的字节后用 WebAssembly 字节实例化，因此不要求流式实例化的 `application/wasm` MIME；仍需正确 HTTP、CORS 和允许 WebAssembly 的 CSP。加载器不会重试，也不会获取材质。浏览器 WebGPU 需要受支持浏览器和安全上下文。

加载后，`validate(source, request?)`、`inspect(source, request?)`、`getNodeCatalog()` 和 `getBuildInfo()` 同步执行，不访问 GPU。core 失败时 `validate` 返回 `{ ok: false, diagnostics }`，`inspect` 抛错。JS 表示错误抛出 `MixtureRuntimeError`；异步初始化和渲染失败以该错误拒绝。错误暴露 `operation`、`code`、不变的引擎 `diagnostics`，以及适用时独立的 `browserFailure`/`evidence`。浏览器代码为 `MIX_BROWSER_WASM_LOAD_FAILED`、`MIX_BROWSER_BUILD_MISMATCH`、`MIX_BROWSER_WEBGPU_UNAVAILABLE`、`MIX_BROWSER_INVALID_ARGUMENT`、`MIX_BROWSER_RUNTIME_BUSY` 和 `MIX_BROWSER_RUNTIME_DESTROYED` 及 `MIX_BROWSER_BINDING_FAILED`（意外绑定 trap/失败）。

源为字符串或普通 `Uint8Array`；原始 UTF-8 直接交给 Rust，不经 JS JSON 解析。字符串拒绝未配对代理项，字节由 Rust UTF-8 解析器检查。异步开始前捕获输入和数组。options/overrides 必须是普通对象和自有可枚举数据属性；拒绝 getter、类实例、undefined、稀疏数组和类型强制转换。请求支持 `size: [width, height]`、唯一 `channels`、公开 ID 的 `overrides` 和部分 `limits`。尺寸默认 `[64,64]`，通道默认 `['baseColor']`。每个显式上限为非负 u64 `bigint`，省略字段使用从 Rust 绑定获得的默认值，不维护重复 JS 策略表。JS 整数覆盖值序列化为整数 token，原始 `.mix` 数值 token 保持严格检查。范围、绑定、所有分支和跨参数约束由 Rust 决定。

成功的校验/检查含计划、材质通道和公开绑定元数据：Rust 参数契约、解析后的源值及有效覆盖值。目录端口通过所在 input/output 列表区分方向；输出的 `default: null` 不表示必接输入。无效文档不返回元数据。

每个 GPU 实例同时接受一个渲染，忙碌调用被拒绝。`destroy()` 立即停止接受请求，等待已接受渲染结束并清理，然后释放设备；重复调用共享 promise。它不取消 GPU 工作，也不分离先前结果。请求通道全部返回或整个渲染失败。像素从 Rust 复制到独立 JS 数组，采用左上起点、紧密 RGBA8 和直通 alpha。颜色 RGB 为 sRGB；标量和已编码法线为线性数据。产品 PNG 导出必须保留这些编码。

Rust u64 和 usize 报告字段投影为 `bigint`；u32 字段和参数值保持 JS number。例如 `plan.estimates.peakBytes`、`report.allocations.liveBytes` 为 bigint，尺寸、计划版本、pipeline cache 命中/未命中数为 number。普通 `JSON.stringify` 无法序列化 bigint；产品可在自己的日志格式中显式编码。精确公开接口见 `src/index.d.ts`；JS、声明、WASM 和 `build-info.json` 共享并检查构建身份。

生产者从引擎仓库执行：

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
node --test packages/runtime/test/runtime.test.mjs
node scripts/browser-runtime/build.mjs
```

需要时设置 `WASM_BINDGEN` 为明确 CLI 路径。构建使用仓库固定 Rust、Cargo.lock 和 wasm-bindgen 0.2.128，产出真实 `target/browser-runtime/openmixture-runtime-0.1.0-alpha.0.tgz`、SHA-256 文件以及包含精确工具版本和源码构建身份的回执。`engineRevision` 标识 HEAD，`engineDirty` 记录是否存在源码修改；构建 ID 还覆盖源码、仓库编译设置、实际编译器/绑定版本和显式 target/profile flags。源码目录不是分发包：生成的 JS/WASM/构建元数据仅存在于暂存归档。消费者执行 `npm install ./vendor/openmixture-runtime-0.1.0-alpha.0.tgz`，不需要 Rust、引擎源码、编译安装钩子或网络 CDN 依赖。保留消费者 package lock。

Node 测试仅通过 fake 底层绑定验证公开请求和生命周期。构建成功或这些测试不证明真实 WebGPU 渲染、原生/浏览器像素等价、PNG 导出保真、设备丢失通知或全浏览器支持；这些需要独立产品的生产服务测试及保留证据。Node GPU/SSR 渲染、编辑器、资源打包、取消和零拷贝纹理仍不支持。
