# Studio 保存文件验收

[English](./studio-qualification.md) | 简体中文

此引擎侧验证接收分离的 Studio 下载字节；产品不导入或构建引擎源码。运行时实现输入必须与归档生产者修订匹配。此次验证批次不授权更改运行时 schema、节点或原生 golden。

## 冻结门槛

像素测量前，[studio-criteria.json](../scripts/browser-runtime/studio-criteria.json)固定七个 1024 × 1024 用例：创作的 4 × 8 棋盘格；默认与创作后的陶瓷（16 × 16 砖块）、皮革（细节 1）和木材（重复 16）。三组材质逐字复用既有 default/fine-tiles/detail-max/coarse-grain 的结构、接缝、非退化、因果及高度／法线规则。棋盘格要求精确黑白交替和精确的默认法线／粗糙度／高度。四个通道同时使用不变的 [M5 浏览器容差](./browser-tolerances.json)。这些标准在验收前冻结；失败不得通过放宽门槛或更新原生 golden 解决。

## 命令

在产品侧捕获，在此准备原生参考，回到产品执行 Player，再在此比较。仓库边界只传递分离数据。

```bash
# Product
npm run capture:studio -- /absolute/studio-downloads
# Engine
MIXTURE_GPU_BACKEND=metal node scripts/browser-runtime/prepare-studio.mjs /absolute/new-native 4b914feb9f3365d292b27ea60c5e0b6004f745e8 /absolute/studio-downloads
# Product
npm run test:studio -- /absolute/new-native /absolute/new-player
# Engine
cargo xtask studio-material-check /absolute/new-native /absolute/new-player
```

准备阶段验证运行时实现身份、唯一用例和保存字节摘要。比较采用失败关闭策略，检查清单与 PNG 来源，比较全通道及单通道 Player 计划，复用现有 Rust 结构、关系和因果检查。原生 golden 保持不变。流程不代表验收结果；需为每个实测环境保留绑定源码的结果。
