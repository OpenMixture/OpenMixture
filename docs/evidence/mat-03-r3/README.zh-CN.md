# MAT-03 配方修订 3（weave-pattern@2）— 在记录范围内验收通过

[English](./README.md) | 简体中文

维护者于 2026-10-10（Asia/Shanghai）以回复 "Accept revision 3" 接受机织物材质配方修订 3；见[人工决定](./human-decision.json)。修订 3 把三个 weave-pattern@1 实例换成已批准的 [weave-pattern@2](../../weave-pattern-v2-design.zh-CN.md)（候选 C：按占位确定可见性并连续叠放高度）。只改变节点版本：拓扑、参数、默认值（underRatio 0.25、crown 0）、用例、尺寸、阈值和预算均与修订 2 相同。[修订 2 的验收](../mat-03/README.zh-CN.md)及其回执作为历史保持不变；其中 combined-high 蝴蝶结限制不适用于修订 3。Rust 0.10.0／browser 0.10.0-alpha.0 仍未发布。

## 源码与冻结计划

全部结果绑定 [binding.json](./binding.json) 记录的干净源码修订（工作区干净，`engineDirty: false`）。[验收计划](../../../fixtures/materials/woven-fabric/qualification-plan.json)在任何运行之前冻结为修订 3；修订 2 按字节保留为 `qualification-plan-v2.json`、`material-v2.mix` 和 `graph-proposal-v2.json`，构建器测试固定其哈希，只允许三个 weave 版本字段不同。

两条带日期的 Stage B 修订让探针预期跟随材质声明的节点版本。两者都由修订 3 的首次运行发现，并且由已批准的 @2 公式推导而来，而不是按像素拟合：

| 修订 | @1 预期（保留为历史） | @2 预期 |
|---|---|---|
| `2026-10-10-weave-v2-crossing-oracle` | 存储高度在 1/1024 内匹配四采样上层纱线高度参照 | 存储高度在同样的 1/1024 内匹配四采样 @2 叠放 `Zw+Zf−Zw·Zf` |
| `2026-10-10-weave-v2-isolation` | crown 和 underRatio 改变 baseColor、roughness、height 和 normal | crown 和 underRatio 只改变 height 和 normal；baseColor、roughness、metallic、coverage 和 warp-share 必须逐字节不变（更严格） |

用 @1 交叉参照去检查 @2 像素，在 257×129 处失败（H 0.99707 对 0.99586），证实第一条修订是必需的。

## 机器门槛 — NVIDIA GeForce GT 1030，Vulkan（NVIDIA 582.66）与 DX12

| 门槛 | 结果 |
|---|---|
| Stage A Native 矩阵 | 每个后端 51/51 行；精确重复、松散文件／包等价、端点、分配统计（[Vulkan](./machine/native-vulkan.json)、[DX12](./machine/native-dx12.json)） |
| 计划成本 | 21 个 pass；2048² 峰值 301,990,480 描述符字节（≤536,870,912），与修订 2 相同 |
| Native 对 Chrome | 255 次通道比较，最大分量差 1/255（[对照](./machine/comparison.json)、[浏览器](./machine/woven-browser.json)） |
| 降采样质量 | plain/varied 最大平均误差 0.3043/255（上限 4/255）；压力用例 1.8189/255 仅记录、不设门槛 |
| Stage B | 六项结构探针和噪声周期性在两个后端全部通过；640 个交叉点；16 个参数隔离变体；织物平移 672 个设门槛比较（[汇总](./machine/stage-b-summary.json)） |
| 奇数尺寸 warp-share（仅观察） | 除 combined-high（7 步）外每个用例最大 1 个半精度步；可见权重最大差 0.00049。修订 2（@1）在 combined-high 曾达 27 步 |

release 计时（冷启动 1K／热中位 1K／热中位 2K，毫秒）：Vulkan plain 253 / 192 / 803，varied 251 / 198 / 795；DX12 plain 892 / 193 / 766，varied 1149 / 197 / 819 — 全部在冻结的硬件预算内（10 000 / 1 000 / 4 000）。

## 视觉评审与决定

未改动的[非金属评审工具](../../../fixtures/materials/woven-fabric/README.zh-CN.md)从干净的 Native/Chrome 对照输出渲染了六张评审图：[plain](./review/plain-pbr.png)、[varied](./review/varied-pbr.png)、[warp-seed](./review/warp-seed-pbr.png)、[weft-seed](./review/weft-seed-pbr.png)、[combined-low](./review/combined-low-pbr.png)、[combined-high](./review/combined-high-pbr.png)。[preview.json](./review/preview.json) 绑定生产构建、Chrome 版本、每张输入贴图的哈希（保留在 `review/inputs/`）以及粗糙度自检。其中 `humanAccepted: false` 是生成时的生产回执，不做改写。

决定前呈现的代理观察：每个预设的露出纱线都是直边（修订 2 的沙漏收窄消失）；combined-high 不再出现蝴蝶结，但 4× 近景里下穿凹陷的阴影偏深；4× 近景因为放大 1024² 贴图而偏软。维护者选择了 "Accept revision 3"，而没有选择把深色凹陷记录为限制。

## 限制与保留

只验收记录的适配器；软件渲染通过从不验收硬件像素。奇数尺寸平移后的 warp-share 仍然仅观察；按第四条修订，软件适配器不运行 2048²。未发布任何包。

Git 保留六张评审图、预览回执、每张绑定的输入贴图以及机器回执（81 个文件，约 17.5 MB，单个均不超过 4 MiB）。完整对照图集、原始日志和逐行 Stage B 输出留在被忽略的 `tmp/`，不做持久保存。
