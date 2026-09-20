# ADR 0006: Bounded browser texture agreement

English | [简体中文](./0006-browser-quality-gates.zh-CN.md)

Status: Implementation decision for the user-authorized browser-gate redesign, 2026-09-20; environment qualification requires a separate result. This supersedes the use of M5-05's sparse changed-pixel limit for **new browser runtime material comparisons**, not the historical calibration or accepted results. Native software goldens, Studio saved-file comparison and node semantics remain unchanged. [ADR 0005's proposal](https://github.com/OpenMixture/OpenMixture/blob/cf4dbfcde63991d38511a34c2470f46c1b11de69/docs/decisions/0005-warp-numerical-compatibility.md) is a separate unaccepted shader-compatibility decision.

## Context

The old browser profile counts any changed component as a changed pixel and permits only extremely sparse differences. That detects numerical differences but conflates their prevalence with material damage. A widespread balanced one-code residual can fail while every pixel remains within the same amplitude bound. We need to preserve sensitivity to drift and local defects without making near-byte identity a general browser requirement.

## Decision

Use the versioned [v2 profile](../browser-quality-v2.json) and its [measurement contract](../browser-quality.md). Keep the maximum RGB difference at one encoded RGBA8 unit; remove changed-pixel ratio and whole-image mean as acceptance conditions. Measure coherent signed bias in overlapping periodic windows. Add channel-specific normal direction/response, height gradient and roughness response checks. Retain the original metrics and verdict under `legacyComparison`; never rewrite a historical failure as a historical pass.

Plans, provenance, encoding, all eleven cases/four channels, existing material structure/seam/causality/relationship checks, lifecycle, installation and production deployment remain mandatory. The new profile is an engineering agreement budget, not a universal perceptual threshold, exact floating-point contract, or license to accept all browser differences. Native is a fixed same-source comparison reference, not a mathematical oracle. A disputed result needs independent scalar diagnosis; changing references to get a pass is prohibited.

Freeze the profile and implementation after independent synthetic perturbation controls and before applying them to browser outputs. No browser-result fitting is allowed. A later profile revision requires a new identifier, rationale, controls and freeze, with all earlier results retained.

## Alternatives

Universal byte equality exceeds the product promise. Simply widening the changed-pixel count still misclassifies spatially coherent drift. Global error averages can hide a local defect. Perceptual-only image scores can miss height/normal errors relevant to downstream use. Browser-specific references can conceal disagreement. The selected rule combines a strict amplitude cap with spatial and output-use measurements; it does not claim to detect every possible imperceptible or harmful change.

## Consequences and migration

The comparator report becomes schema 2, binds the full profile and digest, and separates semantic, structural, numerical and legacy regression results. Current qualification consumers reject older reports and missing/failed current gates. Measurement mode cannot certify a package. Old report files remain valid historical records, but cannot be substituted for a fresh qualification. No `.mix`, node, RenderPlan, runtime API or package version changes.

Cached historical PNGs may be re-evaluated only in a fresh evidence directory, labelled retrospective analysis with original source/environment identities. This is not a new browser execution. New package qualification still requires independent installation and actual browser execution. A change in policy is not acceptance of PR #15/#16 or a new golden.

## Verification

`cargo xtask browser-quality-calibrate <fresh-output>` produces 40 deterministic positive/negative controls and reviewable contact sheets without browser inputs. Focused Rust tests independently check geometry, response equations, local bias and encoding rejection; JavaScript tests reject stale/missing policy evidence. Run `cargo xtask check`, current-candidate browser CI, and post-freeze comparisons of retained ordinary-browser outputs. Record failures as failures. Inspect real material comparisons and document limits before claiming support under v2.

The stopping condition is all required v2 gates passing for the explicitly recorded use scope and no unexplained structural defect. More visible structure, larger errors, coherent drift or failed response checks require diagnosis. Passing this budget does not certify arbitrary world-space displacement scales or every downstream BRDF/light.
