# 显式 GPU 上下文与 doctor

[English](./gpu-context.md) | 简体中文

PR-003 实现无窗口适配器／设备获取。`GpuContext` 拥有实例、适配器、设备和队列，由调用方显式构造并持有。获取过程不创建窗口表面、着色器、渲染／计算 pass、纹理或回读操作。`healthy` 保留给 PR-004 的执行与回读探针。

## 运行 doctor

```bash
cargo run --locked -p mixture-cli -- doctor
cargo run --locked -p mixture-cli -- doctor --json
cargo run --locked -p mixture-cli -- doctor --backend metal --power-preference low-power --json
# 需要已安装的软件 Vulkan 适配器：
cargo run --locked -p mixture-cli -- doctor --backend vulkan --software --json
# 不初始化 GPU，确定性返回适配器不可用诊断；退出码为 1：
cargo run --locked -p mixture-cli -- doctor --backend none --json
```

| 选项 | 含义 |
| --- | --- |
| `--backend auto` | 默认值：允许已编译的原生 Vulkan、Metal 和 DX12 后端，由 wgpu 选择适配器 |
| `--backend vulkan\|metal\|dx12` | 仅允许指定后端；后端不可用时返回错误 |
| `--backend none` | 显式禁用获取；创建实例前返回 `MIX_GPU_ADAPTER_UNAVAILABLE` |
| `--power-preference high-performance\|low-power` | 适配器偏好；默认 `high-performance`，不保证实际设备类型 |
| `--software` | 必须选择 wgpu 的软件／回退适配器类别；不可用时不会重试硬件设备 |
| `--json` | 向 stdout 输出一份完整 JSON 报告 |
| `--help`、`-h` | 显示用法，不初始化 GPU 状态 |

选项值作为独立参数紧随选项。重复、未知或缺少值的选项会被拒绝。库不应用 `WGPU_*` 环境变量覆盖。系统驱动配置仍会影响获取，例如通过 `VK_DRIVER_FILES`／`VK_ICD_FILENAMES` 选择 Vulkan ICD。选择过程发起一次适配器请求和一次设备请求，Mixture 不重试，也不切换执行器。软件 Vulkan 是同一 wgpu 路径背后的驱动。

工作区启用 wgpu 30.0.1 的原生 `vulkan`、`metal`、`dx12` 特性，以及 `std`、`parking_lot` 和 `serde`。实际编译可用性取决于目标平台：macOS 使用 Metal，Linux 使用 Vulkan，Windows 使用 Vulkan／DX12。本 PR 不启用 GL、浏览器 WebGPU、noop 或 WGSL 输入特性。库报告请求策略与已编译后端的交集；已编译后端不等于可用适配器。

## 报告与退出码契约

| 结果 | 判定 | `ok` | 退出码 |
| --- | --- | --- | --- |
| 已获取适配器和设备 | `unverified` | `true` | `0` |
| 没有允许／可用的适配器 | `unhealthy` | `false` | `1` |
| 不支持设备限制或设备请求失败 | `unhealthy` | `false` | `1` |
| 调用无效 | 无运行时报告；即使指定 `--json`，用法错误也写入 stderr | — | `2` |
| 输出 I/O 失败 | 报告可能不完整；说明写入 stderr | — | `1` |

`ok` 只表示获取成功。`computeProbe` 和 `readbackProbe` 均为 `notRun`，失败时也一样。获取成功不能证明渲染可用。人类可读报告同样包含请求策略、适配器身份／能力、实际设备特性／限制，以及可操作的诊断。

JSON 模式版本 `1` 包含以下字段：

- `schemaVersion`、`verdict`、`computeProbe`、`readbackProbe`；
- `requested`：`backend`、`powerPreference`、`softwareAdapter`、排序后的 `effectiveBackends`、排序后的 `requiredFeatures`，以及完整的 `requiredLimits`；
- `adapter`：实际 `name`、`deviceType`、`backend`、数字型 `vendor`／`device`、`driver`／`driverInfo`、完整 `supportedLimits` 和排序后的 `supportedFeatures`；获取适配器前为 null；
- `device`：完整的已启用 `limits` 和排序后的 `features`；设备获取失败时为 null；
- `ok` 和确定性排序的 `diagnostics`，遵循[共享诊断契约](./diagnostics.zh-CN.md)。

后端身份为 `Metal`、`Vulkan` 或 `Dx12`；有效后端标志使用 `METAL` 等 wgpu 名称。设备类型使用 `IntegratedGpu`、`DiscreteGpu` 或 `Cpu` 等 wgpu 名称。限制使用 wgpu 的 camelCase 字段名序列化。驱动提供的值可能随系统变化；它们是证据，不是跨平台快照或语义哈希。

PR-003 不请求可选特性，使用 `wgpu::Limits::default()`。报告适配器限制时不进行限制分桶，并在创建设备前检查请求限制。不支持的限制返回 `gpuDevice` 阶段的 `MIX_GPU_DEVICE_REQUEST_FAILED`，包含首个失败限制的名称及请求／支持值；不会隐式降低要求。这些 GPU 能力限制与核心 `SafetyLimits` 资源上限不同，目前没有材质分配。

原生适配器／设备请求错误保留原始 `std::error::Error::source` 链，并在稳定的 Mixture 诊断下包含 `driverMessage`。设备失败时，报告保留已经选中的适配器。策略／限制预检查失败不会伪造驱动来源。适配器失败使用 `gpuAdapter` 阶段的 `MIX_GPU_ADAPTER_UNAVAILABLE`；两类错误码都提供操作建议。

## 公共 Rust API

直接使用 [context.rs](../crates/mixture-wgpu/src/context.rs) 和 [diagnostics.rs](../crates/mixture-wgpu/src/diagnostics.rs)；JSON 与阻塞式编排归 CLI 所有。

```rust
use mixture_wgpu::{BackendPreference, GpuContext, GpuContextOptions};

async fn acquire() -> Result<GpuContext, mixture_wgpu::GpuContextError> {
    GpuContext::request(GpuContextOptions {
        backend: BackendPreference::Auto,
        ..Default::default()
    }).await
}
```

上下文访问方法借用其实例、适配器、设备、队列和不可变获取报告。没有全局上下文或缓存。报告是获取时的快照，不是实时设备丢失监视器。普通库单元测试和 CLI 测试无需 GPU；显式禁用后端的请求会在初始化 wgpu 前失败。

## 验证与固定软件适配器 CI

```bash
cargo test --locked -p mixture-wgpu context
cargo test --locked -p mixture-cli doctor
cargo xtask check
# 显式访问 GPU，不在普通 check/test 范围内：
cargo xtask gpu-smoke
```

`gpu-smoke` 运行真实 CLI，检查成功报告结构和适配器策略，再运行默认忽略的上下文生命周期测试。该测试获取两个上下文，销毁其中一个设备并验证销毁回调，再轮询另一个设备并确认未收到设备丢失回调；不创建工作负载或回读探针。报告写入已忽略的 `tmp/gpu-smoke/doctor.json` 和 `doctor.stderr.log`；CLI 或适配器断言失败会使任务失败，不会跳过 GPU 覆盖。

只有冒烟测试工具读取 `MIXTURE_GPU_BACKEND`（`auto`、`vulkan`、`metal`、`dx12`；默认 `auto`）、`MIXTURE_GPU_SOFTWARE`（`0` 或 `1`；默认 `0`），以及可选的 `MIXTURE_GPU_EXPECT_ADAPTER`（区分大小写的适配器名称子串）。工具显式向库／CLI 传递选择选项。这些变量不配置生产 `GpuContext` 或直接调用的 `doctor`。

[GPU CI](../.github/workflows/gpu-smoke.yml) 使用 Ubuntu 24.04 和 Clang 18，通过内置 LLVM 构建 [SwiftShader 提交 `694585a05946e1ed49b6bd577ca6537cbb57f025`](https://swiftshader.googlesource.com/SwiftShader/+/694585a05946e1ed49b6bd577ca6537cbb57f025)，将 Vulkan 加载器限制为该 ICD，并要求选中名称含 `SwiftShader` 的 `Cpu`／`Vulkan` 适配器。[准备脚本](../.github/scripts/setup-swiftshader.sh)固定驱动源码，禁用显示集成和上游测试。运行器软件包由发行版管理；固定的是驱动提交，不是整个操作系统镜像。CI 上传完整 doctor 报告、stderr 和构建环境记录；失败后也会上传已有证据。源码／输出布局遵循 [SwiftShader 固定提交的 CMake 配置](https://swiftshader.googlesource.com/SwiftShader/+/694585a05946e1ed49b6bd577ca6537cbb57f025/src/Vulkan/CMakeLists.txt)。

在 Linux 上安装工作流中列出的构建工具和 Vulkan 加载器后，可这样复现：

```bash
CC=clang-18 CXX=clang++-18 bash .github/scripts/setup-swiftshader.sh
export VK_DRIVER_FILES="$PWD/tmp/swiftshader/build/Linux/vk_swiftshader_icd.json"
export VK_ICD_FILENAMES="$VK_DRIVER_FILES"
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

[Apple M5／Metal 获取报告](./evidence/pr-003-apple-m5.json)记录本地硬件证据。本地检查和获取冒烟测试已通过；新添加的远端 SwiftShader 任务和跨平台矩阵仍需真实 CI 运行。M1 保持开放，棋盘格执行、回读、PNG 输出及 `healthy` 验证属于 PR-004。
