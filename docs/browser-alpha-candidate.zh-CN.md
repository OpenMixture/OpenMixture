# 浏览器 Alpha 候选 — 2026-09-20

[English](./browser-alpha-candidate.md) | 简体中文

ALPHA-04 固定一个 `@openmixture/runtime@0.1.0-alpha.0` 交付归档。它是未发布的 npm 候选，不是注册表发行版。此前本地归档也使用同一版本文本，因此消费者必须通过摘要与构建身份识别本候选，不能只看文件名。

| 身份 | 值 |
|---|---|
| 干净生产者修订 | `82b74707b2a8a998190e2f28b16f91fb9614486a` |
| 构建 ID | `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28` |
| 归档 SHA-256 | `a9bcfe8d849f9fb9982a750dd99d026807fdf6453b5be491deeda90e99a2c6ae` |
| Runtime／API schema／Rust 引擎 | `0.1.0-alpha.0`／`1`／`0.1.0` |
| Studio 执行修订 | `6b2d53e3de16b21725b2a4359a2263f98671a6f9` |

归档来自通过的[当前候选材质运行](https://github.com/OpenMixture/OpenMixture/actions/runs/35489430243)，不是测试后重新构建的产物。生产者回执包含 Rust、Node、npm、wasm-bindgen 版本及完整包文件清单。[交付证据](./evidence/alpha-04/README.zh-CN.md)保留归档、回执、保存文件及比较。生产者和 Studio 修订之后的文档提交不冒充已测试的执行修订。

## 发布说明与兼容性

包包含公开 ESM facade、TypeScript 声明、生成绑定、WASM、匹配的构建元数据、双语 README 和两份许可证。安装不编译 Rust，不导入生产者文件，没有安装脚本或运行时 npm 依赖。公开入口仍为 `@openmixture/runtime`；WebGPU 初始化和释放均显式执行。

相对 Studio 原始 `4b914fe` 归档，本候选包含此前已整合的半精度纹理存储前显式最近偶数舍入，减少后端存储舍入差异；不承诺所有 GPU 最终像素相同。材质验收使用冻结的 v2 profile，棋盘规则保持精确，原生 golden 不变。未采纳的编译器／FMA 实验不包含在内。

不需要 `.mix` 迁移，API schema 未改变，也未增加节点或替代渲染器。升级须保留保存字节，替换完整归档，仅更新 runtime 锁完整性，通过 `npm ci` 重装，并在验收前核对实际 `getBuildInfo()`。禁止混用不同构建的 JS、声明或 WASM。Studio 验证已有样例和新创作文件的保存、独立 Player 重开及 PNG 导出；后续归档变化也必须执行完整验收。

## 已验收范围与剩余交付动作

本候选记录覆盖受控 Linux Chromium 包／契约／材质／生命周期／部署 CI、Windows Chromium Studio 保存文件与原生比较、普通 Windows Chrome 产品编辑／导出，以及独立 Linux／WSL 文件系统隔离消费者。准确浏览器、参数、适配器及结果见证据。此前普通 Chrome／Edge／Firefox 材质结果属于其记录中的旧归档，不能独立认证本归档。

支持路径为固定 Studio／Vite 消费者，在安全浏览器上下文中使用 WebGPU。GPU 获取失败时，无 GPU 的验证／编辑仍可使用，渲染返回结构化错误。未认证 Safari、移动端、任意打包器、未测硬件或普遍浏览器支持。Node GPU、CPU／WebGL 回退、新资源、高级 Studio 能力及 M6 不属于此次交付。

[ALPHA-05](./evidence/alpha-05/README.zh-CN.md) 已将两项浏览器检查加入四项原生必需检查，并回读确认远端生效。注册表发布和试用部署属于独立动作。发布前须重新核对注册表命名空间／版本与发行策略；只有分发决策完成后才能发布这份准确的已审查字节。之后，干净消费者须安装准确注册表版本，验证完整性／构建身份并重跑升级门槛。本地 tarball 安装成功不能关闭注册表消费门槛。ALPHA-04 不发布 Rust crate、发行标签或托管试用。
