# weave-pattern@1 节点夹具

[English](./README.md) | 简体中文

[冻结节点用例](../../../docs/weave-pattern-acceptance.zh-CN.md)及[契约](../../../docs/node-contracts.zh-CN.md)由本夹具、Core 验证和生产 WGSL 探针实现，不冻结织物材质计划。input.mix 暴露全部八个参数，cases.json 提供默认、三个模式、上下边界、不等轴、奇数矩形及无效覆盖；空隙和完全覆盖使用固定字节预期，无黄金生成。

执行 cargo xtask test-node weave-pattern。test-only helper 探针覆盖全部交点的解析 parity／严格分层、宽度支持、连续起伏、UV 整数平移、四样本加权 share、跨模式占用一致及重复精确性；原始 half 容差按冻结用例。公开 Native／浏览器的 check-weave.mjs 比较 18 个请求：默认 256／1024／2048、不等轴 257×129、上下边界，均覆盖三个模式，每分量差 <=1/255。

普通报告在 tmp/node-tests/<backend>/，本地交付日志及 1024 模式图在 tmp/weave-node/；不可当作材质 PBR／人工接受。启用硬件时显式记录适配器；无软件自动回退。
