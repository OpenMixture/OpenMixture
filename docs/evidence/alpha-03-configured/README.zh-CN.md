# ALPHA-03 参照环境配置验收——2026-09-17

[English](./README.md) | 简体中文

**普通 Firefox 运行时门槛已通过：11 个案例、44 个通道，像素差异为零。** Windows 11 build 26100、主机 NVIDIA GeForce GT 1030／驱动 `32.0.15.8266` 支持本次记录的 WebGPU 工作负载。此记录补充保留的[初始失败](https://github.com/OpenMixture/OpenMixture/blob/efdac411198bfa87b7eba7ff9e588330ec08baf9/docs/evidence/alpha-03/README.zh-CN.md)和[数值排查](https://github.com/OpenMixture/OpenMixture/blob/efdac411198bfa87b7eba7ff9e588330ec08baf9/docs/evidence/alpha-03-investigation/README.zh-CN.md)，不扩展 Chrome／Edge 材质支持或发布 Alpha。

## 身份与范围

- Firefox 156.0，Mozilla 签名的 Windows 发行包，临时配置、非 headless，没有 GPU／安全／黑名单覆盖。[回执](./firefox/receipt.json)绑定执行文件、版本、启动器／子进程 PID、配置目录及 OS 命令行。运行时适配器身份被隐藏，`isFallbackAdapter=false`；不能用主机清单代替被隐藏的浏览器适配器身份。
- 固定 Studio 消费者 `56c510ab57daa1b68ef660525a648a582730a37e`；[候选记录](./candidate.json)和浏览器回执绑定归档、安装 lock 及实测构建身份。
- [保留归档](./openmixture-runtime-0.1.0-alpha.0.tgz)的 SHA-256 为 `08cdce838500343b34d775a62058d6adfd5c6035f109d86e03e5ac1ff11f1ec2`，版本为 `0.1.0-alpha.0`，构建 ID 为 `sha256:1d05c6c02596921f6c2a4e236fa5a3d3e2e6fa044432f99e8a5bb6a5c4c4eb10`。
- 源码是 CI 合并提交 `ec571816026a945a706067769fef76e24c8398b0`，父提交为 main `7b1cec4` 与实现 `34a0db5`。[CI 35176953041](https://github.com/OpenMixture/OpenMixture/actions/runs/35176953041)第 1 次运行对这些确切归档字节通过 ALPHA-01；[资格回执](./ci-qualification.json)保留门槛与摘要。制品 `10479486219` 于 `2026-10-17T03:14:04Z` 过期，取回的运行时归档已在此保留。

## 参照配置与结果

原生[清单](./native-manifest.json)记录 Release 构建、执行文件／脚本摘要、显式 DX12 策略，以及请求的 Firefox `dxcompiler.dll` SHA-256 `787d2f6ad4b6a5b1327cbfc21b195979bb6ca1c876a0f56f1464f78223aebc60`。[原生证据](./native-leather.json)记录实际 NVIDIA DX12 适配器。正常准备守卫检查了相对候选的运行时源码漂移；最终准备时存在未提交验证工具／文档改动，清单如实标记 dirty，运行时输入完全匹配。

仅切换 Release、保留默认编译器时，皮革仍有差异。将 Firefox 的 DXC DLL 限定到原生子进程 PATH 后，[皮革](./compiler-probe/leather.modules.json)和[木材](./compiler-probe/wood.modules.json)探针进程实际加载了该 DLL，两个默认材质均达到完全一致。[比较结果](./compiler-probe/comparison.json)与[默认编译器对照](./compiler-probe/release-default-comparison.json)保留这一差别。随后按文档配置生成全量原生参照并重新运行浏览器验收。这是修正比较环境，普通用户无需配置 Rust／DXC、修改浏览器 GPU 设置或安装驱动。

[完整比较](./firefox/comparison.json)与[成功回执](./firefox/ordinary.json)显示：现有 11 个 1K 材质案例、44 个通道，原有结构、因果、关系与像素门槛全部通过；最大绝对误差和变化像素比例均为零。加载、显式初始化、后续工作／销毁后自有像素、零报告存活分配、幂等销毁、已接受的在途工作及销毁后拒绝新工作均通过。三个独立注入的失败页面保留预期结构化诊断与可用的 CPU 验证；这不是自然不支持设备的覆盖。

代理已目视检查保留的默认[陶瓷](./firefox/glazed-ceramic.png)、[皮革](./firefox/leather.png)、[木材](./firefox/wood.png)原生／浏览器／差异图：材质特征一致，差异面板为黑色。这是比较图检查记录，不是人工 golden 更新决定。没有修改 golden 或容差。

同一归档还单独通过普通 Chrome 153.0.8010.48 的生产 Player 初始化／渲染／销毁、65×3 精确 PNG 下载和生产构建测试入口排除；保留[回执](./chrome-production/receipt.json)、[PNG](./chrome-production/checker.png)及[截图](./chrome-production/player.png)。这不认证 Chrome 完整材质矩阵或 Firefox 生产下载流程。Studio 保存文件升级验收及公共部署仍属于后续工作。

## 复现与保留

按[普通浏览器流程](../../default-browser.zh-CN.md)使用上述保留归档与源码。设置原生 `dx12`、硬件策略、期望适配器、`MIXTURE_NATIVE_PROFILE=release`，以及指向已核验 Firefox 目录的 `MIXTURE_DX12_COMPILER_DIRECTORY`。确认实际加载的 DLL，准备新的原生目录，在固定消费者中替换／安装确切归档，构建 browser-test 入口并启动静态服务，再以 `-Family firefox` 启动浏览器并向新目录运行验证器。浏览器及原生准备回执绑定实际脚本摘要。不要替换成排查期间另行构建的 Windows 归档。

Git 保留关键比较、身份回执、归档与代表性图片。全量逐案例 PNG、编译产物、Firefox 发行包／配置、完整 CI 制品和日志仍位于临时 `tmp/alpha03-*`，摘要本身不能保存它们。运行时、浏览器、驱动或 OS 改变后需重新验收。本记录完成 ALPHA-03 的已记录环境门槛；PR 整合仍需当前仓库／CI 检查，不声明 ALPHA-04 已可发布。
