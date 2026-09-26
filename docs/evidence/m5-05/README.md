# M5 browser acceptance — 2026-09-15

English | [简体中文](./README.zh-CN.md)

**Storage update (2026-09-25):** Original execution and acceptance conclusions are unchanged. Retrieve complete historical attachments using the [archive instructions](../archives/README.md). Machine receipts, original hashes and capture manifests are unchanged; inspect their paths in the complete restored snapshot. Critical records and review images remain here.

**M5 engineering gates pass for the recorded matrix; the runtime is ready for a bounded Alpha release decision and remains unpublished.** This record closes M5-05 after the earlier [M5-02/M5-03 acceptance](../m5-02-03/README.md) and independent Player workflow. It does not certify untested browsers or authorize Studio/M6. [Engine PR #8](https://github.com/OpenMixture/OpenMixture/pull/8) owns comparison tooling, CI and this assessment; [product PR #5](https://github.com/OpenMixture/Studio/pull/5) owns packaged browser execution, stress and production deployment.

## Sources and ordering

[Calibration](./calibration.md) measured both environments, retained failing candidate comparisons, and froze the [tolerances](https://github.com/OpenMixture/OpenMixture/blob/efdac411198bfa87b7eba7ff9e588330ec08baf9/docs/browser-tolerances.json) in `289a23bf9026c355f557c8d637784c03defb0932`. Both accepted runs started afterward and bind tolerance SHA-256 `f01533c2311e32319c37338126a20f8089957e770e221424c46b10263c40b627`. Acceptance did not widen those gates or update native golden files.

The [machine summary](./summary.json) binds exact source/lock/archive/tool/browser identities, metrics and CI artifact metadata. Local native preparation used clean engine `289a23bf9026c355f557c8d637784c03defb0932`; isolated product execution used clean merged main `659a7ecde5217ef64911600ed49d1f48f98de3f2`. Linux tested PR merge ref `2d0fb9cfc93ee943d2802adfef4a700ddd62be60` for that engine head and product code `56c510ab57daa1b68ef660525a648a582730a37e`. Later evidence commits are not presented as these tested sources.

Both use the unchanged `@openmixture/runtime@0.1.0-alpha.0` tarball SHA-256 `9e245578de160cee1050259ef34358c3fcd3353a21bbb1672d29701bc1ac8084`, produced by engine `4b914feb9f3365d292b27ea60c5e0b6004f745e8`. Native preparation verified no runtime implementation drift against that producer. Runtime/shader/lock sources remain discoverable at that commit; each manifest independently retains input source bytes, contract hashes, overrides, native binary digest and plans. The vendor archive remains in the product repository. Node is 24.20.0, npm 11.19.0, Playwright 1.63.0; the build recipe pins Rust 1.98.1 and wasm-bindgen 0.2.128.

## Accepted matrix

| Environment | Native reference / browser | Material result |
|---|---|---|
| macOS Darwin 25.5.0, arm64 | Explicit Metal reference; Chromium 153.0.8010.12 with `--enable-unsafe-webgpu --ignore-gpu-blocklist` | 11/11 cases, 44/44 channel comparisons; 40 exact |
| Ubuntu 24.04 CI image 20260907.300.1, kernel 6.17.0-1022-azure, x64 | Native Vulkan SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`; Chromium 153.0.8010.12 with the above flags plus `--use-angle=swiftshader --use-webgpu-adapter=swiftshader` | 11/11 cases, 44/44 channel comparisons; 30 exact |

All existing ceramic/leather/wood acceptance variants run at 1024×1024 with baseColor, normal, roughness and height. All exact semantic hashes, normalized plans, structure/seams, non-degeneracy, parameter causality and height/normal relationships pass. [Local measurements](./local-comparison.json) show four wood roughness differences: maximum absolute 1, worst mean 0.00029754638671875, worst changed-pixel ratio 0.000396728515625. [Linux measurements](./linux-comparison.json) show sparse leather differences: maximum absolute 1, worst mean 0.000004291534423828125, worst ratio 0.00001049041748046875. Errors are in decoded RGBA8 units. Exact checker/default/encoding sentinels retain their existing equality gates.

The browser reports `BrowserWebGpu` but redacts adapter name and driver. Local device type is `Other`, Linux is `Cpu`; the Linux flags request Chromium's bundled SwiftShader, whose source revision is not asserted to equal the independently built native driver. Do not infer a hardware name from flags. The initial context's `unverified` verdict describes acquisition before a probe; subsequent successful dispatch/readback and measured PNGs establish execution.

The agent inspected the accepted [ceramic](./accepted-ceramic.png), [leather](./accepted-leather.png) and [wood](./accepted-wood.png) native/browser/difference sheets. No visible material-structure change was observed. This is an agent comparison review; it does not replace or manufacture human acceptance of native goldens.

## Lifecycle, isolation and deployment

Each environment additionally completed four fresh module/device cycles, rendering all three default materials per cycle: 12 extra 1K renders. One 4 MiB channel per cycle survived later renders and destruction with its digest unchanged. All recorded post-render live descriptor bytes were zero; pipeline count grew only to nine. Descriptor estimates exclude JS-owned output copies and do not measure physical GPU memory, browser memory reclamation or long-running soak behavior.

The fresh [local isolation receipt](./local-isolation.json) proves both source checkouts denied and Rust absent from PATH, followed by archive installation, nine Node tests/public type/build checks, [28 real Chromium tests](./local-browser-results.json), material execution and normal production deployment. Those tests include invalid inputs, loader failures, busy/in-flight destruction, controlled real-device loss/mapping cleanup, latest-request behavior and Player PNG correctness. Prior receipts remain linked for their original scope; spontaneous driver loss is not claimed.

[Production deployment](./local-deployment.json) uses the normal built Player under static `/player/`, without the test harness. The browser loads WASM with HTTP 200 and `application/wasm`, completes checker rendering and independently verified 65×3 PNG download, confirms the harness URL is unavailable and disposes the instance. [Screenshot](./local-player.png) retains the visible deployed Player. This is a static local/CI deployment, not public hosting. Material tests use a separate production test build with a public-package harness.

[Linux material run 34942244839](https://github.com/OpenMixture/OpenMixture/actions/runs/34942244839) passed after freeze. [Product main run 34941633999](https://github.com/OpenMixture/Studio/actions/runs/34941633999) passed on merged product `659a7ec`, including 28 browser contracts and normal production deployment. The new browser jobs add execution coverage; the four existing required native checks and active no-bypass ruleset remain unchanged. This record reports these runs; PR/final main status is separately visible in GitHub and must be checked on the integrated SHA.

## Retention and reproduction

[Evidence index](./evidence-index.json) retains all 250 logical files from the two accepted native/browser bundles, including all 176 native/browser channel PNGs, 22 comparison sheets, manifests, receipts and case reports. Identical bytes share content-addressed assets or reference existing tracked native expected PNGs. Original bytes, sizes and SHA-256 values were verified after copying on 2026-09-15; every PNG digest used by the calibration reports is also present. These approximately 84 MB of distinct assets were originally in Git; the content-addressed PNG attachments now live in the verified archive and remain inspectable after CI expiry.

The CI artifact `chromium-material-matrix`, ID `10386545022`, was retrieved and checked. Its service digest is `sha256:bc6f89d73ad7b200674ce72ee891ffb96676e48c61e9be233b540bf0b28f3878`, size 53,651,974 bytes, expiry 2026-10-15T07:38:05Z. Complete ordinary logs and incidental build output remain temporary or in finite 30-day CI artifacts; The dedicated archive retains the complete matrix; Git retains its records, named review images and critical calibration failure. A later rerun is new evidence, not restoration of missing logs.

From the engine repository root, reconstruct and verify the retained original files into a fresh directory, then rerun comparison against the unchanged contracts:

```bash
python scripts/evidence/restore.py m5-05 tmp/retained-m5
python scripts/evidence/replay_m5.py tmp/retained-m5 tmp/m5-05-retained
cargo xtask browser-material-check tmp/m5-05-retained/local-native tmp/m5-05-retained/local-browser
cargo xtask browser-material-check tmp/m5-05-retained/linux-native tmp/m5-05-retained/linux-browser
```

For new GPU executions use the [preparation/comparison guide](../../browser-materials.md) and the pinned [workflow](../../../.github/workflows/browser-materials.yml). The independent product's [qualification guide](https://github.com/OpenMixture/Studio/blob/659a7ecde5217ef64911600ed49d1f48f98de3f2/docs/browser-qualification.md) documents isolation, deployment and browser commands.

No general Safari/Firefox/Windows browser, mobile, physical hardware matrix, SSR/Node GPU, automatic recovery, zero-copy, public hosting or registry publication is accepted here. An npm release still requires an explicit distribution decision, exact archive/version/tag and downstream verification. Studio editing and engine M6 remain separate scope decisions.
