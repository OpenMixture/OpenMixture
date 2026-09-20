# Browser quality v2 — independent controls

English | [简体中文](./README.zh-CN.md)

2026-09-20. This is the pre-browser freeze evidence for the [v2 engineering profile](../../browser-quality.md), not a browser support receipt. The controls were generated before applying v2 to any browser material output. Thresholds were chosen from quantization and response budgets, not browser pass rates.

[calibration.json](./controls/calibration.json) records all 40 predetermined positive/negative labels and measurements: all matched. The directory retains all source/perturbed/difference contact sheets; [control-sha256.json](./control-sha256.json) binds the exact bytes. The agent inspected representative normal balanced-noise, height local-bias and height erased-detail sheets; this is not human material approval. The suite consumes generated 64×64 arrays only and performs no `.mix` or GPU execution.

Reproduce from the repository root:

```sh
cargo test --locked -p xtask browser_quality
cargo xtask browser-quality-calibrate tmp/quality-controls-new
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/default-browser.test.mjs
```

The output directory must be fresh. The Rust tests additionally cover analytic normal/response values, scalar/alpha encoding, direction rotation, sharp-roughness sensitivity and comparison symmetry. One low-roughness single-code perturbation intentionally fails the response gate even though the amplitude/bias gate passes. This prevents interpreting v2 as unconditional acceptance of one-code differences.

## Post-freeze results, 2026-09-20

The profile and metric implementation were frozen in `c85972e23ad684d267af59c9c607d1306a9ffed0` before evaluation. [results.json](./results.json) binds the following **fresh ordinary-profile** executions to installed archive `08cdce838500343b34d775a62058d6adfd5c6035f109d86e03e5ac1ff11f1ec2`, engine `ec571816026a945a706067769fef76e24c8398b0`, and the same recorded native reference. This deliberately tests an existing package under new rules; it does not claim that the Windows executions installed PR #19's new archive. Host: Windows 11 / NVIDIA GT 1030; exact host/driver/adapter metadata is retained in each receipt.

| Browser | v2 channels | Original sparse-pixel channels | Semantics / structure / lifecycle |
|---|---:|---:|---|
| Chrome 153.0.8010.48 | 44/44 | 27/44 | Pass |
| Edge 153.0.4234.32 | 44/44 | 27/44 | Pass |
| Firefox 156.0 | 44/44 | 44/44 | Pass |

Chrome/Edge maximum component difference remains **1**, local signed bias reaches `0.01171875` (budget `0.25`), normal angle `0.5953195122435492°` (budget `1°`), height first-difference error `1/255` (budget `2/255`), and sampled roughness-response change `0.00875639734842093` (budget `0.05`). Firefox pixels are identical; its reported approximately `0.0000017°` normal angle is floating-point measurement roundoff, not a pixel difference. No threshold changed after these observations. Every old failure remains in `legacyComparison`.

[Chrome production](./production-chrome/receipt.json) and [Edge production](./production-edge/receipt.json) also pass exact 65×3 checker download, normal static deployment, absent test harness and disposal. Firefox production download was not run: that verifier remains Chromium-only. The agent inspected [ceramic](./visuals/ceramic-default.png), [leather](./visuals/leather-default.png), [maximum-detail leather](./visuals/leather-detail-max.png), [wood](./visuals/wood-default.png) and [horizontal wood](./visuals/wood-horizontal.png) comparison sheets: no visible structural change at the retained review scale. This is agent comparison review, not replacement human approval of a native golden or exhaustive perceptual validation.

All six CI checks passed on the freeze head. The independent [Linux Chromium qualification](./ci/qualification.json) consumes a **new** archive (`0dfa42bf849fd0cfc7f91eab730edb6f92b167f74b0552f01212d0dd241f16ed`) from actual PR merge checkout `3f0939d1dc925a484b483fc2c5fca9817b089332`. [Run 35485669447, attempt 1](https://github.com/OpenMixture/OpenMixture/actions/runs/35485669447) passes controls, installation/build identity, 28 browser contracts, 11 cases/44 channels, 12 stress renders and production deployment. Its controlled Chromium flags do not constitute an ordinary-profile claim. The [artifact record](./ci/artifact.json) identifies retrieval and expiry. Final documentation/launcher follow-up checks belong to their own PR head, not this experiment revision.

## Evidence integrity, failures and replay

[attempts.json](./attempts.json) preserves rejected historical-input and tooling attempts. Some old local Chrome/Edge PNGs had invalid signatures; an old PR #16 native PNG failed its manifest digest. Those sources were excluded rather than repaired or used to fit thresholds. Fresh runs replaced the missing evidence. The first Edge run failed before rendering because Edge spawned a direct child with `--edge-skip-compat-layer-relaunch`. The launcher now records the actual OS parent/child/executable and the verifier binds CDP's browser PID and exact command line. The switch is never supplied by our launcher; arbitrary extra flags remain rejected. If the launcher exits before observation, that fact is explicit rather than a fabricated observed command. Focused tests cover changed parent/PID/executable, missing child evidence and extra GPU/headless/security flags. This launcher-only follow-up does not change the frozen numerical policy.

Git retains **all 176 decoded 1024² RGBA8 textures**: 44 native plus 44 for each browser, using lossless compressed XOR deltas against the existing committed software goldens. [matrix/index.json](./matrix/index.json) binds original PNG digests, baseline decoded digests, payloads and reconstructed RGBA digests; [matrix/sha256.json](./matrix/sha256.json) binds retained metadata and payloads. The original PNG compression streams are not reconstructed. Exact original decoded pixels are available for full-matrix audit, not just selected witnesses. Native golden bytes are never changed.

```sh
python docs/evidence/browser-quality-v2/verify.py
python docs/evidence/browser-quality-v2/verify.py tmp/v2-replay-new
cargo xtask browser-material-check tmp/v2-replay-new/native tmp/v2-replay-new/chrome
cargo xtask browser-material-check tmp/v2-replay-new/native tmp/v2-replay-new/edge
cargo xtask browser-material-check tmp/v2-replay-new/native tmp/v2-replay-new/firefox
```

Requires NumPy/Pillow. The fresh replay directory contains explicitly **derived** PNG encodings/manifests and `DERIVED-REPLAY.json`; it is an offline diagnostic, not a browser or package qualification. Local replay verified all 176 texture hashes and exactly reproduced all metrics, structure/causality and old/new verdicts for all 132 browser channels. Production screenshots, selected material contact sheets and machine receipts remain in Git. Full routine Linux pixels/logs remain in the finite-retention artifact; no perpetual full-run audit of that separate Linux execution is promised.

Scope: the measured texture/response budget and recorded packages/environments. No universal device support, arbitrary BRDF/physical-displacement promise, acceptance of PR #15/#16, native baseline change, or package publication follows from this result. Remaining warp accuracy work is separately reviewable and is no longer justified solely by the old browser changed-pixel count.
