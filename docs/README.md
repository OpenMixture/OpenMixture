# Documentation

English | [简体中文](./README.zh-CN.md)

The root documents are the active project contract:

- [Mission and setup](../README.md)
- [Contributor and agent guide](../AGENTS.md)
- [Architecture](../ARCHITECTURE.md)
- [Roadmap](../ROADMAP.md)
- [Initial PR sequence](../INITIAL_PRS.md)

[Development](./development.md) describes implemented commands and verification limits.
[Diagnostics and safety limits](./diagnostics.md) defines the PR-002 public API, JSON contract, and planned CLI exit-code policy.
[Architecture decisions](./decisions/README.md) record the four foundation decisions.

The original [review bundle](../mixture-greenfield-docs/README.md) is retained unchanged as source material. Its planned commands are not a claim that later milestones are implemented. Root documentation is maintained with the code from this point onward; the bundle manifest applies only to the original bundle.

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
