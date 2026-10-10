# weave-pattern@2 — 冻结节点契约与验收

[English](./weave-pattern-v2-acceptance.md) | 简体中文

2026-10-09 维护者对 PR #90 的三项决定，在实现前记录并冻结：

1. 批准候选 C：B 的 X/Y/D 占用所有权及 Vw/Vf，连续叠层 H=Zw+Zf−Zw·Zf；有序 2×2 盒，H=ΣH/4、C=ΣC/4、S=ΣVw/ΣC（空覆盖 .5）。@1 所有参数／范围／节点默认值和 48 字节 ABI 不变，无新控制或种子。
2. 批准 @2 与 @1 共存，@1 shader 字节、lowering 和像素不变；旧 runtime 明确拒绝 @2。独立 KernelId::WeavePatternV2／typed invocation，仅一个新 WGSL；18 最新类型，身份 19→20，kernel 16→17。下一未发布候选 Rust 0.10.0／browser 0.10.0-alpha.0；格式 v1、plan/API v3 不变，不发布。
3. 批准节点实现；织物材质修订和 @2 的 MAT-03 A–D 再验收另行进行，不在本 PR。

[已批准公式／连续性论证／参数表](./weave-pattern-v2-design.zh-CN.md)与[机器可读冻结探针](../fixtures/nodes/weave-pattern-v2/acceptance-plan.json)共同构成契约。后者固定每个几何、尺寸、偏移与数值门槛；先提交本文件和计划，再实现。任何冻结失败立即记录并停止，不得放宽。

核心可见宽度 min/median=1，来自 792 点研究在 128／64 px 节距均为 1；测所有冻结几何 1024²／2048² 的经纬全部交点扫描线。不等宽、端点、32 密度、空／部分覆盖、加权盒 share、确定性、无效参数均有固定用例。几何中心 C=1、S=精确奇偶，顶高 1、下高 r，分离至少 .25；helper 半精度比较允许 1/1024。支撑／格边界／lift 连接左右 ±1e−5 UV，高度差 ≤.01，并以稀疏解析值 1/1024 和生产法线公式 encoded 4/255 检查；后者预算覆盖半精度邻点误差在 1K 中心差分中的放大及最终舍入，relief=.025、strength=.5，不是 PBR 接受。

周期：零原点全部尺寸／模式精确；双轴二次幂尺寸全部模式 raw-half 精确；257×129 仅 plain／varied／combined-low／combined-high 的 H/C 门槛为相邻半精度模式最多一步。冻结前独立 f32 研究这四例 H/C 最大均为一步；share 已见 7 步，奇数尺寸 S/P 只记录而不门控，绝不称其通过。使用唯一源码锚点测试副本注入采样原点，生产 shader 和 ABI 不增测试功能。硬件须四尺寸；实际报告 Cpu 时 2048² 明记 notRunOnSoftware，沿用阶段 B 第四修订。

公开 Native/browser 差 ≤1/255；@1 回归逐字节比较实现前 main 的 CLI 输出并校验 shader 哈希。研究参考仅作稀疏测试 oracle，无 CPU 渲染路径。节点实现须满足不变量 8 及节点 playbook；旧材质、golden 与接受记录不改。
