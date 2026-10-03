# weave-pattern@1 node fixtures

English | [简体中文](./README.zh-CN.md)

The [frozen node cases](../../../docs/weave-pattern-acceptance.md) and [contract](../../../docs/node-contracts.md) are exercised by this fixture, Core validation and production-WGSL probes. The woven-material plan remains unfrozen. input.mix exposes all eight parameters; cases.json covers defaults, three modes, endpoints, unequal axes, an odd rectangle and invalid overrides. Empty-gap/full-coverage probes have literal byte expectations, without generated goldens.

Run cargo xtask test-node weave-pattern. Test-only helper probes cover analytical crossing parity/strict layer separation, width support, continuous lift, integer UV translations, four-sample weighted share, cross-mode occupancy consistency and exact repeat; raw-half tolerances follow the frozen cases. Public Native/browser check-weave.mjs compares 18 requests: defaults at 256/1024/2048, unequal 257×129, low/high boundaries, each in all three modes; every component delta <=1/255.

Ordinary reports are under tmp/node-tests/<backend>/; local delivery logs and 1024 mode images are under tmp/weave-node/. These are not material PBR/human acceptance. Hardware runs explicitly record the adapter; no silent software fallback.
