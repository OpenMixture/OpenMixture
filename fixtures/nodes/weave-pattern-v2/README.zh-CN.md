# weave-pattern@2 节点夹具

[English](./README.md) | 简体中文

[冻结节点契约](../../../docs/weave-pattern-v2-acceptance.zh-CN.md)与 [acceptance-plan.json](./acceptance-plan.json)先于实现提交。input.mix 显式使用 @2，原 weave-pattern 夹具保留 @1；不迁移织物材质。无输入、单 Scalar value，mode=height/coverage/warp-share；所有控制及 48 字节 ABI 同 @1。C 使用占用门控所有权和 H=Zw+Zf−Zw·Zf。

运行 cargo xtask test-node weave-pattern-v2；cargo xtask test-node weave-pattern 单独覆盖 @1。前者执行 literal cases、无效参数、中心／支撑／加权盒／精确重复、稀疏 C1／法线 oracle、公开 RGB 核心宽度和全周期 raw-half 探针；均由生产 WGSL／唯一 wgpu 执行。普通 CPU 测试覆盖显式版本共存、lowering 和 ABI；gpu-smoke 包含两者。

硬件执行 256²、1024²、2048²、257×129；实际 Cpu 对 2048² 明记 notRunOnSoftware。奇数 S/P 为观察，不是通过。MIXTURE_NODE_EVIDENCE_DIR 下记录 weave-pattern-v2-probes.json 与 core-probes；门槛和失败停止规则见冻结计划。公开 Native/browser 检查脚本为 node scripts/browser-runtime/check-weave-v2.mjs <candidate目录> <新输出目录>，差 ≤1/255，重复精确。

这些是节点结构／数值证据，不是织物 PBR／人工接受。@2 材质修订与 A–D 再验收仍待后续 PR；不改 @1 goldens。
