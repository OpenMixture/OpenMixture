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

Full repository checks, post-freeze browser comparisons, new CI qualification and real-material visual review are recorded separately when completed. Their completion must not be inferred from these synthetic controls. Native goldens, fixtures and historical tolerance bytes are unchanged.
