# MAT-02 形态处理节点证据

[English](./README.md) | 简体中文

本记录绑定**干净源码 `ad70106fae586777597cf885268e1d19a66fe5d6`**，未发布 Rust 0.7.0 / browser 0.7.0-alpha.0。仅验证记录的 Windows GT 1030 形态处理节点，不接受涂漆金属材质、全部硬件、发布或完整 MAT-02。[PR #56](https://github.com/OpenMixture/OpenMixture/pull/56) 仍须通过全部六项检查。

[浏览器验收](./browser-qualification.json)使用精确[归档身份](./archive-receipt.json)通过全部 18 项测试。[192 组浏览器用例](./browser.json)覆盖四尺寸、四种固定支撑集合、两轴、两操作及半径 0/1/16。[Vulkan](./comparison-vulkan.json)与 [DX12](./comparison-dx12.json) 的每个计划哈希及 RGBA8 分量均精确一致（最大差 0）。[Native Vulkan 像素](./native-vulkan.json)与 [Native DX12 像素](./native-dx12.json)保留完整输出及适配器详情，重复渲染精确一致。[支撑集合图](./support-review.png)无滤波放大所选浏览器像素，仅供智能体检查，不是人工 PBR 决定。绑定[回执](./receipt.json)保留哈希与验收边界。

该源码的完整本地 `cargo xtask check` 已通过，包括打包消费。此前开发检查发现 ABI 夹具位于包外，以及 Clippy 固定块规则；两者在本源码前已修复。软件 [CI 运行](https://github.com/OpenMixture/OpenMixture/actions/runs/35826916523)因旧缓存数量断言 `seen.len() <= 12` **失败**，其中新节点探针已通过。留存 [stdout](./ci-failure.stdout.log)与 [stderr](./ci-failure.stderr.log)保留此区别。后续测试修复使用明确评审过的内核身份集合，代替过时数量。这些 Windows 像素不把失败 CI 改写为成功，后续测试／文档提交也不是本次被测源码。

在该源码运行 `node scripts/browser-runtime/build.mjs`，再用 `MIXTURE_BROWSER_CHANNEL=chrome node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh-browser-output>` 复现。分别显式设置 `MIXTURE_GPU_BACKEND=vulkan` 或 `dx12`、`MIXTURE_GPU_SOFTWARE=0`，运行 `node scripts/browser-runtime/check-morphology.mjs <fresh-browser-output> <fresh-comparison-output>`。原始半精度探针见[节点夹具](../../../fixtures/nodes/scalar-morphology/README.zh-CN.md)。PowerShell 用户通过 `$env:` 赋值设置环境变量。原始归档字节、编译产物及普通日志仍在忽略目录或会过期的 CI 工件；所选回执／像素／失败日志保留在此。不声明永久外部归档。
