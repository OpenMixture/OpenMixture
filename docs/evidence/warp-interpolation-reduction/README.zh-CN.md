# warp 插值缩减 — 2026-09-18

[English](./README.md) | 简体中文

[局部 texel 候选](../warp-local-texel/README.zh-CN.md)剩余的 26 个同输入差异已缩减到：相同采样值、相同权重的插值运算。诊断版显式 `fma(b-a,t,a)` 在本机消除了这些差异，并通过整张 1024×1024 纹理验证。这是修复提案，尚未通过材质验收。产品着色器、golden、容差和 PR15 源码均未更改。

## 测量结果

原生使用 GT 1030/DX12、驱动 32.0.15.8266、普通 registry wgpu 与记录中的 Chrome DXC 模块。Chrome 153.0.8010.48、Edge 153.0.4234.32 使用直接 WebGPU 诊断页面。浏览器适配器报告显示 NVIDIA/Pascal，但未暴露 WebGPU 后端。[摘要](./summary.json)、标量缓冲区、模块回执和适配器上下文绑定结果。

| 测试 | 原生与 Chrome 差异 |
|---|---:|
| 26 个见证点的局部位移 | 0 |
| 26 个见证点的小数权重 | 0 |
| 嵌套 mix、固定权重 mix、`a+(b-a)*t` | 各 26 个 half 差异 |
| 26 个见证点的显式 fma | 0 个 half 差异 |
| 原插值的同输入完整 warp 纹理 | 26 个像素 |
| 显式 fma 的同输入完整 warp 纹理 | 0 个像素 |

Edge 的固定 mix、fma 标量缓冲区及 fma 完整纹理与 Chrome 完全相同。原生改动前后完整纹理字节不变；重新生成的 stretch/distortion 输入也与冻结输入相同。全图实验只将三处插值换成显式 fma，已保留[完整外部着色器](./warp-fma.wgsl)和[管线](./pipeline.json)。warp 基于 `91015e02255f20105586145db23bbd23b3c93546`；诊断管线沿用之前的木材 value-noise 路径，包含未执行的 cellular 候选分支写法，不是 PR15 归档运行时。

每个标量执行仅输出一个结果，避免输出全部中间值改变优化。原 mix 和 half 探针对两端全部 26 个实际纹理 half 值精确复现。[精确有理数对照](./oracle.json)使用存储的 f32 采样值与权重。在 `(922,23)`，`a=0.42626953125`、`b=0.3408203125`、`t=0.9699997901916504`，精确插值为 `1474822221/4294967296`。原生 mix 及两端显式 fma 舍入为 half 位模式 `13695`，Chrome/Edge mix 为 `13694`。这 26 个已选失败点上，原生 mix 和所有显式 fma 都符合精确标量结果的最近偶数 half 舍入，Chrome/Edge mix 均不符合。这仅是固定标量输入的算术对照，不是新材质参照，也不证明全图均为精确舍入。

## 决策与边界

下一项独立候选是保留局部坐标，仅修改 warp 插值。需要补充双轴非零权重、负位移、完整周期、细长及非二次幂纹理的独立回归，再运行冻结软件 golden 和已安装运行时的三浏览器材质矩阵。本次只有默认木材输入，Y 位移为零，尚未证明普遍精度改善或跨环境一致。

2026-09-18 核查的 [WGSL fma](https://gpuweb.github.io/gpuweb/wgsl/#fma-builtin) 允许拆成普通乘法和加法，[重结合与融合规则](https://gpuweb.github.io/gpuweb/wgsl/#reassociation-and-fusion)也允许算术差异。因此现象不能认定为违反规范，显式 fma 也不保证可移植的逐位一致。本次未增加编译策略、依赖分叉或上游 issue。PR15 的冻结软件木材 golden 仍失败；实验保留了其原生像素，未解决该兼容问题，不能通过选择基线或容差宣称验收。

## 复现与保留

运行 `python docs/evidence/warp-interpolation-reduction/verify.py`，仅需 Python 标准库，即可核验纹理绑定的标量见证、精确有理数 half 结果、Edge 重放及全图内容身份。这是离线证据核验，不会重跑 GPU 或认证规范符合性。

重新运行 GPU 时，按已有独立检出流程构建[原生标量工具](../chromium-arithmetic-reduction/native-probe.rs)与[原生纹理工具](../residual-path-reduction/texture_probe.rs)，使用普通锁定依赖，将已记录 DXC 目录限定加入子进程 PATH。标量命令为 `arithmetic_probe.exe <shader.wgsl> <output.bin> 832 <inputs.bin>`；[浏览器标量工具](../chromium-arithmetic-reduction/browser-probe.mjs)在 CDP 地址后使用相同参数。每项输入有 32 个 float，仅第一个输出槽有效。保持保留输入字节不变。

全图先运行 `texture_probe.exe <pipeline.json> <new-native-output-directory>`。解压 [grid-evidence.zip](./grid-evidence.zip) 到临时目录，用原始 stretch/distortion 字节覆盖新原生目录的对应两个输入文件。设置 `MIXTURE_PROBE_PRODUCT` 为含 Playwright 的锁定 Studio 产品目录、`MIXTURE_GRID_PIPELINE` 为保留管线的绝对路径、`MIXTURE_GRID_INPUTS` 为上述原生目录。运行 `node docs/evidence/warp-interpolation-reduction/browser-grid.mjs <CDP-endpoint> <new-browser-output-directory> native`，比较完整 8 MiB rgba16float 纹理。本次浏览器验证、原生着色器编译均在执行前完成。

Git 保留标量原始字节、着色器、有理数结果、上下文及压缩的完整共享输入和失败输出。三个 fma 全图输出都等于保留的改动前原生字节，因此重复内容去重，并在摘要中分别记录散列。可执行文件、常规日志和重复原始输出留在忽略的 `tmp/warp-residual*`。未产生视觉验收或新 golden。

保留了 Chrome 新配置目录的启动回执。Edge 启动脚本未观察到短暂父进程，但已观察并连接其仍存活的专用配置进程和 CDP 地址；[进程记录](./edge-observed.json)说明此限制。这不是普通浏览器配置认证成功回执，未添加 GPU 特性覆盖开关。
