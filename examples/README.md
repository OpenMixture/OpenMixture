# Examples

English | [简体中文](./README.zh-CN.md)

[checker.mix](./checker.mix) is the first executable `.mix v1` validation example: a default checker feeds required `baseColor`, and `frequency` exposes its defaulted `cellsX` parameter. Optional material channels use the versioned defaults. PR-005 validates this source; graph compilation/rendering arrive in PR-006/PR-007.

```bash
cargo run --locked -p mixture-cli -- validate examples/checker.mix --json
```

The [all-M2 fixture](../fixtures/format/valid/all-m2.mix) connects all six initial contracts. The separate `render-builtin checker` command renders the fixed PR-004 probe, without reading a `.mix` document.

See [the file format](../docs/file-format.md), [node contracts](../docs/node-contracts.md), [development commands](../docs/development.md), and [implementation train](../INITIAL_PRS.md).
