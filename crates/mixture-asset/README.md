# mixture-asset

English | [简体中文](./README.zh-CN.md)

Optional CPU-only `.mixpack v1` loader and deterministic writer. Canonical uncompressed USTAR carries exact `.mix` source and tightly packed linear RGBA8 images. This crate uses Core for graph validation, resource identities, dependency slicing and immutable preparation; it has no filesystem, browser or GPU dependencies.

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

`AssetView` borrows pixels; `OwnedAsset::from_vec` moves one allocation, while `copy_from` validates and budgets before copying. Preparation returns independent Core snapshots of selected resources. All resources, including unused branches, must pass integrity and closure checks. Package v1 rejects resource-reference overrides; ordinary Core overrides and loose APIs remain available.

`AssetLimits` combines existing Core policies with lowered-only package ceilings: 67 MiB archive, 64 KiB manifest, 2 MiB source, eight same-size images up to 2048 per axis, 64 MiB resources and 202 MiB charged byte buffers. `preparation_buffer_bytes` reports retained archive capacity + conservative source/manifest scratch + selected snapshots. This is not RSS; caller input, typed objects, allocator overhead and outputs are separate. Callers must account for multiple simultaneous assets themselves.

`AssetError` preserves original typed Core errors or a structured `PackageDiagnostic`: a typed `PackageCode` (stable `MIX_PACKAGE_*` spelling), `package` stage, `error` severity, message, evidence and suggestion, serialized with Core diagnostic field names. No extraction, compression, external resource lookup, fallback or implicit cache exists. The crate is unpublished; the CLI `asset` commands and browser `inspectPackage`/`renderPackage` are its adapters. See [release status](../../docs/release.md) for versions.
