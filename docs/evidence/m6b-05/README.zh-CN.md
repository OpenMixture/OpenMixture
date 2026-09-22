# M6B-05 综合验收

[English](./README.md) | 简体中文

M6-B 实现与验收已在**记录的 Linux 软件适配器矩阵内完成**。[验收记录](./acceptance.json)绑定 [PR #45](https://github.com/OpenMixture/OpenMixture/pull/45) head `92e1a5c7e671595eac57d3b62d5fbaa48d7c9e19`、被测集成提交 `189059f7eca172c8179f416f404f9418a08e3e1d`、六项成功必需检查及精确未发布 0.5.0-alpha.0 npm 归档。后续证据/文档提交不是这些像素的源码版本。阶段 PR 栈尚未合并，发布单独处理。

| 门禁 | 结果 |
|---|---|
| 候选 / registry | [候选](./candidate.json) 16/16；独立已发布 0.3.0-alpha.0 [registry](./registry.json) 13/13；无跳过、重试或失败 |
| 包/散装一致性 | 1024×1024、非对称 65×3、四权重及 height/normal：同运行时计划/像素精确一致，每例重新装载两次，失败恢复、销毁后输出所有权通过 |
| Native/browser | [资产对照](./assets.json)及[浏览器测量](./browser-assets.json)：16 个通道比较**最大差值全部为 0**，包身份与计划一致 |
| Native 源码/归档 | [源码](./native-source.json)、[隔离归档](./native-packaged.json)均运行八例，单次 GPU 存活字节为零；[四个 Cargo 归档](./cargo-packages.json)与移动资产 CLI 消费通过 |
| 原有回归 | [Scalar](./scalar.json)、[资源](./resources.json)、[三材质](./materials.json)的 11 案例/44 通道及[产品/部署验证器](./product.json)通过，未修改 Studio |
| 必需检查 | Linux/macOS/Windows CPU、固定 SwiftShader GPU/材质/归档/2K trace、WASM/npm、Chromium 矩阵全部通过，精确 job URL/时间留存于验收记录 |

取回 npm 归档并独立重算 SHA-256 为 `c1593834217d7a50e1a8abf72cf18937d6ec7350a45495755fbaaae17a1a8f81`，与[构建记录](./build.json)一致。Native 使用固定 SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`，浏览器 Chromium 153.0.8010.12 显式选择 SwiftShader。`.mix v1`、包 v1、计划 v2、API schema 2、fractal-noise v1 均未改变。CPU 平台检查不等于该平台 GPU 验收。

![打包高度/法线验收](./contact.png)

列为 0/0.25/0.5/1 权重，行为高度/法线。1K 面板精确抽样源像素 `(4x,4y)`，底部为非对称图像最近邻预览。Codex 已检查留存图：中间权重仍有周期导入结构，程序细节递增，高度/法线共同变化，端点非空。这是代理视觉检查，不冒称维护者批准。精确数值测试建立包一致性和方向性，预览不能单独证明；验收记录绑定 contact 和全部 16 张原始 PNG 哈希。未接受 shader/golden 变化。

Windows NVIDIA GT 1030 DX12 对安装版 Chrome 的**跨运行时法线一致性仍不合格**。[本次完整失败](./windows-failure.json)记录 1K 四权重最大差 0/1/4/8，65×3 全部为零。[Windows 候选](./windows-candidate.json)通过 16 项，包括该运行时内包/散装像素精确一致；Native 的同后端包/散装像素也一致。此失败复现既有程序噪声/后端限制，不证明具体驱动/编译器根因。≤1 门禁、源版本及[历史失败](../m6a-05/README.zh-CN.md)保持不变；NUM-01 修复和硬件扩展单独处理。

## 复现与保留

按[验收指南](../../m6b-05-qualification.zh-CN.md)和[浏览器 workflow](../../../.github/workflows/browser-materials.yml)，使用干净 checkout、全新输出目录、固定软件适配器及精确候选归档。执行 `cargo xtask check`、`cargo xtask gpu-smoke`、公共候选/registry 消费、`compare-assets.mjs`、Scalar/资源对照及原有材质/部署门禁。fixture 生成器归消费者所有，仅导入公共 Rust API。重新构建产生新记录，不能标成原始运行。

浏览器产物 `chromium-material-matrix` ID `10687810595` 与 GPU 产物 `swiftshader-material-evidence` ID `10688180687` 于 **2026-10-22** 到期，服务元数据/摘要已留存。Git 保留关键原始报告、已检查 contact、失败和源/包身份。完整 PNG、npm/Cargo 归档及常规日志仅在有限期 CI 和忽略的本地副本中，不构成永久完整运行归档；哈希不能恢复过期字节。不隐含通用平台保证、版本发布或 PR 栈合并。
