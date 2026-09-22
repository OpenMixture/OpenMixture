# Brick/paving material candidate

English | [简体中文](./README.zh-CN.md)

This is the unaccepted MAT-01c four-channel [material](./material.mix), implementing the [frozen brief](../../../docs/mat-01-structured-materials.md). Ten passes produce baseColor, roughness, height and normal. Aligned copies of brick-pattern separate height amplitude, color variation and the common surface mask; color and roughness share the mask, and normal derives from final height.

[controls.json](./controls.json) is an acceptance-fixture mapping from conceptual controls to public override IDs, not a new runtime schema or automatic parameter propagation. A caller changes each listed public override together for shared layout controls; heightVariation and colorVariation remain independent. The compiled graph has no hidden linkage between duplicate nodes. MAT-04 owns future graph/parameter reuse.

[qualification-plan.json](./qualification-plan.json) retains frozen cases and budgets. This candidate has no accepted golden, human review or complete cross-runtime verdict yet. Do not install a golden merely because the source renders. MAT-01d must provide the full matrix, PBR review, package/public-browser consumption and retained evidence.

The candidate [Native matrix test](../../../examples/native-consumer/tests/brick_material.rs) uses public APIs to render all twenty case/size combinations, retain eighty channel PNGs, verify exact repeated pixels, descriptor/pass budgets, independent parameter effects and default downsampling, and measure cold/five-warm renders. Its receipt is partial engine evidence, not material acceptance. Browser comparison, periodic-origin probes, aliasing stress, timing-budget verdicts, package roundtrip and human PBR review remain separate unfinished gates.

Run from the repository root in PowerShell, selecting an explicit backend and a new output directory each time:

```powershell
$env:MIXTURE_GPU_BACKEND='vulkan'
$env:MIXTURE_GPU_SOFTWARE='0'
$env:MIXTURE_BRICK_FIXTURE_DIR=(Resolve-Path fixtures/materials/brick-paving).Path
$env:MIXTURE_BRICK_EVIDENCE_DIR=Join-Path $env:TEMP 'mixture-brick-native-run-01'
cargo test --release --locked --manifest-path examples/native-consumer/Cargo.toml --test brick_material brick_material_public_gpu_matrix -- --ignored --nocapture
```

Use `dx12` or `metal` for other native backends; pinned SwiftShader requires the documented Vulkan driver setup and software policy `1`. Ordinary CPU tests compile but do not execute this GPU test. A pre-existing destination is rejected; failures never overwrite earlier evidence. The directory contains exact input copies and `native-matrix.json` with actual adapter/plan identities. A successful receipt covers only its stated scope.

Preview through the public CLI:

```bash
cargo run --locked -p mixture-cli -- render fixtures/materials/brick-paving/material.mix --size 1024 --output baseColor,normal,roughness,height --out tmp/brick-paving-preview --json
```

The existing `test-material` dispatcher has not yet admitted this candidate; MAT-01d adds the material-specific verifier before claiming that command works.
