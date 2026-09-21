# Browser runtime 0.2.0-alpha.0 publication

English | [简体中文](./README.zh-CN.md)

Published publicly on **2026-09-21T05:03:13.279Z** after the user's release request and npm security-key authentication. Install with:

```sh
npm install --save-exact @openmixture/runtime@0.2.0-alpha.0
```

The release adds ENG-04 scalar-blend v1 and the independent SDK entry from ENG-03. API schema 1 and .mix v1 remain unchanged. [ENG-04](../../eng-04-scalar-blend.md) defines semantics, compatibility and implementation evidence. Rust 0.2.0 remains unpublished. Studio upgrades, deployment and product acceptance are outside this release.

## Frozen identity and publication

The exact archive from successful main [browser material CI run 35514729976](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729976), attempt 1, was downloaded and published without rebuilding or repacking. The 345816-byte tarball has SHA-256 `6d7b24f613af9444cb6d854d192749f0e46db6d846d5636e460b16c8fed72cdf`, source `9cd62fe7f16fe51a43badd1d9f70267abd87f843` (clean), and build ID `sha256:2418a0770ba86d7a583cdafcafddf935e3d91ba9d9f316f7835deb3ac5137183`. [Build metadata](./build.json) and [registry metadata](./registry-metadata.json) retain toolchain, integrity and registry timestamp.

Command: `npm publish <frozen-tarball> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/`. npm initially returned success while the package was still processing; public availability was verified subsequently. Observed tags: alpha → 0.2.0-alpha.0, latest → 0.1.0-alpha.0. Use the exact version; this is not a stable release. Tags are mutable. The immutable archive README still describes the previous published Alpha; these release notes and current source documentation supersede that historical wording.

## Qualification and retention

All six protected checks passed on the released source: [three-platform CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729970), [pinned SwiftShader native GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729967), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35514729968) and the browser material run above. [CI SDK receipt](./ci-sdk.json) binds nine passing tests, no skips/flakes, exact installed files and the frozen archive. [Scalar comparisons](./ci-scalar.json) bind eight 1K height/normal outputs at weights 0/0.25/0.5/1; native Vulkan and browser SwiftShader maximum component delta is zero. The same run passed pinned disposable Studio contracts, three-material v2 gates, lifecycle and deployment; its seven bound evidence digests were verified after download.

Post-publication reproduction uses the committed independent consumer and a fresh npm cache:

```sh
node scripts/browser-runtime/consumer.mjs registry - tmp/release-registry
```

Windows verification explicitly sets MIXTURE_BROWSER_CHANNEL=chrome. The registry receipt and copied lock record the verifier's source hashes, dirty-source status when applicable, exact installed bytes, browser identity and nine real browser tests. The delivery documentation commit is not the package's build source. No broader hardware/browser guarantee is inferred; the documented warp precision limitation remains.

Git retains compact source/archive/registry/qualification receipts and the registry lock. Existing [ENG-04 visual evidence](../eng-04/README.md) remains its original source-bound review, not a substitute for this archive's execution. These are repeated qualification runs with no changed goldens or new visual acceptance. Full CI logs, repeated PNGs and product evidence remain in artifact chromium-material-matrix, ID 10605779834, [metadata](./ci-artifact.json), expiring 2026-10-20T14:01:51Z; local repeats stay under ignored tmp/release-020. The registry tarball was retrieved and byte-verified; registry availability is not a promise of permanent full CI artifact retention.

[Publication receipt](./publication.json), [post-publication registry qualification](./registry.json) and [exact registry lock](./registry-lock.json) record the completed Windows Chrome run: all nine tests pass with no skips/flakes, the downloaded SHA-256 and every installed file match the CI candidate.
