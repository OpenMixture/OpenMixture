# M6A-04 — 浏览器资源公开接口

[English](./m6a-04-browser-resources.md) | 简体中文

未发布的 `0.3.0-alpha.0` 候选／API schema 2 现允许 `validate`、`inspect` 和 GPU `render` 接收图像资源。本次在 [M6A-03 执行](./m6a-03-native-resources.zh-CN.md)基础上实现[资源合同](./m6a-resource-contract.zh-CN.md)的浏览器边界。M6A-05 最终资格验证／发布仍独立；已发布的 `0.2.0-alpha.0` 没有资源入口。

## 公开请求

```ts
const request = {
  size: [2, 2] as [number, number],
  channels: ['height', 'normal'] as ('height' | 'normal')[],
  resources: [{
    id: 'heightSource', width: 2, height: 2,
    format: 'rgba8-linear' as const, bytesPerRow: 8,
    data: new Uint8Array([0,11,22,0, 64,33,44,1, 128,55,66,2, 255,77,88,3]),
  }],
  resourceLimits: { resourceCount: 8n, resourcePixels: 16777216n, resourceBytes: 67108864n },
};
const inspection = runtime.inspect(source, request);
const gpu = await runtime.createGpu();
try {
  const pending = gpu.render(source, request);
  request.resources[0].data.fill(0); // 已接受的渲染已经拥有快照。
  const result = await pending;
  // result.plan.hash 等于 inspection.plan.hash；销毁后像素仍由调用方拥有。
} finally { await gpu.destroy(); }
```

`runtime` 是公开 `loadRuntime` 的返回值；`source` 为[高度夹具](../fixtures/nodes/image-input/height.mix)。[安装包浏览器测试](../examples/browser-consumer/tests/resources.spec.mjs)提供可执行示例。资源默认空数组；资源限制与 Rust 默认值合并。限制使用 u64 bigint，尺寸和 `bytesPerRow` 使用精确安全 JS 整数；返回计划中的 Rust u64 步长使用 bigint。数组保留重复 ID，由 Core 拒绝。

仅接受普通 `Uint8Array`，底层必须是非共享、不可调整大小、未脱离的存储。只复制视图的偏移／长度。未知键、访问器、稀疏数组、类实例及非法数值表示通过 `MIX_BROWSER_INVALID_ARGUMENT` 拒绝。busy／closing 检查先于资源读取。Core 拥有 ID／格式／尺寸／步长／长度、所选依赖完整性、覆盖、预算、确定性身份和诊断。`validate` 返回语义失败；`inspect` 抛出、`render` 拒绝相同引擎诊断。准备失败不会执行 GPU 工作，实例仍可继续使用。

## 同步捕获与生命周期

TypeScript 捕获数据属性和字节视图，不复制像素。同步 WASM 准备通过 `AdapterImageBinding`／`ImageData` 调用 Core `prepare_from`。这个小型适配接口声明长度并复制到 Core 分配的目标；全部资源验证完成后才复制所选像素。未使用绑定仍验证但不复制。Core 从实际快照计算摘要，生成与 Native 相同的不透明 `PreparedRender`；不信任调用方摘要，也不暴露可变快照。

浏览器实现对每个所选资源只保留一份引擎拥有的紧密像素副本，低于合同的两份上限。`Uint8Array.copy_to` 直接写入 Core 目标，不构建序列化像素数组或中转 Vec。验证和捕获之间不执行 JS 回调或异步让出。随后修改、脱离像素或替换元数据不会改变已接受请求。内部 WASM 准备句柄由异步渲染消费，不从公开包暴露。成功／失败后释放每次渲染快照及 GPU 描述符；`destroy` 等待活动工作、保持幂等，返回像素继续有效。

## 验证

执行 `npm ci --prefix packages/runtime --ignore-scripts`、`npm test --prefix packages/runtime`、`cargo xtask test-core`、`cargo xtask test-plan`、`cargo xtask check`，以及 M6A-03 的 Native 图像／生命周期门槛。通过 `node scripts/browser-runtime/build.mjs` 构建干净候选后执行：

```text
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/check-resources.mjs tmp/sdk-candidate tmp/sdk-resource-comparison
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

输出目录必须全新。候选必须执行全部 13 个浏览器测试，含四个资源测试；精确注册表版本保留原九项测试。两种模式均禁止跳过／不稳定／失败。安装后的候选覆盖 Native／浏览器同源 1K 高度和法线、权重 0／0.25／0.5／1、字节／方向、1×1／1×256／256×1／65×3、alpha 无关性、偏移视图、接受后修改／脱离、拒绝后恢复、busy／销毁及描述符清理。Native 对照要求八张通道图的最大分量差均 ≤1；现有 Scalar／材质门槛不变。Node 测试独立验证 SharedArrayBuffer 拒绝，不受浏览器跨源隔离条件影响。

测试 canvas 输出的高度／法线 PNG 是比较产物，并非新增公开编码 API。完整例行报告／图像存于忽略的本地目录或 CI 产物，遵循[证据政策](./evidence-policy.zh-CN.md)。M6A-05 负责最终持久发布验收及更广平台声明。本次不包含 CLI 图像解码、Studio 修改、资源缓存、新着色器、包发布或第二执行器。

## 资格边界

集成门槛为 Linux 固定软件 CI 矩阵。本地 Windows Chrome 公开接口测试通过，但这**不代表**其硬件路径达到固定 Native／浏览器分量差上限：在测得的 Native NVIDIA GT 1030／Vulkan 对 Chrome WebGPU 路线上，M6A-03 冻结输入的法线最大差在权重 0.5 为 4、权重 1 为 8。高度最大差均 ≤1，纯导入的权重 0 完全一致。权重 1 在各自运行时也等于直接程序噪声端点，因此该失败在导入像素不贡献输出时仍存在。它必须保持为失败对照，不能被材质容差或新基线吸收。M6A-05 在声明该路线前必须解决或明确限定此硬件精度问题；本次不声明任意适配器跨端资格。

[有界软件矩阵成功与硬件失败回执](./evidence/m6a-04/README.zh-CN.md)分别保留，不能相互替代。
