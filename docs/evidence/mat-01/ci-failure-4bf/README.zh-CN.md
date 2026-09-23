# MAT-01 CI 源码洁净性失败

[English](./README.md) | 简体中文

浏览器运行 [35766417215，第 1 次尝试](https://github.com/OpenMixture/OpenMixture/actions/runs/35766417215)在 PR head `4bf5e9b65d2153aa2fe8ef46459839b4a7e6a8f3` 上失败；实际 CI 检出／运行时修订为合成合并 `2d1ebf0151c63d36107ba3f81788b61082c5af48`，见留存的 [Native manifest](./native-manifest.json)。候选 SDK、Scalar／资源／包／砖材质对照、已安装产品契约、原材质质量及部署检查通过。最终候选绑定因 `native.engineDirty === true` 拒绝验收；后续显式 noise-v2 矩阵被跳过。这是整体浏览器验收失败，不是阶段接受。

## 原因与修复

新增砖材质对照调用独立 Cargo 工作区时遗漏 `--target-dir`，生成了未跟踪的 `examples/native-consumer/target/`。随后 Native 准备记录了脏状态。开发检出的私有 `.git/info/exclude` 条目掩盖了该路径；仓库 `.gitignore` 明确负责 `/target/`，不包含这个嵌套路径。xtask 砖材质入口也有同样遗漏。

两个入口现在均显式使用 `--target-dir target/native-consumer`，与既有独立消费者命令一致。不删除脏源码断言、像素容差、候选身份检查或必需检查。夹具手动命令使用同一路径及全部所需 feature。

[consumer.test.mjs](../../../../scripts/browser-runtime/consumer.test.mjs) 的回归测试创建全新 Git 仓库，只使用受跟踪的根 target 忽略规则，清空私有／全局排除配置，并通过真实砖材质编排调用一个无依赖的小型 Rust 消费者。修复前最终状态断言因 `?? examples/native-consumer/target/` 失败，修复后工作区保持干净。替身只替换 GPU 工作，因此该测试证明构建输出位置，不证明材质像素。全部十九项 Node 编排测试通过。实际 Native 材质、完整仓库检查及当前修订 CI 仍分别必需。

## 留存与范围

[首次失败步骤日志](./failed-step.log)保留原始绑定错误。原 CI 产物为 `chromium-material-matrix`，ID `10714115961`，128,635,403 字节，服务端摘要为 `sha256:6e472c1cf771d4014fd5f8083394dd5269e13e09fe499631d017bd6416350a64`，服务端到期时间为 2026-10-22 18:51:54 UTC。它包含通过的中间砖材质对照及失败的最终绑定。完整普通输出是临时内容；本记录不承诺永久保留完整归档，也不把跳过的迁移检查算作通过。后续修复不会将本次失败重写成成功。
