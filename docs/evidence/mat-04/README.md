# MAT-04 reusable recipes — accepted within recorded scope

English | [简体中文](./README.zh-CN.md)

On 2026-10-13 (Asia/Shanghai) the maintainer accepted MAT-04 and asked that the woven parent's higher descriptor peak be recorded as a limitation; see the [human decision](./human-decision.json). All results bind clean main `d34b278b7c7c59395e9d70a8589c545095139199` ([binding.json](./binding.json)). Rust 0.11.0 / browser 0.11.0-alpha.0 remain unpublished.

## What is accepted

The [MAT-04a contract](../../mat-04a-recipe-contract.md) and [ADR 0010](../../decisions/0010-reusable-recipes.md): caller-supplied `.mixrecipe` documents, `.mix` v2 parents with instances, and Core-only expansion before normalization. RenderPlan v3, the wgpu executor, the node catalog and `.mixpack v1` are unchanged. Four recipes ([fixtures](../../../fixtures/recipes/README.md)) rebuild two accepted materials:

| Parent | Replaces | Instances | Exposed IDs |
|---|---|---|---|
| [brick-paving/material-recipes.mix](../../../fixtures/materials/brick-paving/material-recipes.mix) | MAT-01 brick graph | brick-structure, masked-layer-surface | 24 → 10 |
| [woven-fabric/material-recipes.mix](../../../fixtures/materials/woven-fabric/material-recipes.mix) | MAT-03 r3 woven graph | weave-structure, 2 × directional-grain, masked-layer-surface | 33 → 19 |

`masked-layer-surface` is shared by both families. Parameter fan-out, explicit per-instance seeds and instance isolation are exercised by the frozen cases.

## Gates

| Gate (contract) | Evidence | Result |
|---|---|---|
| 1 Structural equivalence (CPU) | `cargo test -p mixture-core --test recipe_materials`, every frozen case and woven stress case | Expanded parents are isomorphic to the accepted graphs |
| 2 Pixel identity, GT 1030 Vulkan | [native-vulkan.json](./machine/native-vulkan.json) | 71/71 rows, every channel byte-identical |
| 2 Pixel identity, GT 1030 DX12 | [native-dx12.json](./machine/native-dx12.json) | 71/71 rows, every channel byte-identical |
| 2 Pixel identity, pinned SwiftShader (CI) | [native-swiftshader.json](./machine/native-swiftshader.json) | 59 rows identical; 12 woven 2048² rows `notRunOnSoftware` (MAT-03 amendment 4) |
| 2 Chrome subset | [chromium-swiftshader.json](./machine/chromium-swiftshader.json) | 8 rows (default and varied, 64² and 257×129, both families) identical through the public browser runtime |
| 3 Budgets | Native receipts | All descriptor peaks within each material's frozen ceiling; matched-adapter timing budgets pass on Vulkan and DX12 |
| 4 Controls | CPU gate and every receipt | Conceptual controls map onto v2 IDs and reproduce the accepted cases |
| 5 Isolation | `woven_grain_instances_stay_isolated` | Changing `warpSeed` changes only `warp--noise` |
| 6 Negative corpus | `--test recipes` (Core and CLI), runtime unit tests | Every new code and rule covered |
| 7 Compatibility | All six required checks on `d34b278` | v1 fixtures, plan snapshots, goldens and receipts unchanged; v1 entry points and package v1 reject v2 |

Byte identity carries the existing human visual acceptance of both materials over; no new visual review was performed.

Hardware timing for the parent (warm median of five, ms):

| Material | Vulkan 1024² | Vulkan 2048² | DX12 1024² | DX12 2048² |
|---|---:|---:|---:|---:|
| brick | 134 | 589 | 144 | 575 |
| woven | 193 | 831 | 189 | 798 |

## Limitations and observations

- **Limitation (maintainer decision):** at 2048² the woven parent's descriptor peak is **469,762,640 B**, against **301,990,480 B** for the accepted v1 graph (about +55.6%). This leaves about 12.5% headroom under the frozen 536,870,912 B. Expanded IDs change the lexical pass order and texture-slot reuse, as the contract anticipates; pixels are unaffected. Brick peaks are unchanged. A future PERF-MAT slice can address it only with its own measured case.
- **Chrome evidence is software only.** By maintainer choice, the Chrome subset ran in CI on Chromium 153 with SwiftShader; it is not hardware Chrome on the GT 1030. Software passes do not qualify hardware pixels.
- **Backends differ from each other.** Vulkan and DX12 agree on 332 of 335 channel hashes. The other 3 differ between backends, but on each backend the parent equals its v1 graph byte for byte, which is the gate.
- Only the recorded adapters are qualified. Painted metal is not a parent in this slice.

## Reproduce

```bash
cargo test --locked -p mixture-core --test recipes --test recipe_materials
cargo test --locked -p mixture-cli --test recipes -- --include-ignored
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=0 cargo xtask test-material recipe-parents
```

A fresh run writes a new receipt under ignored `tmp/materials/recipe-parents/`; it does not replace these receipts. Rendered images are not retained because identity is the gate, and the accepted brick and woven images remain in their own records. CI artifacts expire on the dates in [binding.json](./binding.json).
