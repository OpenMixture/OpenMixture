# ALPHA-03 ordinary desktop qualification — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**Result: blocked, not accepted.** Both ordinary browsers complete the runtime lifecycle and diagnostic probes, but neither passes the frozen full material comparison. ALPHA-03 and the dependent ALPHA-04 release gate remain open. No tolerance, shader, native golden or runtime implementation was changed. No general Windows/NVIDIA support claim follows from this run.

Delivery: [draft PR #12](https://github.com/OpenMixture/OpenMixture/pull/12). The 11 focused Node tests and full local `cargo xtask check` passed on 2026-09-17; the earlier check during document assembly failed on the then-missing record links and was rerun after those files existed. Remote PR CI was pending when this record was written. Local tooling checks do not convert the browser material failures into passes.

## Bound inputs and environment

- Runtime: clean engine `7b1cec4ad1d42d6269ef6a9912c2e8ba3a2dfdd9`, unpublished `0.1.0-alpha.0`; [producer receipt](./package-receipt.json), [retained archive](./openmixture-runtime-0.1.0-alpha.0.tgz). SHA-256: `88f22ac295c3a1cc6bee2e995ed1e4683ca6669026167e4731ee10564f30d48c`.
- Independent Studio: `56c510ab57daa1b68ef660525a648a582730a37e`; [candidate/lock identity](./candidate.json). The same archive passed [ALPHA-01 CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35112153338), with its [qualification receipt](./ci-qualification.json). That controlled CI pass does not certify these hardware runs.
- Windows 11 IoT Enterprise LTSC, build 10.0.26100, x64; NVIDIA GeForce GT 1030, driver 32.0.15.8266. Chrome 153.0.8010.37; Edge 153.0.4234.32. Fresh disposable desktop profiles, only profile/CDP/about:blank arguments; complete executable identities, observed commands and host/browser GPU reports are in each receipt. No GPU, blocklist, headless or sandbox override.
- Browser request: `powerPreference: high-performance`; observed returned adapter `nvidia`/`pascal`, not fallback. Runtime backend is BrowserWebGpu; its redacted adapter name remains redacted. Host inventory alone is not selected-adapter proof.
- Same-host native references: explicit NVIDIA GT 1030/DX12, software selection disabled; [manifest](./native-manifest.json). Generated at clean verifier commit `7c2206203661da2c7b42dd1d4d10b9b263028394`; runtime-source drift check against `7b1cec4…` passed. This is fresh comparison output, not a golden replacement.

## Results

All comparisons use the existing 11 cases, 1024×1024, four channels and frozen tolerances. A pass requires every gate; a low byte error is not acceptance.

| Run | Pixel channels passing | Largest byte error | Largest changed-pixel ratio | Decision |
|---|---:|---:|---:|---|
| Chrome against historical Linux SwiftShader native references | 12/44 | 2 | 0.21771526336669922 | Failed cross-adapter experiment |
| Chrome against same-host NVIDIA/DX12 | 33/44 | 1 | 0.0007572174072265625 | Failed |
| Edge against same-host NVIDIA/DX12 | 33/44 | 1 | 0.0007572174072265625 | Failed |

Both same-host browsers passed all 44 structure checks, actual archive/build identity, load/init/render, owned RGBA output, zero reported live allocations after readback, ownership after subsequent renders/destruction, idempotent destroy, accepted in-flight completion, and rejection after destroy. Separate synthetic pages passed absent-WebGPU, null-adapter and denied-device structured errors while CPU validation remained available. Synthetic failures do not establish natural unsupported-host coverage.

The normal production Chrome Player separately passed initialization, exact 65×3 checker PNG pixels and sRGB metadata, download, disposal and absence of the test harness: [receipt](./production/receipt.json), [PNG](./production/checker.png), [screen](./production/player.png). This limited pass does not override the material failures or certify Studio open/edit/save/reopen.

Inspect [Chrome comparison](./chrome/comparison.json), [Edge comparison](./edge/comparison.json), and the [cross-adapter comparison](./chrome-cross-adapter/comparison.json). Full runtime observations and negative diagnostics are retained beside each comparison as `receipt.json`; `ordinary.json` stays false and `failure.json` retains the first verifier failure. Representative [leather](./chrome/leather-default-comparison.png) and [wood](./chrome/wood-default-comparison.png) contact sheets retain native/browser/difference content. Visual inspection found no gross layout/content corruption in the leather sheet; this is not human golden acceptance. The numerical failure remains decisive. Compiler/driver arithmetic differences are a hypothesis, not an established root cause.

## Reproduction and retention

Follow the [ordinary-browser procedure](../../default-browser.md). Node 24.20.0 was used. Original local directories were `tmp/alpha03-run`, `tmp/alpha03-paired-run`, `tmp/alpha03-edge-run-3`, `tmp/alpha03-native-windows` and `tmp/alpha03-production`. Native and first Chrome/production runs used verifier commit `7c220620…`; later runs additionally observed the returned adapter, and Edge normalized trailing command whitespace. These tooling edits did not change runtime bytes. The PR diff records those changes; the runs do not certify a later runtime candidate.

Git retains the actual candidate archive, source-bearing native manifest, producer/consumer/CI receipts, complete three comparison reports and runtime failure records, representative failure contact sheets, and the production image/screen. All other per-case PNGs, full logs, browser profiles and build trees remain temporary local output. CI downloads also expire; the retained subset does not promise complete original full-run auditability. Archive integrity was checked during candidate installation and retention on 2026-09-17. No external durable archive is claimed.

Next gate: isolate the failing channels using source-matched node fixtures and determine whether a correct runtime fix or a separately agreed narrower target is appropriate. Any semantic fix requires a new candidate and rerunning ALPHA-01/03; do not reset baselines or enlarge tolerances to mark this run accepted. npm publication, Studio's full saved-file acceptance, other devices/browsers and governance changes are out of scope.
