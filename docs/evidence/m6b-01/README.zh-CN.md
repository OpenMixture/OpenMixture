# M6B-01 进入证据

[English](./README.md) | 简体中文

[测量记录](./measurement.json)来自 2026-09-22 Windows 上的 [CPU-only 可移植资产探针](../../../scripts/probe-portable-assets.mjs)，CLI 通过 main `c2a08e14be9991d75df24e0f045d842d8db4b570` 的 `cargo build --locked -p mixture-cli` 构建。测量时新探针尚未提交，因此工作区标记 dirty；运行时 crate、Cargo.toml 与 Cargo.lock 未改变。精确探针、二进制、源码和合成资源哈希已留存；`.mix` fixture 已入库，原始像素可由探针重新生成。

源码验证及无资源编译通过。在文档旁放入原始字节前后，高度/法线编译均报 `MIX_RESOURCE_MISSING`；报告保留原始结构化错误与退出码。这是预期基线限制，不是新增运行时检查失败。表示体积测量只证明 base64 膨胀，不构成性能、GPU 或二进制格式验收。

构建基线后运行 `node scripts/probe-portable-assets.mjs <fresh-output> [baseline-cli]`。没有引入产品打包命令。[阶段计划](../../m6b-portable-assets.zh-CN.md)区分要求、格式选择和实现。完整 CLI 构建/检查日志及生成原始文件留在忽略的 `tmp/`；不声称永久外部归档。留存 JSON、源码和探针足以检查本次 CPU 观测并复现新测量，但不能认证已不可获取的原始二进制。
