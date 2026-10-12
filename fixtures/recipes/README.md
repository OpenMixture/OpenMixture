# Recipe fixtures

English | [简体中文](./README.zh-CN.md)

MAT-04 reusable-recipe inputs under the accepted [MAT-04a contract](../../docs/mat-04a-recipe-contract.md). These are JSON source fixtures, not pixel goldens.

| Fixture | Purpose |
| --- | --- |
| [masked-layer-surface.mixrecipe](./masked-layer-surface.mixrecipe) | The contract's cross-family recipe: one mask selects between an under-layer color/roughness and connected over-layer inputs |
| [acceptance-plan.json](./acceptance-plan.json) | Machine-readable form of the contract's frozen acceptance section: four recipes, two parents, exposed IDs, budgets, gates and presets |

`cargo test -p mixture-core --test recipes` decodes these fixtures, compares an expanded parent with its hand-flattened v1 graph, and checks the plan against the accepted brick and woven fixtures it references. The remaining recipes, both parents and the preset corpus arrive with MAT-04c; the plan's values do not change when they do.
