# 显式 GPU 上下文与 doctor

[English](./gpu-context.md) | 简体中文

`GpuContext` 显式拥有实例、适配器、设备和队列。`request(options)` 获取这些对象，不创建窗口表面、不执行计算，返回不可变的 `unverified` 获取快照。PR-004 添加[棋盘格计算／回读](./builtin-checker.zh-CN.md)：CLI doctor 默认运行此探针，仅在像素验证通过后报告 `healthy`。`--skip-probe` 保留仅获取上下文的行为。

## 命令与选择策略

```bash
cargo run --locked -p mixture-cli -- doctor
cargo run --locked -p mixture-cli -- doctor --json
cargo run --locked -p mixture-cli -- doctor --skip-probe --json
cargo run --locked -p mixture-cli -- doctor --backend metal --power-preference low-power --json
# 需要已安装的软件 Vulkan 适配器：
cargo run --locked -p mixture-cli -- doctor --backend vulkan --software --json
# 不初始化 GPU 状态，确定性失败；退出码为 1：
cargo run --locked -p mixture-cli -- doctor --backend none --json
```

| 选项 | 含义 |
| --- | --- |
| `--backend auto` | 默认：允许已编译的原生 Vulkan、Metal 和 DX12 后端，由 wgpu 选择适配器 |
| `--backend vulkan\|metal\|dx12` | 仅允许该后端；不可用时失败 |
| `--backend none` | 创建实例前禁用获取，返回 `MIX_GPU_ADAPTER_UNAVAILABLE` |
| `--power-preference high-performance\|low-power` | 偏好，不保证设备类型；默认 `high-performance` |
| `--software` | 必须选择 wgpu 软件适配器类别，不重试硬件 |
| `--skip-probe` | 仅获取上下文，保持 `unverified`，两个探针均为 `notRun` |
| `--json` | 向 stdout 写入一份完整结构化报告 |
| `--help`、`-h` | 显示帮助，不初始化 GPU 状态 |

值作为独立参数紧随选项。重复、未知或缺少值的选项会被拒绝。库不应用 `WGPU_*` 环境变量覆盖。系统驱动配置仍然有效，包括 Vulkan 加载器／ICD 选择。只发起一次适配器请求和一次设备请求，Mixture 不重试、不切换语义执行器。软件 Vulkan 是同一 wgpu 计算路径背后的驱动。

wgpu 30.0.1 启用原生 `vulkan`、`metal`、`dx12`、`std`、`parking_lot`、`serde` 和 WGSL 输入。默认目标在 macOS 支持 Metal，在 Linux 支持 Vulkan，在 Windows 支持 Vulkan／DX12。`mixture-wgpu` 和 `mixture-cli` 的可选 `software-vulkan` 特性还会在 macOS 启用原生 Vulkan，以测试 SwiftShader。GL、浏览器 WebGPU 和 noop 仍禁用。已编译后端不保证驱动／适配器存在。

## 判定与报告

| 结果 | 判定 | `ok` | 退出码 |
| --- | --- | --- | --- |
| 真实棋盘格计算／回读和像素检查通过 | `healthy` | `true` | `0` |
| 显式跳过探针，获取成功 | `unverified` | `true` | `0` |
| 适配器、设备、着色器、执行或回读失败 | `unhealthy` | `false` | `1` |
| 调用无效 | 无运行时报告；写入 stderr，包括 `--json` 模式 | — | `2` |
| 输出 I/O 失败 | 报告可能不完整；stderr 解释失败 | — | `1` |

`healthy` doctor 结果证明固定探针在当前选中上下文上的即时结果，不是设备丢失监视器，也不证明未来材质一定正确。它检查 64×64 输出长度、不透明黑白通道值、相等的黑白像素数及固定像素哨点。这是探针验证器，不是另一套棋盘格渲染器。

JSON 模式版本 `1` 保留 PR-003 字段，并新增可选 `execution`：

- `schemaVersion`、`verdict`、`computeProbe`、`readbackProbe`：探针为 `notRun`、`passed` 或 `failed`；仅获取上下文的快照不声称已计算；
- `requested`：`backend`、`powerPreference`、`softwareAdapter`、排序后的 `effectiveBackends`、排序后的 `requiredFeatures`、完整 `requiredLimits`；
- `adapter`：实际 `name`、`deviceType`、`backend`、数字型 `vendor`／`device`、`driver`／`driverInfo`、完整 `supportedLimits` 和排序后的 `supportedFeatures`；获取前为 null；
- `device`：完整的已启用 `limits` 和排序后的 `features`；获取失败后为 null；
- `execution`：已完成棋盘格的尺寸、pass 数、行布局、字节数、耗时和适配器证据；完整探针成功时存在；
- `ok` 和确定性排序的 `diagnostics`，遵循[共享诊断契约](./diagnostics.zh-CN.md)。

实际后端名称为 `Metal`、`Vulkan` 或 `Dx12`；已编译标志使用 `METAL` 等名称。设备类型使用 `IntegratedGpu`、`DiscreteGpu`、`Cpu` 等 wgpu 名称。限制字段为 camelCase。驱动值和耗时随系统变化，不是语义哈希或可移植快照。人类可读报告包含相同的请求／实际能力证据及探针结果。

设备请求不启用可选特性，使用 `wgpu::Limits::default()`。适配器限制不分桶，并在创建设备前检查，绝不隐式降低要求。不支持的限制返回 `gpuDevice` 阶段的 `MIX_GPU_DEVICE_REQUEST_FAILED`，包含首个失败限制名称及请求／支持值。GPU 能力限制与核心 `SafetyLimits` 资源上限不同。

适配器错误使用 `MIX_GPU_ADAPTER_UNAVAILABLE`／`gpuAdapter`；原生设备请求错误使用 `MIX_GPU_DEVICE_REQUEST_FAILED`／`gpuDevice`。设备失败仍保留已选适配器。实际原生来源通过 `Error::source` 和 `driverMessage` 保留；策略／限制预检查不伪造驱动来源。后续错误保留[准确的执行／回读阶段](./builtin-checker.zh-CN.md)，只有提交实际完成后，计算才会标记为通过。

## API 与验证

[context.rs](../crates/mixture-wgpu/src/context.rs)负责获取，[checker.rs](../crates/mixture-wgpu/src/checker.rs)负责内置计算与探针，[diagnostics.rs](../crates/mixture-wgpu/src/diagnostics.rs)负责报告。上下文访问方法借用其句柄和不可变获取报告。`render_checker(&mut self, request, limits)` 返回 CPU 持有的输出。`probe_checker(&mut self)` 生成新的验证报告。没有全局上下文或缓存，普通测试仍无需 GPU。

```bash
cargo test --locked -p mixture-wgpu context
cargo test --locked -p mixture-cli doctor
cargo xtask shader-check
cargo xtask check
cargo xtask gpu-smoke
```

`gpu-smoke` 显式启用真实 GPU 工作，使用 `--all-features` 使 macOS 软件 Vulkan 特性可用。它运行完整及跳过探针的 doctor，渲染／解码 PNG，将解码 RGBA 与已审查的 SwiftShader 基准比较，再执行默认忽略的库／CLI GPU 测试。报告、PNG、比较 JSON、stderr 和测试日志保存在已忽略的 `tmp/gpu-smoke/`。`check` 和普通工作区测试不初始化 GPU。冒烟失败会使任务失败，绝不更新基准。

只有冒烟工具读取 `MIXTURE_GPU_BACKEND`（`auto`、`vulkan`、`metal`、`dx12`，默认 `auto`）、`MIXTURE_GPU_SOFTWARE`（`0`／`1`，默认 `0`）及可选 `MIXTURE_GPU_EXPECT_ADAPTER`（区分大小写的名称子串）。生产 API／直接 CLI 调用使用显式选项，不使用这些测试变量。

## 固定软件适配器

[GPU CI](../.github/workflows/gpu-smoke.yml) 使用 Ubuntu 24.04／Clang 18，通过内置 LLVM 构建 [SwiftShader 提交 `694585a05946e1ed49b6bd577ca6537cbb57f025`](https://swiftshader.googlesource.com/SwiftShader/+/694585a05946e1ed49b6bd577ca6537cbb57f025)，将 Vulkan 加载器限制到其 ICD，并要求名称包含 `SwiftShader` 的 `Cpu`／`Vulkan` 适配器。[准备脚本](../.github/scripts/setup-swiftshader.sh)固定驱动源码，禁用上游测试和显示集成。运行器软件包仍由发行版管理；固定的是驱动，不是整个操作系统镜像。失败后 CI 也会上传已有证据。

在 Linux 安装工作流中的 CMake、Ninja、Clang 18 和 Vulkan 加载器软件包，然后运行：

```bash
CC=clang-18 CXX=clang++-18 bash .github/scripts/setup-swiftshader.sh
export VK_DRIVER_FILES="$PWD/tmp/swiftshader/build/Linux/vk_swiftshader_icd.json"
export VK_ICD_FILENAMES="$VK_DRIVER_FILES"
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

在 macOS 安装 CMake／Ninja 和 Xcode Command Line Tools，然后运行：

```bash
bash .github/scripts/setup-swiftshader.sh
DYLD_LIBRARY_PATH="$PWD/tmp/swiftshader/build" \
  MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
  MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
```

macOS 准备步骤将 `libvulkan.dylib` 指向 SwiftShader 的直接 Vulkan API 库，不安装系统驱动。[PR-003 仅获取上下文的报告](./evidence/pr-003-apple-m5.json)作为历史证据保留。当前 [Metal](./evidence/pr-004-apple-m5.json) 和 [SwiftShader Vulkan](./evidence/pr-004-swiftshader.json) 完整探针已在本地通过，棋盘格像素相同。远端 Linux SwiftShader 和 Linux／macOS／Windows 非 GPU 矩阵仍待运行；配置文件不代表远端 CI 已完成。
