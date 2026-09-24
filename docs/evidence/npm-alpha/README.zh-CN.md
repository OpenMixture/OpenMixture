# npm Alpha 发布 — 2026-09-20

[English](./README.md) | 简体中文

`@openmixture/runtime@0.1.0-alpha.0` 已于 `2026-09-20T07:26:14.490Z` 公开发布，使用准确 ALPHA-04 tarball，没有重建。用户已授权发布并创建免费 npm 组织 `openmixture`；`krapnik` 为 owner。发布通过 npm 网页安全密钥验证完成。

软件包已可从 [npm](https://www.npmjs.com/package/@openmixture/runtime/v/0.1.0-alpha.0) 获取。请使用准确版本：

```sh
npm install --save-exact @openmixture/runtime@0.1.0-alpha.0
```

[发布时注册表元数据](./registry-at-publication.json)及[下载回执](./registry-pack.json)绑定 348066 字节归档。SHA-256 仍为 `a9bcfe8d849f9fb9982a750dd99d026807fdf6453b5be491deeda90e99a2c6ae`；SHA-512 与注册表完整性及[消费者锁文件](./windows-lock.json)一致。[Windows 安装回执](./windows-install.json)确认全部 12 个安装文件与已验收归档一致。生产者 `82b74707b2a8a998190e2f28b16f91fb9614486a` 和构建 ID `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28` 不变。[候选说明](../../browser-alpha-candidate.zh-CN.md)保留兼容性与支持边界。

发布使用 `npm publish <retained-tarball> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/`。npm 首次发布额外创建了 `latest`。安全密钥验证成功后，`npm dist-tag rm @openmixture/runtime latest` 被注册表以 HTTP 400 拒绝；[最终回读](./registry-readback.json)因此仍保留指向 Alpha 的 `alpha` 和 `latest`。第一次尝试在认证前过期。不声称别名已移除或稳定版已发布。优先使用上述准确版本；当前不带版本安装也会解析到此预发布版。

## 准确注册表版本消费

不可变的已发布 tarball 保留构建时 README 中“未发布”的表述。当前仓库包文档及本发布记录取代该历史状态文字；没有为修改文字而重新打包归档。后续 CI 候选属于独立构建，不替换已发布归档。

Studio 版本 `87ded9351e1c426e03aa7fb2b4c641f32399b85b` 只修改运行时准确依赖及注册表解析 URL。完整性和可执行字节不变。Windows 使用全新 npm 缓存；Linux 在全新文件系统隔离消费者内安装，无法读取源码检出，也没有 Rust。保留的 vendor 归档作为身份／测试夹具，不是安装来源或回退路径。

Windows 和隔离 Linux 均通过公开类型、20 项单元测试、生产构建、**52/52 浏览器契约**、正常静态部署，以及针对七个 ALPHA-04 保存材质的新 Player 执行和 **28/28 通道比较**。[隔离配方](./linux-isolation.sh)、[边界回执](./linux-isolation.json)及[注册表安装](./linux-install.json)标识 Linux 运行。Node `24.20.0`、npm `11.19.0`、受控 Chromium `153.0.8010.12`、显式 Linux SwiftShader 参数及记录中的 Windows 浏览器参数与有界候选配方一致。Windows [普通 Chrome 证据](./ordinary/receipt.json)还通过编辑／修复／历史／保存／重开／四通道 PNG／销毁，以及单独注入的 GPU 不可用情况。

这是针对注册表安装字节的新执行。保存材质输入及原生参考复用 ALPHA-04，不声称重新创作输入或重新渲染原生参考。它们保留创作版本 `6b2d53e3de16b21725b2a4359a2263f98671a6f9`；新 Player 回执标识注册表消费者版本。未修改容差、着色器或基线。不声称更广浏览器／硬件支持、Rust 发布、托管试用重新部署或真人试用结果。

## 保留与复现

[重放索引](./replay-index.json)绑定全部 104 个 Windows／Linux 重放文件。88 个文件（包括重复 PNG 和比较）与既有 [ALPHA-04 包](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/alpha-04/saved-file-bundles.tar.gz)字节一致，复用其保留内容。其余 16 个文件保留在 `replay/`；普通浏览器截图／下载保留在 `ordinary/`。[文件哈希](./files.json)绑定新增保留内容。完整普通日志、浏览器配置和 npm 缓存留在忽略的本地产物中，不作为长期证据。

```sh
node docs/evidence/npm-alpha/verify.mjs
node docs/evidence/npm-alpha/verify.mjs tmp/npm-alpha-replay
cargo xtask studio-material-check tmp/npm-alpha-replay/native tmp/npm-alpha-replay/windows
cargo xtask studio-material-check tmp/npm-alpha-replay/native tmp/npm-alpha-replay/linux
```

第二条命令要求全新输出目录，先验证旧包，再覆盖新的注册表消费者回执。新执行需检出上述 Studio 版本，运行 `npm ci`、`npm run check`、`npm run test:browser`、`npm run test:deployment`、`npm run test:ordinary -- chrome <new-output>` 和 `npm run test:studio -- <detached-native> <new-player>`，再用上述引擎命令比较。Linux 配方记录本机工具路径，复现时需相应准备。注册表元数据和 dist-tags 可变，历史快照不能替代新的远端回读。
