# 普通桌面浏览器配置验收——ALPHA-03

[English](./default-browser.md) | 简体中文

本流程在普通 GPU 设置下验证特定浏览器／OS／驱动与运行时归档，独立于受控 Chromium CI 和 Alpha 发布。[验收记录](./evidence/alpha-03-configured/README.zh-CN.md)定义实际支持范围；浏览器版本本身不代表通用硬件支持保证。

## 输入与启动规则

使用已经通过 ALPHA-01 的确切归档、生产者回执及 SHA-256。按[候选验证流程](./browser-materials.zh-CN.md)安装到固定的独立消费者，执行 `npm ci` 并核对安装字节。产品原始提交和其他 lock 条目保持不变。保留的候选可能早于后续文档或验证工具提交，不能将后者称为已测运行时。

Windows [启动器](../scripts/browser-runtime/launch-default-browser.ps1)使用新的临时配置目录启动已安装的桌面浏览器，只增加 `--user-data-dir`、回环 CDP 端口及 `about:blank`。不添加 headless、unsafe-WebGPU、忽略黑名单、ANGLE／软件适配器、沙箱或功能覆盖参数，不修改个人配置、驱动或浏览器设置。CDP 只是测试通信方式，不是最终用户的前置条件。[验证器](../scripts/browser-runtime/default-browser.mjs)同时核对 OS 实测命令行和浏览器自身报告的命令行，只规范化 Chromium 的空 flag-switch 标记和首尾空白。额外参数或复用启动目录都会被拒绝。

记录 OS／构建、已安装浏览器版本／可执行文件摘要、完整启动参数、浏览器 GPU 信息及运行时可见的适配器证据。被隐藏或不可用的字段保持原样。观察用 `requestAdapter` 包装器原样转发选项并返回原适配器；失败注入仅发生在独立页面，并明确标为合成情形。

## 复现

消费者需要其固定版本的 Node／npm／Playwright 依赖，环境需要真实桌面浏览器和桌面会话；Linux headless CI 不能替代本机验收。

1. 按材质指南用 `candidate.mjs stage` 准备候选消费者及 `candidate.json`，执行 `npm ci` 和 `candidate.mjs installed`。
2. 在目标主机上从同一运行时实现生成新的原生参考。显式选择原生适配器／后端策略，确认实际适配器，并保留相对候选引擎版本的源码漂移检查。记录的 Windows NVIDIA 主机使用：

```powershell
$env:MIXTURE_GPU_BACKEND = 'dx12'
$env:MIXTURE_GPU_SOFTWARE = '0'
$env:MIXTURE_GPU_EXPECT_ADAPTER = 'NVIDIA GeForce GT 1030'
node scripts/browser-runtime/prepare-materials.mjs tmp/default-native 7b1cec4ad1d42d6269ef6a9912c2e8ba3a2dfdd9
./scripts/browser-runtime/launch-default-browser.ps1 -OutputDirectory tmp/default-launch
```

3. 在独立消费者中执行 `npx vite build --mode browser-test` 构建测试入口，然后在单独终端执行 `node scripts/static-server.mjs`。静态服务器只在 `http://127.0.0.1:4173/player/` 提供产品 `dist`。本记录使用回环地址安全上下文；公共部署仍须满足[浏览器指南](./browser-runtime.zh-CN.md)中的服务与安全要求。
4. 从引擎检出执行，使用绝对路径及新输出目录：

```bash
node scripts/browser-runtime/default-browser.mjs run /absolute/product /absolute/candidate-evidence /absolute/native-reference /absolute/launch/launch.json /absolute/new-browser-output
```

运行时门槛检查真实构建身份、全部 11 个既有 1K 用例／四通道、稳定计划身份、回读后报告的 live allocation 为零、后续渲染及销毁后仍有效的自有像素、幂等销毁、已接受工作的完成及销毁后新工作的拒绝。既有冻结 `browser-material-check` 执行结构、因果性、关系及像素容差检查。验证器不会修改原生 golden、容差或旧报告。

5. 在产品中执行 `npm run build`，用正常生产资源替换测试资源，保持静态服务器运行，再执行：

```bash
node scripts/browser-runtime/default-browser.mjs production /absolute/product /absolute/candidate-evidence /absolute/launch/launch.json /absolute/new-production-output
```

此步骤检查 Player 显式初始化／渲染／销毁、65×3 棋盘格 PNG 下载的独立解码 RGBA 精确字节和 sRGB 元数据，以及生产构建不包含测试入口。保留 PNG、截图和回执。收集证据后只关闭验收专用的浏览器／配置，不关闭个人浏览器会话。

## Firefox 与原生参照配置

启动 Firefox 时传入 `-Family firefox -BrowserPath <firefox.exe>`，只增加新配置目录、回环 BiDi 端口和空白页。验证器将版本、非 headless 状态、配置目录及实际子进程 PID／命令行与启动记录绑定。`run` 支持 Firefox；`production` 的实际下载检查目前仅支持 Chromium，不能据此声称 Firefox Player 下载已验收。

成功获取 GPU 并渲染，但逐像素比较失败，不等于设备不支持 WebGPU。原生参照还依赖构建模式及实际着色器编译器。`prepare-materials.mjs` 默认保留 `debug`；可显式设置 `MIXTURE_NATIVE_PROFILE=release`。Windows DX12 可用 `MIXTURE_DX12_COMPILER_DIRECTORY` 指定包含 `dxcompiler.dll` 的目录，仅前置到原生子进程 PATH。清单记录构建模式、DLL 请求路径／摘要、脚本及可执行文件摘要；另需记录实际加载的模块，不能把 PATH 配置本身当作加载证明。这些是开发验收参照工具设置，浏览器用户无需配置 Rust 或 DXC。

```powershell
$env:MIXTURE_NATIVE_PROFILE = 'release'
$env:MIXTURE_DX12_COMPILER_DIRECTORY = '<经核验的 Firefox 目录>'
node scripts/browser-runtime/prepare-materials.mjs tmp/configured-native <候选引擎完整提交>
```

继续使用原有后端／适配器要求和源码漂移检查，为每次参照与浏览器运行创建新目录。保留先前失败、golden 和容差。结果仅证明记录的配置配对，不能推导所有编译器逐位一致。

## 失败与证据边界

负向页面分别注入 WebGPU 缺失、适配器返回 null、设备创建拒绝，必须保留 `MIX_BROWSER_WEBGPU_UNAVAILABLE`、`MIX_GPU_ADAPTER_UNAVAILABLE`／`gpuAdapter`、`MIX_GPU_DEVICE_REQUEST_FAILED`／`gpuDevice`。引擎失败保留可操作建议，CPU 文档验证仍可使用。这些是受控诊断探针，不是自然不支持主机的覆盖，也不是备用像素执行器。

失败运行保持原样。将硬件浏览器与不同软件／硬件适配器生成的参考比较，是独立的可移植性实验，不能悄悄重新定义同机浏览器／原生验收。保留这类失败比较，并为目标主机配对使用新的、源码匹配的参考与输出目录。不能通过放宽容差关闭门槛。

三项 Node 守卫测试在浏览器 CI 中运行，但不在那里启动或认证普通桌面浏览器。修改验证器后运行 `node --test scripts/browser-runtime/default-browser.test.mjs` 和 `cargo xtask check`。后续候选、OS／浏览器／驱动变化、其他浏览器、自定义配置／扩展／策略、公共部署及完整 Studio 保存文件流程，均需单独证据。已接受记录、临时完整输出及归档过期遵循[证据保留政策](./evidence-policy.zh-CN.md)。
