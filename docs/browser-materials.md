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

## Frozen criteria

[Calibration review](./evidence/m5-05/calibration.md) records local and Linux measurements, failed candidate comparisons and the frozen per-channel gates. Acceptance must run after this freeze.

[Formal acceptance and complete pixels](./evidence/m5-05/README.md) retain both post-freeze results and audit commands.
