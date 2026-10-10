# weave-pattern@2 node fixtures

English | [简体中文](./README.zh-CN.md)

The [frozen node contract](../../../docs/weave-pattern-v2-acceptance.md) and [acceptance-plan.json](./acceptance-plan.json) were committed before implementation. input.mix explicitly uses @2; the original weave-pattern fixture retains @1. No woven material migration. No inputs, one Scalar value, mode=height/coverage/warp-share; all controls and the 48-byte ABI remain @1's. C uses occupancy ownership and H=Zw+Zf−Zw·Zf.

Run cargo xtask test-node weave-pattern-v2; cargo xtask test-node weave-pattern separately covers @1. The former runs literal cases, invalid controls, centers/support/weighted box/exact repeats, sparse C1/normal oracles, public RGB core widths and full-period raw-half probes through production WGSL/the sole wgpu executor. CPU tests cover explicit version coexistence, lowering and ABI; gpu-smoke includes both.

Hardware executes 256²,1024²,2048²,257×129; actual Cpu marks 2048² notRunOnSoftware. Odd S/P is an observation, never a pass. MIXTURE_NODE_EVIDENCE_DIR retains weave-pattern-v2-probes.json and core-probes; frozen thresholds and stop rules live in the plan. Public Native/browser comparison: node scripts/browser-runtime/check-weave-v2.mjs <candidate-directory> <fresh-output>, delta≤1/255, exact repeats.

These are node structural/numerical observations, not woven PBR/human acceptance. An @2 material revision and A–D requalification remain later PRs; no @1 golden change.

Unlike the retained @1 visibility formula, @2 ownership is independent of crown and underRatio: these two controls affect height only. Review material control-isolation expectations when a later PR migrates the woven recipe; this node PR does not migrate it.
