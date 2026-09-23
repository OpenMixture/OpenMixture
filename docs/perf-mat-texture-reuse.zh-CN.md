# PERF-MAT 物理纹理槽

[English](./perf-mat-texture-reuse.md) | 简体中文

611e9c4 候选未通过 Linux 软件执行的涂漆金属法线对照。[留存失败与限定诊断](./evidence/perf-mat-numerics/README.zh-CN.md)记录不变的门槛及 Windows 优化前后对照的证明范围；PR #59 保持草稿。

工作中的 Rust 0.8.0／browser 0.8.0-alpha.0 候选实现 [ADR 0009](./decisions/0009-transient-texture-reuse.zh-CN.md)。验收、集成和发布仍分别处理。[原始 2K 拒绝及 1K 像素](./evidence/perf-mat-before/README.zh-CN.md)保持不变；开发运行成功不关闭 MAT-02。

前置集成：PR #56 以 `313074451cd6ddb5e5f82cce933c3d4fe3b4ed38` 集成形态处理，PR #57 以 `cbeb367261bf09b3b7acd2540b4efbee3000e7e7` 集成减法。组合 main 源码通过[三平台 CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35832101430)、[GPU／包检查](https://github.com/OpenMixture/OpenMixture/actions/runs/35832101419)、[WASM 打包](https://github.com/OpenMixture/OpenMixture/actions/runs/35832101468)及 [Chromium 验收](https://github.com/OpenMixture/OpenMixture/actions/runs/35832101483)。PR #58 在最终 head 六项检查通过后，以 `f8dccfee8b94015690a891d20c02d99479fda2b4` 接受分配设计。这些运行只证明各自精确 0.7 源码，不验收新的 0.8 候选。

Core 的[分配规划器](../crates/mixture-core/src/compiler/allocation.rs)在裁剪和稳定 pass 排序后执行。每个逻辑输出记录最后消费者，请求输出别名固定到 pass 数量哨兵。只有前一资源最后消费者严格早于当前 pass，生产者才复用最低编号、完整描述兼容的槽。同 pass 输入／输出重叠被禁止。描述仍为 rgba16float，Scalar、Color、Normal 槽分开；外部 RGBA8 输入不进池。

不可变计划的 `allocation()` 暴露槽描述、逻辑资源到槽映射及最后使用位置。`texture_count`／`textureCount` 统计物理 pass 槽，`logical_texture_bytes`／`logicalTextureBytes` 统计全部逻辑结果，`texture_bytes`／`textureBytes` 统计实际 pass 槽。不变的 uniform、上传及串行读回规则计算保守峰值／累计预算，默认 512 MiB 上限不变。

wgpu [资源所有者](../crates/mixture-wgpu/src/resources.rs)每次调用只创建一次各物理纹理，按逻辑映射生成每个 pass 的绑定，将槽保留至读回／清理。保持独立 compute pass 及原着色器。`AllocationReport.texture_count` 包含物理 pass 槽及独立上传图像，`reused_bytes` 记录省去的 pass 结果分配字节，不代表跨调用保留内存。每个唯一纹理销毁前清空绑定组；成功报告存活字节为零，释放计费平衡。驱动开销、CPU 像素及即时物理显存回收不属于这些计数。

Plan／hash v3、浏览器 API schema 3 和图 CLI 报告 schema 3 显式表达新的内存语义。`.mix`／`.mixpack` 保持 v1；doctor、固定 checker、asset 外层保持 schema 1；`validate` 仍无版本。从 `.mix` 及请求重新编译，使已存计划／哈希缓存失效；不提供计划反序列化或自动源迁移。薄 TypeScript 投影包含类型化 allocation 数组和 bigint 估算。源码／lock 同级版本一起更新，不执行 npm 发布。

[Core 调度测试](../crates/mixture-core/tests/allocation.rs)包含链／菱形／重复输入／别名／默认／裁剪的显式预期，及冻结 2K 材质十四槽映射、等于上限通过和低一字节拒绝。GPU 探针覆盖多尺寸、较早结果固定、输出别名、重复计费、销毁后像素所有权，以及故障注入的同 pass 重叠清理／重试。既有上传／读回失败测试仍为必需。

十一份原始材质哈希使用有限、精确的 [v2→v3 分配记录](./plan-v3-migration.json)，连接不变的 v1→v2 记录。`node scripts/verify-plan-v3.mjs` 验证类型化 v3 哈希，仅移除 allocation 元数据并恢复保留全部资源的估算，重建精确旧 v2 哈希；内核、pass、输出与数值 token 不得改变。若新构建 CLI 不在 `target/debug`，设置 `MIXTURE_CLI`。历史 v2 验证器继续配合其原始 v2 运行时使用。旧计划快照保留，新增 v3 快照显式更新当前测试预期，不重置像素 golden。

候选[公开浏览器测试](../examples/browser-consumer/tests/reuse.spec.mjs)执行四组涂漆金属请求（带种子的奇数尺寸、完整涂层端点、默认 1K 和默认 2K），检查重复／裁剪输出、类型化 v3 分配元数据及销毁后的像素。候选消费现要求二十项测试；精确已发布注册表消费仍要求十三项。干净候选构建／消费后运行 `node scripts/browser-runtime/check-reuse.mjs <candidate-qualification> <fresh-output>`。它以 release 模式运行[独立 Rust 测试](../examples/native-consumer/tests/texture_reuse.rs)，按冻结 ≤1/255 门槛检查二十组通道对照、一致的计划／分配元数据、包往返、物理计数，以及冻结 MAT-02 计划中匹配适配器的冷启动／五次热样本时间预算。Native 与浏览器均保留 PNG 和报告。既有上传／包验收继续必需。Chromium 任务加入此检查，不移除已有门槛。

独立 Native 探针需要设置 `MIXTURE_REUSE_ROOT` 为工作区，`MIXTURE_REUSE_EVIDENCE` 为父目录已存在的新目录，并明确 `MIXTURE_GPU_BACKEND`／`MIXTURE_GPU_SOFTWARE`；运行 `cargo test --release --locked --all-features --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --test texture_reuse -- --ignored --nocapture`。可选将 `MIXTURE_REUSE_BROWSER` 指向包含 `reuse-browser.json` 及 PNG 的对照目录。`MIXTURE_REUSE_BEFORE` 仅为实测 GT 1030 Vulkan 适配器选择复用前的留存目录，要求解码后的 1K 字节完全相同。失败的时间／一致性结果保留在 `native.json`；未知适配器的时间结果不具备资格，对照包装脚本要求匹配且通过预算。

`cargo xtask trace-2k` 保持既有三材质工作负载、排序及 512 MiB 门槛；新的 `m3-pooled-2k-allocation-trace` 类型验证物理槽计数、预期复用、估算一致和清理。此回归与涂漆金属验收不同。完成仍须绑定干净候选源码的 1K 优化前后精确像素、2K 五通道真实执行、冻结冷／热时间、Native／浏览器及包消费、原始／显式 noise-v2 材质矩阵、六项 CI 和合并后检查。MAT-02 因果、接缝、金属 PBR 及人工审查仍待完成。
