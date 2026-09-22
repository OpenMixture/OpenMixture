# M6B-04 — CLI 与浏览器资产适配

[English](./m6b-04-adapters.md) | 简体中文

2026-09-22 已在评审分支实现。[共享 CPU codec](./m6b-03-cpu-assets.zh-CN.md)现服务于显式本地文件 CLI 命令及浏览器公共字节 API。版本仍为未发布 Rust 0.5.0 / browser 0.5.0-alpha.0。综合像素验收及阶段收尾属于 M6B-05；本切片不改变 shader、源格式、包 v1 或散装输入 API。

## CLI 文件

```bash
mixture asset pack material.mix --size 65x3 --image Input input.rgba --out material.mixpack --json
mixture asset inspect material.mixpack --json
mixture asset inspect material.mixpack --plan --size 65x3 --output height --json
mixture asset render material.mixpack --size 65x3 --output height --out rendered --json
```

`pack` 通过重复的 `--image <逻辑ID> <文件>` 显式提供文档全部默认资源，包括未使用图像。文件须为同一正数 `--size` 下紧密排列的 `rgba8-linear` 原始字节；无 PNG 解码、相邻文件发现或网络解析。默认尺寸 64×64，源字节保持原样。重复 ID、缺失/额外绑定、错误长度均失败。输出独占创建，不覆盖已有资产；写入失败可能留下不完整的新文件，使用前应检查。

输入必须为显式普通文件，不能是目录或符号链接。适配器在分配前检查元数据长度，读取时检查截断/增长；从不解包归档条目。命令显式选择行为，扩展名不参与判断；移动/改名不改变资产身份。以横线开头的输入路径放在 `--` 后。

`inspect` 无 GPU 检查整个包；`--plan` 额外准备选中 Core 计划，`--size`、`--output`、`--set id=JSON` 须与它一起使用。默认输出为 `baseColor`。`render` 先准备并释放传输存储，再调用现有 wgpu 执行器与 PNG 写入器。现有 backend、software、power-preference 选项仅用于 render。坏包在 GPU 获取或创建输出目录前失败。资源引用覆盖即使值不变也返回 `MIX_PACKAGE_RESOURCE_OVERRIDE`，合法普通覆盖保持 Core 语义。

三个命令都接受只能降低的十进制无符号 `--package-bytes`、`--manifest-bytes`、`--package-buffer-bytes`；默认/上限为 67 MiB、64 KiB、202 MiB。CLI 装载计入实际归档容量、源/manifest 暂存和选中资源快照；打包先为调用方仍持有的原始图像缓冲预留预算，再调用共用写入器。这是字节缓冲预算，不是进程 RSS 上限。

JSON 资产外层 schema 为 1，含 `operation`、`input`、`ok`、`diagnostics` 和可空的 `asset`、`plan`、`render`。pack 增加 `output`、`writtenBytes`；render 嵌套现有 schema-2 渲染报告，保留适配器、PNG 与部分写入诊断。JSON u64 仍为数字，需要精确 u64 的 JavaScript 调用方应使用浏览器 API。退出码：0 成功，1 I/O/GPU 失败，2 非法调用/包/请求；调用语法错误按现有命令习惯写入 stderr。保留 Core 诊断码/阶段；包错误使用 `package` 阶段及类型化证据，包括资源覆盖 ID 数组。

## 浏览器字节

```typescript
import { loadRuntime } from '@openmixture/runtime';
const runtime = await loadRuntime();
// 字节由宿主取得，SDK 不解析资产路径或 URL。
const asset = runtime.inspectPackage(bytes, {
  packageLimits: { packageBytes: 10n * 1024n * 1024n },
});
const gpu = await runtime.createGpu();
try {
  const result = await gpu.renderPackage(bytes, {
    size: [65, 3], channels: ['height'],
    packageLimits: { packageBufferBytes: 32n * 1024n * 1024n },
  });
  // result.channels 拥有像素，destroy() 后仍可使用。
} finally { await gpu.destroy(); }
```

`RuntimeModule.inspectPackage(Uint8Array, PackageOptions?)` 在 `loadRuntime()` 后同步执行，纯 CPU。`PackageOptions` 仅包含可选 `limits`、`resourceLimits`、`packageLimits`，不接受渲染选项。`GpuRuntime.renderPackage(Uint8Array, PackageRenderRequest?)` 接受现有 size/channels/overrides/limits/resourceLimits 及 packageLimits，不接受散装 `resources`。部分包限制与 Rust 权威默认值合并。所有包字节限制及检查结果 u64 均为 `bigint`；API schema 仍为 2，包报告 schema 为 1。

检查结果包含精确包/源 SHA-256 和长度、源版本、排序的资源身份/尺寸/行长/摘要，以及 `buffers.jsPackageBytes`、`rustPackageBytes`、`chargedBytes`。验证所有资源，但不捕获选中 Core 像素；即使 `navigator.gpu` 抛错也不触碰 GPU。

选项须为普通对象，字段必须是自有可枚举数据属性且属于已知字段。字节须为固定 ArrayBuffer 上的普通 Uint8Array；拒绝共享、可调整大小、已分离、代理及子类视图。拒绝相关视图属性的 accessor/自有覆盖；不读取无关属性。偏移视图仅复制可见范围。受理时在首次让出执行前同步快照字节和选项，调用方之后的修改不影响渲染。包/散装渲染共用 busy 状态、失败恢复及幂等 destroy；关闭中/busy 调用在读取输入前失败。

JS 快照和 Rust Vec 都是真实副本。Uint8Array 绑定避免 wasm-bindgen 切片自动再复制。账本为 `JS P + Rust capacity(P) + D + M + 选中 S`；复制前检查策略及 `2P`，随后 Rust 检查实际容量、暂存和资源捕获；异步 GPU 渲染前释放传输存储。`AssetLimits::with_retained_bytes` 在相同 codec 策略中预留适配器缓冲，`AssetView::loading_buffer_bytes` 报告装载计量。`MixtureRuntimeError` 保留包错误码、阶段和证据；浏览器参数/busy/destroy 错误码不变。普通 npm 导入仍无副作用。

## 复现与范围

```bash
cargo test --locked -p mixture-asset
cargo xtask test-consumer
cargo xtask check
npm test --prefix packages/runtime
node --test scripts/browser-runtime/consumer.test.mjs
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/check-assets.mjs
# 需要干净且一致的源码/归档身份，以及全新的输出目录：
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/m6b04-browser-consumer
```

CPU WASM 验证器仅导入构建后的公共包，检查全部 22 个格式语料及精确传输预算；包目录参数后可追加大归档路径以测量实际复制。报告为 `tmp/m6b04-wasm-assets.json`。独立浏览器 fixture 为 [asset.mix](../examples/browser-consumer/public/asset.mix) 与 [asset.mixpack](../examples/browser-consumer/public/asset.mixpack)：65×3 原始像素重复 `[128,37,91,255]`，以上述 CLI 命令、ID `Input` 打包。height 输出须重复 `[128,128,128,255]`，包/散装计划哈希须一致。

候选浏览器验收运行 15 项，新增 CPU 包检查及真实 WebGPU 生命周期/所有权/预算渲染；registry 0.3.0-alpha.0 保留 13 项，因为已发布版本没有包 API。Native 源码与隔离归档 CLI 消费者打包/移动/检查资产，在获取 GPU 前拒绝坏包；现有显式 GPU 消费套件额外检查包像素。原材质门禁仍须通过。完整四权重、非对称对照、重复装载、Native/browser 包矩阵及六检查收尾仍属 M6B-05。不包含发布、Studio 修改、节点迁移或平台支持扩展。
