# 历史证据存储

[English](./README.md) | 简体中文

让重要结论仍可检查，同时让每次检出不必携带全部历史运行。维护者于 2026-09-25 选择专用的 [GitHub 证据归档](https://github.com/OpenMixture/OpenMixture/releases/tag/evidence-archive-2026-09-25)。它**不是产品发布**、新执行、新资格认定或 registry 发布。不重写历史及原始验收回执。

## 保留与迁移范围

当前 `.mix` 输入、变体、验收契约、预期 PNG、人工验收记录及绑定的 PBR 图像继续保留在 Git。M5 校准失败、具名比较图、关键测量、包／源码标识和原始证据索引也继续保留。完整历史附件按需获取：

| 组 | 完整恢复的快照 | 当前目录精简 |
|---|---|---|
| `alpha-04` | 原 ALPHA-04 目录、验证器、包含 152 个文件的嵌套归档和 runtime 包 | 仅移出 `saved-file-bundles.tar.gz` |
| `m5-05` | 原 M5-05 目录及证据索引引用的全部已跟踪 golden | 移出按内容寻址的 PNG 矩阵；保留 JSON／文本记录及具名评审／校准图片 |
| `early-runs` | 完整 PR-015 目录及 wood 原始 Metal／软件 smoke 目录 | 移出普通日志和逐次 CLI 运行目录；保留状态／标识／测量、包归档、消费者摘要及缺 shader 的拒绝证据 |

PR-015 的八张 2K PNG 与保留的 wood 2K 图片逐字节相同。[迁移清单](./relocations.json)记录本地共享副本，以及每个移除路径、原始长度、SHA-256 和归档组。这减少工作区重复图片；Git 已对相同 blob 去重，因此不能当作同等 pack 体积收益。

原始机器回执与捕获清单仍记录原路径和哈希，描述原始快照，不声称全部附件仍在当前检出目录。重放这些路径前恢复完整组。Markdown 引用移出的目标时使用迁移前固定提交。不得改写旧回执来让不完整目录看似完整。

## 获取与验证

只需要 Python 3.10+ 标准库。从仓库根目录执行，目标目录必须不存在。下载是显式操作，不属于普通构建或 CI 检查。

```bash
python scripts/evidence/restore.py alpha-04 tmp/retained-alpha04
node tmp/retained-alpha04/docs/evidence/alpha-04/verify.mjs tmp/alpha04-replay

python scripts/evidence/restore.py m5-05 tmp/retained-m5
python scripts/evidence/replay_m5.py tmp/retained-m5 tmp/m5-05-retained

python scripts/evidence/restore.py early-runs tmp/retained-early
```

恢复命令先核对固定归档长度与 SHA-256，再读取目录清单，验证完整成员集合、来源快照、每个文件的长度和摘要。拒绝路径穿越、链接、含糊的 Windows 路径及已存在目标；失败不会留下看似完成的恢复目录。`--archive <downloaded.zip>` 离线执行相同检查。ZIP 包含原始文件清单；M5 重放另校验原始 250 个逻辑文件索引，ALPHA-04 原验证器另校验嵌套的 152 个文件。

`manifest.json` 固定传输标识。[取回验证](./retrieval.json)记录从已发布 Release 下载的结果；这属于存储完整性检查，不是新像素资格认定。ZIP 保留 `e249d57d9ce78e73bbe8da554a6fcce0f3375303` 的不可变 Git blob 字节，避免检出换行转换。在含该提交的克隆中运行 `python scripts/evidence/pack.py <fresh-directory>` 可重建。工具／zlib 版本可能改变 ZIP 压缩字节；重建文件不得静默替换固定资产。

## 保留责任与限制

资产由 OpenMixture 仓库维护者负责。无计划到期时间，不删除或替换此标签／资产。更换位置须先复制字节、独立下载并验证摘要／成员，再新增取回记录及更新位置。GitHub Release 不代表不可变存储或独立备份保证。保留的原 Git 提交提供恢复来源；普通限期 CI artifacts 不承担归档职责。取回失败必须如实报告为不可用，不得视为验证成功。

本调整减少当前源码目录；完整 Git 历史仍含原 blob，不声称普通完整克隆或已有 `.git` pack 会缩小。不改变 golden 像素、shader、包语义、硬件支持范围或历史失败。

## 维护检查

```bash
python scripts/evidence/test_restore.py
cargo xtask evidence
cargo xtask links
cargo xtask check
```

必需 CPU 工作流执行离线归档测试，不下载这些资产。修改保留映射后，除普通仓库检查，还须实际恢复并执行原始成员校验。同一 PR 缩减过期例外计数。配套策略和 Agent Guide 定义此保留例外；这不授权普遍外置当前 golden 或人工验收图像。
