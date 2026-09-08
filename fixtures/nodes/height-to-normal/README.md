# height-to-normal fixtures

English | [简体中文](./README.zh-CN.md)

Flat and non-flat inputs, zero/max strength, one-texel dimensions and invalid strength. Literal ramp probes in the actual shader establish rectangular UV derivatives, repeat edges and tangent Y direction.

```bash
cargo xtask test-node height-to-normal
```

See [the contract](../../../docs/node-contracts.md), [cases](./cases.json), and [input](./input.mix). Tests validate invalid cases before GPU work, preserve golden files, and save actual adapter/plan evidence under `tmp/node-tests/<backend>/`. The shader is the only pixel implementation.
