# mixture-asset

[English](./README.md) | 简体中文

可选、纯 CPU 的 `.mixpack v1` 装载器与确定性写入器。规范未压缩 USTAR 保存精确 `.mix` 源码和紧密排列的线性 RGBA8 图像。图验证、资源身份、依赖切片和不可变准备均由 Core 完成；此 crate 不依赖文件系统、浏览器或 GPU。

```rust
use mixture_asset::{AssetLimits, AssetView, OwnedAsset, write};
use mixture_core::CompileRequest;

let source = br#"{"version":1,"nodes":[
 {"id":"base","type":"constant-color","version":1},
 {"id":"out","type":"material-output","version":1}],"edges":[
 {"from":{"nodeId":"base","portId":"color"},"to":{"nodeId":"out","portId":"baseColor"}}]}"#;
let limits = AssetLimits::default();
let bytes = write(source, &[], &limits)?;
let view = AssetView::load(&bytes, &limits)?;
assert_eq!(view.source(), source);
let prepared = view.prepare(&CompileRequest::default())?;
drop(view);
let owned = OwnedAsset::from_vec(bytes, &limits)?;
assert_eq!(owned.prepare(&CompileRequest::default())?.plan().hash(), prepared.plan().hash());
# Ok::<(), mixture_asset::AssetError>(())
```

`AssetView` 借用像素；`OwnedAsset::from_vec` 移动一个分配，`copy_from` 先验证和检查预算再复制。准备返回所选资源的独立 Core 快照。所有资源（含未使用分支）都必须通过完整性和闭包检查。包 v1 拒绝资源引用覆盖；普通 Core 覆盖及散装 API 保留。

`AssetLimits` 组合现有 Core 策略及只能降低的包上限：67 MiB 归档、64 KiB manifest、2 MiB 源码、最多八张各轴不超过 2048 的同尺寸图像、64 MiB 资源和 202 MiB 计费字节缓冲。`preparation_buffer_bytes` 报告留存归档容量 + 保守源码/manifest 临时预算 + 所选快照。这不是 RSS；调用方输入、类型化对象、分配器开销和输出另计。调用方须自行累计同时存在的多个资产。

`AssetError` 保留原始类型化 Core 错误，或提供结构化 `PackageDiagnostic`：类型化 `PackageCode`（稳定的 `MIX_PACKAGE_*` 拼写）、`package` 阶段、`error` 严重级别、消息、证据与建议，序列化字段名与 Core 诊断一致。没有解包、压缩、外部资源查找、回退或隐式缓存。本 crate 尚未发布；CLI `asset` 命令与浏览器 `inspectPackage`/`renderPackage` 是其适配层。版本见[发布状态](../../docs/release.zh-CN.md)。
