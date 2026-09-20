# Browser Runtime Alpha delivery closeout

English | [简体中文](./browser-alpha.zh-CN.md)

## Current direction — 2026-09-20

Browser runtime and Studio material comparisons use one frozen [v2 quality profile](./browser-quality.md). Current reports contain only current acceptance checks. The native software goldens and exact checker contract are unchanged. PR #19 integrated the gate redesign; the follow-up removes old sparse-pixel execution and unadopted research from the active workflow.

Native M4/M4.1, bounded M5 and the recorded Studio MVP are complete. Packages remain unpublished. M6 and new nodes are not the next step.

## Engine-owned work

| Item | State | Completion / next action |
|---|---|---|
| ALPHA-01 — Current candidate qualification | Implemented and exercised by PR #19 CI | Every delivery candidate must independently install its exact archive and pass build identity, public contracts, materials, lifecycle and deployment. A prior candidate does not certify a new package. |
| ALPHA-02 — Status alignment | Updated | This page owns current work; historical run records describe their own sources and environments. |
| ALPHA-03 — Ordinary desktop browser qualification | Complete for the recorded Windows 11 / GT 1030 scope | [Chrome, Edge and Firefox each passed 11 cases / 44 channels under v2](./evidence/browser-quality-v2/README.md), with source/adapter identity and lifecycle evidence. Windows used the recorded existing archive; fresh PR packages are separately tested in Linux CI. This is not universal hardware support. |
| ALPHA-04 — Publishable npm Alpha candidate | Exact candidate and recorded Studio upgrade gates passed | [Candidate release notes](./browser-alpha-candidate.md) fix the archive and support scope; [new execution evidence](./evidence/alpha-04/README.md) records 7 cases / 28 channels and 52 contracts on Windows and isolated Linux, plus ordinary Chrome save/Player/export. Registry publication and exact registry-version consumption remain separate actions. |
| ALPHA-05 — Browser required checks | Active and verified | [Live enforcement evidence](./evidence/alpha-05/README.md) records both browser checks alongside the four native checks in active ruleset 23016046, effective main rules and PR required-check readback. Strict base synchronization, GitHub Actions source binding and no bypass actors are preserved. |

## Product handoff

OpenMixture owns the runtime candidate, declarations, diagnostics and package identity. Studio owns open/edit/save/Player reopen/export behavior, including useful CPU editing when GPU acquisition fails. Joint upgrade evidence must bind both revisions and the same archive; changing the comparator alone is not a new Studio execution.

## Research closeout

PRs #13–#18 are closed as unadopted research. Their experiments do not change the shipped shaders and are no longer release prerequisites. No compiler strategy, FMA rewrite or new baseline is needed merely to align last bits across browsers. New numerical work needs a concrete failed current contract or consumer defect; it starts as a focused task with independent expectations.

Git/PR history retains the experiments. Current navigation and commands do not depend on them. See the [quality contract](./browser-quality.md), [Studio comparison](./studio-qualification.md), [package qualification](./browser-materials.md), and [governance](./governance.md).

[Cleanup verification](./evidence/quality-closeout/README.md) records exact browser replay and Studio migration checks.
