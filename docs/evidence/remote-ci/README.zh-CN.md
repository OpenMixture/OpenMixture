# 远端 CI 验收

[English](./README.md) | 简体中文

**进行中，2026-09-12（Asia/Singapore）。** 已使用现有 `gh` 账号关联 `OpenMixture/OpenMixture`。CI 修复位于 `codex/remote-ci`。发布仍禁用，本轮不包含发布 tag 或合并。

## 已记录运行

| 版本 | 验证 | 结果 |
|---|---|---|
| `cda3f4b` | [首次 CPU 矩阵](https://github.com/OpenMixture/OpenMixture/actions/runs/34617436927) | Windows 无法替换运行中的 `xtask.exe`；Linux 通过；macOS 被后续运行替代。 |
| `9f6918a` | [CPU 矩阵](https://github.com/OpenMixture/OpenMixture/actions/runs/34618010038) | Linux／macOS 通过；Windows 到达包验证阶段，但拒绝了同一文件的另一种路径写法。 |
| `9f6918a` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34618010162) | smoke、打包消费、三种 1K 材质及 2K 跟踪通过。 |
| `b9d2ca5` | [CPU 矩阵](https://github.com/OpenMixture/OpenMixture/actions/runs/34619090585) | 三平台全部通过，包括隔离包验证。 |
| `b9d2ca5` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34619090576) | 并发节点测试进程因 SIGSEGV 退出；材质及 2K 步骤未运行。 |

[CPU 运行](./cpu-run.json)、[制品身份](./cpu-artifacts.json)、[外层包回执](./cpu-packages.json)和 [GPU 失败运行](./gpu-failed-run.json)保留精确版本及结果。GitHub 制品包含完整日志与报告，但有保留期限。包暂存路径属于原始 runner，成功运行的暂存树已清理。

## 修复与限制

- `9f6918a` 将 xtask 启动程序与工作区集成测试重新构建的可执行文件分开，保留 Windows 上的全部测试。
- `b9d2ca5` 比较规范化清单／目标路径，接受等价 Windows 特殊前缀路径，同时拒绝缺失文件、错误包身份和包目录外目标。文件系统回归覆盖规范化与普通路径，以及真实存在的外部目标。
- 后续修复串行运行独立 GPU 测试，保留全部测试及各测试内部的独立上下文检查。仅凭 Linux 信号无法确定驱动根因，也不能认证任意并发设备销毁。
- golden／trace 报告不再硬编码远端 CI 暂缓。描述性 `remoteCi` 字段指向实际 CI 运行／版本判定。原始报告保持不变，包括首次成功 Linux 运行中已过时的文字。
- 前两次完成的 Linux GPU 运行分别耗费约 16 和 29 分钟构建 SwiftShader。精确驱动构建缓存覆盖平台、工具版本和固定工作流／准备脚本。命中缓存后仍验证源码、配置／构建并运行全部验收；测试前即保存成功构建，不使用部分键恢复。

运行时 crate、shader、锁文件、十一节点词汇及已接受 golden 均未变化。CPU 通过和首次 GPU 成功不掩盖后续 SIGSEGV；串行测试必须远端通过后才能关闭剩余门槛。本轮不启动 M5，也不授权发布。
