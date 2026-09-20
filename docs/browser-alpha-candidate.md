# Browser Alpha candidate — 2026-09-20

English | [简体中文](./browser-alpha-candidate.zh-CN.md)

ALPHA-04 selects one delivery archive of `@openmixture/runtime@0.1.0-alpha.0`. It is an unpublished npm candidate, not a registry release. The same version text was used by earlier local archives: consumers must identify this candidate by its digest and build identity, not its filename alone.

| Identity | Value |
|---|---|
| Clean producer revision | `82b74707b2a8a998190e2f28b16f91fb9614486a` |
| Build ID | `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28` |
| Archive SHA-256 | `a9bcfe8d849f9fb9982a750dd99d026807fdf6453b5be491deeda90e99a2c6ae` |
| Runtime / API schema / Rust engine | `0.1.0-alpha.0` / `1` / `0.1.0` |
| Studio executable revision | `6b2d53e3de16b21725b2a4359a2263f98671a6f9` |

The archive comes from the successful [current-candidate material run](https://github.com/OpenMixture/OpenMixture/actions/runs/35489430243), not a rebuild after testing. Its producer receipt includes Rust, Node, npm and wasm-bindgen versions and the complete package file list. The [delivery evidence](./evidence/alpha-04/README.md) retains the archive, receipts, saved files and comparisons. Documentation commits after the producer and Studio revisions are not presented as tested executable revisions.

## Release notes and compatibility

The package contains the public ESM facade, TypeScript declarations, generated bindings, WASM, matching build metadata, paired README files and both licenses. Installation does not compile Rust or import producer files. It has no install scripts or runtime npm dependencies. The public entry remains `@openmixture/runtime`; WebGPU initialization and disposal are explicit.

Compared with Studio's original `4b914fe` archive, this candidate includes the already integrated explicit round-to-nearest-even conversion before half-float texture storage. It reduces backend-dependent storage rounding; it does not promise identical final pixels on every GPU. Material qualification uses the frozen v2 profile, with exact checker rules and unchanged native goldens. The unadopted compiler/FMA experiments are not included.

There is no `.mix` migration, API schema change, new node or alternate renderer. Preserve saved bytes, replace the entire archive, update only the runtime lock integrity, reinstall with `npm ci`, and check actual `getBuildInfo()` before accepting an upgrade. Never mix JS, declarations or WASM from different builds. Studio demonstrates existing samples and newly authored files through save, independent Player reopen and PNG export; the full qualification is mandatory for later archive changes too.

## Qualified scope and remaining delivery actions

This candidate's recorded coverage comprises controlled Linux Chromium package/contracts/material/lifecycle/deployment CI, Windows Chromium Studio saved-file/native comparisons, ordinary Windows Chrome product editing/export, and a separate Linux/WSL filesystem-isolated consumer. Exact browsers, flags, adapters and results are in the evidence. Earlier ordinary Chrome/Edge/Firefox material results concern their recorded older archive and do not independently certify this archive.

The supported recipe is the pinned Studio/Vite consumer in a secure browser context with WebGPU. GPU-free validation/editing remains useful when acquisition fails; rendering then returns a structured error. Safari, mobile, arbitrary bundlers, untested hardware and universal browser support are not certified. Node GPU, CPU/WebGL fallback, new resources, advanced Studio features and M6 remain outside this delivery.

ALPHA-05 still owns the browser required-check decision and live enforcement. Registry publication and trial deployment are separate actions. Before publication, recheck the registry namespace/version and release policy; publish these exact reviewed bytes only after the distribution decision. A clean consumer must subsequently install the exact registry version and verify integrity/build identity plus the upgrade gates. A successful local tarball install does not close that registry-consumption gate. No Rust crates, release tag or hosted trial are published by ALPHA-04.
