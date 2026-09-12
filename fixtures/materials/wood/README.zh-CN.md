# 方向性木材候选

[English](./README.md) | 简体中文

PR-010 添加温暖棕色的类木材表面，具有细长、轻微弯曲的纹理。[material.mix](./material.mix) 使用八个计算 pass：显式种子的 value 分形噪声 → 整数重复／旋转 → 第二个带种子的场扰动纹理 → levels 高度 → gradient-map 颜色、height-to-normal 及反向 levels 粗糙度。四个 1024×1024 通道全部连接。内置词汇共十一种节点、九个 kernel。

[报告](./reports/README.zh-CN.md)分别记录全图比较、机器测量、代理检查与[人工接受](./reports/human-review.json)。木材人工接受已记录。陶瓷与皮革已获用户接受，其基准 PNG 保持不变。[完整 M3 评审](../../../docs/m3-review.zh-CN.md)已完成。PR-010 时远端 CI 曾暂缓；现已关闭已记录的[远端 CI 门槛](../../../docs/evidence/remote-ci/README.zh-CN.md)。历史报告保持不变。

## 控件与预期效果

| 公开控件 | 节点契约范围／材质默认值 | 目的 |
| --- | --- | --- |
| `grainSeed` | u32 `[0,4294967295]` / `161803` | 显式确定性源排列。独立扰动种子固定为 `314159`。 |
| `grainRepeat` | 整数 `[1,64]` / `32` | 源场的 X 重复次数；`16` 使纹理更宽。 |
| `warpStrength` | 浮点 `[-1,1]` / `0.018` | 以 UV 为单位的有符号 X 采样位移；`0` 取消扰动。 |
| `orientation` | 整数 `[0,3]` / `0` | 扰动前纹理可见的顺时针四分之一圈旋转；`1` 将纵纹改为横纹。 |

[变体](./variants/)每次只改变一个公开控件：`coarse-grain`、`straight-grain` 与 `horizontal-grain`。全部通道必须改变，粗纹高度场的对比度归一化梯度能量必须低于默认值。旋转改变拉伸的纹理；独立位移场与 X 扰动方向保持固定，因此并非最终材质整体的精确旋转。方向标签指图像空间纹理，而非相机空间外观。

源纹理使用 scale 4、四个 octave、persistence 0.45。扰动使用 scale 3、三个 octave、persistence 0.45。高度重映射为 `[0.12,0.88]`、gamma 0.7；法线强度为 0.0015；粗糙度由高度从 0.65 映射到 0.45。调色端点为场景线性 `[0.05,0.017,0.006,1]` 与 `[0.34,0.17,0.063,1]`。各案例固定这些参数。这是由普通各向同性 BRDF 消费的方向性贴图结构，不是物理各向异性反射模型。

## 平铺与方向性

两个噪声场均周期化。整数变换缩放与四分之一圈旋转保留单位正方形的周期性；transform／warp 在像素中心采用显式循环四点双线性采样。height-to-normal 使用循环导数。见[节点约定](../../../docs/node-contracts.zh-CN.md)。滤波为双线性，不是通用各向异性或 mipmap 滤波；高频及缩小预览可能混叠。

每组案例要求高度跨度 ≥100 字节、标准差 ≥20、两轴相邻相关性 ≥0.6，且横跨纹理／沿纹理梯度能量 ≥4。能量为含循环边缘的相邻红通道差平方均值；分母下限为一个字节的平方。跨度、方差与相关性保护会拒绝平面或打乱的场。字面转置测试证明相同直方图不能让错误方向通过。默认／粗纹／直纹声明纵向；旋转案例声明横向。

所有通道的重复边缘／内部相邻跳变比 ≤2.5。法线须不透明、正 Z、接近单位长度，并与可测高度坡度的方向一致。至少 20% 的轴样本可测，其中 ≥98% 必须一致。粗纹案例采用更低的最小平均倾斜（0.0003，其他为 0.001）：固定高度与法线强度时，更宽特征具有更缓坡度。其初始 0.001 门槛在约 0.00062 处失败；已保留[初始测量](./reports/metal-initial-measurement.json)。该门槛在首版基准建立前设置，仍拒绝中性法线图。没有覆盖失败 golden。

## 复现

从仓库根目录运行，并使用显式 [GPU 配置](../../../docs/gpu-context.zh-CN.md)：

```bash
DYLD_LIBRARY_PATH="$PWD/tmp/pr-004/swiftshader-build/bin" \
MIXTURE_SWIFTSHADER_SOURCE="$PWD/tmp/pr-004/swiftshader-source" \
MIXTURE_GPU_BACKEND=vulkan MIXTURE_GPU_SOFTWARE=1 \
MIXTURE_GPU_EXPECT_ADAPTER=SwiftShader cargo xtask test-material wood

MIXTURE_GPU_BACKEND=metal MIXTURE_GPU_SOFTWARE=0 MIXTURE_GPU_EXPECT_ADAPTER='Apple M5' cargo xtask test-material wood
cargo xtask golden check
cargo xtask trace-2k
cargo xtask check
```

`golden check` 与 `trace-2k` 使用相同 GPU 策略环境变量。前者以 1K 检查全部三种材质。后者以 2K 检查十一组案例，选取估算峰值最高者并实测；独立追踪绝不更新 1K golden。软件像素比较须精确一致；木材硬件容差为最大绝对误差 ≤2 字节、RGBA 均值 ≤0.20 字节，超过一字节的像素最多 0.025%。已保留[初次比较失败](./reports/metal-initial-tolerance-failure.json)与[中间阶段精度测量](./reports/precision/measurements.json)：噪声／transform／warp 导出最多相差一字节，levels 重映射可放大到两字节。根据实测精度传播仅调整木材容差；软件基准 PNG 与结构门槛均不变。见[受保护更新](../../../docs/material-goldens.zh-CN.md)；`golden update wood --accept` 消费已评审的软件候选，不重新渲染并拒绝 CI。

## 观感与范围

可选[受控评审](./review/README.zh-CN.md)在相同光照、相机、几何与 BRDF 下消费实际验证过的 PNG。导出法线只应用一次；高度保留作诊断，不叠加第二次 bump 或 displacement。消费者不添加节疤、额外纤维、程序噪声或表面浮雕。目标为可用的风格化类木材表面，不是扫描树种或物理校准的木材。

[2K 证据](./reports/README.zh-CN.md)在任何生命周期优化前，将描述符分配实测与既有 512 MiB 峰值预算比较。实测负载符合预算时允许保留所有 pass 纹理；没有需求就不添加复用。描述符字节不含驱动开销与 CPU 图像缓冲区。PR-010 范围外：新纹理格式、通用优化、没有实测失败的最后消费者复用、M4 API 稳定化、绑定、守护进程、编辑器、嵌入资源、远端 CI 关闭及第十三种节点。
