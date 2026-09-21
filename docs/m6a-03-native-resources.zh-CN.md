# M6A-03 — Native 图像上传与执行

[English](./m6a-03-native-resources.md) | 简体中文

M6A-03 在 [M6A-02](./m6a-02-core-resources.zh-CN.md) 上实现[资源合同](./m6a-resource-contract.zh-CN.md)中的 Native 执行部分，以本 PR 的六项必需检查为集成门槛。浏览器资源参数仍属 M6A-04，跨平台图像资格仍属 M6A-05。本次不发布软件包。

## 公开 API 与所有权

借用紧密 RGBA8 字节同步调用 `mixture_core::prepare`，再调用 `renderer.render_prepared(&prepared).await`。Core 拥有与计划绑定的不可变快照；准备后修改或释放调用方字节不影响渲染。保留准备请求意味着继续持有 CPU 快照。每次渲染重新上传，渲染器及输出不持有快照。请求与渲染器销毁后像素仍有效。[独立消费者](../examples/native-consumer/tests/resources.rs)提供完整例子。

普通 `render(&plan)` 对选中的图像节点在分配、查找管线或提交前返回 `MIX_RESOURCE_MISSING`，附资源 ID、计划哈希及适配器证据。无延迟绑定、调用方可信摘要、可变快照、图像缓存或备用执行器。

## GPU 执行及计数

每个选中 ID 分配一个非 sRGB 的 `rgba8unorm` 纹理及一个显式拥有的暂存缓冲区。紧密行复制进按 256 字节对齐的映射暂存区，不增加紧密 CPU 副本。受检查的复制提交完成后才执行计算。同 ID 的多个节点共享纹理，裁剪掉的绑定不上传。每次调用守卫持有上传分配，在成功、失败或 future 丢弃时显式销毁，保留保守的全部持有生命周期。

唯一图像 WGSL 内核以整数 `textureLoad` 读取 R，经 `mixture_half4` 写出 `(value,0,0,1)`。保留的零 uniform 属于固定 ABI，由入口引用。无采样器、gamma 转换、alpha 相乘、重采样或接缝修复。height-to-normal 保留环绕差分。缓存最多十一种内核身份，与像素内容无关。

上传失败保留首个结构化 GPU 错误／来源链，附 `operation=resourceUpload` 和 `resourceId`；继续保留适配器、设备丢失及清理证据。上传前设备不可用保留既有早期丢失诊断。

实际 `allocations` 增加成功提交 ID／紧密字节的 `resourceCount`／`resourceUploadBytes`，及已分配描述符的 `resourceTextureBytes`／`resourceStagingBytes`。后两者包含在既有纹理／暂存总量中。纹理计数包含 pass 与图像纹理，暂存计数包含上传与回读；无资源计划的原数值不变。驱动内存及失败的不完整分配批次仍按既有报告合同排除。65×3 的紧密／纹理字节为 780，上传暂存为 1,536。清理须零存活字节；Core 估计覆盖保守描述符生命周期。

## 验证

[固定测试](../crates/mixture-wgpu/tests/support/image_probe.rs)及[源码夹具](../fixtures/nodes/image-input/height.mix)覆盖：

- 1×1、1×256、256×1、非对称 2×2、65×3 和 1K；全部 256 个 R 值精确回读为不透明 Scalar。
- G/B/A 改变不影响像素但改变身份；调用方修改、同 ID 替换、共享引用及有界缓存。
- 非周期边界保留与环绕法线方向；1K 周期高度块权重 0／0.25／0.5／1，直接导入／程序化端点相等，中间因果关系误差不超一字节。
- 重复成功／拒绝、上传后部分回读失败及恢复、设备销毁、清理及独立自有输出。
- 源码和隔离 Cargo 归档的公开 Rust 消费，不引用私有实现或生产者夹具。

运行 `cargo xtask test-node image-input`、`shader-check`、`test-consumer`、`gpu-smoke` 和 `check`。节点执行 JSON 与四组 1K 高度／法线 RGBA 写入 `tmp/node-tests/<backend>/`；gpu-smoke 使用 `tmp/gpu-smoke/nodes/`，覆盖现有节点、Scalar、生命周期和打包消费者。CI 还保留不变的三种材质金图、2K 跟踪和浏览器检查。

选定证据见 [Native 证据记录](./evidence/m6a-03/README.zh-CN.md)，普通重复日志保留于忽略目录或 CI 产物。Native 结果不认证浏览器图像执行。CLI 解码、浏览器资源 API、池化、新依赖、Studio 修改及发布均不在范围内。
