# MAT-02 减法节点证据

[English](./README.md) | 简体中文

本记录绑定干净源码 `e7b25c4e36f6d7a2052fdba11d8cdfffad762a9d`，对应未发布的 Rust 0.7.0／browser 0.7.0-alpha.0。只验收记录的 Windows GT 1030 减法用例；涂漆金属验收、最终集成检查与发布仍分别处理。[收据](./receipt.json)绑定源码及保留文件哈希。

精确[候选归档](./archive-receipt.json)通过全部 19 项[浏览器消费测试](./browser-qualification.json)。64 组[浏览器用例](./browser.json)覆盖八对输入、四种尺寸及直接／放大输出。[Vulkan 对照](./comparison-vulkan.json)与 [DX12 对照](./comparison-dx12.json)最大分量差均为零，计划哈希及重复输出精确一致。完整 [Native Vulkan 像素](./native-vulkan.json)与 [Native DX12 像素](./native-dx12.json)保留适配器身份。放大用例验证 0.5 减 0.499755859375 在 RGBA8 转换前保留为 1/4096。这些算术探针不声明人工 PBR 认可。

本次纯证据更新前，本地 `cargo xtask check` 已通过，包含隔离包消费及 281 项文档链接检查。实现期间 Vulkan 定向探针通过；合入前置 main 提交后，在干净 `a49edb031f6381a1143c18b151a79493a613609a` 上补充的 DX12 原始半精度检查也通过。普通日志留在忽略目录中的 `tmp/subtract-check-initial.log` 与 `tmp/subtract-dx12-a49edb0.log`。最终 PR 检查只认证其自身精确 checkout，不替代这些较早候选字节的身份。

在绑定源码上执行 `node scripts/browser-runtime/build.mjs`，再设置 `MIXTURE_BROWSER_CHANNEL=chrome` 执行 `node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh-browser-output>`。分别设置 `MIXTURE_GPU_BACKEND=vulkan` 或 `dx12` 及 `MIXTURE_GPU_SOFTWARE=0`，运行 `node scripts/browser-runtime/check-subtract.mjs <fresh-browser-output> <fresh-comparison-output>`。[节点夹具指南](../../../fixtures/nodes/scalar-subtract/README.zh-CN.md)说明原始半精度检查。PowerShell 使用 `$env:` 设置环境变量。

选定原始 JSON 字节与完整像素保留在 Git 中。原始包字节、构建产物及普通日志仍在忽略目录或有限期 CI 制品中，不声明外部永久归档。本记录不替换历史[组合减法反例](../mat-02-subtraction/README.zh-CN.md)。
