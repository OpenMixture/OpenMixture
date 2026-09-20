# Browser material comparison — M5-05

English | [简体中文](./browser-materials.zh-CN.md)

**2026-09-20 gate redesign:** New runtime comparisons use [profile v2](./browser-quality.md): bounded amplitude, local bias and channel-specific responses. The same profile now serves Studio material comparisons; native goldens and exact checker checks remain unchanged. Superseded sparse-pixel verdicts are removed from current reports. New browser support still needs source-bound qualification.

The native reference producer calls the public CLI on all 11 existing acceptance cases, at 1024 × 1024 with baseColor/normal/roughness/height. It preserves original source bytes and public overrides. Preparation requires an explicit backend and a fresh directory; it rejects runtime implementation drift relative to the archive's producer revision. Ordinary source, fixture and build identities are separate from plan hashes.

```bash
MIXTURE_GPU_BACKEND=metal node scripts/browser-runtime/prepare-materials.mjs tmp/browser-native 4b914feb9f3365d292b27ea60c5e0b6004f745e8
# In the independent product checkout:
npm run test:materials -- /absolute/native-reference /absolute/new-browser-output
# Back in the engine checkout:
cargo xtask browser-material-measure tmp/browser-native /absolute/new-browser-output
cargo xtask browser-material-check tmp/browser-native /absolute/new-browser-output
```

On Linux use the existing pinned SwiftShader setup and explicit Vulkan/software policy. The product consumes only the detached reference manifest and installed tarball. Its production assets are statically served under `/player/`; missing WASM or unavailable WebGPU is a failure. The engine comparison decodes browser PNGs and reuses existing material structure, seam, non-degeneracy, causality and height/normal relationship checks without changing native goldens.

`browser-material-measure` records differences and checks structural/semantic gates, but explicitly does not accept pixel tolerances. `browser-material-check` enforces [profile v2](./browser-quality.md). Both write `comparison.json` and per-case native/browser/difference contact sheets in the browser output directory. The plan comparison preserves integers exactly, normalizes f32 JSON projection, and requires identical semantic hashes. The original manifest and PNG digests are checked before comparison.

## Current-candidate qualification — ALPHA-01

The command example above reproduces the historical archive. The [material workflow](../.github/workflows/browser-materials.yml) now builds and consumes a new archive in the same job at the checked-out engine revision (including a PR merge revision). It does not download an unbound latest artifact. This is implemented verification tooling, not a new accepted matrix until that revision's run passes.

The consumer remains Studio `56c510ab57daa1b68ef660525a648a582730a37e`. The [candidate verifier](../scripts/browser-runtime/candidate.mjs) requires clean producer metadata, the expected engine and consumer revisions, and the archive SHA-256. It replaces only the disposable consumer's vendor archive, build receipt and runtime lock entry. The new entry uses the archive's SHA-512; every other dependency remains locked. `npm ci` then installs it, and every installed package file is compared with the candidate archive. Product source and committed vendor history are unchanged. A future package version requires an explicit consumer-contract update.

After installation the workflow runs `npm run check`, all 28 pinned browser contracts, an actual browser build-identity probe, the 11-case/44-channel material matrix plus 12 lifecycle renders, frozen comparison, and normal production deployment. The probe also supplies the historical WASM bytes to current JS and requires `MIX_BROWSER_BUILD_MISMATCH`. Public types and real bigint/owned-output behavior are covered by the independent consumer. Both browser jobs run the focused rejection tests:

```bash
node --test packages/runtime/test/runtime.test.mjs scripts/browser-runtime/candidate.test.mjs
```

For a fresh candidate, first commit the engine changes and build with `node scripts/browser-runtime/build.mjs`. Prepare native references using that exact full engine SHA, retaining the existing source-drift guard. With a fresh checkout of the consumer pin, use:

```bash
# From the engine root; use fresh output directories for every run.
node scripts/browser-runtime/candidate.mjs stage target/browser-runtime /absolute/product /absolute/candidate-evidence <full-engine-sha> 56c510ab57daa1b68ef660525a648a582730a37e
# In /absolute/product: npm ci; npx playwright install chromium; npm run check; npm run test:browser
node /absolute/engine/scripts/browser-runtime/candidate.mjs installed /absolute/product /absolute/candidate-evidence
# Keep the product as the working directory for probe (it serves the browser-test build).
node /absolute/engine/scripts/browser-runtime/candidate.mjs probe /absolute/product /absolute/candidate-evidence
# Run test:materials, then the engine's browser-material-check as above.
# In the product, run npm run test:deployment after material comparison.
node /absolute/engine/scripts/browser-runtime/candidate.mjs verify /absolute/product /absolute/candidate-evidence /absolute/native-reference /absolute/browser-output
```

On Linux, unset `VK_ICD_FILENAMES` and `VK_DRIVER_FILES` for browser commands and use the workflow's explicit Chromium SwiftShader arguments. `probe` must run before deployment rebuilds assets without the test harness. `verify` resets prior qualification success before any checks, binds observed build info, archive/lock/native identities, browser/deployment results and comparison digests, and fails on skipped or missing gates. `qualification.json` is the final receipt; `candidate.json` retains original and substituted lock/archive identities, product revision and CI run/attempt. The workflow uploads those records, the candidate archive, native/browser pixels, test reports and deployment evidence with 30-day retention. Accepted results still require the [evidence retention policy](./evidence-policy.md).

This job uses an independent package consumer, but does not claim OS sandbox denial of the engine checkout or default-browser support. Chromium still uses controlled test flags; ordinary user configuration, npm publication and Studio's full saved-file upgrade qualification remain separate Alpha gates. Native goldens and the frozen v2 numerical budgets are unchanged.

## Historical v1 frozen criteria

[Calibration review](./evidence/m5-05/calibration.md) records local and Linux measurements, failed candidate comparisons and the frozen per-channel gates. Acceptance must run after this freeze.

[Formal acceptance and complete pixels](./evidence/m5-05/README.md) retain both post-freeze results and audit commands.
