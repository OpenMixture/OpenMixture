# Brick/paving material candidate

English | [简体中文](./README.zh-CN.md)

This is the unaccepted MAT-01c four-channel [material](./material.mix), implementing the [frozen brief](../../../docs/mat-01-structured-materials.md). Ten passes produce baseColor, roughness, height and normal. Aligned copies of brick-pattern separate height amplitude, color variation and the common surface mask; color and roughness share the mask, and normal derives from final height.

[controls.json](./controls.json) is an acceptance-fixture mapping from conceptual controls to public override IDs, not a new runtime schema or automatic parameter propagation. A caller changes each listed public override together for shared layout controls; heightVariation and colorVariation remain independent. The compiled graph has no hidden linkage between duplicate nodes. MAT-04 owns future graph/parameter reuse.

[qualification-plan.json](./qualification-plan.json) retains frozen cases and budgets. This candidate has no accepted golden, human review or complete cross-runtime verdict yet. Do not install a golden merely because the source renders. MAT-01d must provide the full matrix, PBR review, package/public-browser consumption and retained evidence.

The candidate [Native matrix test](../../../examples/native-consumer/tests/brick_material.rs) uses public APIs to render all twenty case/size combinations, retain eighty channel PNGs, verify exact repeated pixels, descriptor/pass budgets, independent parameter effects and default downsampling, verify exact source/plan/pixel package roundtrips, and enforce cold/five-warm budgets when the recorded adapter matches the frozen hardware/software target. Its receipt is partial engine evidence, not material acceptance. The candidate browser consumer renders the same twenty cases. `node scripts/browser-runtime/check-brick.mjs <candidate-qualification> <fresh-output>` binds all eighty comparisons to source/build identities and runs the Native harness in release mode. `test-node brick-pattern` checks test-only periodic sampling origins against wrapped production pixels and validates raw f16 range. The Native matrix records 64×64-cell stress at 256² versus 1024²; it does not relax the default gate. Human PBR review remains separate. Unknown adapters receive no timing-budget qualification; debug timings do not certify release performance.

Run from the repository root in PowerShell, selecting an explicit backend and a new output directory each time:

```powershell
$env:MIXTURE_GPU_BACKEND='vulkan'
$env:MIXTURE_GPU_SOFTWARE='0'
$env:MIXTURE_BRICK_FIXTURE_DIR=(Resolve-Path fixtures/materials/brick-paving).Path
$env:MIXTURE_BRICK_EVIDENCE_DIR=Join-Path $env:TEMP 'mixture-brick-native-run-01'
cargo test --release --locked --all-features --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --test brick_material brick_material_public_gpu_matrix -- --ignored --nocapture
```

Use `dx12` or `metal` for other native backends; pinned SwiftShader requires the documented Vulkan driver setup and software policy `1`. Ordinary CPU tests compile but do not execute this GPU test. A pre-existing destination is rejected; failures never overwrite earlier evidence. The directory contains exact input copies and `native-matrix.json` with actual adapter/plan identities. A successful receipt covers only its stated scope.

Both the xtask and browser comparison entry points explicitly build under `target/native-consumer`, as does the command above. This keeps generated Cargo files inside the repository's tracked ignore policy, independent of a developer's private Git excludes. Qualification still rejects dirty sources; the [recorded CI failure](../../../docs/evidence/mat-01/ci-failure-4bf/README.md) explains the regression and fix.

Preview through the public CLI:

```bash
cargo run --locked -p mixture-cli -- render fixtures/materials/brick-paving/material.mix --size 1024 --output baseColor,normal,roughness,height --out tmp/brick-paving-preview --json
```

`cargo xtask test-material brick-paving` runs the release Native matrix with the normal explicit GPU policy and fresh evidence under `tmp/materials/brick-paving/`. It verifies properties and packages; it does not install golden pixels or assert human acceptance. Original three-material `golden check` remains unchanged. The required software GPU CI runs this additional material gate.

Controlled PBR review: run `npm ci --ignore-scripts --prefix examples/browser-consumer`, then `node scripts/brick-material-preview.mjs <native-evidence> <fresh-output>`. Chrome WebGPU displays existing PNGs with fixed camera, lights and dielectric GGX settings, producing plane/sphere and 1×/3× tiling views for every preset. It does not execute material graphs or displace geometry. The receipt binds input PNG and preview-source digests; it never manufactures human acceptance.

Measured development-host limit: on GT 1030 Vulkan, 64×64 cells at 256² have height mean downsample error about 14.13/255 and baseColor red error about 7.99/255, exceeding the default 4/255 gate. This setting is outside the default quality guarantee. Retain the measurements; do not reset gates or silently modify inputs.
