# 浏览器质量 v2：独立控制案例

[English](./README.md) | 简体中文

2026-09-20。本文是 [v2 工程规则](../../browser-quality.zh-CN.md)应用于浏览器前的冻结证据，不是浏览器支持收据。控制案例在将 v2 应用于任何浏览器材质输出前生成。阈值依据量化和响应预算选择，不依据浏览器通过率。

[calibration.json](./controls/calibration.json)保留全部 40 项预先规定的正负标签及测量，全部符合预期。目录保留全部原图/扰动/差异对照图，[control-sha256.json](./control-sha256.json)绑定确切字节。Agent 检查了代表性的法线平衡噪声、高度局部偏移和高度细节抹除图；不代表人工材质批准。测试只消费生成的 64×64 数组，不执行 `.mix` 或 GPU。

从仓库根目录复现：

```sh
cargo test --locked -p xtask browser_quality
cargo xtask browser-quality-calibrate tmp/quality-controls-new
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/default-browser.test.mjs
```

输出目录必须全新。Rust 测试另外覆盖法线/响应解析值、标量/Alpha 编码、方向偏转、尖锐粗糙度敏感性及比较对称性。一项低粗糙度单级扰动在幅度/偏移通过时，按预期未通过响应门槛，防止把 v2 理解为无条件接受单级差异。

完整仓库检查、冻结后浏览器比较、新 CI 资格及实际材质视觉审查将在完成后单独记录，不能由本合成控制推断完成。原生金图、夹具及历史容差字节不变。
