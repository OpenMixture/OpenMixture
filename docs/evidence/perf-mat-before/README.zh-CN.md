# PERF-MAT 进入证据

[English](./README.md) | 简体中文

默认涂漆金属配方在干净运行时源码 `e7b25c4e36f6d7a2052fdba11d8cdfffad762a9d`（Rust 0.7.0）上产生真实预算失败。[收据](./receipt.json)绑定选定输入、原始报告、五张基线 PNG 及相关运行时／着色器／契约源码。这是优化基线，不是 MAT-02 材质或人工 PBR 验收。

[精确输入](./material.mix)在参考尺寸 1024 解析冻结配方的默认控制。[编译计划](./plan-1024.json)有 23 个像素 pass 和五个连接输出。记录的 Windows NVIDIA GeForce GT 1030 上 [1K Vulkan 渲染](./render-1024-vulkan.json)成功：描述符峰值 201,327,056 字节、存活字节 0、复用字节 0、报告冷执行总时间 2,621.9843 ms。CPU 自有输出、PNG 编码及驱动开销不属于 GPU 描述符内存。未测 GPU 时间戳时长和热渲染中位数；报告阶段时间是墙钟测量，不是硬件 GPU 时间戳。

[真实 2K 拒绝](./rejected-2048.json)在编译阶段报告 `MIX_LIMIT_TRANSIENT_BYTES_EXCEEDED`：实测估算 805,306,832 字节，上限 536,870,912 字节。两个 radius 覆盖均为 8，保持冻结的分辨率映射。`inspect` 不获取 GPU，没有提高上限，这份优化前记录没有 2K 像素。此失败启动按测量触发的 [PERF-MAT ADR](../../decisions/0009-transient-texture-reuse.zh-CN.md)。

基线像素保留为 [baseColor](./baseColor-1024.png)、[normal](./normal-1024.png)、[roughness](./roughness-1024.png)、[metallic](./metallic-1024.png) 和 [height](./height-1024.png)。它们用于未来同一适配器上的优化前后精确比较，不是已安装的 golden。BaseColor RGB 为 sRGB，Scalar／Normal PNG 为线性。Agent 检查 baseColor 看到大块蓝漆与裸露基底，锈色较弱；这不关闭待完成的材质美术、因果、接缝或 PBR 门槛。运行时复用必须保持这些字节，配方改进应在材质审查中单独保留证据。

执行 `node docs/evidence/perf-mat-before/verify-design.mjs` 可验证保留字节哈希、独立解析当前冻结默认配方并与保留输入比较，以及审计拟议最低空闲槽算法。显式预期映射有 14 个完整描述兼容槽，模拟 2K 峰值为 503,316,944 字节（480 MiB + 464 字节）。这只是 CPU 图分析，不是第二像素执行器或已实现 GPU 分配器。逻辑存活峰值小于物理池总量，因为不同类型的槽独立保留。

在绑定源码上构建 `cargo build --locked -p mixture-cli`，将保留输入传给收据中的命令，即可复现原运行时。1024 不需要覆盖；2048 使用 `--set radiusX=8 --set radiusY=8`。实现后，原 v2 拒绝仍是历史记录，新源码绑定结果必须证明真实复用执行及像素不变。原始二进制、构建产物及普通日志仍在忽略目录；选定输入／报告／像素保留于此，不声明外部永久归档。
