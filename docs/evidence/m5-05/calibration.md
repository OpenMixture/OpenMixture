# Frozen browser comparison criteria — 2026-09-15

English | [简体中文](./calibration.zh-CN.md)

This record freezes [browser-only per-channel tolerances](https://github.com/OpenMixture/OpenMixture/blob/efdac411198bfa87b7eba7ff9e588330ec08baf9/docs/browser-tolerances.json) after measurement and agent review, before an acceptance run. It does not update or loosen native goldens.

[Local calibration](./local-calibration.json) uses clean native producer `3c851ca` and isolated product `56c510a`, Chromium 153.0.8010.12 on macOS arm64. All 11 semantic hashes and normalized plans agree; all quality/relationship gates pass. Forty of 44 channels are exact. The four wood roughness cases differ by at most 1/255; worst mean absolute error is 0.0002975464 RGBA8 units and worst changed-pixel ratio is 0.0003967286.

[Linux calibration](./linux-calibration.json) comes from [run 34941156861](https://github.com/OpenMixture/OpenMixture/actions/runs/34941156861), native pinned SwiftShader and independently bundled Chromium SwiftShader. All 11 semantics and quality gates pass. Fourteen leather channels have differences of one unit; the worst changed ratio is 0.0000104905 (11 pixels in a 1K image). The [earlier candidate](./candidate-tolerances.json) demanded exact color/normal/height, so 12 Linux comparisons are false in the retained measurement report. A successful **measurement** job did not accept those failed pixel gates. [Context](./linux-calibration-context.json) records actual browser, source/manifest and driver evidence.

| Channel | Maximum absolute error | Maximum mean absolute error | Maximum changed-pixel ratio |
|---|---:|---:|---:|
| baseColor | 1 | 0.00001 | 0.00001 |
| normal | 1 | 0.00001 | 0.00002 |
| roughness | 1 | 0.001 | 0.001 |
| height | 1 | 0.00001 | 0.00001 |

All values use decoded RGBA8 units, not normalized floats. Pixel threshold is zero: any component difference counts as a changed pixel. Limits allow only sparse single-unit variation, with small explicit margins over measured maxima; broad per-pixel drift still fails. Structure, seams, causality, relationships and exact plan hashes remain independent mandatory gates. Single-unit differences are consistent with quantization variation, but this record does not claim to isolate a particular compiler instruction as their cause.

The agent inspected [local wood](./local-wood-comparison.png) and [Linux leather](./linux-leather-comparison.png) native/browser/difference sheets; no visible material-structure change was observed. This is browser comparison review, not a new human acceptance of engine goldens. Complete accepted matrix pixel content will be retained with the subsequent acceptance receipt. These criteria apply only to the recorded implementations and tested environments; other browsers/hardware require new evidence and review, not silent tolerance widening.

[Initial CI failure](./initial-ci-failure.log) occurred before comparison when initialization returned no transported context. The successful run separates Chromium's driver environment from native Vulkan variables and transports context as JSON text with browser errors captured. Both changes were made before rerunning; the record does not attribute the failure to either change alone.
