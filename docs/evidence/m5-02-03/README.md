# M5-02 / M5-03 local acceptance — 2026-09-14

English | [简体中文](./README.zh-CN.md)

**The initial browser execution and isolated packaged-consumer gates passed locally.** The independent product's [acceptance record](https://github.com/OpenMixture/Studio/blob/523caddd845e21762a160fcc26a45fb01eb0cec4/docs/evidence/m5-02-03/README.md), delivered in [Studio PR #2](https://github.com/OpenMixture/Studio/pull/2), retains thirteen passing Chromium checks, zero failures/skips, controlled real-device loss, mapping failure cleanup and eight independent modules/devices with 32 checker renders. Its clean tested source is `d38bf68ddc470a33c608592363e3554f05887fe1`; the evidence commit is separate.

The product consumes the unchanged `@openmixture/runtime@0.1.0-alpha.0` archive from clean engine `4b914feb9f3365d292b27ea60c5e0b6004f745e8`, SHA-256 `9e245578de160cee1050259ef34358c3fcd3353a21bbb1672d29701bc1ac8084`. Runtime source/build inputs are unchanged at engine `964a0784850a6993c226aae0ef894f5ed22c4fd7`. No new runtime code or archive was necessary to close these initial gates.

## Decisive browser evidence

The product's checked-in `scripts/verify-isolated.mjs` archives a clean product commit and runs `npm ci`, `npm run check` and `npm run test:browser` under macOS filesystem denial of both original engine and product checkouts. A negative probe proves that denial and the absence of cargo/rustc from PATH. All steps passed. Production assets, including WASM, are served at `/player/`; public build identity is checked against archive provenance and numeric projections against the declared API.

The recorded environment is macOS 26.5.1 arm64, Node 24.20.0, npm 11.19.0 and Chromium 153.0.8010.12 with explicit `--enable-unsafe-webgpu` and `--ignore-gpu-blocklist`. The browser reports `BrowserWebGpu` and redacts adapter identity. The product record retains actual limits, flags, package/lock/fixture identities, test outcomes and failure/cleanup attachments.

Real `GPUDevice.destroy()` between renders and at mapping delivers browser loss. The first map failure remains `MIX_READBACK_FAILED`; subsequent work reports `MIX_GPU_DEVICE_LOST`. Readback cleanup, balanced scopes, zero tracked live allocation bytes, retained pixels and explicit new-device execution pass. A separate misaligned real map verifies actual validation failure, later successful rendering and disposal while a rejected render settles. Injected adapter/device acquisition failures are classification coverage only.

## Engine verification

The following all exited zero on clean engine `964a0784850a6993c226aae0ef894f5ed22c4fd7`; later documentation changes receive a separate repository check before delivery.

```bash
cargo xtask check
cargo xtask shader-check
cargo check --locked -p mixture-wasm --target wasm32-unknown-unknown
node --test packages/runtime/test/runtime.test.mjs
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 cargo xtask gpu-smoke
```

The native run selected Apple M5 / Metal, doctor healthy and checker exact, and passed source and packaged Rust/CLI GPU consumers. Retained support: [doctor](./native-doctor.json), [checker comparison](./native-checker-comparison.json), [native loss regression](./native-device-loss.json) and [smoke result](./native-gpu-summary.txt). [Summary](./summary.json) binds sources, commands and retained files. Native evidence does not certify browser hardware.

## Acceptance boundary and remaining work

This closes the bounded M5-02/M5-03 local checkpoint, not all M5. Controlled destruction is not spontaneous hardware/driver failure or tab termination; explicit new-device creation is not automatic recovery. Descriptor accounting is not physical GPU memory measurement. Full material 1K regression, broader lifetime stress, Player controls/channels/export/scheduling, formal browser CI acceptance and compatibility assessment remain M5-04/M5-05. The existing native required checks remain mandatory for PR integration. GitHub check results are separate from the local receipt.

The runtime remains unpublished, and no website deployment or Studio editor is included. Historical M3/M4/M4.1 and initial browser records remain unchanged. Critical compact results and the actual product archive stay in Git; complete routine logs and temporary outputs are not permanently retained. Reproduction creates a new run.
