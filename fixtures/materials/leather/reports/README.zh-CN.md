# 皮革验证证据

[English](./README.md) | 简体中文

- [软件材质报告](./software.json)和 [doctor](./software-doctor.json)：固定 SwiftShader 对十六张 1K PNG 的逐字节比较。
- [Metal 材质报告](./metal.json)和 [doctor](./metal-doctor.json)：实际 Apple M5 与声明容差；保留[初始阈值失败](./metal-initial-tolerance-failure.json)。
- [初始接受记录](./bootstrap-acceptance.json)、[初始候选](./bootstrap-candidate.json)及[初始检查](./bootstrap-render.json)：仅建立新材质首份基准，初始接触表明确标记修改前图像缺失。
- [通道总览](./overview.png)、[重复预览](./tiling.png)及 [PBR 对比](./pbr/comparison.png)：检查空间结构和受控观感；[review.json](./pbr/review.json)记录完整输入／脚本／输出身份。
- [验证索引](./verification.json)：定向检查、节点报告、冒烟证据和仓库检查。
- [人工评审](./human-review.json)：用户已针对受控 PBR 对比接受；代理检查仍单独标明。

预期 PNG 哈希自首次建立后未变。初始记录中的源码哈希描述当次捕获；后续工具测试及已说明的硬件容差调整由后续检查报告绑定。已接受的陶瓷基准与评审不变。远端 CI 暂缓，这些报告不关闭 M3。复现见[材质指南](../README.zh-CN.md)。

最终[软件源码身份](./software-candidate.json)及 [Metal 源码身份](./metal-candidate.json)与当前源码文件匹配。两个适配器也重新检查了未变的陶瓷基准。

对比图保留捕获时的“Human acceptance pending”原始标注，后续接受决定记录于 human-review.json；图像像素未改动。
