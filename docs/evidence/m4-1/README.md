# M4.1 governance acceptance

English | [简体中文](./README.zh-CN.md)

Accepted on 2026-09-12 for repository governance at implementation merge `cc98dc9298add5ed172e0c1752d8a768b61a0eb5`, integrated by [PR #1](https://github.com/OpenMixture/OpenMixture/pull/1). The [machine receipt](./acceptance.json) binds the revisions, run attempts, protection settings and preservation checks. The later documentation commit carrying this record has its own required PR and main checks; it is not the source tested by the runs below.

## Verified change

`main` was created from accepted commit `99704e8c6a05e9e0b60e4264aa2a5901fbb391c6` without rewriting history, then made the default branch after bootstrap CI passed. Historical branches were retained. Active ruleset `23016046` requires a pull request, resolved conversations, an up-to-date base and all four named GitHub Actions checks; it blocks deletion and non-fast-forward updates and has no bypass actors. The single-maintainer policy requires zero approving reviews. The [committed ruleset](../../../.github/main-ruleset.json) matched the live API at acceptance; the [governance guide](../../governance.md) provides current-state verification commands.

PR #1 updated CI triggers to pull requests, pushes to `main` and manual dispatch, with 30-day retention requested for new artifacts. Test commands, job names, the pinned software adapter and serial GPU execution were preserved. Active documentation and navigation were synchronized in English and Simplified Chinese, including the distinction between historical implementation batch IDs and actual GitHub PR numbers, evidence retention and tested platform limits.

## Recorded checks

| Source | CPU workflow | GPU workflow |
|---|---|---|
| Bootstrap `main`, `99704e8` | [Passed](https://github.com/OpenMixture/OpenMixture/actions/runs/34676251027) | [Passed](https://github.com/OpenMixture/OpenMixture/actions/runs/34676251035) |
| PR #1, reported head `befb96b` | [Passed](https://github.com/OpenMixture/OpenMixture/actions/runs/34676473145) | [Passed](https://github.com/OpenMixture/OpenMixture/actions/runs/34676473224) |
| Integrated `main`, `cc98dc9` | [Passed](https://github.com/OpenMixture/OpenMixture/actions/runs/34677769615) | [Passed](https://github.com/OpenMixture/OpenMixture/actions/runs/34677769619) |

Each CPU workflow checks Linux, macOS and Windows. Each GPU workflow checks the configured Linux pinned SwiftShader Vulkan workload: smoke and independent source/package consumption, three 1K materials, and the largest material at 2K. The receipt records GitHub's reported head SHA for PR runs; it does not claim that this is the synthetic merge SHA checked out by `actions/checkout`.

Local `cargo xtask check` passed for the implementation change. Workflow comparison confirmed only trigger/retention changes; the four required check names and GitHub Actions source matched the ruleset. The implementation and its merge have identical Git trees. A comparison of 32 protected Git objects against the bootstrap revision confirmed that runtime code, dependencies, tooling, original source bundle, material inputs/baselines, historical human receipts and bound visual reports/images were unchanged. This is governance acceptance, not a new visual or platform acceptance.

## Retention and limits

This summary, the receipt and governance configuration remain in Git. Existing accepted runtime and visual evidence remains at its original tracked paths under the [evidence policy](../../evidence-policy.md). Complete output from these ordinary repeated CI runs is temporary: bootstrap artifacts report expiry on 2026-12-11 under the previous policy; PR #1 artifacts report expiry on 2026-10-12 under the new policy. Integrated-main artifacts also report expiry on 2026-10-12. Artifact digests are service-reported metadata, not a claim that the bundles were downloaded and verified locally. No durable external archive is claimed. After expiry, the recorded gate results remain, but inspection of the complete original run output may no longer be possible.

No checkpoint tag was created. Packages remain unpublished with `publish = false`; M5 has not started. Product readiness and hardware limitations remain defined by the [release record](../../release.md).
