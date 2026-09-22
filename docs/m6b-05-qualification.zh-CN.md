# M6B-05 — 综合验收与收尾

[English](./m6b-05-qualification.md) | 简体中文

验收进行中，基于六项远程检查已通过的 M6B-04。本切片仅改变消费者测试/工具，保持包 v1、文档 v1、计划 v2、API schema 2 及 fractal-noise v1。Rust 0.5.0 / npm 0.5.0-alpha.0 仍未发布。M6-B 收尾必须有精确候选和六检查结果，不能仅靠测试定义。

独立 Native 消费者拥有 `tests/asset_support`：不变的 M6A 高度/法线图、1024×1024 冻结三角图案及 65×3 非对称对照（`R=(17x+71y)%256`，其他通道 19/201/0）。其 `asset-fixtures` 示例仅用公共 Rust codec 在临时浏览器消费者内写入两个单文件资产，不分发作者伴随文件。浏览器仅获取归档；散装参考源/像素由测试另行提供用于对照。

八个案例覆盖两种尺寸及 0/0.25/0.5/1 权重，输出 height/normal。同后端包/散装计划及每个像素须精确一致；每例重新装载/渲染两次，并在成功前拒绝畸形输入验证恢复。浏览器受理后的输入修改不影响结果。每次渲染须零存活单次 GPU 字节，拥有像素在销毁执行器后保持可用。Native 准备测试还销毁拥有的包并修改其原传输字节。M6B-04 移动文件 CLI、严格字节视图、busy/destroy 和包预算用例继续执行。

`cargo xtask gpu-smoke` 在源码及隔离规范 Cargo 归档两条路径执行新 Native 矩阵。候选浏览器资格检查现须 16 项，registry 保持 13 项。跨运行时验证须全部八例身份、16 张通道 PNG，最大分量差仍为 1。Windows 噪声限制与同后端包精确一致性分别记录，不放宽阈值或重置 golden。原 Native/browser 三材质及六项必需 CI 门禁仍全部必需；CI 对照使用固定 SwiftShader Native 和记录的 Chromium WebGPU 适配器。

```bash
cargo xtask check
cargo test --locked --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --all-features --test asset_qualification
# ignored GPU 测试须设置显式 backend/software 环境：
cargo test --locked --manifest-path examples/native-consumer/Cargo.toml --target-dir target/native-consumer --all-features --test asset_qualification -- --ignored --nocapture
# 需要干净源码版本及全新目录：
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/m6b05-browser
node scripts/browser-runtime/compare-assets.mjs tmp/m6b05-browser tmp/m6b05-comparison
```

Native 报告路径通过 `MIXTURE_ASSET_EVIDENCE` 指定。对照脚本设置 `MIXTURE_ASSET_BROWSER_DIR`，拒绝缺失/重复产物、身份不一致和任一数值门禁失败。浏览器结果留存包 SHA-256、计划哈希、构建/适配器/浏览器身份、分配报告、通道原始像素哈希及 PNG。普通完整输出存于忽略目录或保留 30 天的 CI 产物，简洁绑定收尾测量和必要视觉证据另行留存；复现产生新证据，不恢复原始产物。

发布、合并阶段 PR 栈、Studio 升级、NUM-01 噪声迁移及平台支持扩展属于其他任务。
