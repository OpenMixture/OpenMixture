# MAT-02 subtraction composition counterexample

English | [简体中文](./README.zh-CN.md)

This is bounded design evidence for the [MAT-02 draft](../../mat-02-layered-weathering.md), not acceptance of a new node or material. A fresh CLI build from clean source `4c49186097290a842c0c0f11a9f6a62ad29b229b` reproduced the counterexample on the recorded NVIDIA GeForce GT 1030 through Vulkan and DX12. Existing nodes behave according to their half-float contracts; no old shader, tolerance or golden is changed.

## Observation

[input.mix](./input.mix) tries to synthesize `max(a-b,0)` using three existing nodes: invert b with levels, mix a and the inverse at weight 0.5, then remap [0.5,1] to [0,1]. The exactly representable inputs are a=0.5 and b=0.499755859375; their exact positive difference is 0.000244140625 (1/4096). The first intermediate rounds `1-b` to 0.5, so the composed result is zero.

A final levels node amplifies the difference by 4096 before RGBA8 encoding: the rendered `height` pixel is `[0,0,0,255]`. The separate constant `roughness` reference is `[255,255,255,255]`, the amplified analytical expectation. Both backends reproduce these bytes. The reference does **not** execute a proposed subtraction kernel. Without amplification both values could quantize to the same 8-bit pixel, hiding the lost internal value.

[Vulkan report](./vulkan.json), [DX12 report](./dx12.json) and [receipt](./receipt.json) retain the exact commands, selected adapters, plan hashes, toolchain, binary/lock/verifier identities and input/output hashes. The 1x1 pixels remain in the `vulkan/` and `dx12/` directories. This proves a specific composition limitation; it does not prove arbitrary graph precision, a visible painted-metal defect, performance targets or correctness of an unimplemented node. MAT-02a must still decide catalog admission and freeze its material plan. MAT-01 human acceptance remains independent.

## Reproduction and retention

From a clean checkout, build its CLI and run the retained verifier with an explicit binary, input and **fresh** output directory:

```powershell
cargo build --locked -p mixture-cli
node docs/evidence/mat-02-subtraction/reproduce.mjs target/debug/mixture.exe docs/evidence/mat-02-subtraction/input.mix tmp/subtraction-new-run
```

If `CARGO_TARGET_DIR` is set, supply its actual CLI path. The verifier runs Vulkan and DX12 explicitly, rejects changed sources/binary during execution and independently decodes the single RGBA8 pixel. It deliberately requires both backends; it does not fall back on unsupported hosts. Current source creates a new record, not a reproduction of the original binary identity. To rebuild the original runtime use the recorded source revision and the retained input/verifier bytes in an ignored directory; these evidence files were added after that revision.

Git retains the small complete selected input, reports, receipt, pixels and verifier. Original paths inside reports describe the run and are not rewritten after copying. The built executable and routine build logs are temporary; their hashes do not constitute permanent binary retention. The build used the development profile, and no timing claim is made.
