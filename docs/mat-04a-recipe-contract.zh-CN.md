# MAT-04a — 可复用配方契约

[English](./mat-04a-recipe-contract.md) | 简体中文

**状态：** MAT-04 已在[记录范围](./evidence/mat-04/README.zh-CN.md)内接受（2026-10-13）。本契约于 2026-10-12 接受，并与 [ADR 0010](./decisions/0010-reusable-recipes.zh-CN.md) 一起经 PR #99 集成。MAT-04b 实现本契约；实现、验收和发布仍是不同状态。契约 PR 未修改代码、fixture、golden、schema 或版本。本契约在维护者 2026-10-12 的方向内回答 [MAT-04 入口](./mat-04-reusable-recipes.zh-CN.md)中的问题：由调用方单独提供的配方文档、仅松散输入、禁止嵌套。

## 概要

**配方**是独立的、带版本的 JSON 文档，包含一个带类型化接口的图片段。新的 **`.mix` v2** 父文档在普通节点旁放置配方**实例**。Core 针对调用方显式提供的配方闭包验证父文档，并在规范化之前把每个实例**展开**为普通内置节点。展开之后的一切——规范化、切片、降级、RenderPlan v3、wgpu kernel——均不变。`.mix v1`、`.mixpack v1`、现有入口点及每个已验收像素保持其确切含义。

## 配方文档

UTF-8 JSON，约定扩展名 `.mixrecipe`，按与 `.mix` 相同的严格程度解码（拒绝未知或重复字段、注释、非整数版本、NaN 及尾随数据）。

```json
{
  "version": 1,
  "id": "masked-layer-surface",
  "recipeVersion": 1,
  "nodes": [
    { "id": "underColor", "type": "constant-color", "version": 1, "parameters": { "value": [0.16, 0.15, 0.13, 1] } },
    { "id": "underRoughness", "type": "constant-scalar", "version": 1, "parameters": { "value": 0.9 } },
    { "id": "color", "type": "blend", "version": 1 },
    { "id": "roughness", "type": "scalar-mask-blend", "version": 1 }
  ],
  "edges": [
    { "from": { "nodeId": "underColor", "portId": "color" }, "to": { "nodeId": "color", "portId": "a" } },
    { "from": { "nodeId": "underRoughness", "portId": "value" }, "to": { "nodeId": "roughness", "portId": "a" } }
  ],
  "inputs": [
    { "id": "mask", "targets": [ { "nodeId": "color", "portId": "mask" }, { "nodeId": "roughness", "portId": "mask" } ] },
    { "id": "overColor", "targets": [ { "nodeId": "color", "portId": "b" } ] },
    { "id": "overRoughness", "targets": [ { "nodeId": "roughness", "portId": "b" } ] }
  ],
  "outputs": [
    { "id": "baseColor", "from": { "nodeId": "color", "portId": "color" } },
    { "id": "roughness", "from": { "nodeId": "roughness", "portId": "value" } }
  ],
  "parameters": [
    { "id": "underColor", "targets": [ { "nodeId": "underColor", "parameterId": "value" } ] },
    { "id": "underRoughness", "targets": [ { "nodeId": "underRoughness", "parameterId": "value" } ] }
  ]
}
```

| 字段 | 规则 |
|---|---|
| `version` | 配方文档格式版本；只支持 `1`，其他值以 `MIX_FORMAT_UNSUPPORTED_VERSION` 失败。 |
| `id` | 配方身份，`[a-z][a-z0-9-]{0,63}`，与内置节点类型 ID 相互独立。 |
| `recipeVersion` | 无符号 32 位整数，≥ 1。身份是 `id@recipeVersion` 这一对。 |
| `nodes`、`edges` | 与 `.mix v1` 节点和边的形状及验证相同，仅限内置目录。禁止 `material-output` 和 `image-input`（`MIX_RECIPE_FORBIDDEN_NODE`）。没有 `instances` 字段，因此配方不能嵌套另一个配方；出现 `instances` 键按未知字段失败。 |
| `inputs` | 接口输入端口。每个指向一个或多个内部**未连接**输入端口。所有目标必须具有相同端口种类，该种类即接口种类。若任一目标端口没有契约默认值，该输入为**必需**；否则为可选，父图不连接时每个目标保持其契约默认值。 |
| `outputs` | 接口输出端口。每个恰好指向一个内部输出端口；两个输出不得别名同一端口。 |
| `parameters` | 接口参数。每个指向一个或多个参数契约（类型、范围、枚举值）**完全相同**的可变内部参数。映射规则见下文。 |

接口 ID 使用节点 ID 模式；输入、输出和参数是三个独立命名空间，各自唯一。内部输入端口要么由内部边连接，要么由一个接口输入指向，不能两者兼有，也不能被指向两次。内部参数最多被一个接口参数指向一次。接口错误以 `MIX_RECIPE_INVALID_INTERFACE` 失败，并指明配方、接口 ID 和出错目标。

配方单独验证：节点、版本、参数、边、无环和端口种类遵循 v1 规则；例外是被接口指向的必需输入无需内部连接，被接口指向的必需参数无需源值。

## 参数映射与默认值

- 接口参数把其值**直接**赋给每个目标。不做算术、缩放或表达式。
- **默认值：** 若所有目标在配方源中的有效值（显式值或契约默认值）相同，该值即接口默认值。若**没有**任何目标有值（无默认值的契约参数，例如种子），该接口参数为**必需**。值混合或不相等属于接口错误。
- 未被接口指向的配方参数由配方字节固定；父图既不能读取也不能设置。
- 各节点契约的跨参数关系在展开图上、施加实例值和覆盖值之后检查。诊断携带实例 ID、接口参数和展开后的节点 ID。

## 种子

每个没有契约默认值的内置参数——目前即显式种子——**必须**被某个接口参数指向，且该接口参数为必需：每个实例在父图中显式设置它。因此同一配方的两个实例绝不会意外共享种子；共享只能通过显式设置相同值实现。不从实例 ID、位置或哈希推导种子，因此不会有现有像素因推导而改变。

## `.mix` v2 父文档

```json
{
  "version": 2,
  "nodes": [ { "id": "out", "type": "material-output", "version": 1 } ],
  "instances": [
    { "id": "surface", "recipe": "masked-layer-surface", "recipeVersion": 1,
      "parameters": { "underColor": [0.015, 0.012, 0.01, 1], "underRoughness": 0.95 } }
  ],
  "edges": [
    { "from": { "nodeId": "surface", "portId": "baseColor" }, "to": { "nodeId": "out", "portId": "baseColor" } }
  ],
  "exposedParameters": [
    { "id": "backingColor", "nodeId": "surface", "parameterId": "underColor" }
  ]
}
```

| 对象 | 规则 |
|---|---|
| 文档 | 必需 `version: 2`、`nodes`、`edges`；可选 `instances` 和 `exposedParameters`，默认 `[]`。与 v1 相同，普通节点中恰有一个 `material-output`。 |
| 实例 | 必需 `id`、`recipe`、`recipeVersion`；可选 `parameters`（接口值）。实例 ID 与节点 ID 共用一个命名空间，合起来必须唯一。 |
| 边 | 端点指向节点端口或实例接口端口。种类精确匹配；每个输入（包括接口输入）最多接受一条边。 |
| 暴露参数 | 形状同 v1。`nodeId` 指向节点或实例；`parameterId` 指向节点参数或接口参数。一个公开 ID 绑定一个目标；两个公开 ID 不得绑定同一目标。 |

v2 中节点和实例 ID 不得包含 `--`，该序列保留给展开使用。实例参数按接口验证（未知 → `MIX_PARAMETER_UNKNOWN`，无效 → `MIX_PARAMETER_INVALID_VALUE`，缺少必需值 → 带实例证据的 `MIX_PARAMETER_INVALID_VALUE`）。缺少必需接口输入以 `MIX_PORT_REQUIRED_CONNECTION` 失败，种类不匹配以 `MIX_PORT_TYPE_MISMATCH` 失败，未知接口端口以 `MIX_PORT_UNKNOWN` 失败。没有实例的 v2 文档有效，除 `documentVersion` 外行为与其 v1 等价物相同。

## 闭包与身份

- 调用方以字节形式随父文档一起提供配方文档。Core 绝不读取路径、URL 或注册表。
- 闭包必须**精确**：每个实例恰好解析到一个提供的 `id@recipeVersion`（`MIX_RECIPE_MISSING`，以该 ID 已提供的版本作为证据）；两个身份相同的文档失败（`MIX_RECIPE_DUPLICATE`）；没有实例使用的配方失败（`MIX_RECIPE_UNUSED`）。
- Core 记录每个配方精确字节的 SHA-256 及字节长度。检查结果把闭包报告为按身份排序的 `[{ id, recipeVersion, sha256, byteLength }]`。本切片中父图不锁定摘要；语义内容仍通过展开进入计划哈希。

## 展开与计划身份

1. 验证配方集合和父文档，然后在父图层面施加实例参数值和请求覆盖值。
2. 把每个实例替换为其配方节点的副本，ID 为 `<实例ID>--<配方节点ID>`。每个展开 ID 必须满足 v1 节点 ID 模式，包括 64 字符上限（带实例证据的 `MIX_NODE_INVALID_ID`）。由于 v2 ID 不能包含 `--`，第一个 `--` 能唯一还原实例。
3. 重写接口边：进入接口输入的边变为每个目标一条边；离开接口输出的边从其内部端口出发。内部边按改名后的端点复制。
4. 用 v1 图规则（循环、多重性、必需连接、跨参数关系）及现有节点、边和暴露参数上限验证展开图。
5. 按现有方式规范化、切片、降级和计算哈希。

后果：RenderPlan 保持 v3，无需新字段。`documentVersion` 记录 `2`。pass 来源携带展开后的节点 ID。由于 pass 按节点 ID 字典序排序，父图与其替代的手写 v1 图可能有不同的 pass 和纹理槽顺序，因此**计划哈希和分配估算可能不同**；但它们的 kernel、参数和依赖结构相同，像素必须逐字节相同。冻结的时间和内存预算仍适用于父图。

## 上限

新增调用方拥有的 `RecipeLimits`，与 `SafetyLimits`、`ResourceLimits` 和 `PackageLimits` 并列；调用方可调低上限。

| 上限 | 默认值 | 代码 |
|---|---|---|
| 提供的配方文档数 | 16 | `MIX_LIMIT_RECIPE_COUNT_EXCEEDED` |
| 一个父图中的实例数 | 32 | `MIX_LIMIT_INSTANCE_COUNT_EXCEEDED` |
| 配方总字节数 | 2 MiB | `MIX_LIMIT_RECIPE_BYTES_EXCEEDED` |

每个配方文档同样受 `SafetyLimits` 约束（解码字节、节点、边；接口参数受 `exposedParameters` 约束）。展开图受现有节点（128）和边（512）上限约束，并在物化展开图**之前**根据实例数和配方规模检查。计数在分配之前检查；与 v1 相同，解码在第一个超出上限的条目处停止。

## 诊断

新增代码，除上限和解码失败属于 `parse` 阶段外均属于 `validation` 阶段：`MIX_RECIPE_MISSING`、`MIX_RECIPE_DUPLICATE`、`MIX_RECIPE_UNUSED`、`MIX_RECIPE_INVALID_INTERFACE`、`MIX_RECIPE_FORBIDDEN_NODE`、`MIX_LIMIT_RECIPE_COUNT_EXCEEDED`、`MIX_LIMIT_INSTANCE_COUNT_EXCEEDED`、`MIX_LIMIT_RECIPE_BYTES_EXCEEDED`。其余情况复用现有代码，并新增证据键 `recipeId`、`recipeVersion`、`instanceId` 和 `interfaceId`。配方内部的诊断指明配方及其局部节点 ID；展开后的诊断指明实例和展开后的节点 ID。排序遵循共享诊断顺序。循环在展开图上检测，因此父图中穿过实例的路径只有在其内部也成环时才构成循环；报告的路径使用展开 ID。

## 公开接口与归属

| 接口 | 契约 |
|---|---|
| Core（Rust） | 增加配方解码、已验证配方集合、`RecipeLimits`，以及接受配方集合并返回现有 `ValidatedDocument` 的 v2 解码／验证入口；`compile`、`normalize` 和 `prepare` 不变地消费它。现有 `MaterialDocument::decode` 和 `into_validated` 保持只支持 v1，并以 `MIX_FORMAT_UNSUPPORTED_VERSION` 拒绝 v2。对绑定接口参数的公开 ID 的覆盖作用于每个目标。确切类型名在 MAT-04b 审查中确定并记录到格式指南。 |
| wgpu | 不变。 |
| `mixture-asset` | 不变。包 v1 仍要求 v1 文档，因此 v2 父图以现有的不支持版本错误失败。 |
| CLI | `validate`、`inspect` 和 `render` 接受可重复的显式 `--recipe <file>` 选项；不扫描目录。v1 输入的报告保持逐字节相同；v2 报告增加闭包列表。报告 schema 保持 3，validate 信封不变。`asset pack` 拒绝 v2。 |
| 浏览器 | validate、inspect 和 render 请求增加可选的 `recipes: Source[]` 和 `recipeLimits`，在让出控制权前与源一起同步捕获。包 API 拒绝 v2。API schema 保持 3；v1 调用不变。WASM 和 JS 仍是 Core 之上的薄层。 |

第一次实现选择未发布的 **Rust 0.11.0 / browser 0.11.0-alpha.0** 候选，因为 Core 增加了公开类型和诊断代码。节点目录不变（无新节点类型或版本）。

## 冻结验收用例

MAT-04b PR 的第一个提交把本节转换为机器可读的计划，不改变任何值。

**配方**（位于 `fixtures/recipes/`）：

| 配方 | 内部节点 | 接口 |
|---|---|---|
| `masked-layer-surface@1` | constant-color、constant-scalar、blend、scalar-mask-blend | 输入 mask（Scalar，2 个目标）、overColor、overRoughness；输出 baseColor、roughness；参数 underColor、underRoughness |
| `brick-structure@1` | 三个 `brick-pattern@1`（height、color、mask；mask 的 variation 固定为 0） | 输出 height、color、mask；参数 seed、columns、rows、rowOffset、mortarX、mortarY、bevel（各 3 个目标）、heightVariation、colorVariation（各 1 个目标） |
| `weave-structure@1` | 三个 `weave-pattern@2`（模式固定为 warp-share、coverage、height） | 输出 warpShare、coverage、height；参数 warpCount、weftCount、warpWidth、weftWidth、bevel、crown、underRatio（各 3 个目标） |
| `directional-grain@1` | fractal-noise@2、transform-2d、levels、gradient-map | 输出 grain（Scalar）、color（Color）；参数 seed（必需）、quarterTurns、darkColor、color |

**父图：**

| 父图 | 替代的已验收图 | 实例 | 暴露 ID（v1 → v2） |
|---|---|---|---|
| `fixtures/materials/brick-paving/material-recipes.mix` | `brick-paving/material.mix`（MAT-01） | brick-structure、masked-layer-surface | 24 → 10：seed、columns、rows、rowOffset、mortarX、mortarY、bevel、heightVariation、colorVariation、normalStrength |
| `fixtures/materials/woven-fabric/material-recipes.mix` | `woven-fabric/material.mix`（MAT-03 r3） | weave-structure、两个 directional-grain（经、纬）、masked-layer-surface | 33 → 19：warpSeed、weftSeed、七个织纹控制、warpDark、warpColor、weftDark、weftColor、backingColor、roughnessMin、yarnRoughness、backingRoughness、relief、normalStrength |

`masked-layer-surface` 是两个父图共用的跨族配方；结构配方演示扇出；同一父图中的两个 `directional-grain` 实例演示实例隔离。彩漆金属不在本切片的父图中：其锈层使用相反的混合极性，适配它需要第二个配方或参数化极性，而目前没有用例需要。

**门槛：**

1. **结构等价（CPU）。** 对默认值以及冻结的砖块和机织物验收计划中的每个用例，展开后的父图在节点 ID 改名下与已验收 v1 图同构：节点类型、版本、规范化参数和边相同。二者编译得到相同的 pass 多重集（kernel 调用和参数）及输出连接。
2. **像素相同。** 对两个冻结矩阵的每个用例和尺寸，父图的每个请求通道在记录的适配器上与已验收 v1 图**逐字节相同**：CI 中固定版本的 SwiftShader，以及 GT 1030 Vulkan 和 DX12；Chrome 覆盖每个材质族的默认用例和一个变化用例。逐字节相同使现有人工视觉验收得以沿用，无需新的视觉评审。任何差异都会停止本增量。
3. **预算。** 父图计划不超过各材质冻结的 `maxDescriptorPeakBytes`（砖块 469,762,048；机织物 536,870,912）及冻结时间预算；记录相对 v1 估算的变化。
4. **控制。** 概念控制请求（砖块 [controls.json](../fixtures/materials/brick-paving/controls.json)、机织物请求构建器）一一映射到 v2 公开 ID，并复现门槛 2。
5. **隔离。** 覆盖一个 `directional-grain` 实例时，另一个实例的展开节点保持不变；种子不同的两个实例产生不同的展开种子，且不共享任何节点。
6. **负面语料。** 本文每条规则一个聚焦 fixture：每个新代码、每个带实例证据的复用代码、每个上限的等于和超出情形、含 `--` 的 ID、展开 ID 溢出、嵌套尝试、禁止节点、默认值不相等、未设置必需种子、缺失／重复／未使用的配方、穿过与不穿过实例的循环。
7. **兼容性。** 所有 v1 格式 fixture、计划哈希快照、golden 和已验收材质回执不变；v1 入口点拒绝 v2；包 v1 拒绝 v2；CLI v1 报告逐字节相同；六项必需检查通过。

**预设。** `fixtures/recipes/presets/` 下的小型语料：每个父图三个命名覆盖映射，取自现有冻结用例（砖块 regular、staggered、varied；机织物 plain、varied、combined-low）。预设是按父图暴露 ID 验证的请求覆盖 fixture，**不是**运行时或文档格式。

## 范围之外

配方嵌套、内联定义、随包传递配方、父图锁定摘要、参数表达式或范围求交、彩漆金属提取、新节点类型、隐式查找、注册表、自动 v1→v2 转换、Studio 变更及发布。每项都需要自己的已证明案例。
