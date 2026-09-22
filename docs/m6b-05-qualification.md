# M6B-05 — Qualification and closeout

English | [简体中文](./m6b-05-qualification.zh-CN.md)

[Qualification completed](./evidence/m6b-05/README.md) on the recorded Linux software matrix: all 16 package channel comparisons are exact and all six required checks pass. This slice changes consumer tests/tooling only, retaining package v1, document v1, plan v2, API schema 2 and fractal-noise v1. Rust 0.5.0 / npm 0.5.0-alpha.0 remain unpublished. The linked receipt identifies the exact tested commit/archive; later documentation commits do not replace that identity.

The independent Native consumer owns `tests/asset_support`: the unchanged M6A height/normal graph, 1024×1024 frozen triangular image and 65×3 asymmetric control (`R=(17x+71y)%256`, other channels 19/201/0). Its `asset-fixtures` example authors two single-file assets using only the public Rust codec into the disposable browser consumer. No author sidecars are shipped. The browser fetches only the archive; the loose reference source/pixels are supplied separately by the test for comparison.

Eight cases cover both dimensions and weights 0/0.25/0.5/1, requesting height/normal. Each compares the loose and packaged plan and every pixel exactly on the same backend, then reloads/renders twice with malformed input rejection before recovery. Input mutation after browser acceptance cannot affect output. Every render requires zero live per-call GPU bytes; owned pixels survive renderer destruction. Native preparation coverage also drops the owned package and mutates its former source bytes. Existing M6B-04 moved-file CLI, strict byte views, busy/destroy and package-budget cases stay required.

`cargo xtask gpu-smoke` runs the new Native matrix both from source and isolated normalized Cargo archives. Candidate browser qualification now requires 16 tests; registry qualification stays at 13. The cross-runtime verifier requires all eight case identities, 16 channel PNGs and the unchanged maximum component delta of 1. Windows noise limitations are recorded separately from exact same-backend packaging equivalence; no tolerance is relaxed and no golden is reset. The existing three-material Native/browser gates and all six required CI checks remain mandatory. CI uses pinned SwiftShader for Native comparison and its recorded Chromium WebGPU adapter.

```bash
cargo xtask check
cargo test --locked --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --all-features --test asset_qualification
# Explicit backend/software environment is required for ignored GPU tests:
cargo test --locked --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --all-features --test asset_qualification -- --ignored --nocapture
# Clean source revision and fresh directories required:
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/m6b05-browser
node scripts/browser-runtime/compare-assets.mjs tmp/m6b05-browser tmp/m6b05-comparison
```

Use `MIXTURE_ASSET_EVIDENCE` for the Native report path. The comparison script supplies `MIXTURE_ASSET_BROWSER_DIR` and rejects missing/duplicate artifacts, identity mismatches and any failed numerical gate. Browser results retain package SHA-256, plan hashes, build/adapter/browser identity, allocation reports, per-channel raw pixel hashes and PNGs. Ordinary complete outputs stay in ignored directories or 30-day CI artifacts; concise bound closeout measurements and necessary visual evidence are retained separately. Reproduction creates new evidence, not the original artifact.

Publication, merging the milestone stack, Studio upgrades, NUM-01 noise migration and any expansion of platform support are separate tasks.
