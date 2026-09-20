# ALPHA-05 — Browser required checks

English | [简体中文](./README.zh-CN.md)

On 2026-09-20, [PR #22](https://github.com/OpenMixture/OpenMixture/pull/22) made `WASM and npm package` and `Chromium WebGPU material matrix` required alongside the four native checks. The existing active ruleset `23016046` was updated, with no duplicate ruleset or bypass. This record certifies remote policy activation, not a package release or broader browser support.

## Retained evidence

- [Activation receipt](./activation.json): timestamp, baseline main revision `507852c59b69708b122fe24bdc240c68983bf84d`, six successful baseline check runs, source app IDs and job URLs.
- [Before](./before-ruleset.json), [exact update](./applied-ruleset.json), [live readback](./after-ruleset.json) and [effective main rules](./effective-main-rules.json): all controls preserved except the two added required contexts. Strict base synchronization, GitHub Actions app `15368`, PR/review-thread requirements, deletion/non-fast-forward protection and no bypass actors remain enforced.
- [PR required-check snapshot](./pr-required-checks.json): `gh pr checks 22 --required --json name,state,link,workflow` at head `d09fb048c529a9b9113d0065275a66d4a5a64d7d`, immediately after activation. All six were running and already classified as required. This is enforcement evidence, not a claim that this snapshot passed.

The existing browser workflows trigger for every PR, main push and manual dispatch without path filters. Their names and runtime workloads are unchanged. A failing or missing browser result now prevents the normal protected merge. No deliberately failing PR, direct protected push or bypass experiment was performed. The live effective rules and GitHub's required-check classification are the evidence of enforcement. The branch endpoint's classic protection summary can have empty contexts; it does not override active repository rulesets.

## Reproduction and limits

From the repository root, run `node docs/evidence/alpha-05/verify.mjs` for focused snapshot/config/workflow consistency checks, then `cargo xtask check`. Verify current remote state separately:

```bash
gh api repos/OpenMixture/OpenMixture/rulesets/23016046
gh api repos/OpenMixture/OpenMixture/rules/branches/main
gh pr view 22 --repo OpenMixture/OpenMixture --json headRefOid,mergeCommit,state
gh pr checks 22 --repo OpenMixture/OpenMixture --required
gh run list --repo OpenMixture/OpenMixture --branch main
```

The PR's final head and merged main each require fresh six-check results; earlier snapshots do not certify later revisions. The PR/CI records identify these delivery results. Full routine logs and generated packages stay in ignored local output or CI artifacts with requested 30-day retention; this directory retains the compact policy evidence permanently in Git. No new visual acceptance was needed or claimed. Existing ALPHA-04 archive, runtime, shaders, golden pixels, Studio sources and historical records are unchanged. Registry publication and exact registry-version consumption remain separate actions.
