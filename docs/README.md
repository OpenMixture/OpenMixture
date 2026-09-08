# Documentation

English | [简体中文](./README.zh-CN.md)

The root documents are the active project contract:

- [Mission and setup](../README.md)
- [Contributor and agent guide](../AGENTS.md)
- [Architecture](../ARCHITECTURE.md)
- [Roadmap](../ROADMAP.md)
- [Initial PR sequence](../INITIAL_PRS.md)
- [M3 review and evidence](./m3-review.md)
- [M4 implementation train](../M4_PRS.md)

[Development](./development.md) describes implemented commands and verification limits.
[Public native Rust consumption](./native-sdk.md) defines PR-011 API ownership, dependency exposure, the independent application, and release measurements.
[Native CLI reports and exit codes](./cli-contract.md) defines PR-012 JSON presence/types, complete human context, independent process tests and partial-write behavior.
[Diagnostics and safety limits](./diagnostics.md) defines the PR-002 public API, JSON contract, and shared CLI exit-code policy.
[GPU context and doctor](./gpu-context.md) describes PR-003 acquisition, report fields, exit codes, and pinned software CI.
[Built-in checker](./builtin-checker.md) defines PR-004 pixels, readback, PNG output, golden provenance, and execution evidence.
[Strict .mix v1 format](./file-format.md) defines PR-005 decoding, validation, CLI behavior, and source fixtures.
[Eleven built-in node contracts](./node-contracts.md) define typed ports, parameters, defaults, and declared semantics before graph execution.
[Deterministic RenderPlan](./render-plan.md) defines PR-006 requests, normalization, slicing, typed resources, estimates, hashes, and CLI inspection.
[Eleven-node graph rendering](./graph-rendering.md) defines PR-007/009/010 execution, caching, readback/PNG encoding, examples, tests, and adapter evidence.
[Architecture decisions](./decisions/README.md) record the four foundation decisions.

The original [review bundle](../mixture-greenfield-docs/README.md) is retained unchanged as source material. Its planned commands are not a claim that later milestones are implemented. Root documentation is maintained with the code from this point onward; the bundle manifest applies only to the original bundle.

[Material goldens](./material-goldens.md) defines PR-008/009/010 guarded updates, fixture schema, metrics, and the ceramic/leather/wood review workflows.

## Supporting documentation

- [Examples](../examples/README.md)
- [Node fixtures](../fixtures/nodes/README.md)
- [Material fixtures](../fixtures/materials/README.md)
- [Pull request description template](../.github/pull_request_template.md)

## Languages and synchronization

- Active project documents are paired in the same directory: English uses `*.md`, Simplified Chinese uses `*.zh-CN.md`. Each page provides language links at the top.
- Update both languages in the same PR when behavior, commands, scope, or acceptance criteria change. Chinese documents preserve the full requirements rather than replacing the source with a summary.
- Keep commands, paths, API/type/node identifiers, and executable examples consistent. Chinese prose links to Chinese documents where available; source code, configuration, licenses, and the original review bundle retain their original targets.
- The original `mixture-greenfield-docs/` remains a historical source. Use the current paired project documents for implementation status. If the languages diverge, consult the authoritative contracts and implementation using [the guide's authority order](../AGENTS.md), then correct both versions.
