# Unresolved painted-metal software parity failure

English | [简体中文](./README.zh-CN.md)

The [Chromium run](https://github.com/OpenMixture/OpenMixture/actions/runs/35836054345) for PR #59 head `611e9c4afad7e4b6558a1c5a4bf538d13196650e`, tested merge `9df804336d295c74b72f32b81371dc81a1b425b9`, fails the frozen painted-metal Native/browser normal gate: maximum difference is 4/255 at 1K and 8/255 at 2K, against the unchanged ≤1/255 limit. Cost and timing checks pass; that does not qualify pixels. Later original/noise-v2 browser material steps did not execute. The candidate remains unaccepted and unpublished.

[receipt.json](./receipt.json) binds the retained reports and images by size and SHA-256, records the source identities and artifact expiry, and distinguishes the Windows control from Linux qualification. [ci-native.json](./ci-native.json) records the failed comparisons. Native/browser normal and height images at both resolutions remain here for inspection; complete ordinary output remains in artifact `10739483613`, expiring 2026-10-23. The material input is the unchanged [before fixture](../perf-mat-before/material.mix).

The Windows control used Chrome 153.0.8010.53's bundled software Vulkan driver: source `ddb2e831e0397ccc0e07eebb27c8684166072a11` (0.7 release) and `611e9c4afad7e4b6558a1c5a4bf538d13196650e` (0.8 debug) produced identical five-channel 1K PNG bytes. Different profiles prohibit a timing comparison. Decoded Windows pixels also match the retained Linux browser pixels exactly. The [decoded comparison](./decoded-comparison.json) finds 422 differing normal pixels against Linux Native, maximum 4. This control does **not** establish Linux before/after equivalence or identify the first divergent node. No shader repair or precision-policy decision follows from it alone.

For the next bounded diagnostic, the existing Chromium job invokes [probe-painted-metal.py](../../../scripts/probe-painted-metal.py) only after the reuse gate fails. It uses the production CLI and the same pinned Native adapter to render eleven fixed Scalar stages through height and normal at 1K. Each derived `.mix`, command, plan hash, CLI report and image is retained under `tmp/painted-metal-stages/` in that run's artifact. Slicing changes allocation, so these outputs locate candidate divergence rather than prove unchanged full-graph behavior. The diagnostic is not a passing gate, an alternative renderer or a platform qualification. No tolerance, golden, required check or shader changes.

Local reproduction with an explicitly configured software Vulkan driver and freshly built CLI:

```bash
python scripts/probe-painted-metal.py --cli target/release/mixture --output tmp/painted-metal-stages
```

Use the actual CLI path on Windows. The output directory must not exist. The script needs only Python's standard library. WSL experiments were stopped; no WSL build or result is used as acceptance evidence. Guide impact: no operational rule changed; this record preserves failure evidence and the diagnostic keeps the existing six gates intact.
