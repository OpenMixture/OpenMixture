# NUM-01：版本化稳定 value noise

[English](./README.md) | 简体中文

2026-09-22，维护者授权显式迁移噪声语义并重新验收材质。[PR #40](https://github.com/OpenMixture/OpenMixture/pull/40) 叠加于[调查 PR #39](https://github.com/OpenMixture/OpenMixture/pull/39)，实现 [Q0.24 契约](../../stable-noise.zh-CN.md)。这是尚未发布的 0.4 候选。记录的 Windows 资源/Scalar 回归已在显式迁移到 v2 value noise 的输入上修复；不代表任意硬件图、cellular noise 或另行记录的 warp 限制已获验收。

## 绑定源码的结果

- **Windows 包：** 干净源码 `2cadd6deb5a9b38263bbcbe6a1dbbc7bbeb0a26c`，archive SHA-256 `8d1801d7f4b8d6ce480f1f99ebfc1c88bea6a0c936e08548f3596964638d5caa`，build `sha256:bc7a5acda8b047f6098c94a6c2ec4e8f8a1463ad7825104a9b053d1c73bf94f8`。[SDK 记录](./windows-sdk.json)：13 项公共浏览器测试全部通过，无跳过或 flaky。Chrome 153.0.8010.48 选择 NVIDIA/Pascal，非回退适配器。Native 选择 GeForce GT 1030，NVIDIA 582.66 / DX12 驱动 32.0.15.8266。
- **公共接口 1K 像素：** 四种权重（0、0.25、0.5、1）、height/normal 两通道、Vulkan/DX12 两后端均与 Chrome 完全一致：最大分量差 **0**，历史资源法线最大差为 **8**。原有 ≤1 门槛不变。留存报告：[Vulkan 资源](./resources-vulkan.json)、[DX12 资源](./resources-dx12.json)、[Vulkan Scalar](./scalar-vulkan.json)、[DX12 Scalar](./scalar-dx12.json)、[浏览器上下文](./browser-resources.json)。
- **原始诊断探针：** [六组参数](./raw-probes.json)覆盖 scale 7/13/128、最大 seed、六 octave 及 persistence 0/0.1/0.5/1；两个 Native 后端相对 Chrome 的原始 f16 高度与法线差异均为零。Vulkan/Chrome 捕获早于干净源码提交，以精确 shader/参数/输出哈希单独绑定；DX12 在 `991ae1d` 之后重跑相同生产 shader。这些探针补充干净包记录，本身不替代包验收。
- **CI：** head `991ae1d2302322b5302f460f77c43532e26cb0e3` 的[六项必需检查](./ci-checks.json)全部通过。材质任务测试合成合并提交 `dae2cf58ccde3a1cf23063bf6084fc99e9ecaa9a`；其独立 archive SHA-256 为 `fc47727724b0e2ddd4ffff7f47b52794aa68758ddff7097a468720797dd40f8f`。[独立迁移验收](./ci-noise-v2-qualification.json)绑定不变的公共接口/部署门禁及全部 11 个迁移用例、44 通道。[对照](./ci-material-comparison.json)：陶瓷和木材最大差 0，cellular 皮革最大差 1；结构、因果、通道关系与数值门禁均通过，原始矩阵也通过。CI 是固定软件 GPU 证据，不是 Windows 全材质验收。
- **本地检查：** `cargo xtask check`、`shader-check`、`test-core`、`test-plan`、Vulkan/DX12 的 `test-node fractal-noise` 及 Vulkan `gpu-smoke` 通过；后者包含独立 Rust/CLI 与隔离包 GPU 消费。新增精确 v2 节点基线也通过固定 SwiftShader CI。[计划兼容性](./plan-compatibility.json)确认 11 个旧材质计划保持预期哈希，未替换原有 golden。

## 材质迁移视觉评审

Windows Vulkan 前后渲染使用上述干净包源码，1024×1024，四通道、原有 11 用例。[全分辨率测量](./migration-comparison.json)绑定源码/计划/PNG 身份及留存对照图。陶瓷与 cellular 皮革逐字节相同；木材仅少量分量变化 1/255。代理检查默认/粗纹木材、最大细节皮革和细网格陶瓷，未发现可见新增伪影，材质结构、方向、通道关系保留；也检查了 CI 默认对照图。这是**代理评审**，不声称人类已经批准新图像。没有重置基线或发布包。

每个 `<material>-<case>.png` 按 v1 / v2 / 绝对差异 ×16 排列，从 1024 缩为 320 像素。细小孤立差异可能在预览中消失，应以全分辨率测量判断其范围。每个 `ci-<material>-<case>.png` 是 CI 验证器生成的 Native/browser/×4 原始对照图。全部 22 张图留存 Git，[评审绑定](./visual-review.json)记录摘要及范围。代表图：[木材迁移](./wood-default.png)、[木材 CI](./ci-wood-default.png)、[皮革 CI](./ci-leather-default.png)。

## 复现与留存

复现记录应使用精确测试源码；后续仅证据提交不是原始构建。遵循[契约迁移命令](../../stable-noise.zh-CN.md)。先运行 `node scripts/browser-runtime/build.mjs`，再在 `MIXTURE_BROWSER_CHANNEL=chrome` 下执行 `scripts/browser-runtime/` 中的 `consumer.mjs candidate target/browser-runtime <fresh-sdk-directory>`。针对该记录分别设置 `MIXTURE_GPU_BACKEND=vulkan`、`dx12`，运行 `check-resources.mjs` 与 `check-scalar.mjs`，每次使用全新目录。这些检查要求记录的消费者提交与 HEAD 相同。

生成迁移对照时，以显式 Vulkan 和相同完整 runtime 提交，常规准备 `tmp/stable-noise/native-materials-v1`，另以 `--noise-v2` 准备 `tmp/stable-noise/native-materials-v2`。`python docs/evidence/stable-noise/make-review.py` 仅用 Pillow/NumPy 比较已渲染 PNG，生成留存的 11 张图，不在 CPU 上渲染材质。浏览器材质 CI 工作流复现两个矩阵、精确候选安装与独立迁移验收。

[制品元数据](./ci-artifacts.json)记录材质制品 `10675392337`，154,202,040 字节，到期 **2026-10-22T03:28:55Z**，已于 2026-09-22 下载并检查。完整日志、原始 f16、archive、manifest 和全分辨率重复材质输出留在忽略的本地目录或会过期的 CI 制品；不声称永久外部归档。Git 留存选定记录、测量及实际评审图，制品过期后不足以逐像素重审全部原始全分辨率输出。[历史 v1 失败](../windows-numerics/README.zh-CN.md)保持不变。
