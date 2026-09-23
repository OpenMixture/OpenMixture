# MAT-01 工具集成 — 材质接受待定

[English](./README.md) | 简体中文

[PR #52](https://github.com/OpenMixture/OpenMixture/pull/52) 于 2026-09-23 03:56:42 UTC 合入，提交为 `0611d7273e368b12628bc827627779ea338dbb92`。两个节点已通过 PR #50–51 集成；本次集成公开消费者验收工具及留存候选证据。人工视觉接受仍待定，不启动 MAT-02 实现，也不发布包。

## 被测源码与门槛

PR head `1dfb879b86998f038841931a6297676221d7ac4d` 的六项合并前检查全部通过：[三平台 CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446565)、[软件 GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446393)、[WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446500) 和 [Chromium](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446555)，均为 attempt 1。浏览器／运行时实际使用干净的合成合并提交 `9345ac22d4da7dc6d55a2d62adc295b12b3dd501`，不能与 PR head 或后来的集成提交混同。

[SDK 回执](./sdk-qualification.json)、[原始材质绑定](./browser-qualification.json)和[显式 noise-v2 绑定](./noise-v2-qualification.json)全部通过。它们绑定未发布归档 SHA-256 `302b5220c66b86e3e5102bae5d54f705c8c678c64c020e8924333d07a72f6181` 及运行时构建 `sha256:a4dbb21caada8e9d11260132cfd2c46f3b3aa394bc07115c17b97f478e29366f`。此前[脏源码失败](../ci-failure-4bf/README.zh-CN.md)仍保留失败结论；新的成功绑定验证了 Cargo 输出路径修复，未削弱干净源码检查。

[砖材质比较](./brick-comparison.json)及其 [Native 矩阵](./brick-native.json)保留二十个用例、八十通道对照：最大分量差为 0，重复渲染与包往返全部精确一致。这是记录的 Linux 软件／Chromium 范围，不新增硬件认证。两份回执均保留 `materialAccepted: false`。独立来源的 [Windows 证据及人工评审图](../README.zh-CN.md)继续保留原始源码身份与待定的人工决定。

合并后的 main 检查独立记录，准备本记录时仍在运行：[CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332292)、[GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332264)、[WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332218)、[Chromium](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332263)。合并前结果不代表这些运行已通过。

## 留存

浏览器产物 `chromium-material-matrix`，ID `10716612741`，大小 182,294,555 字节，SHA-256 为 `501782b8f78ce138513911eace06caf9f9f10903c07eecbaf5721d6bcd771dea`，服务到期时间为 2026-10-22 19:55:15 UTC。已于 2026-09-23 下载并核对摘要。上述选定回执逐字节复制到 Git；完整日志、归档字节、重复 PNG 及其他报告仍为临时本地产物／CI 产物。回执中的哈希链接不承诺完整包在过期后仍可获取。本记录支持工具集成和所述测量，不承诺长期复查全部 CI 像素。已有人工评审像素继续单独留存。

按[夹具指南](../../../../fixtures/materials/brick-paving/README.zh-CN.md)及被测提交的工作流复现。新运行使用新的源码／归档身份。本记录不改变格式、节点语义、迁移策略或发布状态。
