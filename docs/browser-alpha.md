# Browser Runtime Alpha delivery closeout

English | [简体中文](./browser-alpha.zh-CN.md)

## Current direction — 2026-09-20

Browser runtime and Studio material comparisons use one frozen [v2 quality profile](./browser-quality.md). Current reports contain only current acceptance checks. The native software goldens and exact checker contract are unchanged. PR #19 integrated the gate redesign; the follow-up removes old sparse-pixel execution and unadopted research from the active workflow.

Native M4/M4.1, bounded M5 and the recorded Studio MVP are complete. The [npm Alpha and exact registry consumer](./evidence/npm-alpha/README.md) are delivered; Rust crates remain unpublished. M6 and new nodes are not the next step.

## Engine-owned work

| Item | State | Completion / next action |
|---|---|---|
| ALPHA-01 — Current candidate qualification | Implemented and exercised by PR #19 CI | Every delivery candidate must independently install its exact archive and pass build identity, public contracts, materials, lifecycle and deployment. A prior candidate does not certify a new package. |
| ALPHA-02 — Status alignment | Updated | This page owns current work; historical run records describe their own sources and environments. |
| ALPHA-03 — Ordinary desktop browser qualification | Complete for the recorded Windows 11 / GT 1030 scope | [Chrome, Edge and Firefox each passed 11 cases / 44 channels under v2](./evidence/browser-quality-v2/README.md), with source/adapter identity and lifecycle evidence. Windows used the recorded existing archive; fresh PR packages are separately tested in Linux CI. This is not universal hardware support. |
| ALPHA-04 — Publishable npm Alpha candidate | Exact candidate and recorded Studio upgrade gates passed | [Candidate release notes](./browser-alpha-candidate.md) fix the archive and support scope; [new execution evidence](./evidence/alpha-04/README.md) records 7 cases / 28 channels and 52 contracts on Windows and isolated Linux, plus ordinary Chrome save/Player/export. [Subsequent publication and registry consumption](./evidence/npm-alpha/README.md) passed against these same bytes. |
| ALPHA-05 — Browser required checks | Active and verified | [Live enforcement evidence](./evidence/alpha-05/README.md) records both browser checks alongside the four native checks in active ruleset 23016046, effective main rules and PR required-check readback. Strict base synchronization, GitHub Actions source binding and no bypass actors are preserved. |

## Review reconciliation and ownership — 2026-09-20

The supplied review used engine `f824cbf` and Studio `dcb2be7`. Engine main subsequently integrated [PR #23](https://github.com/OpenMixture/OpenMixture/pull/23) as `458d928`. The [publication record](./evidence/npm-alpha/README.md) closes publication authorization, exact archive publication and clean registry consumption. This planning update uses retained evidence; it does not claim a fresh registry query, hosted deployment inspection or GPU run.

| Priority / item | Owner and state | Acceptance / next action |
|---|---|---|
| P1 / ALPHA-06 — Exact npm delivery | Engine; complete in PR #23 | Published `0.1.0-alpha.0` retains the ALPHA-04 digest and build identity in the candidate notes. Authorization and registry receipts are retained. Do not rebuild or republish this identity. |
| P1 / ALPHA-07 — Registry consumer handoff | Joint; recorded execution complete | Studio execution `87ded9351e1c426e03aa7fb2b4c641f32399b85b` binds exact version, lock integrity, installed files and `getBuildInfo()`. Windows and isolated Linux each pass 52 contracts and 28 channel comparisons, plus recorded ordinary Chrome product gates. This does not certify the hosted trial. |
| P2 / ALPHA-08 — Support and precision scope | Engine; documented in this change | [Candidate notes](./browser-alpha-candidate.md) distinguish historical three-browser coverage, published archive coverage and untested environments. Keep the warp precision limitation visible; do not mark it fixed. |
| On a concrete failure / ALPHA-09 — Consumer defect response | Engine; conditional, no implementation scheduled | Require package/build identity, `.mix`, request, browser/adapter and first structured error or failed current gate. Reproduce the smallest case, add focused independent expectations and qualify any changed archive before handoff. |

Studio owns the next delivery sequence: confirm registry dependency integration, update the trial's allowed runtime identity and scope, build `trial.json`, deploy, verify online `getBuildInfo()` and ordinary-browser workflows, then observe 3–5 non-developer desktop users. It also owns correcting `docs/external-trial.md` to distinguish historical PR #12 failure, accepted main and the hosted version. The review identified the old digest guard in `scripts/prepare-trial.mjs`; update it with the qualified identity and scope rather than deleting it. These are handoff requirements, not completed work in this repository. Registry acceptance proves neither redeployment nor human trial results.

Future engine releases advance the version when content changes, freeze a new archive identity, repeat qualification, then publish and verify clean exact-version registry consumption. Changed versions or bytes cannot inherit this release's acceptance. Preserve the recorded `alpha`/`latest` dist-tag caveat and use exact versions; Alpha is not a stable release.

Do not restart completed M4/M5 acceptance, browser required-check setup, Studio MVP or closed research merges. Preserve existing checks; extend tooling only for a demonstrated current failure. M6, new nodes, subgraphs, full 3D preview, GPU zero-copy, another backend, generalized optimization, marketplace and collaboration remain unscheduled. Small product improvements follow observed user obstacles.

## Product handoff

OpenMixture owns the runtime candidate, declarations, diagnostics and package identity. Studio owns open/edit/save/Player reopen/export behavior, including useful CPU editing when GPU acquisition fails. Joint upgrade evidence must bind both revisions and the same archive; changing the comparator alone is not a new Studio execution.

## Research closeout

PRs #13–#18 are closed as unadopted research. Their experiments do not change the shipped shaders and are no longer release prerequisites. No compiler strategy, FMA rewrite or new baseline is needed merely to align last bits across browsers. New numerical work needs a concrete failed current contract or consumer defect; it starts as a focused task with independent expectations.

Git/PR history retains the experiments. Current navigation and commands do not depend on them. See the [quality contract](./browser-quality.md), [Studio comparison](./studio-qualification.md), [package qualification](./browser-materials.md), and [governance](./governance.md).

[Cleanup verification](./evidence/quality-closeout/README.md) records exact browser replay and Studio migration checks.
