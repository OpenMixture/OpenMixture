# ALPHA-03 数值差异排查 — 2026-09-17

[English](./README.md) | 简体中文

**这是修正参照环境配置之前的排查记录。** 本记录补充[首次失败记录](../alpha-03/README.zh-CN.md)，不改写其结果。[PR #12](https://github.com/OpenMixture/OpenMixture/pull/12) 提议显式半精度存储舍入，并将普通浏览器测试扩展到 Firefox。未修改容差或既有 golden；此阶段尚无普通配置通过全部 44 通道门槛。最大差异比例下降不能替代完整通过。

## 发现与处理

原始源码为干净的 `7b1cec4ad1d42d6269ef6a9912c2e8ba3a2dfdd9`，使用已保留的 Alpha 归档。在 Windows 11 26100／NVIDIA GT 1030 上，[节点隔离](./node-isolation.json)发现木材 baseColor 有 740 个变化像素，其中 729 个都是红色字节 133 → 132。[常量探针](./implicit-storage-probe.json)确认：原生 DX12 和普通 Chrome 对 `0.23265`、`0.2326660007238388` 均导出 132，对 `0.2326660305261612` 导出 133；[可精确表示的半精度输入](./exact-half-transfer-probe.json)在两端一致。这确认了存储舍入敏感性，但不能解释所有剩余算术差异。

尝试 `quantizeToF16` 内建函数后，该主机仍得到旧边界结果。保留的修复在唯一 WGSL 路径中使用整数位运算，写纹理前执行最近 binary16、平局取偶数舍入。零容差 constant-color 回归已通过；GPU 探针将 [-1,1] 所有 binary16 区间的有符号精确值、中点和相邻 f32 值，共 122,884 个输入，与独立 `half` crate 转换比较。改动前后的既有 DX12 节点夹具均通过。

该修复从干净的 `34a0db5a6b93fa58acb2c5aae2f803c5dde2023b` 构建。[候选目录](./rounding-candidate/)保留归档、生产／消费者身份与原生清单，归档 SHA-256 为 `52c21ab756c40ca19c353f91b929729251de3e560fc50559e80e9f89d515106b`；消费者仍为 Studio `56c510ab57daa1b68ef660525a648a582730a37e`。[固定原生 GPU CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35176953039)通过既有节点／材质 golden 与 2K 检查，[受控浏览器 CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35176953041)通过完整矩阵。CI 构建了自身绑定修订的包，不能证明另行构建的 Windows 归档已通过全部 ALPHA-01 消费者门槛。

另在 `c9226bf10eff5b03c3396af1fe9d427f1d5e8e48` 尝试显式 `fma` 插值，导致 [CI](https://github.com/OpenMixture/OpenMixture/actions/runs/35177471107)中未改动的木材 golden 失败；保留[错误输出](./rejected-interpolation-ci.log)及普通浏览器比较。该改动已由 `dfcf106` 撤回，不属于当前修复。原生 Vulkan 参照、原生着色器优化及 DXC 实验也未关闭门槛；它们是诊断实验，不是替换验收参照。

## 普通浏览器结果

下表均使用同源原生 DX12 参照、现有 11 个 1K 案例、四通道及原有冻结门槛。各目录保留完整比较和运行时／失败回执。

| 候选／浏览器 | 像素通道通过 | 最大变化像素比例 | 结果 |
|---|---:|---:|---|
| 显式舍入／Chrome 153.0.8010.48 | 31/44 | 0.0000438690185546875 | [失败](./rounding-chrome/comparison.json) |
| 已撤回插值／Chrome 153.0.8010.48 | 30/44 | 0.0000438690185546875 | [失败，已撤回](./rejected-interpolation/comparison.json) |
| 原候选／Firefox 156.0 | 32/44 | 0.0011339187622070312 | [失败](./firefox-original/comparison.json) |
| 显式舍入／Firefox 156.0 | 32/44 | 0.0010519027709960938 | [失败](./rounding-firefox/comparison.json) |

最大字节误差均为 1。Firefox 的陶瓷与木材全部通道通过，失败集中在皮革的 baseColor、height 和 normal。加载／渲染／自有输出／销毁与三个独立注入的诊断场景均完成，但完整材质验收仍失败。Chrome 在初次 `.37` 后经已安装浏览器更新变为 `.48`；回执记录实际版本，没有把二者当成同一环境。

Firefox 来自 Mozilla 签名的 Windows 正式版 MSIX，解压到忽略的工作区目录，使用新配置，没有改动 GPU 偏好。运行时适配器字段被隐藏，`isFallbackAdapter` 为 false；不能用主机 NVIDIA 清单替代被隐藏的选中适配器身份。WebDriver BiDi 能力记录将版本、非 headless 状态、配置目录及实际浏览器 PID 与 OS 实测启动器／子进程命令绑定；另保留仓库[启动器验证](./firefox-launcher.json)。初始尝试因启动器／子进程 PID 不符，以及不安全 `about:blank` 页面中的观察器错误被拒绝，不能计为通过。观察器现在容忍导航前缺失 WebGPU，但真实安全应用页面仍必须初始化成功。

## 复现与保留

遵循[普通浏览器指南](../../default-browser.zh-CN.md)。保留原始及显式舍入归档；已拒绝插值归档是临时产物，过期后不能只靠摘要逐字节恢复。Git 另保留完整失败比较／回执、代表性的原生／浏览器／差异图、数值探针结果及被拒绝的 CI 错误。其余逐案例 PNG、配置目录、诊断脚本、全量日志和构建树位于临时 `tmp/alpha03-*`。Firefox 可执行文件不在 Git 中再分发；启动回执记录分发包及执行文件摘要，最终启动器检查使用同一签名执行文件。未声明外部永久归档或全量运行永久可审计。

这些是开发排查，不是已接受的视觉改动。运行时归档／源码身份精确；部分诊断期间验证器处于未提交状态，未捕获其当时的完整文件摘要。PR 保留最终验证器实现，不声称每次历史运行都使用该最终实现。未来运行在开始时记录验证器／传输工具摘要；使用历史报告时必须保留此审计限制。

用户选择保留完整材质门槛并修正环境。后续[参照配置验收记录](../alpha-03-configured/README.zh-CN.md)记录了通过的 Firefox 运行。此前失败保持原样，不能据此声明 ALPHA-04 可发布。发布、容差调整、golden 替换、第二执行器和新节点仍不在范围内。
