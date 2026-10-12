# Recipe fixtures

English | [简体中文](./README.zh-CN.md)

MAT-04 reusable-recipe inputs under the accepted [MAT-04a contract](../../docs/mat-04a-recipe-contract.md). These are JSON source fixtures, not pixel goldens.

| Fixture | Purpose |
| --- | --- |
| [masked-layer-surface.mixrecipe](./masked-layer-surface.mixrecipe) | The contract's cross-family recipe: one mask selects between an under-layer color/roughness and connected over-layer inputs |
| [brick-structure.mixrecipe](./brick-structure.mixrecipe) | Three `brick-pattern@1` sharing seven layout controls and the seed; height and color variation stay separate |
| [weave-structure.mixrecipe](./weave-structure.mixrecipe) | Three `weave-pattern@2` (warp-share, coverage, height) sharing seven weave controls |
| [directional-grain.mixrecipe](./directional-grain.mixrecipe) | Seeded noise, quarter-turn direction, levels and gradient map; the seed is required on every instance |
| [presets/](./presets/) | Override maps for each parent, three per material, each reproducing one frozen case |
| [acceptance-plan.json](./acceptance-plan.json) | Machine-readable form of the contract's frozen acceptance section: four recipes, two parents, exposed IDs, budgets, gates and presets |

The parents are [brick-paving/material-recipes.mix](../materials/brick-paving/material-recipes.mix) and [woven-fabric/material-recipes.mix](../materials/woven-fabric/material-recipes.mix). `cargo test -p mixture-core --test recipes` covers the Core rules; `--test recipe_materials` runs the CPU gates: for every frozen case (and woven stress case) the expanded parent is isomorphic to the accepted v1 graph, public IDs and descriptor budgets match the plan, grain instances stay isolated, and presets reproduce their cases. Pixel identity on GPU adapters is MAT-04d.
