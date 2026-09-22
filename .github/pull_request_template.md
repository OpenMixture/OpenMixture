English | [简体中文](./pull_request_template.zh-CN.md)

## Goal

Describe the concrete problem and resulting behavior.

## Why now

Identify the current milestone exit criterion or blocking defect.

## Design

Explain ownership, data flow, and relevant tradeoffs.

## Agent Guide impact

Major adjustments must update `AGENTS.md` and `AGENTS.zh-CN.md` in this PR. List the updated sections, rationale, ownership/compatibility impact and verification links; otherwise explicitly state that no operational rule changed.

## Evidence

- Targeted verification:
- `cargo xtask check`:
- GPU adapter/backend and visual evidence, if applicable:

## Risks

Describe possible regressions and the checks covering them.

## Out of scope

List adjacent work intentionally left for later PRs.
