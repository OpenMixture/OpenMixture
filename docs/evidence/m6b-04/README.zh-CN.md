# M6B-04 本地适配验证

[English](./README.md) | 简体中文

[PR #44](https://github.com/OpenMixture/OpenMixture/pull/44)实现 [CLI/浏览器资产契约](../../m6b-04-adapters.zh-CN.md)。[receipt.json](./receipt.json)将本地验证绑定到干净实现提交 `3fef6093f186e56b2d1311722ccb03711df69dd8`，后续文档提交不是被测运行时版本。没有发布包或合并 PR。

`cargo xtask check` 通过，包括独立源码及隔离归档消费。运行时契约 9 项、资格校验器 6 项、真实公共 WASM 全部 22 个 codec 用例通过。Windows Native DX12（NVIDIA GeForce GT 1030）通过 15 次 CLI 调用；精确干净 npm 归档在 Windows Chrome 通过全部 15 项，无跳过/重试。浏览器适配器仅报告 `BrowserWebGpu`，未暴露 GPU 名称，不能从 Native 结果推断其物理适配器。包/散装及 Native/browser 计划共享留存哈希；65×3 资产移动后渲染的全部 195 个 height 像素精确等于 `[128,128,128,255]`。未改变视觉语义或 golden 基线。

公共 WASM 检查测量一份 JS 快照与一份 Rust 副本：普通包 4,197,888 字节、总计 8,396,646 字节；最大资源包 67,118,592 字节、总计 134,240,169 字节，均含源/manifest 暂存。比测量预算少 1 字节均拒绝。这是实际缓冲容量账本检查，不是进程 RSS 或全尺寸 GPU 验收。普通 fixture 为一张 1024×1024 图，最大为八张 2048×1024 图（共 64 MiB 原始像素），均重复 `[128,37,91,255]`。复现时以受跟踪的浏览器资产源图为基础，为最大案例增加断开连接的 `image-input` 节点 `image1`…`image7`，资源 ID 为 `Input1`…`Input7`，用 `mixture asset pack` 打包，再将两个归档路径放在公共 WASM 验证器的包目录参数后。精确源/资源身份留存在 receipt。

捆绑 Chromium 尝试在启动进程时失败（`spawn UNKNOWN`），14 个页面测试均未执行，一个静态构建测试通过；独立安装 Chrome 的另一次运行通过。这不构成捆绑 Chromium 通过记录。完整失败/成功日志保留在忽略目录 `tmp/m6b04-browser-consumer` 与 `tmp/m6b04-browser-chrome`，不保证原始文件永久可用；源码测试和受跟踪 fixture 可复现新运行。

复现命令见适配指南。本次 Windows 浏览器设置 `MIXTURE_BROWSER_CHANNEL=chrome`；Native 的 ignored `cli_contract_gpu` 测试设置 `MIXTURE_GPU_BACKEND=dx12`、`MIXTURE_GPU_SOFTWARE=0`、`MIXTURE_CONSUMER_CLI` 为已构建可执行文件、`MIXTURE_CONSUMER_EVIDENCE_DIR` 为全新绝对目录，使用 Native 消费者独立 Cargo manifest。完整日志/归档/PNG 和大包均为忽略的本地产物，此摘要留存关键测量。记录时远程 CI 尚在运行；完整 M6B-05 材质/包验收、六项必需检查及发布仍属后续工作。
