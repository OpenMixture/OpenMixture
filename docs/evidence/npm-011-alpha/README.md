# Browser Alpha 0.11.0 publication

English | [简体中文](./README.zh-CN.md)

Published on **2026-10-15T17:26:31.644Z** by the user with their own npm authentication. The [publication receipt](./publication.json), [registry metadata](./registry-metadata.json), [13-test registry qualification](./registry.json) and [exact registry lock](./registry-lock.json) confirm that the registry tarball is byte-identical to the frozen CI candidate and that every installed file matches it. Tags after publication: alpha → 0.11.0-alpha.0; latest → 0.1.0-alpha.0 (unchanged).

```sh
npm install --save-exact @openmixture/runtime@0.11.0-alpha.0
```

## What changed since 0.3.0-alpha.0

API schema 2 → **3** and plan v2 → **v3**. Consumers must invalidate stored plans and hashes and handle the newer report and estimate fields. `.mix` v1 documents keep their meaning, and new versions are opt-in. Added since 0.3:

- Stable value noise `fractal-noise@2` ([NUM-01](../stable-noise/README.md)).
- Material nodes `brick-pattern`, `scalar-mask-blend` ([MAT-01](../mat-01/README.md)), `scalar-morphology`, `scalar-subtract` ([MAT-02](../mat-02/README.md)) and `weave-pattern@1`/`@2` ([MAT-03 r3](../mat-03-r3/README.md)). That makes 18 types and 20 supported identities.
- Per-render texture reuse (PERF-MAT, plan v3).
- `.mixpack` assets through `inspectPackage`/`renderPackage` ([M6-B](../m6b-05/README.md)).
- Explicit `.mixrecipe` closures for `.mix` v2 recipe parents through the `recipes` request field ([MAT-04](../mat-04/README.md)).

## Source, archive and qualification

The release source is clean main **`ffb10f71ab68689eb472b21fdb89cafb0b941322`**. All 11 [check runs](./main-checks.json) on it succeeded, covering the six required checks and the three Chromium partitions. The exact 510,006-byte archive comes from main [browser material run 38345148032](https://github.com/OpenMixture/OpenMixture/actions/runs/38345148032), artifact `chromium-material-matrix-core` ([metadata](./ci-artifact.json)), without rebuild or repack:

- SHA-256: `5265cda8931bd9eee7788c8d3cbb58f1449ab053f40dee17586d449540842dfc`.
- Build ID: `sha256:3b0fb8f74959c4b5a4eae7900c82c3443e4aa1f0f3a0facd58b1be892d123843` ([build receipt](./build.json)): `engineDirty` false, pinned Rust 1.98.1 / wasm-bindgen 0.2.128.
- The [candidate qualification](./ci-candidate.json) in the same artifact (`ok: true`) qualified this SHA-256 at the same source and ran every core-partition spec, including `recipes.spec.mjs`. The woven and painted partitions ran in the same workflow.

The release qualification scope is the union of the accepted records: the recorded Linux SwiftShader / Chromium software matrices, plus GT 1030 Vulkan/DX12 and Chrome where each material record says so. Software passes do not qualify hardware pixels, and no general hardware guarantee is made. Recorded limitations remain, including the [MAT-04](../mat-04/README.md) woven recipe parent's higher 2048² descriptor peak, the warp precision limit, and the Windows v1 resource-normal failure (explicit `fractal-noise@2` inputs are qualified). No Rust crate publication, stable promotion, CLI distribution, Studio upgrade or deployment is included.

## Publication and registry consumption

The user ran `npm publish <frozen-tarball> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/` on this Windows host. Before publication, the agent verified the archive identity and confirmed that 0.11.0-alpha.0 was absent from the registry. The preserved `latest` tag is unchanged, and exact-version installation is recommended for Alpha.

After publication, the downloaded registry tarball was byte-identical to the frozen archive (npm `shasum` and `integrity` match). The independent consumer was then pinned to the exact registry version and integrity, and the change was committed (`ea3a4e1`). `MIXTURE_BROWSER_CHANNEL=chrome node scripts/browser-runtime/consumer.mjs registry - tmp/registry-0.11` passed on this Windows / GT 1030 host with a clean consumer tree (Node v24.21.0, npm 11.19.0, Playwright 1.63.0). It ran **13 tests: 13 passed, 0 skipped, 0 flaky** (`sdk.spec.mjs` and `resources.spec.mjs`). Candidate-only specs are excluded in registry mode by design.

The archive README describes this release. The shipped bytes are never edited after publication.

## Retention

Git retains the build, candidate, registry, lock, metadata, dist-tag, check and artifact receipts above. The archive bytes, CI logs and PNGs remain in [CI artifact 11788986576](./ci-artifact.json), which expires 2026-11-12. The local registry-run output under ignored `tmp/registry-0.11/` is temporary. This record does not promise permanent full-run artifact retention.
