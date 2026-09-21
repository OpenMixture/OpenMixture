# Browser Alpha 0.3.0 publication

English | [简体中文](./README.zh-CN.md)

Published on **2026-09-21T18:01:14.383Z** after the user completed npm security-key authentication. [Publication receipt](./publication.json), [registry metadata](./registry-metadata.json), [13-test registry qualification](./registry.json) and [exact registry lock](./registry-lock.json) confirm the downloaded archive and every installed file match the qualified CI candidate. Observed tags: alpha → 0.3.0-alpha.0; latest → 0.1.0-alpha.0.

```sh
npm install --save-exact @openmixture/runtime@0.3.0-alpha.0
```

This release adds `image-input@1`, explicit resource bindings and budgets, immutable synchronous capture, and Native/browser prepared-resource execution through the sole wgpu renderer. The SDK is authored in strict TypeScript. `.mix` stays v1; public browser API schema advances to 2 and all plans/hashes to v2. Consumers must handle `resourceRef`, the new resource report fields, and invalidate old plan caches. Images must be same-size tightly packed linear RGBA8; R supplies height, with no implicit decoding, gamma conversion or resizing. See the [browser resource contract](../../m6a-04-browser-resources.md).

## Source, archive and qualification

PRs #35–37 were reviewed and integrated into main; [integration](./integration.json) records their distinct merge commits and the review scope. All six [main checks](./main-checks.json) passed at `acabc3a911c14c3b8a2775d184f8b9cd759209c9`. The exact 387,049-byte archive comes from main [browser material run 35624051146](https://github.com/OpenMixture/OpenMixture/actions/runs/35624051146), without rebuild or repack:

- SHA-256: `af2ba690f0d56c347ab8336cfd5ffffac31b5098e4f3356a94a01958c01042d4`.
- Build ID: `sha256:721eed5458ce82d55c73fcae004a1edb497ec1a8aab9f470c12c034c1543d602`.
- [Build receipt](./build.json), [13 public candidate tests](./ci-sdk.json), [eight resource comparisons](./ci-resources.json), [Scalar regressions](./ci-scalar.json), [11 material cases / 44 channels](./ci-materials.json), and [outer product verifier](./ci-product.json).

Archive SHA-256, build identities and all seven product evidence digests were verified after download. Resource comparisons have maximum component delta **0** for all four 1K weights and both height/normal channels. Existing material and Scalar gates remain unchanged.

Qualification is bounded to the recorded Linux Native SwiftShader / Chromium software matrix. Windows hardware Native/browser resource normals remain **not qualified**, with recorded maximum delta 8 against the unchanged ≤1 gate. [M6A-05](../m6a-05/README.md) retains the reviewed contact sheet and failure evidence. Historical warp precision limits remain. Interface tests on ordinary Chrome do not establish cross-runtime hardware parity. No Rust publication, stable promotion, CLI distribution, Studio upgrade or deployment is included.

## Publication and registry consumption

Publish only the frozen archive with `npm publish <archive> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/`. The user explicitly authorized review, merge and publication. npm account security verification is separate from that authorization. Preserve the existing `latest` tag; exact-version installation is recommended for Alpha.

After registry availability, update the independent consumer's exact manifest/lock to this version and run `node scripts/browser-runtime/consumer.mjs registry - <fresh-output>` with `MIXTURE_BROWSER_CHANNEL=chrome` on this Windows host. Candidate and registry qualification now require all 13 tests, including the four resource cases, with no skips/flakes. Compare downloaded archive integrity and every installed package file against the frozen CI candidate. The delivery documentation/verifier commit is not the package's build source.

The immutable archive README contains earlier publication status; this release record and current source documentation supersede that historical wording. The shipped bytes are never edited after publication.

Git retains selected source/archive/qualification/registry receipts and the exact registry lock. Full repeated PNGs, archive bytes and routine logs remain in [CI artifact 10651552461](./ci-artifact.json), expiring 2026-10-21, and ignored temporary local storage. The existing source-bound M6A-05 visual review is not relabelled as a new human decision. This record does not promise permanent full-run artifact retention.
