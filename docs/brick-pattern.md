# Brick pattern v1

English | [简体中文](./brick-pattern.zh-CN.md)

## Bounded use case and ownership

The maintainer selected a tileable brick height field with mortar and edge transitions. Existing checker emits Color without mortar or relief; this increment adds only `brick-pattern@1`, a Scalar generator. Core owns validation/lowering and wgpu owns one shader. No randomness, new crate, file format, image decoder, editor, or second executor is added. Implementation on this branch is not qualification, integration or publication.

## Contract

No inputs; output `value: Scalar` in [0,1]. Defaults and bounds:

| Parameter | Type | Default | Accepted values |
|---|---|---|---|
| columns | Integer | 4 | 1–256 |
| rows | Integer | 8 | 1–256; even for half-offset |
| layout | Enum | half-offset | aligned, half-offset |
| gap | Float | 0.08 | 0–0.5 |
| bevel | Float | 0.08 | 0–0.25 |

Coordinates are top-left-origin pixel centers. Integer division/remainders select cells; odd rows shift by half a cell in half-offset mode. Odd row counts in that mode fail with `MIX_PARAMETER_INVALID_VALUE`, including defaults and overrides on unused nodes. Aligned mode accepts odd rows.

For each axis, measure distance to the nearest cell boundary as a fraction of that axis's cell extent. Let d be the smaller distance, and t = d - gap/2. Height is zero for t <= 0; otherwise it is 1 when bevel is zero, or min(t/bevel,1). Thus gap is the total mortar width, and bevel extends inward from each mortar edge. Widths are relative to each axis independently: rectangular cells do not imply equal mortar widths in pixels. The maximum bounds retain a nonnegative plateau width. Zero gap and zero bevel still assign exact boundary samples to mortar. This is point sampling without antialiasing; subpixel mortar or bevels may disappear, and tiny images are correctness controls, not quality examples. f64 parameters lower to f32 and intermediates use explicit half rounding. No universal cross-hardware last-bit promise is added.

Both layouts repeat across a full UV tile. Half-offset uses an even number of rows so the row pattern repeats vertically. No texture sampling or implicit resize occurs in this node.

## Compatibility

Rust 0.6.0 and browser 0.6.0-alpha.0 are unpublished candidates. The new exhaustive `KernelId::BrickPattern` and `KernelInvocation::BrickPattern` variants require downstream match review. Explicit catalog review changes fourteen types/twelve kernels, retaining both noise versions. `.mix v1`, plan v2/hash domain, API schema 2 and `.mixpack v1` remain unchanged. Old documents and goldens are not migrated. Older runtimes must reject the new type. Candidate consumption changes the producer-owned disposable host catalog/version expectations only; registry qualification remains pinned to published 0.3.0-alpha.0.

## Reproduction and acceptance

[The complete graph](../fixtures/nodes/brick-pattern/input.mix) connects height to existing normal, gradient-map and levels nodes for height, normal, baseColor and roughness. The default is half-offset. Override layout to aligned for stacked tiles, or gap to 0.2 for wide mortar. These are three variants of one fixture, not three new node types.

```sh
cargo xtask test-core
cargo xtask test-plan
cargo xtask shader-check
cargo xtask test-node brick-pattern
cargo run --locked -p mixture-cli -- render fixtures/nodes/brick-pattern/input.mix --size 1024 --output baseColor,height,normal,roughness --out tmp/brick-preview
cargo xtask check
```

Required evidence includes literal mortar/plateau/linear-ramp samples, hard edges, 1-pixel axes, non-square controls, invalid parameters, deterministic hashes, complete 2x2 pixel repetition, half-row shift, and monotonic mortar/bevel changes. Native/browser public consumers must compare height/normal from the same graph and variants against the unchanged <=1 component gate. Review 1K height/normal and 2x2 contact images. Existing original and noise-v2 materials and all six required checks remain gates. Passing local checks alone does not qualify an archive or platform. Evidence and publication status must be reported separately; no old golden is reset.
