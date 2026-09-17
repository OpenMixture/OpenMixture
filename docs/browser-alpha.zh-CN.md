# Browser Runtime Alpha 交付收尾

[English](./browser-alpha.md) | 简体中文

## 当前状态与决定——2026-09-16

Native M4／M4.1 和 [M5 有界浏览器验收](./evidence/m5-05/README.zh-CN.md)已完成。独立 Studio MVP 也已通过[记录的 macOS 保存文件验收](./evidence/studio-qualification/README.zh-CN.md)。这些结果只认证记录的源码、归档及环境，不认证此后每次构建或默认浏览器配置。Cargo 包仍为未发布的 `0.1.0`；浏览器归档为未发布的 `@openmixture/runtime@0.1.0-alpha.0`。

尚无稳定版本兼容性保证；[兼容性记录](./compatibility.zh-CN.md)定义已测试契约。

引擎下一阶段定为 **Browser Runtime Alpha 交付收尾**，不进入 M6 或新一轮功能里程碑。本页统一维护当前状态与已认领的后续工作；带日期的 M5 记录继续作为历史证据。本计划采纳用户提供的 2026-09-16 双仓库审查中的引擎部分，其审查快照为引擎 `c41fcfb`、Studio `5138afc`。本地检查引擎 `c41fcfb` 确认下述工作流缺口；本次规划修改未独立复核审查中的远端 CI／保护状态。

## 引擎认领工作

以下是工作项 ID，不是 GitHub PR 编号。ALPHA-01 已有[候选验收工具](./browser-materials.zh-CN.md)，每个候选均须有通过的 CI 回执。ALPHA-02 文档已通过 [PR #10](https://github.com/OpenMixture/OpenMixture/pull/10) 合并；其余实现与验收仍待完成。后续每个实施 PR 按[治理规则](./governance.zh-CN.md)记录确切源码、真实 PR URL、检查与明确排除项。

| 工作项 | 优先级／状态 | 范围与完成标准 |
|---|---|---|
| ALPHA-01 — 当前候选包浏览器验收 | P1／已实现；每个候选须分别验收 | 构建当前引擎版本的真实 `.tgz`，由固定版本的独立消费者安装；通过公开声明检查、生产构建／部署、浏览器契约及既有 11 用例／44 通道材质比较。断言浏览器实际 `getBuildInfo()` 与候选身份一致。旧归档和混用 JS／WASM 必须失败。 |
| ALPHA-02 — 当前状态统一 | P2／已由 PR #10 合并 | README、Roadmap、SDK／构建指南、发布状态与导航指向本页，一致说明 M5／Studio 有界完成、软件包未发布、默认浏览器覆盖尚未验证。保留历史证据与带日期检查点。必须同步双语并运行仓库检查。 |
| ALPHA-03 — 默认桌面浏览器验收 | P1／已记录 Firefox 环境通过；待 PR 整合（[记录](./evidence/alpha-03-configured/README.zh-CN.md)） | 至少选择并记录一组目标桌面 OS／浏览器／版本，使用普通用户设置，不带 unsafe-WebGPU、忽略 blocklist 或强制软件适配器参数。同一候选完成加载、显式 GPU 初始化、渲染、自有输出及销毁，并验证结构化不支持／获取失败诊断。失败环境不能算作已支持：修复或收窄目标并记录失败。 |
| ALPHA-04 — 可发布 npm Alpha 候选 | P2／已认领，待执行；依赖 ALPHA-01／03 | 干净构建带明确版本的候选，包含完整 JS／声明／WASM、构建身份、归档摘要、变更说明、服务要求和实测支持范围。交付 Studio 做精确候选升级验收。发布单独执行；发布后独立安装注册表精确版本，复核身份及消费后才能宣称交付完成。 |
| ALPHA-05 — 浏览器必需检查决策 | P2／已认领，等待 ALPHA-01 | 候选链通过后读取实时分支保护／rulesets，评审是否将 `WASM and npm package`、`Chromium WebGPU material matrix` 与既有四项 Native 检查一起设为必需。若采纳，通过 PR 更新期望配置，应用并回读，保留实时证据。受跟踪的规则文件或通过的可选检查不能证明强制保护。 |

### ALPHA-01 实施边界

审查快照 c41fcfb 中，[打包工作流](../.github/workflows/browser-runtime.yml)构建新归档，[材质工作流](../.github/workflows/browser-materials.yml)却安装 Studio `56c510ab57daa1b68ef660525a648a582730a37e` 及其历史 vendor 归档。[原生准备脚本](../scripts/browser-runtime/prepare-materials.mjs)已有对 `crates`、`Cargo.lock` 和 `Cargo.toml` 相对归档运行时版本的漂移拒绝，必须保留。尚未覆盖的输入包括公开 JS 接口、声明与打包工具；两条工作流通过不代表本次完整包已在浏览器中运行。

通过绑定版本的 job 依赖连接生产与消费，或在同一工作流构建并消费候选。不能仅按“最近成功运行”选择 artifact。记录实际测试引擎 SHA（使用 PR 合并 SHA 时记录该 SHA）、干净源码状态、消费者提交／lock 身份、归档 SHA-256、build ID、JS／WASM 身份、原生参考身份和运行／attempt。独立比较浏览器实测构建信息与生产者 receipt。仅检查声明不足：实际返回值须与公开类型共同验证，包括 bigint 和自有像素结果。保留小型手写声明，不将代码生成作为前置条件。

消费者版本可以固定，安装的运行时必须换成本次候选。产品运行时消费不能依赖引擎源码、Rust 或开发者绝对路径；原生参考仅作为分离的比较输入。增加替换旧归档、错误构建身份及混合包组件的拒绝验证。失败必须传播，不能留下可复用的上次成功报告。冻结的浏览器容差与原生 golden 保持不变；从候选源码重新生成原生参考不等于允许重置已接受基线。按[证据政策](./evidence-policy.zh-CN.md)保留已验收内容。

## 两仓库交接

| 职责 | 所有者与交接内容 |
|---|---|
| 运行时候选 | OpenMixture 提供精确引擎版本、包版本、归档／摘要／构建 receipt、公开类型、支持限制、变更说明和验收结果。Studio 返回其精确提交／lock／归档身份及升级结果。 |
| 普通浏览器流程 | OpenMixture 负责运行时行为与诊断。Studio 负责打开／编辑／保存／Player 重开／PNG 导出，以及 GPU 不可用时仍有用的图查看和编辑行为。联合记录把双方版本绑定到同一候选及普通浏览器配置。 |
| 完整保存文件回归 | 运行时升级、保存逻辑变化及 Alpha 发布前，Studio 执行七用例／28 通道跨消费者验收。OpenMixture 提供分离的原生参考与比较工具。保留 Studio 保存的原始字节。普通 CSS 修改无需重复完整矩阵。 |
| 产品维护 | Studio 负责分支保护、Player／Studio 入口分离、编辑器模块小幅提取、公共部署和真实用户试用。这些是外部依赖，不是本引擎仓库的实现认领。 |

执行顺序：ALPHA-02 文档可先落地；ALPHA-01 闭合候选验证；然后 ALPHA-03 确立普通浏览器覆盖，同时 ALPHA-05 处理治理。ALPHA-04 绑定已验证候选及 Studio 升级结果。npm 预发布、精确注册表版本消费与产品公共试用随后作为独立交付动作记录。npm 交付不要求先发布 Rust crate、原生安装器或可嵌入 Player 包。

## 验证与停止规则

本次文档修改运行 `cargo xtask links` 和 `cargo xtask check`；缺少前置条件应报告为检查未完成。后续工作流／工具修改先跑针对性测试及 `cargo xtask check`，再要求实际测试版本的候选浏览器 CI 通过。既有构建与材质命令见[运行时指南](./browser-runtime.zh-CN.md)和[材质指南](./browser-materials.zh-CN.md)；本计划不增加任何已实现命令或新的已验收运行。

候选、独立消费者、普通浏览器支持声明与所需证据一致后，才可关闭交付收尾；必须明确发布状态。打包成功不能关闭浏览器验收，这些新增交付门槛也不重开历史 M5 验收。

M6、新节点、子图、资源容器、第二像素执行器、零拷贝 GPU 互操作、通用优化、N-API、3D 预览及市场／协作均不在本阶段范围。Core 所有权、显式 GPU 状态、`.mix` 语义和唯一 `wgpu` 路径不变。后续功能规划以真实消费者反馈或可测阻塞为依据。
