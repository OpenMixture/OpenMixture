# 砖块图案本地视觉检查

[English](./README.md) | 简体中文

这些是新砖块图在本地 Windows 硬件上的预览，不代表跨运行时或归档验收。[记录](./local.json) 保存源图哈希、实际适配器、plan 身份和十二张原始 PNG 哈希。[节点指南](../../brick-pattern.zh-CN.md) 提供契约和生成命令。三个变体分别使用默认值、layout=aligned、gap=0.2，在 1024x1024 下输出 baseColor,height,normal,roughness。

![变体：颜色、高度、法线、粗糙度](./contact.png)

代理检查：仅错行变体交替偏移砖行；高度图砖缝为暗色，离开边缘过渡处的砖缝法线为中性。增大 gap 明显缩小砖面平台。法线边缘颜色与高度边界对应。这是程序化几何基础，不是写实砖材质。接触图缩至 256x256，数值检查应使用保留的原始 PNG。不声称维护者视觉批准或软件/browser 发布验收。

![高度 2x2 重复](./tiled-height.png)

平铺图重复默认高度的 512x512 显示缩图。独立 GPU 测试将分辨率与单元数翻倍的完整输出逐字节比较为四份原图；预览本身不是数值证明。原始及迁移后的历史材质 golden 均未改变。

## Windows 公共消费者结果

[精确候选记录](./windows.json) 与 [Native/browser 比较](./windows-comparison.json) 绑定源版本 847fe53c6a42dfac573ee498ccfdc15b1e978b79 及干净、未发布的 0.6.0-alpha.0 归档。17 项候选浏览器测试全部通过。三个 1K 变体与一个 65x3 控制案例 plan 哈希相同，8 个高度/法线比较的最大分量差全部为 **0**，门槛保持 <=1。仅覆盖记录的 Windows 适配器与已安装 Chrome，不代表任意硬件或已发布版本。cargo xtask check、着色器验证、节点 GPU 测试及独立 Native 消费者已在本地通过。六项远端检查单独记录，当前状态见 PR #47。首次浏览器报告写入因 BigInt 序列化失败；上述版本已修复并完整重新验收。

从上述源版本生成干净候选并执行：

```sh
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/brick-browser-candidate
node scripts/browser-runtime/check-bricks.mjs tmp/brick-browser-candidate tmp/brick-native-browser
```

此 Windows 主机用 MIXTURE_BROWSER_CHANNEL=chrome 选择已安装浏览器，Native 使用记录中的 auto 适配器策略。使用新的输出目录，候选与 registry 身份保持分离。软件 CI 矩阵在 Chromium artifact 的 sdk-brick-comparison 中保留独立报告。
