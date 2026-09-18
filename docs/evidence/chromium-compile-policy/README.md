# Chrome / Edge compiler-policy investigation — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**Formal full-material certification remains pending.** Ordinary Chrome 153.0.8010.48 and Edge 153.0.4234.32 each pass 27/44 pixel gates against the unmodified Release/DXC reference used for [Firefox acceptance](../alpha-03-configured/README.md). An isolated native compiler-flag experiment matches both browsers exactly in all 44 channels. The experiment changes the dependency graph and is not an accepted reference for the archived candidate.

## Bound inputs and results

Both browsers used the exact previously qualified archive `08cdce838500343b34d775a62058d6adfd5c6035f109d86e03e5ac1ff11f1ec2`, engine `ec571816026a945a706067769fef76e24c8398b0`, pinned Studio `56c510ab57daa1b68ef660525a648a582730a37e`, and the original 11 cases / 44 channels. The archive and native manifest remain in the [configured-reference record](../alpha-03-configured/README.md). Host: Windows 11 build 26100, NVIDIA GT 1030, driver `32.0.15.8266`. Fresh-profile launch arguments and observed hardware/build evidence are in each browser receipt; no GPU, blocklist, headless or security overrides were added.

| Browser | Unmodified reference | Strict native pixel diagnostic | Production Player |
|---|---|---|---|
| Chrome 153.0.8010.48 | [27/44; failed](./chrome/comparison.json) | 44/44 exact | [Passed](./chrome/production.json) |
| Edge 153.0.4234.32 | [27/44; failed](./edge/comparison.json) | 44/44 exact | [Passed](./edge/production.json) |

The failed comparisons have maximum byte difference 1 and maximum changed-pixel ratio `0.001064300537109375`. Full runtime receipts retain completed loading, initialization, owned outputs, lifecycle and synthetic failure probes. Production checks independently verified the same archive, exact 65×3 checker PNG download, disposal and absent test harness. Those successes do not override the failed material gate.

## Isolated experiment

Using Chrome's DXC DLL with unmodified wgpu still produced the Firefox-like reference. The isolated [two-site patch](./diagnostic-only.patch) adds `D3DCOMPILE_IEEE_STRICTNESS` for FXC and `-Gis` for DXC in `wgpu-hal 30.0.1`. Release/DXC compilation with this patch produced [zero differences for all 88 browser/channel comparisons](./strict-comparison.json). [Diagnostic provenance](./diagnostic.json) records the patched source and binary digests. Only the DXC arm was exercised by that full experiment; the FXC arm is not independently certified.

The inspected [Dawn D3D12 implementation](https://github.com/google/dawn/blob/main/src/dawn/native/d3d12/ComputePipelineD3D12.cpp) requests IEEE strictness by default unless its toggle changes that policy. Chromium's [descriptor IDL](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/modules/webgpu/gpu_shader_module_descriptor.idl) restricts its `strictMath` override to developer features. This source inspection and the controlled flag experiment identify compiler arithmetic policy as the practical mismatch; they do not prove the exact internal backend/toggle state of each installed browser. GL/ANGLE/compositor fields are not WebGPU backend identity.

The diagnostic used an isolated checkout of `2cc36863eb5cb4a0f419722b172fb5aceaec239f`, an explicit local Cargo patch for `wgpu-hal`, and Chrome's `dxcompiler.dll`. Its lock delta replaced the registry source/checksum of that crate with the local patched source. The main checkout's dependencies, shaders, goldens and tolerances were unchanged. The diagnostic executable was isolated after the experiment and the regular Release executable rebuilt with the original locked dependency graph.

## Formal implementation decision

The current wgpu API does not expose this compiler flag. The package verifier intentionally rejects unreviewed non-registry dependencies in [registry_pins](../../../xtask/src/package.rs). A local Cargo patch also does not propagate automatically to separately consumed Cargo archives. Do not relax the source-drift guard or label the diagnostic manifest as the original candidate to manufacture certification.

The proposed formal route is a minimal pinned dependency branch exposing an explicit per-context DX12 math policy, retaining the existing default for Firefox references. It needs a narrowly reviewed dependency-source policy, exact revision pinning in independent package verification, focused compiler-option tests, unchanged golden checks, and a newly built/qualified browser candidate. Then repeat ordinary Chrome/Edge gates against the actual candidate's declared reference configuration. Alternatively retain the failure record until upstream provides the option. The user has been asked to choose this dependency-maintenance path; no fork, dependency-policy exception, browser feature override or release has been created.

## Reproduction and retention

First reproduce the ordinary baseline using the [existing procedure](../../default-browser.md), retained archive and native manifest. For diagnosis only: create an isolated checkout at the source above; copy the published `wgpu-hal 30.0.1` sources to a separate directory; apply the retained patch; add an explicit local Cargo patch in the isolated manifest and generate its diagnostic lock. Build Release, scope Chrome's DXC directory to the render process PATH, and render the same fixture cases/overrides at 1024 with `baseColor,normal,roughness,height`. Compare decoded RGBA pixels to the retained browser-run outputs. Record the modified dependency identity and keep this outside ordinary qualification.

Git retains complete ordinary failure reports/receipts, the minimal diagnostic patch, all 88 diagnostic pixel measurements and production receipts. Full PNGs, downloaded sources, isolated checkout, executable and logs are temporary under `tmp/chromium-cert-*`. No visual golden was changed or accepted, and no permanent full-bundle auditability is claimed.
