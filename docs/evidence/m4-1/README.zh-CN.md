# M4.1 治理验收

[English](./README.md) | 简体中文

2026-09-12 已验收通过 [PR #1](https://github.com/OpenMixture/OpenMixture/pull/1) 合入的实施版本 `cc98dc9298add5ed172e0c1752d8a768b61a0eb5` 的仓库治理结果。[机器回执](./acceptance.json)绑定版本、运行尝试、保护设置和保留检查。承载本记录的后续文档提交具有独立的必需 PR 和主分支检查，不是下表运行测试的源码。

## 已验证变更

`main` 从已接受提交 `99704e8c6a05e9e0b60e4264aa2a5901fbb391c6` 创建，没有改写历史；初始主分支 CI 通过后被设为默认分支。历史分支均保留。生效规则集 `23016046` 要求 PR、解决审查对话、与基分支同步，并通过四项指定的 GitHub Actions 检查；禁止删除和非快进更新，未配置绕过主体。单维护者策略要求零个批准审查。[已提交规则集](../../../.github/main-ruleset.json)在验收时与实时 API 一致；[治理指南](../../governance.zh-CN.md)提供当前状态的验证命令。

PR #1 将 CI 触发范围调整为 PR、推送到 `main` 和手动触发，新附件请求保留 30 天。测试命令、任务名称、固定软件适配器和 GPU 串行执行均保留。当前文档与导航已同步英文和简体中文，包括区分历史实施批次编号与真实 GitHub PR 编号、证据保留规则及已测试平台限制。

## 已记录检查

| 来源 | CPU 工作流 | GPU 工作流 |
|---|---|---|
| 初始 `main`，`99704e8` | [通过](https://github.com/OpenMixture/OpenMixture/actions/runs/34676251027) | [通过](https://github.com/OpenMixture/OpenMixture/actions/runs/34676251035) |
| PR #1，报告的 head 为 `befb96b` | [通过](https://github.com/OpenMixture/OpenMixture/actions/runs/34676473145) | [通过](https://github.com/OpenMixture/OpenMixture/actions/runs/34676473224) |
| 集成后 `main`，`cc98dc9` | [通过](https://github.com/OpenMixture/OpenMixture/actions/runs/34677769615) | [通过](https://github.com/OpenMixture/OpenMixture/actions/runs/34677769619) |

每次 CPU 工作流检查 Linux、macOS 和 Windows。每次 GPU 工作流检查配置的 Linux 固定 SwiftShader Vulkan 工作负载：smoke、独立源码／包消费、三种 1K 材质以及最大材质的 2K 案例。回执记录 PR 运行中 GitHub 报告的 head SHA，不声称它就是 `actions/checkout` 实际检出的临时合并 SHA。

实施变更已通过本地 `cargo xtask check`。工作流比较确认仅修改触发／保留设置；四项必需检查名称和 GitHub Actions 来源与规则集匹配。实施提交和合并提交的 Git 树相同。与初始版本比较的 32 个受保护 Git 对象确认：运行时代码、依赖、工具、原始来源包、材质输入／基准、历史人工回执及绑定的视觉报告／图片均未改变。这是治理验收，不是新的视觉或平台验收。

## 保留与限制

本摘要、回执和治理配置保留在 Git 中。既有运行时与视觉验收证据按[证据规则](../../evidence-policy.zh-CN.md)保留在原有跟踪路径。这些普通重复 CI 运行的完整输出是临时内容：初始主分支附件按旧策略报告于 2026-12-11 到期；PR #1 附件按新策略报告于 2026-10-12 到期。集成主分支附件同样报告于 2026-10-12 到期。附件摘要是服务报告的元数据，不声称已下载并在本地验证附件内容。未声称存在持久外部归档。到期后仍保留已记录的门槛结果，但可能无法再检查原运行的完整输出。

未创建检查点标签。软件包仍未发布，保持 `publish = false`；M5 尚未启动。产品就绪程度及硬件限制继续由[发布记录](../../release.zh-CN.md)定义。
