# M4 发布状态与清单

[English](./release.md) | 简体中文

**砖块图案候选：** Rust 0.6.0、浏览器 0.6.0-alpha.0 增加新的穷尽 plan 枚举及节点目录。两者均未发布；已发布浏览器仍为 0.3.0-alpha.0。源基线已集成 M6-B 与 NUM-01。[契约与必要验收](./brick-pattern.zh-CN.md) 定义本增量。下文旧证据不验收此候选。

**M6B-05，2026-09-22：** [可移植资产验收](./evidence/m6b-05/README.zh-CN.md)在记录的 Linux 软件矩阵内完成：源码/归档与浏览器消费、16 个包精确对照、原材质及六检查全部通过。Windows 硬件法线一致性仍未合格。0.5.0 / 0.5.0-alpha.0 未发布，集成和发布单独处理，下方早期状态保留为历史。

**当前发布（2026-09-22）：** [浏览器 0.3.0-alpha.0](./evidence/npm-030-alpha/README.zh-CN.md)已从通过六项检查的 main 归档发布。API schema 2 / 计划 v2；Rust crate 尚未发布。Windows 硬件资源法线一致性仍未通过，不扩大支持范围。

**M6A-05 验收，2026-09-21：** [留存验收](./evidence/m6a-05/README.zh-CN.md)关闭记录的 Linux 软件矩阵内综合验收：八组资源通道逐字节一致，Scalar、三材质回归和六项必需检查通过。Windows 硬件一致性仍失败，不属于已验收范围；≤1 门槛不变。0.3.0 Rust 源码 / 0.3.0-alpha.0、API schema 2 浏览器候选均未发布。实现 PR 集成和发布仍是独立动作。

**历史检查点（2026-09-20）：** 原生 M4／M4.1、有界 M5 和记录范围内的 Studio MVP 已验收。普通 Windows Chrome／Edge／Firefox 均通过记录中已有归档的 [v2 材质矩阵](./evidence/browser-quality-v2/README.zh-CN.md)。[npm Alpha 发布与准确注册表消费](./evidence/npm-alpha/README.zh-CN.md)已在记录范围内完成。Rust crate 仍未发布。[Alpha 收口](./browser-alpha.zh-CN.md)记录交付及浏览器强制检查；托管试用重新部署仍为独立动作，不启动 M6。

以下带日期的检查点保留当时状态，不作为当前未完成项清单。

**M5 验收，2026-09-15：** [浏览器验收记录](./evidence/m5-05/README.zh-CN.md)关闭记录的 macOS／Linux Chromium 矩阵内 M5-05 门槛：冻结后两端各通过 11 个 1K 用例／44 通道比较、语义与质量检查、12 次额外生命周期渲染，以及独立产品隔离安装、28 项浏览器契约和正常生产静态部署。完整像素与来源证据已保留。Alpha 就绪限于实测范围，npm 仍未发布；Studio／M6 需另行决定。下方较早的日期记录保留其当时状态。

**M4 已完成本地及远端验收，2026-09-12；软件包仍未发布。** PR-011–015 及 CI 工具修复通过已记录验收矩阵。此前暂缓的干净检出 CPU 及 Linux SwiftShader 门槛现已关闭，见[远端 CI 证据](./evidence/remote-ci/README.zh-CN.md)。全部产品包仍为 pre-alpha `0.1.0`，禁用发布。记录的 CI 修复已推送实现及修正，当时不包含合并或发布 tag。后续默认分支、PR 及检查点工作遵循 [M4.1 仓库治理](./governance.zh-CN.md)，不发布软件包、安装器或二进制分发。[PR-015 证据](./evidence/pr-015/README.zh-CN.md)保留此前本地评估。

## M4 退出条件评估

| 退出条件 | 本地证据与结果 |
|---|---|
| 独立原生消费者 | PR-011 应用使用公开 parse／validate／compile／render API、显式覆盖／通道／上下文，以及 renderer drop 后的 CPU 自有输出。 |
| CLI 边界 | PR-012 在独立工作目录验证报告外层结构、退出码、诊断、真实 PNG 及部分写入行为。 |
| 失败契约 | PR-013 添加类型化设备丢失／OOM 分类、上下文自有丢失状态、首个错误保留及受保护清理。OOM 分类使用合成类型化错误；销毁测试使用真实 GPU 设备。 |
| 过期工作与保留状态 | PR-014 限制为一个活跃／一个待执行请求，只发布最新代次，清理自有目录，并验证九内核缓存／生命周期边界。 |
| 软件包消费 | PR-015 在仓库外临时工作区构建并测试真实规范化 Cargo 归档，检查固定外部依赖，拒绝缺失嵌入 shader，并运行打包 Rust／CLI CPU 及 GPU 消费者。 |
| 契约与材质 | 成对公开文档与测试一致。两种本地适配器策略下，陶瓷、皮革、木材的 1K 机器／基准检查及排序选出的 2K 跟踪均通过，未改动已接受基准。 |

独立消费者可通过解析自真实本地包内容的公开 API 加载 `.mix`、验证、覆盖公开参数、请求通道、渲染，并消费输出／指标。不需要生产方私有导入或外部仓库运行时资源。[包解析及限制](./package-consumption.zh-CN.md)准确说明本地 patch 和独立验证锁。

本地实现及已记录远端矩阵完成 M4 验收，并关闭此前暂缓的 M0／M1 干净检出／平台门槛。这不自动批准 M5，下一轮计划仍需明确选择；分发仍需完成下述发布清单。

## 已验证主机／后端矩阵与开放项

| 环境 | 已记录状态 |
|---|---|
| 当前本地 macOS／aarch64，CPU | `package-check`、源码消费者、仓库检查、隔离包单元测试／Rustdoc 及 32 个打包 CLI CPU 用例通过。这是既有本地工作树，不是远端干净检出证据。 |
| 同一主机，Apple M5／Metal | 完整 GPU smoke、源码及打包公开 Rust／CLI 消费、三种 1K 材质及最大 2K 跟踪通过。 |
| 同一主机，固定 SwiftShader Device (LLVM 10.0.0)／Vulkan／CPU 适配器 | 同样的 GPU／包／材质／trace 门槛通过，记录源码固定版本 `694585a05946e1ed49b6bd577ca6537cbb57f025`。这不是 Linux CI 结果。 |
| 远端 Linux／macOS／Windows CPU 矩阵 | **版本 `8b43c84` 通过。** [三平台运行](https://github.com/OpenMixture/OpenMixture/actions/runs/34622547271)执行包含隔离包验证的 `cargo xtask check`，并保留证据。 |
| 远端 Linux 固定 SwiftShader GPU／材质任务 | **版本 `8b43c84` 通过。** [Linux 运行](https://github.com/OpenMixture/OpenMixture/actions/runs/34622547778)包含串行 GPU 测试、打包消费、三种 1K 材质及 2K 跟踪。[证据与并发限制](./evidence/remote-ci/README.zh-CN.md)保留此前失败的并发运行。 |
| 其他原生硬件／驱动，包括 Windows／DX12 | **本次矩阵未认证。** 暴露适配器选项不证明所有支持后端／设备都通过验收矩阵。 |

当前最大用例选中 wood/default，2048×2048、八个 pass。预算仍为既有 512 MiB 描述符限制；报告区分估算／记录峰值、累计字节、释放和零复用。这是有界正确性／资源跟踪，不是新的延迟基准或物理 VRAM 测量。[跟踪语义](./development.zh-CN.md)及最终原始证据定义其范围。

## 可复现验证

```bash
cargo xtask package-check
cargo xtask test-consumer
cargo xtask check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask gpu-smoke
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask golden check
MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 \
MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask trace-2k
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask gpu-smoke
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask golden check
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask trace-2k
```

macOS loader 路径是本地准备路径，不是可移植安装说明；按目标主机使用 [GPU 指南](./gpu-context.zh-CN.md)或固定 Linux CI 配置。`golden check` 从不接受新基准，不用 `golden update` 替代失败的发布检查。

## 实际发布前清单

- [x] 保留独立可审查的 PR-011–015 本地提交、对应双语文档和本地证据。
- [x] 验证真实本地包源码／资源／许可、精确同组版本、独立消费者及缺失资源拒绝。
- [x] 记录源码／锁／归档身份及已测试主机／后端策略；已接受材质像素不变。
- [x] 获得同一版本的远端干净检出 CPU 及固定软件 GPU／材质／trace 结果，见[已验收 CI 证据](./evidence/remote-ci/README.zh-CN.md)。
- [ ] 选择拟发布版本和支持范围；依据[兼容性记录](./compatibility.zh-CN.md)审查 API／依赖／线上协议兼容性及必要迁移。
- [ ] 为目标分发审查最终注册表／包锁／安装元数据。当前本地归档省略锁，使用独立固定验证器；单独授权发布改动前保留 `publish = false`。
- [ ] 在发布版本审查最终说明、实际包内容及源码身份。本地归档或成功 CI 本身不授权发布、push／merge 或创建发布 tag。

## 本地未发布说明

PR-011 证明公开 Rust 消费和自有输出，包括显式上下文与重复渲染。PR-012 修复人类诊断缺失的端口／参数上下文，并验证既有 CLI 报告／退出码／文件。PR-013 让类型化 GPU 丢失／OOM 可被处理，并修复销毁缓冲区 unmap 的未捕获失败。PR-014 添加仅消费者所有的新鲜度与有界保留，不取消 GPU。PR-015 现已验证包内资源及隔离归档消费者，添加精确同组依赖元数据和 README／许可文件，继续禁用发布并保留既有锁文件。

PR-011–015 未改变文档／节点／计划版本、十一节点词汇、九个 WGSL 实现及已接受材质观感。诊断词汇新增两个 PR-013 代码，严格旧解码器需相应处理。未引入备用执行器、隐藏 GPU 状态、新产品 crate、运行时依赖、资源池优化器、WebAssembly 或编辑器。下一步决策应依据本次 M4 评估、已完成远端矩阵及剩余兼容性／分发清单。
