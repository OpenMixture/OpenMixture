# weave-pattern@1 — 已冻结的节点验收

[English](./weave-pattern-acceptance.md) | 简体中文

依据维护者 2026-10-03 对 PR #80（eb3ea30）的决定，在实现前冻结。[已批准的节点公式、参数、采样与 ABI](./mat-03-woven-surfaces.zh-CN.md)为规范。仅冻结节点用例，织物材质验收另行处理。不增加公开端口、CPU 渲染器或 schema。

| 门槛 | 冻结用例与预期 |
|---|---|
| 准入 | 默认值；height/coverage/warp-share 模式；偶数数量 4、8、12、32；宽度 .55、.7、.9；bevel .02、.08、.12；crown 0、.5、1；underRatio .25、.5、.75。含端点均合法。 |
| 拒绝 | 任一轴奇数 5/31、越界 3/33、浮点 token 8.0、字符串/null/bool、越界宽度、错误枚举、未知参数、未支持节点版本。MIX_PARAMETER_INVALID_VALUE 带节点与参数 ID；未知参数为 MIX_PARAMETER_UNKNOWN。源文档、覆盖及未选中分支均检查，不取整。 |
| 降低／ABI | 单个无输入 pass，Scalar rgba16float，8×8 调度；48 字节，模式码 0/1/2，四个填充 word 为零。默认与显式默认同 hash；有效参数改变 hash，节点／边顺序及未用值不改变。保持 .mix v1／plan v3。 |
| 解析交点 | 默认 8×8 与不等轴 12×8 的全部交点：偶数 i+j 为 Hw=1,Hf=r，奇数相反；C=1、Vw=1/0、H=1。r=.25,.5,.75，crown=0,.5,1；正 relief 的解析交点严格上高下低（不承诺任意微小 half 起伏）。原始 helper 容差 1/1024。 |
| 支持域／轮廓 | UV (0,0) 为 H=C=Vw=0、S=.5。暴露经线中心 (.5/Nw,0) 满足 C=1、H>=r。向内半 bevel 处占用为正，宽度边界及外部为零；横向冠形从中心严格下降。无轴向占用端帽。 |
| 周期／连续 | 生产 helper 比较 UV 与 UV+(1,0),(0,1),(1,1),(-1,-1)，覆盖接缝邻域及不等轴。原始 half 容差 1/1024；不比较对边像素。可见权重／高度在边界 ±1e-5 UV 趋于同一极限（差 <=.01），空隙选择器除外。 |
| 滤波／模式一致 | 按规范顺序的四个固定偏移。部分覆盖时对照生产 helper 样本：H=sumH/4、C=sumC/4、S=sumVw/sumC，sumC=0 时 .5（原始 half 容差 1/1024）。至少一个加权与非加权 share 差 >.01 的足迹。分开 half 存储后 C*S=sumVw/4，容差 2/1024。 |
| 公开执行 | 默认、模式、端点、不等轴 12×8 与 257×129 奇数矩形；节点渲染 256、1024、2048 及奇数矩形。有限归一值，同适配器重复精确一致；Native／浏览器每个 RGBA8 分量差 <=1/255。空隙与完全覆盖交点的字面量夹具探针。 |
| 回归 | 明确评审过的 18 类型／16 内核目录；shader/core/plan/node/consumer/gpu-smoke/runtime 测试、干净浏览器候选与硬件一致性、最终 cargo xtask check。不改变既有语义／黄金。 |

GPU helper 观察仅用于测试，通过生产 WGSL 及唯一 wgpu 执行器。解析断言为稀疏 oracle，不是 CPU 渲染路径。普通报告及模式图放在忽略的 tmp/weave-node/ 或 CI 工件。节点图只是合理性证据，不代表织物外观验收。
