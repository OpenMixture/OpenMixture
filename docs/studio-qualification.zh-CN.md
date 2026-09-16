# Studio 保存文件验收

[English](./studio-qualification.md) | 简体中文

此引擎侧验证接收分离的 Studio 下载字节；产品不导入或构建引擎源码。运行时实现输入必须与归档生产者修订匹配。此次验证批次不授权更改运行时 schema、节点或原生 golden。

## 冻结门槛

像素测量前，[studio-criteria.json](../scripts/browser-runtime/studio-criteria.json)固定七个 1024 × 1024 用例：创作的 4 × 8 棋盘格；默认与创作后的陶瓷（16 × 16 砖块）、皮革（细节 1）和木材（重复 16）。三组材质逐字复用既有 default/fine-tiles/detail-max/coarse-grain 的结构、接缝、非退化、因果及高度／法线规则。棋盘格要求精确黑白交替和精确的默认法线／粗糙度／高度。四个通道同时使用不变的 [M5 浏览器容差](./browser-tolerances.json)。这些标准在验收前冻结；失败不得通过放宽门槛或更新原生 golden 解决。

原生准备与比较命令实现后再记录。仅冻结不代表验收结果。产品证据必须记录真实 Studio 界面下载、确切源码／构建身份、独立 Player 执行、PNG 及记录环境。
