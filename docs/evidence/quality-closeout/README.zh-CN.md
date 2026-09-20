# 质量规则收口 — 2026-09-20

[English](./README.md) | 简体中文

PR #19 已合并 v2 质量规则。维护者随后要求从新项目的当前流程移除已替代规则和研究。浏览器报告改为 schema 3，仅有三项当前门槛；Studio 报告为 schema 2，共用相同冻结规则。着色器、原生金图、材质输入、规则数值及指标实现均未改变。

[验证摘要](./summary.json)绑定已有的完整浏览器及 Studio 数据集。离线重放精确复现全部 132 个浏览器通道的 v2 指标、结构／因果及案例关联结果；Studio 全部 28 个通道通过共用规则及不变的保存文件检查。这是对记录像素的新比较，不是新浏览器执行或交付候选资格。原报告保持原始结果。

聚焦验证包含六项质量指标测试、两项 Studio 测试及九项 JavaScript 候选／普通浏览器测试。Studio 回归证明：四个通道中，材质预算允许的一级变化仍被精确棋盘格规则拒绝。候选测试拒绝报告 schema 1／2、不完整矩阵和缺失／失败的当前门槛。集成仍要求完整仓库检查及当前包浏览器 CI。

使用[完整解码纹理验证器](../browser-quality-v2/verify.py)重建浏览器数据，然后对三个浏览器目录分别运行 `cargo xtask browser-material-check`。按 [Studio 证据记录](../studio-qualification/README.zh-CN.md)中检查摘要的代码重建 88 个文件，对 native／player 目录运行 `cargo xtask studio-material-check`。使用新输出目录，保留原记录不动。浏览器像素重建需要 Python NumPy／Pillow。聚焦测试：

```sh
cargo test --locked -p xtask browser_quality
cargo test --locked -p xtask golden::studio
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/default-browser.test.mjs
cargo xtask check
```

PR #13–#18 已关闭，未采用其着色器或兼容性提案。删除的早期 ALPHA-03 实验及旧容差文件仍可从提交 `efdac411198bfa87b7eba7ff9e588330ec08baf9` 获取；保留的历史链接指向该固定版本。当前代码不再依赖这些文件。[当前收口计划](../../browser-alpha.zh-CN.md)列出后续 Alpha 包交付及浏览器分支保护决策。
