# MAT-04 可复用配方 — 在记录范围内接受

[English](./README.md) | 简体中文

2026-10-13（Asia/Shanghai），维护者接受 MAT-04，并要求把机织物父图更高的描述符峰值记为限制；见[人工决定](./human-decision.json)。所有结果绑定干净的 main `d34b278b7c7c59395e9d70a8589c545095139199`（[binding.json](./binding.json)）。Rust 0.11.0／browser 0.11.0-alpha.0 仍未发布。

## 接受的内容

[MAT-04a 契约](../../mat-04a-recipe-contract.zh-CN.md)与 [ADR 0010](../../decisions/0010-reusable-recipes.zh-CN.md)：由调用方提供的 `.mixrecipe` 文档、带实例的 `.mix` v2 父文档，以及在规范化前只在 Core 中展开。RenderPlan v3、wgpu 执行器、节点目录和 `.mixpack v1` 不变。四个配方（[fixture](../../../fixtures/recipes/README.zh-CN.md)）重建两个已验收材质：

| 父图 | 替代 | 实例 | 暴露 ID |
|---|---|---|---|
| [brick-paving/material-recipes.mix](../../../fixtures/materials/brick-paving/material-recipes.mix) | MAT-01 砖块图 | brick-structure、masked-layer-surface | 24 → 10 |
| [woven-fabric/material-recipes.mix](../../../fixtures/materials/woven-fabric/material-recipes.mix) | MAT-03 r3 机织物图 | weave-structure、2 × directional-grain、masked-layer-surface | 33 → 19 |

`masked-layer-surface` 由两个材质族共用。冻结用例覆盖了参数扇出、每个实例的显式种子和实例隔离。

## 门槛

| 门槛（契约） | 证据 | 结果 |
|---|---|---|
| 1 结构等价（CPU） | `cargo test -p mixture-core --test recipe_materials`，每个冻结用例及机织物压力用例 | 展开后的父图与已验收图同构 |
| 2 像素相同，GT 1030 Vulkan | [native-vulkan.json](./machine/native-vulkan.json) | 71/71 行，各通道逐字节相同 |
| 2 像素相同，GT 1030 DX12 | [native-dx12.json](./machine/native-dx12.json) | 71/71 行，各通道逐字节相同 |
| 2 像素相同，固定 SwiftShader（CI） | [native-swiftshader.json](./machine/native-swiftshader.json) | 59 行相同；12 行机织物 2048² 为 `notRunOnSoftware`（MAT-03 第四次修订） |
| 2 Chrome 子集 | [chromium-swiftshader.json](./machine/chromium-swiftshader.json) | 8 行（两个材质族的默认与变化用例，64² 和 257×129）经公开浏览器运行时逐字节相同 |
| 3 预算 | Native 收据 | 所有描述符峰值在各材质冻结上限内；Vulkan 与 DX12 上匹配适配器的耗时预算通过 |
| 4 控制 | CPU 门槛及全部收据 | 概念控制映射到 v2 ID 并复现已验收用例 |
| 5 隔离 | `woven_grain_instances_stay_isolated` | 改变 `warpSeed` 只改变 `warp--noise` |
| 6 负面语料 | `--test recipes`（Core 与 CLI）、运行时单元测试 | 覆盖每个新代码和规则 |
| 7 兼容性 | `d34b278` 上的全部六项必需检查 | v1 fixture、计划快照、golden 和回执不变；v1 入口点和包 v1 拒绝 v2 |

逐字节相同使两个材质已有的人工视觉验收得以沿用；没有进行新的视觉评审。

父图的硬件耗时（五次 warm 的中位数，毫秒）：

| 材质 | Vulkan 1024² | Vulkan 2048² | DX12 1024² | DX12 2048² |
|---|---:|---:|---:|---:|
| 砖块 | 134 | 589 | 144 | 575 |
| 机织物 | 193 | 831 | 189 | 798 |

## 限制与观察

- **限制（维护者决定）：** 2048² 时机织物父图的描述符峰值为 **469,762,640 B**，已验收 v1 图为 **301,990,480 B**（约 +55.6%）。在冻结的 536,870,912 B 之下还剩约 12.5% 余量。如契约所预期，展开 ID 改变了字典序 pass 顺序和纹理槽复用；像素不受影响。砖块峰值不变。以后的 PERF-MAT 切片只有在有自己的实测用例时才能处理它。
- **Chrome 证据仅为软件。** 按维护者选择，Chrome 子集在 CI 中以 Chromium 153 + SwiftShader 运行；它不是 GT 1030 上的硬件 Chrome。软件通过不验收硬件像素。
- **后端彼此不同。** Vulkan 与 DX12 在 335 个通道哈希中有 332 个一致，其余 3 个在两个后端之间不同；但在每个后端上，父图都与其 v1 图逐字节相同，这才是门槛。
- 只验收记录的适配器。彩漆金属不在本切片的父图中。

## 复现

```bash
cargo test --locked -p mixture-core --test recipes --test recipe_materials
cargo test --locked -p mixture-cli --test recipes -- --include-ignored
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=0 cargo xtask test-material recipe-parents
```

新的运行会在被忽略的 `tmp/materials/recipe-parents/` 下写入新收据，不会替换这些收据。由于门槛是逐字节相同，渲染图片不予保留，已验收的砖块和机织物图片仍在各自记录中。CI artifact 在 [binding.json](./binding.json) 记录的日期过期。
