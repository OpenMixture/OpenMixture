# ALPHA-03 numerical investigation — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**Historical investigation before reference configuration was corrected.** This supplements the [initial failure record](../alpha-03/README.md), without changing its results. [PR #12](https://github.com/OpenMixture/OpenMixture/pull/12) proposes explicit half-storage rounding and extends ordinary-browser testing to Firefox. No tolerance or existing golden was changed; at this stage no ordinary configuration passed all 44 channel gates. A smaller largest error ratio does not override a failed gate.

## Findings and disposition

The original source is clean engine `7b1cec4ad1d42d6269ef6a9912c2e8ba3a2dfdd9` and its retained Alpha archive. On Windows 11 build 26100 / NVIDIA GT 1030, [node isolation](./node-isolation.json) found 740 changed wood base-color pixels; 729 shared the same red-byte transition, 133 → 132. The [constant input probe](./implicit-storage-probe.json) established that values `0.23265` and `0.2326660007238388` exported 132, while `0.2326660305261612` exported 133, in both native DX12 and ordinary Chrome. [Exactly representable half inputs](./exact-half-transfer-probe.json) agreed between the two. This identifies a storage-rounding sensitivity; it does not prove the cause of every remaining arithmetic difference.

The attempted `quantizeToF16` built-in still produced the old boundary result on this host. The retained fix uses integer operations in the sole WGSL path to round to nearest binary16, ties to even, before texture storage. The zero-tolerance constant-color fixture now passes. A GPU probe checks 122,884 signed exact values, midpoints and neighboring f32 values across all binary16 intervals in [-1,1], against the independent `half` crate conversion. Existing node fixtures passed on DX12 before and after the change.

This fix was built from clean `34a0db5a6b93fa58acb2c5aae2f803c5dde2023b`. The [candidate directory](./rounding-candidate/) retains its archive, producer/consumer identity and native manifest. Archive SHA-256 is `52c21ab756c40ca19c353f91b929729251de3e560fc50559e80e9f89d515106b`; consumer remains Studio `56c510ab57daa1b68ef660525a648a582730a37e`. The [pinned native GPU CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35176953039) passed existing node/material goldens and 2K checks, and [controlled browser CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35176953041) passed its full matrix. CI built its own revision-bound package; it is not proof that the separately built Windows archive passed every ALPHA-01 consumer gate.

Explicit `fma` interpolation was separately tried at `c9226bf10eff5b03c3396af1fe9d427f1d5e8e48`. It failed the unchanged wood golden in [CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35177471107); [failure output](./rejected-interpolation-ci.log) and its ordinary-browser comparison are retained. It was reverted by `dfcf106`; it is not part of the proposed runtime fix. Native Vulkan reference selection, native shader optimization and a DXC experiment also did not close the gate. Those were diagnostic experiments, not replacement acceptance references.

## Recorded ordinary-browser results

All rows use same-source native DX12 references, 11 existing 1K cases, four channels, and unchanged frozen gates. Complete comparison and runtime/failure receipts are retained in each directory.

| Candidate / browser | Pixel channels passing | Largest changed-pixel ratio | Result |
|---|---:|---:|---|
| Explicit rounding / Chrome 153.0.8010.48 | 31/44 | 0.0000438690185546875 | [Failed](./rounding-chrome/comparison.json) |
| Rejected interpolation / Chrome 153.0.8010.48 | 30/44 | 0.0000438690185546875 | [Failed; reverted](./rejected-interpolation/comparison.json) |
| Original candidate / Firefox 156.0 | 32/44 | 0.0011339187622070312 | [Failed](./firefox-original/comparison.json) |
| Explicit rounding / Firefox 156.0 | 32/44 | 0.0010519027709960938 | [Failed](./rounding-firefox/comparison.json) |

All largest byte errors are 1. Firefox passes every ceramic and wood channel; its failures are the leather base-color, height and normal channels. Full load/render/owned-output/destruction and the three separately injected diagnostic cases completed; full material acceptance still failed. The browser identity changed from the initial Chrome `.37` to `.48` through an installed-browser update; the receipts record actual versions, rather than treating them as the same environment.

Firefox is Mozilla's signed stable Windows MSIX, extracted into ignored workspace storage, with a fresh profile and no GPU preference changes. Its runtime adapter fields are redacted; `isFallbackAdapter` is false. Host NVIDIA inventory must not be substituted for the redacted selected-adapter identity. WebDriver BiDi capabilities bind version, non-headless state, profile and actual browser PID to the observed OS launcher/child commands. The separate [launcher check](./firefox-launcher.json) verifies the repository launcher. Initial attempts rejected a launcher/child PID mismatch and an observer error in the insecure `about:blank` realm; those are verifier failures, not successful acceptance. The observer now tolerates absent WebGPU before navigation, while the actual secure application still must initialize successfully.

## Reproduction and retention

Use the [ordinary-browser guide](../../default-browser.md). The initial and rounding archives are retained; the rejected interpolation archive is temporary and cannot be reconstructed byte-for-byte from its digest after expiry. Git also retains full failed comparisons/receipts, representative native/browser/difference contact sheets, the numerical probe results, and the rejected CI error. Other per-case PNGs, profile directories, diagnostic scripts, full logs and build trees are temporary under `tmp/alpha03-*`. The Firefox executable is not redistributed in Git; the launch receipt records distribution and executable hashes. The final launcher check used the same signed executable. No external durable archive or full-run permanent auditability is claimed.

These are development investigations, not accepted visual changes. The runtime archive/source identities are exact. Some verifier edits were uncommitted during the diagnostic runs and their exact historical verifier hash was not captured; the PR retains the resulting verifier implementation, not a claim that every historical run used that final implementation. Future runs record verifier/transport hashes at start. Preserve this audit limit when using the reports.

The user chose to keep the full material gate and correct the environment. The subsequent [configured-reference record](../alpha-03-configured/README.md) documents the passing Firefox run. These earlier failures remain unchanged; they do not certify ALPHA-04 release readiness. Publication, tolerance changes, golden replacement, a second executor and new nodes remain out of scope.
