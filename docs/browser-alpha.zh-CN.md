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

## 产品交接

OpenMixture 负责运行时候选、声明、诊断和包身份。Studio 负责打开／编辑／保存／Player 重开／导出，以及 GPU 获取失败时仍可使用的 CPU 编辑行为。联合升级证据必须绑定两个仓库版本及同一归档；只修改比较器不代表新 Studio 执行。

## 研究收口

PR #13–#18 作为未采用研究关闭。实验不改变交付着色器，也不再是发布前置条件。不能仅为不同浏览器末位对齐而引入编译策略、FMA 改写或新金图。后续数值工作需要具体的当前契约失败或消费者缺陷，以独立预期建立聚焦任务。

Git／PR 历史保留实验，当前导航和命令不依赖它们。参见[质量契约](./browser-quality.zh-CN.md)、[Studio 比较](./studio-qualification.zh-CN.md)、[包资格](./browser-materials.zh-CN.md)及[治理](./governance.zh-CN.md)。

[清理验证](./evidence/quality-closeout/README.zh-CN.md)记录浏览器精确重放及 Studio 迁移检查。
