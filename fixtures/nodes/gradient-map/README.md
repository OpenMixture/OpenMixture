# gradient-map fixtures

English | [简体中文](./README.zh-CN.md)

Default ramp, endpoints, interior interpolation, straight-alpha color, and invalid endpoint colors. Sentinel pixels verify the linear-to-sRGB boundary.

```bash
cargo xtask test-node gradient-map
```

See [the contract](../../../docs/node-contracts.md), [cases](./cases.json), and [input](./input.mix). Tests validate invalid cases before GPU work, preserve golden files, and save actual adapter/plan evidence under `tmp/node-tests/<backend>/`. The shader is the only pixel implementation.
