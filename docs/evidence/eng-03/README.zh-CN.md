# ENG-03 独立浏览器消费者证据

[English](./README.md) | 简体中文

2026-09-20，候选包和精确版本的公开注册表包分别通过全部 8 项浏览器检查，跳过、意外失败和不稳定结果均为零。两次运行均使用干净的消费者提交 `20ad93c2cfcc6e64d9acc1349275d76403972ca8`。后续证据／文档提交不是被测源码。[示例说明](../../../examples/browser-consumer/README.zh-CN.md)定义命令和覆盖范围；[PR #28](https://github.com/OpenMixture/OpenMixture/pull/28)记录集成及独立的 CI 结果。

| 运行 | 包的引擎修订 | 构建 ID |
|---|---|---|
| [候选包回执](./candidate.json) | `20ad93c2cfcc6e64d9acc1349275d76403972ca8` | `sha256:a30e066c9b44ce1f418681d95d6508b5f5582d0492db4249ecbf3fd390cad5f8` |
| [注册表包回执](./registry.json) | `82b74707b2a8a998190e2f28b16f91fb9614486a` | `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28` |

两个包均标识 runtime `0.1.0-alpha.0`，但归档和构建身份不同。候选包未发布。回执保留 UTC 时间、源码／验证器／包文件哈希、归档 SHA-256／SHA-512、工具版本、浏览器选项和完成数量。实际安装的精确依赖锁分别保留在 [candidate-lock.json](./candidate-lock.json) 和 [registry-lock.json](./registry-lock.json)。

## 执行与限制

这些是本地 Windows 11 x64 运行，显式选择 Chrome `153.0.8010.48`，使用 `--enable-unsafe-webgpu --ignore-gpu-blocklist`。浏览器报告返回 `BrowserWebGpu`，适配器名称／厂商／设备信息被隐藏，设备类型为 `Other`。无法获得硬件型号及原生驱动／后端；结果不能证明硬件矩阵或普通浏览器配置支持。CI 独立使用其配置的 Chromium／SwiftShader 环境。

测试覆盖无副作用导入与延迟 WASM 加载、纯 CPU 公共操作、`/consumer/` 下包相对 WASM 路径、结构化非法输入／GPU 不可用错误、精确棋盘／标量像素、输出通道选择及暴露参数覆盖、独立输出所有权、busy／closing／destroy 行为、示例页面，以及生产构建排除测试代码。保留的[候选渲染](./candidate-render.json)和[注册表渲染](./registry-render.json)包含实际适配器／限制、分配报告、计划哈希及两个 65×3 RGBA8 通道的全部字节，每通道 780 字节。精确期望值和请求保留在对应源码版本的浏览器测试中。

实现代理已目视检查[示例截图](./example.png)：成功页面显示黑白棋盘和均匀粗糙度预览、通道标签、控件，以及确认 GPU 所有权已释放的完成消息。这是 SDK 示例检查，不是人工材质质量验收。原有三材质 golden 和完整的固定 Studio 资格验证保持不变，仍为必需检查。

在上述验收运行之前，本地缓存的 Playwright Chromium 可执行文件在进程启动时报告 `spawn ...chrome.exe UNKNOWN`，runtime 尚未执行。该次尝试失败，不计入上述通过结果。新运行显式选择 Chrome，验证器没有静默回退。启动问题不构成修改像素容差或 runtime 行为的理由。

## 复现与保留

使用被测提交的干净检出、浏览器构建指南中的固定工具链和新的输出路径。在该 Windows 主机上，通过 `WASM_BINDGEN` 选择精确的 `wasm-bindgen` 0.2.128 可执行文件，再运行：

```powershell
$env:MIXTURE_BROWSER_CHANNEL = 'chrome'
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/sdk-candidate
node scripts/browser-runtime/consumer.mjs registry - tmp/sdk-registry
```

每种模式在检出目录之外的新系统临时目录中安装，验证安装字节，执行 TypeScript 检查、生产构建和 8 项浏览器测试，然后清理成功的暂存目录。注册表消费保留已提交的锁文件并使用新 npm 缓存，不继承候选包验收。本项工作没有发布包。

[files.json](./files.json)绑定保留的回执、锁文件、完整渲染测量和已检查截图。完整命令日志、Playwright 报告和归档属于普通本地／CI 临时输出，未在此持久归档。哈希可标识字节，但无法恢复已过期原件。保留内容支持此有限范围的验收；复现产生新运行，不是恢复原始产物。后续 CI 结果绑定各自的修订，不改写这些本地回执。
