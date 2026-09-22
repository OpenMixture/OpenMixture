# fractal-noise fixtures

English | [简体中文](./README.zh-CN.md)

Explicit source seed, both periodic bases, full u32 boundaries, octave/persistence equivalence, different seeds, and warm-cache repeatability. Two whole-image 129×65 RGBA baselines use pinned SwiftShader; hardware permits at most one byte error.

```bash
cargo xtask test-node fractal-noise
```

See [the contract](../../../docs/node-contracts.md), [cases](./cases.json), and [input](./input.mix). Tests validate invalid cases before GPU work, preserve golden files, and save actual adapter/plan evidence under `tmp/node-tests/<backend>/`. The shader is the only pixel implementation.

## Version 2

[NUM-01](../../../docs/stable-noise.md) adds `stable.mix`, small rectangular/boundary cases and a separate exact `stable-defaults-129x65.rgba`. The new baseline was captured on Windows GT 1030 Vulkan and is checked with zero tolerance on every adapter, including pinned software CI. Original v1 baselines are unchanged. Production arithmetic is tested against independent u64/f64 operations, and both versions retain seed/cache/octave invariants.
