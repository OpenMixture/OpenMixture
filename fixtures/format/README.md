# .mix v1 format fixtures

English | [简体中文](./README.zh-CN.md)

PR-005 source validation fixtures follow the [file format](../../docs/file-format.md) and [M2 node contracts](../../docs/node-contracts.md). These are JSON input/diagnostic fixtures, not pixel goldens.

| Fixture | Purpose |
| --- | --- |
| [valid/checker.mix](./valid/checker.mix) | Minimal checker → baseColor, default parameters, exposed default frequency |
| [valid/all-m2.mix](./valid/all-m2.mix) | All six M2 contracts, levels/blend mask, explicit roughness |
| [invalid/duplicate-key.mix](./invalid/duplicate-key.mix) | Reject duplicate JSON keys, even equal values |
| [invalid/unknown-field.mix](./invalid/unknown-field.mix) | Reject undeclared top-level layout |
| [invalid/unsupported-version.mix](./invalid/unsupported-version.mix) | Reject document version 2 without migration |
| [invalid/duplicate-ids.mix](./invalid/duplicate-ids.mix) | Reject ambiguous node IDs |
| [invalid/duplicate-edges.mix](./invalid/duplicate-edges.mix) | Reject duplicate edge identity and single-input conflict |
| [invalid/unknown-port.mix](./invalid/unknown-port.mix) | Reject an undeclared source port |
| [invalid/type-mismatch.mix](./invalid/type-mismatch.mix) | Scalar cannot connect to required Color baseColor |
| [invalid/cycle.mix](./invalid/cycle.mix) | Disconnected a → b → a cycle with a non-cycle downstream node |
| [invalid/cycle.diagnostics.json](./invalid/cycle.diagnostics.json) | Reviewed, path-independent exact cycle diagnostic |
| [invalid/missing-base-color.mix](./invalid/missing-base-color.mix) | Required connection is not supplied by a default |
| [invalid/invalid-parameters.mix](./invalid/invalid-parameters.mix) | Range, color shape, and unknown parameter failures |
| [invalid/invalid-exposed.mix](./invalid/invalid-exposed.mix) | A node version cannot be exposed as a mutable parameter |

Run `cargo xtask test-format`. Core tests also construct byte/count boundary inputs, deep nesting, duplicate escaped keys, wrong directions, self-loops, source-order permutations, and parameter/public-binding variants. CLI tests read these fixtures without modification and assert exit codes and document paths. No fixture update command is implemented.
