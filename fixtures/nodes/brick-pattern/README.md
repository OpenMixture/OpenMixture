# Brick pattern fixtures

English | [简体中文](./README.zh-CN.md)

MAT-01b implements the [versioned brick contract](../../../docs/mat-01-structured-materials.md). The [source](./input.mix) exposes every parameter. [Cases](./cases.json) use literal interior/gap sentinels for aligned and staggered cells, odd rectangular dimensions and a one-pixel output; default and maximum-seed cases exercise the production shader.

Core tests reject missing/invalid seeds, unsupported versions and odd staggered rows, including atomic overrides; they verify slicing, parameter identity and original-source preservation. GPU tests additionally check exact repeated output, changed cell amplitudes with stable resolved gaps, zero-variation seed independence and degenerate dimensions. Quantized sub-byte bevel pixels are not used to infer occupancy.

Run `cargo xtask test-node brick-pattern` with an explicit adapter policy. The pinned software matrix and existing material regressions remain required. These node fixtures do not establish MAT-01 material or cross-runtime qualification; the frozen material acceptance contract remains separate.
