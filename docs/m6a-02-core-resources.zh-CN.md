# M6A-02 — Core 资源语义与验证

[English](./m6a-02-core-resources.md) | 简体中文

M6A-02 实现[已接受资源合同](./m6a-resource-contract.zh-CN.md)的 Core 部分，增加 `image-input@1`、`resourceRef`、`ImageBinding`、独立 `ResourceLimits`、不可变 `ResourceSnapshot` 和不透明 `PreparedRender`。这不代表图像节点的 GPU／浏览器纵向实现已完成：原生上传／执行仍属于 M6A-03，浏览器资源请求属于 M6A-04。本次不接受新图像像素或发布包。

## 公开准备入口

通过普通 `MaterialDocument` API 解码和验证 `.mix`，然后调用：

```rust,ignore
let prepared = mixture_core::prepare(
    &validated_document, &compile_request, &image_bindings, &resource_limits,
)?;
let plan = prepared.plan();
let captured = prepared.resources();
```

上方变量由调用方提供，不是完整可运行示例。[独立 Rust 消费者测试](../examples/native-consumer/tests/resources.rs)是可执行的公开 API 示例。`ImageBinding` 在同步准备期间借用紧密字节，返回后快照拥有独立字节。计划和准备输入都不暴露修改／反序列化构造入口。快照 Debug 与计划序列化均不包含原始像素。

所有源码节点和公开资源 ID 覆盖均验证，包括未使用分支。仅选中节点要求绑定完整；未知绑定按规范化完整图判定。重复条目不能互相覆盖。提供但未使用的资源仍计入输入预算并检查格式／尺寸／长度，但不捕获、哈希或上传。每个不同的选中 ID 仅产生一份快照及上传估计，不因引用次数重复；不同 ID 的相同内容不去重。

资源策略默认 8 个条目、16,777,216 像素及 64 MiB 紧密字节。有检查的运算、宿主长度转换、精确跨度／长度及全部输入预算在复制／哈希前验证。原生快照分配使用可返回失败的预留。既有 GPU 瞬态上限包含外部纹理及保守对齐上传暂存。等于上限通过，零保持为零；策略上限不进入计划哈希。

普通 `compile` 使用空资源集：选中图像节点返回 `MIX_RESOURCE_MISSING`，被裁剪图像不阻止无资源计划。CLI `validate` 保持仅源码验证。CLI `inspect --plan` 和浏览器 inspect／validate 当前没有资源交付参数，选中切片需要图像时明确失败。

## 计划与消费者兼容

全部计划采用版本 2、哈希前缀 `mixture-render-plan-v2\0`、按字典序排列的 `imageResources` 身份表及四个显式资源估计字段。内容摘要按合同绑定捕获的域／尺寸／全部 RGBA 字节。可执行 v1 哈希预期替换为独立命名的 v2 快照；原始 v1 JSON 保留为未修改的历史记录。既有程序化内核和材质金图不变。

Rust workspace 包为 0.3.0，未发布浏览器候选为 0.3.0-alpha.0、API schema 2。CLI inspect 及图 render 报告 schema 为 2，validate、doctor 和固定 checker 信封保留原版本。类型声明暴露 `resourceRef`、计划图像身份及 bigint 资源估计；尚不暴露浏览器上传参数。独立注册表消费者继续安装精确已发布 0.2.0-alpha.0。候选资格显式将固定可丢弃宿主的目录预期调整为十三项，包版本预期调整为 0.3.0-alpha.0，保留原始／适配测试摘要及其他全部检查。这不修改 Studio，也不接受候选资源执行。

穷尽内核映射包含唯一图像 WGSL 源码，使用整数 R 采样及既有半精度存储约定，并有 shader 解析／ABI 检查。M6A-03 接通准备资源执行前，执行器在 GPU 分配、管线查找及提交前拒绝图像调用，保留结构化资源及适配器证据。这不是回退；未实现上传路径和验收门槛前不得删除该保护。

## 验证与保留输入

- `cargo xtask test-format`、`test-core`、`test-plan`：源码／覆盖错误、切片、重复／未知／缺失绑定、不可变捕获、元数据／预算溢出和精确边界、确定性身份及旧文档行为。
- [Core 资源测试](../crates/mixture-core/tests/resources.rs)：不对称字节输入、全通道摘要敏感性、同名内容替换、绑定／源码重排、共享 ID 引用、65×3 上传填充、尺寸身份及策略独立性。
- `cargo xtask shader-check`：新 shader 验证／ABI 及既有 shader 检查，不代表新节点 GPU 资格。
- `cargo xtask test-consumer`：独立编写的文档／像素通过公开准备 API 消费，以及既有 Rust／CLI 合同。
- `cargo xtask check` 及六项必需 CI 继续作为集成门槛。GPU／Native／浏览器材质 CI 使用新计划与包身份执行既有程序化内容，不声称已执行上传图像。

新增 `plan-v2-*.json` 由 Node crypto 独立构造并计算 SHA-256。无资源旧计划保留原数值 token（含 `.0`），修改计划版本、在估计前增加四个零资源字段并追加空图像表，再对 v2 前缀及紧凑正文哈希。[图像计划快照](../crates/mixture-core/tests/snapshots/plan-v2-images.json)独立列举两个 2×2 图像、内容摘要、三个 pass，以及 1,712 字节峰值／累计估计。测试将生产输出与独立预期比较。未重新生成像素金图。

下一步 M6A-03 消费准备组合、每个选中 ID 上传一次、绑定图像内核、统计实际资源分配，并执行已知值／生命周期／Native 像素门槛。浏览器缓冲区捕获、GPU 资格、CLI 解码器、打包、缓存及发布仍在 M6A-02 范围之外。
