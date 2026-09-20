# 仓库治理

[English](./governance.md) | 简体中文

M4.1 在原生 M4 验收后建立长期使用的集成分支、真实 GitHub PR、必需检查及证据保留规则。这是仓库维护，不是新的运行时里程碑，也不授权启动 M5。产品范围及兼容性继续由[路线图](../ROADMAP.zh-CN.md)和[发布清单](./release.zh-CN.md)定义。

## 分支与变更身份

使用 `main` 作为默认集成分支。新工作基于当前 `main` 创建 `codex/` 分支，必需检查通过后经 GitHub PR 合入。各项变更应可独立审查，保留有意义的实施提交。本规则不要求重写既有历史或删除旧分支。

[初始计划](../INITIAL_PRS.zh-CN.md)及 [M4 计划](../M4_PRS.zh-CN.md)中的历史标识 `PR-001` 至 `PR-015` 是实施批次编号，不是 GitHub PR 编号，不得将其改造成虚构的追溯审查。新工作应分别记录里程碑工作项、真实 GitHub PR 地址／编号及实施提交。例如，`M4.1-02` 是工作项，PR 编号由 GitHub 分配。

使用配对的 [PR 模板](../.github/pull_request_template.zh-CN.md)记录具体问题、里程碑工作项、设计边界、验证、风险及明确的不在范围内事项。修改公开行为和当前文档时同步简体中文版。[智能体指南](../AGENTS.zh-CN.md)继续约束实现及测试。

## 主分支保护

[main-ruleset.json](../.github/main-ruleset.json) 是可审查的目标分支规则配置，通过 GitHub 管理操作应用；仅提交此文件不会启用保护。规则是否执行以实时 API 为准。

- 仅匹配 `refs/heads/main`，禁止删除和非快进更新。
- 要求真实 PR，并解决全部审查对话。
- 当前单维护者流程要求零个批准审查，保留 PR 审查记录，同时避免依赖不可用的第二名审查者。不要求 CODEOWNERS 或最后一次推送由他人批准。
- 要求分支与基分支保持同步，并通过以下六项检查。
- 仅接受 GitHub Actions（`integration_id: 15368`）提供的检查，不配置绕过规则的主体。

| 必需检查 | 覆盖范围 |
|---|---|
| `Check (ubuntu-latest)` | Linux 上的锁定依赖仓库检查及隔离包检查 |
| `Check (macos-latest)` | macOS 上相同的 CPU 检查 |
| `Check (windows-latest)` | Windows 上相同的 CPU 检查 |
| `Pinned SwiftShader Vulkan materials and packaged consumption` | Linux 固定软件 GPU smoke、源码及打包消费者、全部三种 1K 材质及最大案例的 2K 跟踪 |
| `WASM and npm package` | 锁定依赖的 WASM 构建、JavaScript／包契约及准确 npm 归档生成 |
| `Chromium WebGPU material matrix` | 在固定 Studio 中安装准确归档、构建身份、浏览器契约、v2 材质、生命周期及生产部署 |

保持检查名称稳定。任务改名或检查来源变化时，必须协调验证规则；不得通过删除必需检查合并失败的变更。不要添加可能导致必需检查不报告结果的工作流路径过滤。规则修改本身也应经过审查，并在应用后记录实时结果。

## CI 触发与保留

[CPU](../.github/workflows/ci.yml)、[GPU](../.github/workflows/gpu-smoke.yml)、[浏览器包](../.github/workflows/browser-runtime.yml)及[浏览器材质](../.github/workflows/browser-materials.yml)工作流均响应 PR、推送到 `main` 及手动触发。普通功能分支推送不会额外启动一套分支 push 运行。合并后仍在 `main` 上运行，验证集成结果。现有按工作流／引用划分的并发控制取消已被取代的运行，不取消无关分支或 PR。

GPU 任务保留串行测试及固定 SwiftShader 构建缓存。缓存命中后仍验证源码版本、配置／构建驱动并执行每项验收。缓存状态不是测试通过的证据。

新上传的 CPU／GPU／浏览器证据附件请求保留 30 天。接受运行时记录服务实际报告的到期时间；仓库或服务限制可能缩短可用期。此设置不追溯改变已有附件。普通运行输出留在 CI 附件或忽略的本地目录；已接受的视觉内容、关键失败证据和摘要遵循[证据保留规则](./evidence-policy.zh-CN.md)，不得仅靠会到期的附件保存长期验收记录。

## ALPHA-05 浏览器强制检查 — 2026-09-20

[PR #22](https://github.com/OpenMixture/OpenMixture/pull/22) 将两项现有浏览器任务加入活动规则集 `23016046`。[保留的 API 证据](./evidence/alpha-05/README.zh-CN.md)包含原策略、准确更新、实时规则集、main 有效规则及 PR 六项必需检查。其他保护设置全部保留。分支端点的传统保护摘要可能显示空检查列表，而规则集仍在强制执行；应核对规则集及有效规则。

```bash
gh api repos/OpenMixture/OpenMixture/rulesets/23016046
gh api repos/OpenMixture/OpenMixture/rules/branches/main
gh pr checks 22 --repo OpenMixture/OpenMixture --required
```

下方四项检查的 M4.1 记录保留历史含义。浏览器 CI 强制检查不会发布或独立认证冻结的 ALPHA-04 归档。

## M4.1 启用与验证

**已于 2026-09-12 启用并验证：** 默认分支为 `main`，规则集 `23016046` 生效，[PR #1](https://github.com/OpenMixture/OpenMixture/pull/1) 合并为 `cc98dc9298add5ed172e0c1752d8a768b61a0eb5`，合并前后四项必需检查均通过。[完成记录](./evidence/m4-1/README.zh-CN.md)保留精确版本、运行、已应用策略及证据限制。下列顺序记录启用过程；后续状态通过实时 API 验证。

初始主分支来源为 `99704e8c6a05e9e0b60e4264aa2a5901fbb391c6`，其 [CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/34626312709) 和 [GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/34626312588) 运行均通过。从此既有提交创建 `main` 会保留完整验收历史，不使用旧默认分支制造追溯 PR。

按以下顺序完成启用：

初始主分支 CI 运行期间可准备草稿 PR。完成下述默认分支及保护验证后，才能标记为可审查并合并；创建草稿不豁免合并门槛。

1. 重新核对远端引用及来源 SHA。若本地 `main` 是其祖先则快进，非强制创建尚不存在的远端 `main`，保留旧分支。
2. 等待该来源版本在 `main` 上的四项检查通过，再将 `main` 设为默认分支。
3. 检查已有规则集，应用或更新匹配的目标规则，并读回核对，避免重复规则集。
4. 通过真实 PR 提交工作流及文档变更，验证 PR 当前版本的必需检查，并经正常保护路径合并；随后验证 `main` 的运行。
5. 保留精简完成回执，记录初始来源／主分支／PR 版本、运行结果、规则集 ID／内容、默认分支及任何检查点标签。记录这些实时检查前，本地配置不证明启用已完成。

只读验证使用现有已登录的 `gh` 账号：

```bash
gh repo view OpenMixture/OpenMixture --json defaultBranchRef
gh api repos/OpenMixture/OpenMixture/branches/main
gh api repos/OpenMixture/OpenMixture/rulesets
gh run list --repo OpenMixture/OpenMixture --branch main
gh pr list --repo OpenMixture/OpenMixture --state all
```

声称所选 PR 完成前，还需检查当前 head、合并状态、必需检查及合并后的提交。保留精确运行尝试及版本；较早版本的检查通过不能认证后续编辑。

可用描述性的 M4 检查点标签标识已接受的初始提交。它是源码检查点，不是软件包版本或发布；不得将已有标签移动到其他内容。软件包仍未发布，保持 `publish = false`。M5 进入、新节点、硬件支持承诺和分发仍需单独决定。
