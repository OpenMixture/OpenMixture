# Evidence retention policy

English | [简体中文](./evidence-policy.zh-CN.md)

This policy governs evidence added from M4.1 onward. It keeps acceptance reviewable while avoiding a new copy of every successful run in Git. It does not rewrite historical execution outcomes, alter material gates, or add a runtime or verification framework. [Release status](./release.md), [compatibility](./compatibility.md), and the [native SDK contract](./native-sdk.md) remain the sources for product readiness and public behavior.

## Historical records and current guidance

Keep executable golden pixels/manifests and the source-bound content that supports current acceptance. Superseded research, rejected experiments and duplicate diagnostics may be removed from the current tree once the maintainer chooses a direction; Git history and closed PRs retain their original outcomes. Remove obsolete navigation and executable dependencies at the same time. Link a specific historical commit when a retained record needs an old file. Do not rewrite a failed result as a successful run, alter a bound receipt, or delete inputs still required by current verification. The original `mixture-greenfield-docs/` source bundle is independent reference material.

Current navigation and explanatory guides may be updated in both languages to point to a newer accepted revision and explain which older statements are historical. A correction to a historical finding belongs in a new dated record that links to the original; it must not replace the original result or claim the old run tested new sources. See the [remote CI record](./evidence/remote-ci/README.md) for separate failed and accepted runs.

## What new evidence retains

Classify evidence by its role, not just by file extension or size.

| Evidence | Retention for new work |
|---|---|
| Executable acceptance inputs | Keep source `.mix` files, acceptance schema/contracts, parameter variants, expected PNGs and manifests, focused fixtures, and reproduction scripts/commands in Git. They remain usable from a checkout. |
| Human acceptance and visual decisions | Keep the decision receipt and its bound comparison images/reports in Git, including the relevant before/after/difference evidence. A small summary or hash cannot substitute for the image that was accepted. Preserve failures used to justify a tolerance or semantic decision. |
| Accepted milestone or material results | Keep a concise, source-bound summary in Git with the critical machine measurements, gate results, limitations, and human decision references. Retain the actual supporting content needed to inspect those conclusions. |
| Ordinary repeated runs | Retain complete stdout/stderr, repeated JSON reports, unselected candidates, duplicate PNGs, and large routine traces in ignored local output directories or finite-retention CI artifacts. Do not copy every successful run into Git. Promote the necessary subset if it becomes evidence for an acceptance or failure decision. |
| Build and staging data | Keep compiler products and SwiftShader builds in ignored directories or caches. Successful owned package extraction trees may be removed by the verifier; preserve its receipts and required archive evidence before cleanup. Caches are not acceptance evidence. |

For a newly accepted visual change, all files referenced by its human receipt must remain available and bound to the accepted bytes. For ordinary runs whose full outputs are intentionally temporary, say so in the summary. Reproduction instructions can generate a new run; they do not recover or authenticate an unavailable original artifact.

## Minimum accepted-run summary

Use a small structured record and a paired human explanation where needed. This is a retention requirement, not a newly implemented report schema or command. Existing report formats remain authoritative; identify their schema versions, and mark non-applicable or unavailable fields explicitly instead of inventing values.

- **Source:** full tested commit SHA, working-tree cleanliness, and relevant source/input hashes when uncommitted work is included; toolchain, lockfile, shader, fixture, acceptance-contract, and baseline identities needed to reproduce the result. A later documentation commit must not be presented as the tested commit.
- **Run:** timestamp and timezone, local or CI origin, run ID and attempt where available, workflow/job identity, commands and relevant options/environment, and each required gate's completion/result. Keep failures and skipped or incomplete work distinguishable from success.
- **Execution:** host OS/architecture, requested adapter policy, actual adapter/backend, verified software source revision where applicable, resolution, channels, variants/overrides, plan hashes, and the decisive comparison/resource measurements. Do not infer hardware GPU coverage from CPU checks.
- **Acceptance:** result and scope, known limitations, selected failure evidence, and references/digests for the human decision and reviewed content where required. Machine checks and `golden update --accept` do not assert human review.
- **Artifacts:** relative paths or retrieval locations, archive/artifact name and ID, content digest and size, creation/expiry information when available, and the supporting file identities needed to verify an extracted bundle. Package acceptance also retains archive/source/lock identity and the outer verifier's provenance and consumption results.
- **Retention:** which contents remain in Git, which are temporary, any durable archive location and responsible maintainer, and the last successful retrieval/integrity check when durable retention is claimed. State unavailable original content and resulting audit limits explicitly.

A plan hash identifies normalized semantics; it does not identify original source spelling, implementation/shader bytes, GPU pixels, package contents, or request freshness. A commit SHA alone does not identify a dirty working tree. An artifact digest verifies retrieved bytes; it does not keep those bytes available. Preserve these distinctions from the [compatibility record](./compatibility.md) and [package verification method](./package-consumption.md).

## CI expiry and durable retention

The [CPU workflow](../.github/workflows/ci.yml) and [GPU workflow](../.github/workflows/gpu-smoke.yml) retain their generated evidence through CI artifacts. Those artifacts have finite retention. Record the configured retention and the reported expiry for an accepted run where available; do not promise permanent download from a run URL. Source and driver caches may disappear independently.

The maintainer recording an acceptance owns its retention decision. Before relying on an external durable archive, that maintainer must select an accessible location with a documented retention owner/policy, store the required bundle, retrieve it, verify its digest and required contents, and record that check. Retain the original run/source identity when copying a bundle. A durable copy is separate from a new execution or a release publication.

When no usable durable archive is available, keep the minimum necessary acceptance content in Git and state which complete logs or incidental outputs will expire. If the intended audit requires the original full bundle, retain that bundle or narrow the claim; hashes and run IDs alone cannot support a claim of long-term full-run auditability. Expiry does not turn a previously recorded pass into a failure, but it limits later inspection of the original run. Record that limitation honestly.

No external archive service or automatic preservation job is introduced by this policy. Superseded research may be pruned under the current-guidance rules above.

## Preserve verification and ownership

The [golden workflow](./material-goldens.md) continues to separate rendering, visual inspection, guarded baseline installation, and human acceptance. Candidate integrity covers runtime/shader/tooling sources, inputs, previous baselines, and reviewed artifacts. In particular, [candidate input hashing](../xtask/src/golden/files.rs) includes `xtask/src`: a tooling edit can invalidate an unaccepted candidate even when pixels are unchanged. Rerun and review it; never rewrite its hashes to bypass that protection.

Local documentation links must still resolve, and human acceptance bindings must still identify available content. The [link checker](../xtask/src/links.rs) excludes external URLs, so a passing link check does not establish artifact availability. Preserve source fixtures and the independent consumer inputs required by [isolated package verification](./package-consumption.md); historical evidence is not a package runtime dependency.

Retention changes must not weaken failure propagation, source/archive checks, material comparisons, or independent consumer checks. File size alone is not a reason to split `xtask`, add a crate, or introduce a general evidence framework. Any future tooling change follows the owning command's focused checks and `cargo xtask check` under the [development workflow](./development.md).

## Describe verification scope accurately

These categories describe evidence coverage, not new support tiers or service guarantees. Exact accepted revisions, hosts, results, and limitations remain in [release status](./release.md) and its linked records.

| Category | Current evidence boundary |
|---|---|
| Ongoing CI verification | Linux/macOS/Windows CPU checks and the configured Linux pinned SwiftShader Vulkan GPU/material/package/trace workload. A workflow's presence is not a pass for an untested revision. |
| Recorded development-host verification | The recorded Apple M5/Metal and local pinned SwiftShader runs on macOS. Include the host, date, source, and workload; do not promote one host to a general Metal or Tier 1 promise. |
| Native hardware without recorded acceptance | Other Metal hardware, hardware Vulkan, and Windows/DX12 configurations outside the recorded matrix. Exposed backend options do not certify these devices. |
| Browser work not started | Browser WebGPU has no completed implementation or accepted browser matrix in M4.1. A proposed browser CI environment is not current evidence. |

This policy does not expand platform support, start M5, publish packages, change goldens, or reopen accepted historical decisions.
