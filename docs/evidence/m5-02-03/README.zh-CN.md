# M5-02／M5-03 本地验收——2026-09-14

[English](./README.md) | 简体中文

**初始浏览器执行与隔离打包消费者门槛已在本地通过。** 独立产品的[验收记录](https://github.com/OpenMixture/Studio/blob/523caddd845e21762a160fcc26a45fb01eb0cec4/docs/evidence/m5-02-03/README.zh-CN.md)通过 [Studio PR #2](https://github.com/OpenMixture/Studio/pull/2)交付，保留十三项 Chromium 检查通过、零失败／跳过、可控真实设备丢失、映射失败清理，以及八个独立模块／设备与 32 次棋盘格渲染。干净的已测源码为 `d38bf68ddc470a33c608592363e3554f05887fe1`；证据提交单独标识。

产品消费未变更的 `@openmixture/runtime@0.1.0-alpha.0` 归档，来自干净引擎 `4b914feb9f3365d292b27ea60c5e0b6004f745e8`，SHA-256 `9e245578de160cee1050259ef34358c3fcd3353a21bbb1672d29701bc1ac8084`。引擎 `964a0784850a6993c226aae0ef894f5ed22c4fd7` 的运行时源码／构建输入未变。关闭这些初始门槛无需新增运行时代码或归档。

## 决定性浏览器证据

产品已提交的 `scripts/verify-isolated.mjs` 将干净产品提交归档，并在 macOS 拒绝访问原引擎及产品检出的条件下执行 `npm ci`、`npm run check` 与 `npm run test:browser`。负向探针证明拒绝生效且 PATH 中没有 cargo／rustc。全部步骤通过。包括 WASM 在内的生产资源服务于 `/player/`；公开构建身份与归档来源核对，数值投影与声明的 API 核对。

记录环境为 macOS 26.5.1 arm64、Node 24.20.0、npm 11.19.0、Chromium 153.0.8010.12，使用显式 `--enable-unsafe-webgpu` 与 `--ignore-gpu-blocklist`。浏览器报告 `BrowserWebGpu`，适配器身份被隐去。产品记录保留实际限制、参数、包／锁／夹具身份、测试结果与失败／清理附件。

在两次渲染之间及映射边界调用真实 `GPUDevice.destroy()`，浏览器交付设备丢失。首个映射失败保持 `MIX_READBACK_FAILED`；后续工作报告 `MIX_GPU_DEVICE_LOST`。读回清理、作用域平衡、跟踪的实时分配字节归零、像素保留及显式新设备执行均通过。单独的不对齐真实映射验证实际校验失败、后续成功渲染及失败渲染结束期间的释放。注入的适配器／设备获取失败仅作为分类覆盖。

## 引擎验证

下列命令均在干净引擎 `964a0784850a6993c226aae0ef894f5ed22c4fd7` 上退出零；后续文档变更在交付前另行执行仓库检查。

```bash
cargo xtask check
cargo xtask shader-check
cargo check --locked -p mixture-wasm --target wasm32-unknown-unknown
node --test packages/runtime/test/runtime.test.mjs
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 cargo xtask gpu-smoke
```

原生运行选择 Apple M5／Metal，doctor 健康、棋盘格精确一致，源码和打包 Rust／CLI GPU 消费者均通过。保留支持内容：[doctor](./native-doctor.json)、[棋盘格比较](./native-checker-comparison.json)、[原生丢失回归](./native-device-loss.json)与[smoke 结果](./native-gpu-summary.txt)。[摘要](./summary.json)绑定源码、命令和保留文件。原生证据不认证浏览器硬件。

## 验收边界与剩余工作

本记录关闭有界的 M5-02／M5-03 本地检查点，不代表完整 M5。可控销毁不等于自发硬件／驱动故障或标签页终止；显式创建新设备不是自动恢复。描述符核算不是物理 GPU 内存测量。完整材质 1K 回归、更广生命周期压力、Player 参数／通道／导出／调度、正式浏览器 CI 验收及兼容性评估仍属于 M5-04／M5-05。PR 集成继续要求现有原生必需检查。GitHub 检查结果与本地回执分别记录。

运行时仍未发布，本次不含网站部署或 Studio 编辑器。历史 M3／M4／M4.1 与初始浏览器记录未变更。关键精简结果及实际产品归档保留在 Git；完整常规日志与临时输出不永久保留。复现创建新的运行。
