# Browser Runtime Alpha delivery closeout

English | [简体中文](./browser-alpha.zh-CN.md)

## First-delivery closeout — 2026-09-20

Browser runtime and Studio material comparisons use one frozen [v2 quality profile](./browser-quality.md). Current reports contain only current acceptance checks. The native software goldens and exact checker contract are unchanged. PR #19 integrated the gate redesign; the follow-up removes old sparse-pixel execution and unadopted research from the active workflow.

Native M4/M4.1, bounded M5 and the recorded Studio MVP are complete. The [npm Alpha and exact registry consumer](./evidence/npm-alpha/README.md) are delivered; Rust crates remain unpublished. This page retains first-delivery scope and limitations. The [Post-Alpha roadmap](../ROADMAP.md) owns current priorities; ENG-01/02 do not implement M6 or new nodes.

## First-delivery work record

| Item | State | Completion / next action |
|---|---|---|
| ALPHA-01 — Current candidate qualification | Implemented and exercised by PR #19 CI | Every delivery candidate must independently install its exact archive and pass build identity, public contracts, materials, lifecycle and deployment. A prior candidate does not certify a new package. |
| ALPHA-02 — Status alignment | Updated | This page retains first-delivery status; the roadmap owns current work. Historical run records describe their own sources and environments. |
| ALPHA-03 — Ordinary desktop browser qualification | Complete for the recorded Windows 11 / GT 1030 scope | [Chrome, Edge and Firefox each passed 11 cases / 44 channels under v2](./evidence/browser-quality-v2/README.md), with source/adapter identity and lifecycle evidence. Windows used the recorded existing archive; fresh PR packages are separately tested in Linux CI. This is not universal hardware support. |
| ALPHA-04 — Publishable npm Alpha candidate | Exact candidate and recorded Studio upgrade gates passed | [Candidate release notes](./browser-alpha-candidate.md) fix the archive and support scope; [new execution evidence](./evidence/alpha-04/README.md) records 7 cases / 28 channels and 52 contracts on Windows and isolated Linux, plus ordinary Chrome save/Player/export. [Subsequent publication and registry consumption](./evidence/npm-alpha/README.md) passed against these same bytes. |
| ALPHA-05 — Browser required checks | Active and verified | [Live enforcement evidence](./evidence/alpha-05/README.md) records both browser checks alongside the four native checks in active ruleset 23016046, effective main rules and PR required-check readback. Strict base synchronization, GitHub Actions source binding and no bypass actors are preserved. |

## Review reconciliation and ownership — 2026-09-20

The supplied review used engine `f824cbf` and Studio `dcb2be7`. Engine main subsequently integrated [PR #23](https://github.com/OpenMixture/OpenMixture/pull/23) as `458d928`. The [publication record](./evidence/npm-alpha/README.md) closes publication authorization, exact archive publication and clean registry consumption. This planning update uses retained evidence; it does not claim a fresh registry query, hosted deployment inspection or GPU run.

| Priority / item | Owner and state | Acceptance / next action |
|---|---|---|
| P1 / ALPHA-06 — Exact npm delivery | Engine; complete in PR #23 | Published `0.1.0-alpha.0` retains the ALPHA-04 digest and build identity in the candidate notes. Authorization and registry receipts are retained. Do not rebuild or republish this identity. |
| P1 / ALPHA-07 — Registry consumer evidence | Historical recorded execution complete | Studio execution `87ded9351e1c426e03aa7fb2b4c641f32399b85b` binds exact version, lock integrity, installed files and `getBuildInfo()`. Windows and isolated Linux each pass 52 contracts and 28 channel comparisons, plus recorded ordinary Chrome product gates. This does not certify the hosted trial or assign future Studio work to the engine. |
| P2 / ALPHA-08 — Support and precision scope | Engine; documented in this change | [Candidate notes](./browser-alpha-candidate.md) distinguish historical three-browser coverage, published archive coverage and untested environments. Keep the warp precision limitation visible; do not mark it fixed. |
| On an upstream issue / ALPHA-09 — Consumer defect triage | Engine; issue-triggered, no implementation scheduled | Studio submits an OpenMixture issue with exact identity, minimal input/request, environment, expected/actual behavior and failure evidence. Triage ownership, reproduce engine defects here, add focused regression coverage and deliver a qualified fix/version through the issue. Studio owns its upgrade and product acceptance. |

Studio dependency upgrades, product acceptance, deployment, trial documentation and user feedback belong to Studio's own plan. They are not engine backlog items or execution instructions. Registry acceptance proves neither redeployment nor human trial results. Cross-project dependencies are communicated through upstream issues under the [agent task boundary](../AGENTS.md); shared evidence does not authorize taking over Studio work.

Future engine releases advance the version when content changes, freeze a new archive identity, repeat qualification, then publish and verify clean exact-version registry consumption. Changed versions or bytes cannot inherit this release's acceptance. Preserve the recorded `alpha`/`latest` dist-tag caveat and use exact versions; Alpha is not a stable release.

Completed M4/M5 and first-Alpha records remain historical acceptance; new candidates still require current qualification. Preserve existing required checks and extend verification for approved engine use cases as well as defects. ENG-03/04 are planned in the roadmap, not implemented by this closeout update. M6, subgraphs, full 3D preview, GPU zero-copy, generalized optimization, marketplace and collaboration remain unscheduled; a second pixel backend remains forbidden. Product improvements belong to their product plan.

## Product handoff

OpenMixture owns the runtime candidate, declarations, diagnostics, package identity and producer-side qualification. Studio owns open/edit/save/Player reopen/export behavior, including useful CPU editing when GPU acquisition fails, and independently verifies its upgrades. Evidence spanning both projects must bind their revisions and the same archive; changing the comparator alone is not a new Studio execution. Existing engine CI may exercise its pinned disposable consumer host without modifying Studio or managing product delivery.

Engine work may originate from approved milestones, maintainer-defined use cases, measurements, regressions or external issues. OpenMixture determines scope and acceptance independently; Studio delivery is not a prerequisite. ALPHA-09 remains the external defect-triage route, not the sole source of work. An issue or this closeout record does not automatically authorize unrelated scope.

## Research closeout

PRs #13–#18 are closed as unadopted research. Their experiments do not change the shipped shaders and are no longer release prerequisites. No compiler strategy, FMA rewrite or new baseline is needed merely to align last bits across browsers. New numerical work needs an approved engine use case, measured limitation or concrete contract failure, with independent expectations and an explicit semantic version/compatibility decision where behavior changes.

Git/PR history retains the experiments. Current navigation and commands do not depend on them. See the [quality contract](./browser-quality.md), [Studio comparison](./studio-qualification.md), [package qualification](./browser-materials.md), and [governance](./governance.md).

[Cleanup verification](./evidence/quality-closeout/README.md) records exact browser replay and Studio migration checks.
