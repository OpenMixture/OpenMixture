# Browser material comparison — M5-05

English | [简体中文](./browser-materials.zh-CN.md)

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

`browser-material-measure` records differences and checks structural/semantic gates, but explicitly does not accept pixel tolerances. `browser-material-check` also enforces [per-channel tolerances](./browser-tolerances.json). Both write `comparison.json` and per-case native/browser/difference contact sheets in the browser output directory. The plan comparison preserves integers exactly, normalizes f32 JSON projection, and requires identical semantic hashes. The original manifest and PNG digests are checked before comparison.

## Calibration in progress

The first local Metal/Chromium measurement produced identical hashes for all 11 cases. Forty of 44 channels were byte-identical. Four wood roughness outputs differed by at most one RGBA8 unit, with changed-pixel ratios from 0.0001783371 to 0.0003967285. The current candidate gates require exact baseColor/normal/height and roughness max absolute 1, mean absolute 0.001, pixel threshold 0 and changed ratio 0.001. These are provisional until the pinned Linux browser matrix is measured and the calibration record is reviewed. No complete M5-05 acceptance is claimed by this tooling commit.

The new CI matrix pins an independent product commit and preserves all existing native required checks. Native SwiftShader and Chromium's bundled SwiftShader have separate provenance. A passing measurement job does not freeze tolerances or establish broad hardware compatibility. Publication, public website hosting, Studio editing and M6 are outside scope.
