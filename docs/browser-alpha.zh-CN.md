# 浏览器运行时 Alpha 交付收口

[English](./browser-alpha.md) | 简体中文

## 当前方向 — 2026-09-20

浏览器运行时与 Studio 材质比较共用冻结的 [v2 质量规则](./browser-quality.zh-CN.md)。当前报告只包含当前验收检查。原生软件金图及精确棋盘格契约不变。PR #19 已集成门槛重设计，后续清理从当前流程移除旧稀疏像素执行及未采用研究。

原生 M4／M4.1、有界 M5 和记录范围内的 Studio MVP 已完成。[npm Alpha 与准确注册表消费者](./evidence/npm-alpha/README.zh-CN.md)已交付，Rust crate 仍未发布；下一步不是 M6 或新增节点。

## 引擎负责的工作

| 工作项 | 状态 | 完成范围／下一步 |
|---|---|---|
| ALPHA-01 — 当前候选资格 | 已实现，PR #19 CI 已执行 | 每个交付候选仍须独立安装准确归档，通过构建身份、公开契约、材质、生命周期和部署检查。旧候选不认证新包。 |
| ALPHA-02 — 状态对齐 | 已更新 | 本页定义当前工作；历史运行记录只描述自身来源及环境。 |
| ALPHA-03 — 普通桌面浏览器资格 | 记录的 Windows 11／GT 1030 范围内完成 | [Chrome、Edge、Firefox 均通过 v2 的 11 案例／44 通道](./evidence/browser-quality-v2/README.zh-CN.md)，包含来源／适配器身份及生命周期证据。Windows 使用记录中的已有归档；新 PR 包另经 Linux CI 验证，不代表普遍硬件支持。 |
| ALPHA-04 — 可交付 npm Alpha 候选 | 准确候选及记录范围内的 Studio 升级验收通过 | [候选发布说明](./browser-alpha-candidate.zh-CN.md)固定归档与支持范围；[新执行证据](./evidence/alpha-04/README.zh-CN.md)记录 Windows 和隔离 Linux 各 7 用例／28 通道、52 项契约及普通 Chrome 保存／Player／导出。[后续发布与注册表消费](./evidence/npm-alpha/README.zh-CN.md)已针对同一字节通过。 |
| ALPHA-05 — 浏览器必需检查 | 已生效并验证 | [远端执行证据](./evidence/alpha-05/README.zh-CN.md)记录活动规则集 23016046 中的两项浏览器及四项原生必需检查，以及 main 有效规则和 PR 必需检查回读。保留严格基分支同步、GitHub Actions 来源绑定及无绕过主体设置。 |

## 审查对齐与认领 — 2026-09-20

提供的审查基于引擎 `f824cbf` 和 Studio `dcb2be7`。此后引擎 main 已通过 [PR #23](https://github.com/OpenMixture/OpenMixture/pull/23) 合入 `458d928`。[发布记录](./evidence/npm-alpha/README.zh-CN.md)已闭环发布授权、精确归档发布及干净注册表消费。本次规划更新依据保留证据，不声称重新查询注册表、检查线上部署或执行 GPU 测试。

| 优先级／工作项 | 负责人及状态 | 验收／下一步 |
|---|---|---|
| P1／ALPHA-06 — 精确 npm 交付 | 引擎；PR #23 已完成 | 已发布 `0.1.0-alpha.0` 保持候选说明中的 ALPHA-04 摘要和构建身份，保留授权及注册表回执。不重建或重新发布该身份。 |
| P1／ALPHA-07 — 注册表消费者交接 | 双方；记录范围内的执行已完成 | Studio 执行 `87ded9351e1c426e03aa7fb2b4c641f32399b85b` 绑定精确版本、锁完整性、安装文件及 `getBuildInfo()`。Windows 和隔离 Linux 各通过 52 项契约和 28 通道比较，并有普通 Chrome 产品门槛记录；不认证线上试用。 |
| P2／ALPHA-08 — 支持与精度范围 | 引擎；本次修改已文档化 | [候选说明](./browser-alpha-candidate.zh-CN.md)区分历史三浏览器覆盖、已发布归档覆盖及未测环境。保留 warp 精度限制，不标为已修复。 |
| 具体失败触发／ALPHA-09 — 消费者缺陷响应 | 引擎；条件性认领，尚未安排实现 | 提供包／构建身份、`.mix`、请求、浏览器／适配器及首个结构化错误或失败的当前门槛。最小案例复现、添加聚焦独立预期，变化归档重新验收后交接。 |

下一条交付链由 Studio 负责：确认注册表依赖集成，更新试用允许的运行时身份和范围，构建 `trial.json`，部署，核对线上 `getBuildInfo()` 和普通浏览器流程，再观察 3–5 名未参与开发的桌面用户。Studio 同时负责修正 `docs/external-trial.md`，区分历史 PR #12 失败、已验收 main 与线上版本。审查指出 `scripts/prepare-trial.mjs` 仍锁定旧摘要；应以已验收身份和范围更新保护，不能直接删除断言。这些是交接要求，不是本仓库已完成的工作；注册表验收不证明重新部署或真实用户试用完成。

后续引擎发行在内容变化时推进版本，冻结新归档身份，重做验收，再发布并验证干净环境的精确注册表版本消费。版本或字节改变后不能继承本次发行结论。保留记录中的 `alpha`／`latest` dist-tag 限制并使用精确版本；Alpha 不属于稳定发行。

不重新启动已完成的 M4／M5 验收、浏览器必需检查建设、Studio MVP 或已关闭研究的合并。保留已有检查，仅为明确的当前失败扩展工具。M6、新节点、子图、全量 3D 预览、GPU 零拷贝、第二后端、通用优化、市场与协作仍未排期。产品小改进依据真实用户卡点安排。

## 产品交接

OpenMixture 负责运行时候选、声明、诊断和包身份。Studio 负责打开／编辑／保存／Player 重开／导出，以及 GPU 获取失败时仍可使用的 CPU 编辑行为。联合升级证据必须绑定两个仓库版本及同一归档；只修改比较器不代表新 Studio 执行。

## 研究收口

PR #13–#18 作为未采用研究关闭。实验不改变交付着色器，也不再是发布前置条件。不能仅为不同浏览器末位对齐而引入编译策略、FMA 改写或新金图。后续数值工作需要具体的当前契约失败或消费者缺陷，以独立预期建立聚焦任务。

Git／PR 历史保留实验，当前导航和命令不依赖它们。参见[质量契约](./browser-quality.zh-CN.md)、[Studio 比较](./studio-qualification.zh-CN.md)、[包资格](./browser-materials.zh-CN.md)及[治理](./governance.zh-CN.md)。

[清理验证](./evidence/quality-closeout/README.zh-CN.md)记录浏览器精确重放及 Studio 迁移检查。
