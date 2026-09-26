# Repository governance

English | [简体中文](./governance.zh-CN.md)

M4.1 establishes a durable integration branch, actual GitHub pull requests, required checks, and evidence retention after native M4 acceptance. It is repository maintenance, not a new runtime milestone or authorization to begin M5. Product scope and compatibility remain defined by the [roadmap](../ROADMAP.md) and [release checklist](./release.md).

## Branches and change identity

Use `main` as the default integration branch. Begin new work on a `codex/` branch based on current `main`, and integrate it through a GitHub pull request after the required checks pass. Keep each change independently reviewable and preserve meaningful implementation commits. This policy does not require rewriting existing history or deleting old branches.

The historical identifiers `PR-001` through `PR-015` in the [initial train](../INITIAL_PRS.md) and [M4 train](../M4_PRS.md) are implementation batch identifiers. They are not GitHub pull request numbers and must not be turned into fictitious retrospective reviews. For new work, identify the milestone item separately from the actual GitHub PR URL/number and implementing commit. For example, `M4.1-02` is a work item; the PR number is assigned by GitHub.

Use the paired [PR template](../.github/pull_request_template.md) to record the concrete problem, milestone item, design boundary, verification, risks, and explicit out-of-scope work. Public behavior and active documentation changes include their Simplified Chinese counterparts. The [agent guide](../AGENTS.md) continues to govern implementation and tests.

## Main protection

[main-ruleset.json](../.github/main-ruleset.json) is the reviewable desired branch ruleset. It is applied through GitHub administration; committing this file alone does not activate protection. The live API is authoritative for whether the rules are enforced.

- Match only `refs/heads/main`; block deletion and non-fast-forward updates.
- Require an actual pull request and resolution of review conversations.
- Require zero approving reviews while this repository has a single-maintainer workflow. This preserves PR review records without requiring an unavailable second person. There is no CODEOWNERS or latest-push approval requirement.
- Require the branch to be up to date with its base and all six checks below to pass.
- Accept those checks only from GitHub Actions (`integration_id: 15368`), with no configured bypass actors.

| Required check | Coverage |
|---|---|
| `Check (ubuntu-latest)` | Locked repository and isolated package checks on Linux |
| `Check (macos-latest)` | The same CPU checks on macOS |
| `Check (windows-latest)` | The same CPU checks on Windows |
| `Pinned SwiftShader Vulkan materials and packaged consumption` | Linux pinned software GPU smoke, source and packaged consumers, all three 1K materials, and largest-case 2K trace |
| `WASM and npm package` | Locked WASM build, JavaScript/package contracts and exact npm archive generation |
| `Chromium WebGPU material matrix` | Independent SDK candidate consumption (exact registry consumption on `main` and manual runs) plus exact archive installation in pinned Studio, build identity, browser contracts, v2 materials, lifecycle and production deployment |

Keep these check names stable. A renamed job or changed check source requires coordinated ruleset verification; never remove a required check to merge a failing change. Do not apply workflow path filters that can prevent a required check from being reported. Rule changes are themselves reviewed changes, with the live result recorded after application.

The GPU and browser material workflows each start with a `Qualification scope` job running [qualification-scope.sh](../.github/scripts/qualification-scope.sh). A pull request whose every changed path is documentation-only (Markdown outside `crates/`, `packages/`, `fixtures/`, `examples/`, `xtask/` and `scripts/`, or anything under `docs/evidence/` and `docs/reviews/`) skips the heavy job; the skipped job still reports its unchanged required check as successful. Tool-read documentation such as `docs/*.json` and crate/npm READMEs always qualifies, as do pushes to `main`, manual runs and a failed scope job. A skipped check is not qualification evidence: acceptance records must cite runs that actually executed.

## CI triggers and retention

The [CPU](../.github/workflows/ci.yml), [GPU](../.github/workflows/gpu-smoke.yml), [browser package](../.github/workflows/browser-runtime.yml) and [browser material](../.github/workflows/browser-materials.yml) workflows all run on pull requests, pushes to `main`, and manual dispatch. A normal push to a feature branch does not also start a branch-push run. A merged change still runs on `main`, verifying the integrated state. Existing per-workflow/ref concurrency cancels superseded runs without cancelling unrelated branches or PRs.

The GPU job retains serial test execution and the pinned SwiftShader build cache. A cache hit still verifies the source revision, configures/builds the driver, and executes every acceptance gate. Cache state is not proof of a passing test.

Every workflow restores a pinned `Swatinem/rust-cache` Cargo build cache after installing the repository toolchain. Only `main` saves it; pull requests restore. It speeds up compilation only: every step still builds with `--locked` and executes its gates, and the cached `~/.cargo/bin` lets the exact `wasm-bindgen-cli` install be reused.

New uploaded CPU, GPU and browser evidence artifacts request 30 days of retention. Record the service-reported expiry for accepted runs; repository or service limits may shorten availability. This setting does not change existing artifacts retroactively. Ordinary run output stays in CI artifacts or ignored local directories. Accepted visual content, critical failure evidence, and summaries follow the [evidence retention policy](./evidence-policy.md); do not rely on expiring artifacts as the only long-term acceptance record.

## ALPHA-05 browser enforcement — 2026-09-20

[PR #22](https://github.com/OpenMixture/OpenMixture/pull/22) adds the two existing browser jobs to active ruleset `23016046`. [Retained API evidence](./evidence/alpha-05/README.md) includes the prior policy, exact update, live ruleset, effective main rules and six required checks on the PR. All other protection settings are preserved. The classic protection summary from the branch endpoint can show empty contexts even when rulesets enforce checks; verify the ruleset and effective rules instead.

```bash
gh api repos/OpenMixture/OpenMixture/rulesets/23016046
gh api repos/OpenMixture/OpenMixture/rules/branches/main
gh pr checks 22 --repo OpenMixture/OpenMixture --required
```

The four-check M4.1 record below remains historical. Browser CI enforcement does not publish or independently certify the frozen ALPHA-04 archive.

## M4.1 activation and verification

**Activated and verified on 2026-09-12:** default `main`, active ruleset `23016046`, and [PR #1](https://github.com/OpenMixture/OpenMixture/pull/1) merged as `cc98dc9298add5ed172e0c1752d8a768b61a0eb5` with all four required checks passing before and after merge. The [completion record](./evidence/m4-1/README.md) retains the exact revisions, runs, applied policy and evidence limits. The sequence below documents the activation procedure; use the live API to verify subsequent state.

The bootstrap source is `99704e8c6a05e9e0b60e4264aa2a5901fbb391c6`, with passing [CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/34626312709) and [GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/34626312588) runs. Creating `main` from this existing commit preserves the entire accepted history. The older default branch is not used to manufacture a retrospective PR.

Complete activation in this order:

A draft PR may be prepared while bootstrap CI is running. Mark it ready and merge only after default-branch and protection verification below; drafting is not an exception to the merge gates.

1. Recheck the remote refs and source SHA. Fast-forward the local `main` if it is an ancestor, create the missing remote `main` without force, and retain the old branches.
2. Wait for all four checks to pass on `main` at that source revision; then make `main` the default branch.
3. Inspect existing rulesets, apply or update the matching desired ruleset, and read it back. Avoid duplicate rulesets.
4. Submit the workflow and documentation changes as a real PR. Verify the required checks on its current revision and merge through the normal protected path. Verify the resulting `main` runs too.
5. Retain a compact completion receipt identifying the bootstrap/main/PR revisions, run results, ruleset ID/content, default branch, and any checkpoint tag. Until those live checks are recorded, the local configuration is not proof of completed activation.

Read-only verification uses the existing authenticated `gh` account:

```bash
gh repo view OpenMixture/OpenMixture --json defaultBranchRef
gh api repos/OpenMixture/OpenMixture/branches/main
gh api repos/OpenMixture/OpenMixture/rulesets
gh run list --repo OpenMixture/OpenMixture --branch main
gh pr list --repo OpenMixture/OpenMixture --state all
```

For a selected PR, also inspect its current head, merge status, required checks, and the merged commit before claiming completion. Preserve exact run attempts and revisions; a passing earlier revision does not certify a later edit.

A descriptive M4 checkpoint tag may identify the accepted bootstrap commit. It is a source checkpoint, not a package version or release publication; never move an existing tag to different content. The packages remain unpublished with `publish = false`. M5 entry, additional nodes, hardware support promises, and distribution remain separate decisions.
