# ALPHA-05 — 浏览器必需检查

[English](./README.md) | 简体中文

2026-09-20，[PR #22](https://github.com/OpenMixture/OpenMixture/pull/22) 将 `WASM and npm package` 和 `Chromium WebGPU material matrix` 加入四项原生必需检查。更新既有活动规则集 `23016046`，没有重复创建规则集或绕过保护。本记录认证远端策略生效，不代表软件包发布或扩大浏览器支持。

## 保留证据

- [启用回执](./activation.json)：时间戳、基准 main 版本 `507852c59b69708b122fe24bdc240c68983bf84d`、六项成功的基准检查、来源应用 ID 及任务 URL。
- [修改前](./before-ruleset.json)、[准确更新](./applied-ruleset.json)、[实时回读](./after-ruleset.json)及 [main 有效规则](./effective-main-rules.json)：除新增两项必需检查外，保留全部控制。严格基分支同步、GitHub Actions 应用 `15368`、PR／审查对话要求、禁止删除／非快进及无绕过主体继续生效。
- [PR 必需检查快照](./pr-required-checks.json)：启用后立即对 head `d09fb048c529a9b9113d0065275a66d4a5a64d7d` 执行 `gh pr checks 22 --required --json name,state,link,workflow`。六项均在运行，且已被归类为必需检查。这证明强制执行，不声称该快照已通过。

现有浏览器工作流对每个 PR、main 推送及手动触发运行，没有路径过滤。名称和运行时负载不变。浏览器检查失败或缺失现在会阻止正常受保护合并。没有故意创建失败 PR、直接推送受保护分支或尝试绕过。有效规则及 GitHub 必需检查分类是强制执行证据。分支端点的传统保护摘要可能显示空检查列表；这不会覆盖活动仓库规则集。

## 复现及边界

在仓库根目录运行 `node docs/evidence/alpha-05/verify.mjs`，进行快照／配置／工作流一致性定向检查，然后运行 `cargo xtask check`。另行验证当前远端状态：

```bash
gh api repos/OpenMixture/OpenMixture/rulesets/23016046
gh api repos/OpenMixture/OpenMixture/rules/branches/main
gh pr view 22 --repo OpenMixture/OpenMixture --json headRefOid,mergeCommit,state
gh pr checks 22 --repo OpenMixture/OpenMixture --required
gh run list --repo OpenMixture/OpenMixture --branch main
```

PR 最终 head 和合并后的 main 均需新的六项检查结果；较早快照不能认证后续版本。PR／CI 记录标识这些交付结果。完整普通日志及生成软件包保留在忽略的本地产物或请求保留 30 天的 CI 附件中；本目录在 Git 中长期保留精简策略证据。本次无需也不声称新的视觉验收。既有 ALPHA-04 归档、运行时、着色器、黄金像素、Studio 源码及历史记录不变。注册表发布及准确注册表版本消费仍为独立动作。
