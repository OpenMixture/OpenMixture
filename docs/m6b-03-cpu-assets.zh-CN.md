# M6B-03 — 共享 CPU 资产 codec

[English](./m6b-03-cpu-assets.md) | 简体中文

可选 [mixture-asset crate](../crates/mixture-asset/README.zh-CN.md)实现 [M6B-02 字节契约](./m6b-package-format.zh-CN.md)。源码和独立 Cargo 消费者无需 GPU 即可写入、验证、检查和准备 `.mixpack v1`。Rust 源码为 0.5.0；浏览器构建身份推进至未发布的 0.5.0-alpha.0，注册表消费者仍固定于已发布 0.3.0-alpha.0。CLI/浏览器包 API 和包像素验收仍属 M6B-04/05。不声称发布或新增硬件数值保证。

## 公共操作与所有权

| API | 所有权与验证 |
|---|---|
| `write(source, bindings, &limits)` | 借用精确源码/像素；验证完整闭包和元数据、重算身份、一次预留最终归档并返回确定字节 |
| `AssetView::load(bytes, &limits)` | 借用归档，验证严格头/manifest、全部摘要与源码/闭包，保留指向原分配的范围 |
| `OwnedAsset::from_vec(bytes, &limits)` | 移动归档分配，检查并计费实际容量（含空余容量） |
| `OwnedAsset::copy_from(bytes, &limits)` | 先验证和检查预算，再进行一次可失败的归档预留/复制 |
| `view.source/resources/image/bindings` | 只读源码、元数据和借用载荷访问 |
| `view.inspect()` | schema 1 传输/源码身份及有序资源元数据，与计划哈希分离 |
| `view.preparation_buffer_bytes(&request)` | 验证请求，报告留存归档容量 + D + M + 所选快照字节 |
| `view.prepare(&request)` / `owned.prepare(&request)` | 取策略交集，拒绝 resourceRef 覆盖，将切片/捕获/计划委托给 Core；快照可独立于资产存活 |

Core 导出 `resources::{valid_image_id, image_identity, document_image_ids, image_references}`。资源清单与编译通过共用遍历选择相同节点。codec 没有第二套图遍历或资源摘要算法。Core 元数据验证与捕获共用实现，不将行为放进生成绑定。新 crate 只依赖 Core 和已有 serde/serde_json/sha2。Core/wgpu 均不依赖它；Native 消费者和归档验证包含它，CLI/WASM 尚未依赖它。

## 错误与内存契约

`AssetError::Package` 提供可序列化诊断，含稳定 `MIX_PACKAGE_*` 码、阶段、消息、证据与建议。`Document`、`Compile` 变体保留原始 Core 错误/报告及来源链。在 Core 验证覆盖 ID/类型后，即便 resourceRef 覆盖等于默认值或处于输出切片外，也会拒绝。其他覆盖沿用 Core 行为；没有改变语义的通用回退。

检查所有源码/资源条目，包括断开的图像。索引前资源表限定为八项，仅接受精确序号名称。严格类型化 JSON 拒绝重复/未知字段、浮点/指数整数、负零、溢出和 BOM。规范头逐字节比较同时拒绝扩展字段、链接、路径穿越、非零填充和其他八进制写法。必须恰有两个终止块并抵达 EOF。全部摘要重算，写入不规范化源码字节。

即使借用切片避免复制，缓冲计费也保守保留 D+M 临时预算。借用准备计费 D+M+S；拥有准备另加实际留存 Vec 容量。借用写入计费 P+D+M。大缓冲使用 `try_reserve_exact` 一次预留检查后的尺寸，预留失败结构化返回。Core 拥有所选像素捕获；类型化 manifest/图分配另受边界限制。不引入全局分配器探针或 unsafe。[内存测试](../crates/mixture-asset/tests/memory.rs)通过指针身份、实际归档容量、快照尺寸和精确预算拒绝测量真实 Rust 路径。宿主须累计并发资产、调用方输入和输出；浏览器双副本计费仍属 M6B-04。

## 复现

```sh
cargo test --locked -p mixture-asset -- --nocapture
cargo xtask test-core
cargo xtask test-consumer
cargo xtask package-check
cargo check --locked -p mixture-asset --target wasm32-unknown-unknown
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/consumer.test.mjs
tar -tf fixtures/packages/mixpack-v1/valid.mixpack
cargo xtask check
```

[Codec 测试](../crates/mixture-asset/tests/codec.rs)消费所有已提交语料、精确复现独立生成的归档、拒绝每个截断前缀及全部头位置的单字节变异、对比四权重的散装/包计划哈希，并覆盖移动/复制/借用生命周期。包内单测覆盖严格 JSON、零/等值/超限及原始 Core 错误。[外部 Native 测试](../examples/native-consumer/tests/assets.rs)只使用公共 API 及自有源码/像素，也针对隔离解包归档执行。内存测试使用 4 MiB、64 MiB 资源载荷，并拒绝减少一字节的准备预算。这些是 CPU/所有权结论，不是包 GPU/浏览器验收。
