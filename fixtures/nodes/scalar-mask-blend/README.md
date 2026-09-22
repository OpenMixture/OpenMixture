# Scalar mask blend fixtures

English | [简体中文](./README.zh-CN.md)

The [MAT-01 contract](../../../docs/mat-01-structured-materials.md) defines required Scalar a/b/mask inputs and opacity. [Cases](./cases.json) exercise exact mask/opacity endpoints, partial opacity, rectangular and degenerate dimensions. Core tests reject missing/wrong-kind masks and bad versions even at zero opacity, preserve repeated bindings and verify sliced hashes.

`cargo xtask test-node scalar-mask-blend` additionally feeds literal negative/above-one mask and input texels through the production shader, verifying saturation and exact endpoints without a CPU renderer. The existing uniform scalar-blend tests share only GPU upload/readback plumbing and retain their original expectations.
