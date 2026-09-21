# M6A-03 Native evidence

English | [简体中文](./README.zh-CN.md)

The [implementation record](../../m6a-03-native-resources.md) defines scope and reproduction. Local Windows/NVIDIA GeForce GT 1030 Vulkan execution covers the frozen image probe, including exact byte ramp, padding, shared IDs, crossfade and lifetime gates. This is host-specific evidence, not a general hardware-support promise. Pinned SwiftShader results are recorded separately after the PR checks.

The [run receipt](./run.json) binds 110 source/input files, the actual adapter, 22 execution cases, descriptor measurements, plan identities and image digests. The source was a dirty working tree based on the recorded M6A-02 commit; hashes identify the tested files. Agent visual review is recorded separately from maintainer acceptance.

The selected contact sheet has columns at weights 0, 0.25, 0.5 and 1; height is above normal. Each panel is nearest-neighbor sampled from 1K to 256×256, with no color adjustment. Reproduce with `node scripts/image-resource-contact.mjs tmp/node-tests/auto <output.png> docs/evidence/m6a-03` after `cargo xtask test-node image-input`.

| Height / normal | 0 | 0.25 | 0.5 | 1 |
|---|---|---|---|---|
| height | ![height 0](./height-0.png) | ![height 0.25](./height-0.25.png) | ![height 0.5](./height-0.5.png) | ![height 1](./height-1.png) |
| normal | ![normal 0](./normal-0.png) | ![normal 0.25](./normal-0.25.png) | ![normal 0.5](./normal-0.5.png) | ![normal 1](./normal-1.png) |

Retained source identities, measurements and selected visual content accompany the completed run receipt. Full repeated logs and raw RGBA outputs remain in ignored local directories or finite-retention CI artifacts. Browser image execution and M6A-05 release qualification are not claimed.
