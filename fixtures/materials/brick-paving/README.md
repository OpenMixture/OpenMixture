# Brick/paving material candidate

English | [简体中文](./README.zh-CN.md)

This is the unaccepted MAT-01c four-channel [material](./material.mix), implementing the [frozen brief](../../../docs/mat-01-structured-materials.md). Ten passes produce baseColor, roughness, height and normal. Aligned copies of brick-pattern separate height amplitude, color variation and the common surface mask; color and roughness share the mask, and normal derives from final height.

[controls.json](./controls.json) is an acceptance-fixture mapping from conceptual controls to public override IDs, not a new runtime schema or automatic parameter propagation. A caller changes each listed public override together for shared layout controls; heightVariation and colorVariation remain independent. The compiled graph has no hidden linkage between duplicate nodes. MAT-04 owns future graph/parameter reuse.

[qualification-plan.json](./qualification-plan.json) retains frozen cases and budgets. This candidate has no accepted golden, human review or complete cross-runtime verdict yet. Do not install a golden merely because the source renders. MAT-01d must provide the full matrix, PBR review, package/public-browser consumption and retained evidence.

Preview through the public CLI:

```bash
cargo run --locked -p mixture-cli -- render fixtures/materials/brick-paving/material.mix --size 1024 --output baseColor,normal,roughness,height --out tmp/brick-paving-preview --json
```

The existing `test-material` dispatcher has not yet admitted this candidate; MAT-01d adds the material-specific verifier before claiming that command works.
