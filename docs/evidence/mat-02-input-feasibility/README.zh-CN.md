# MAT-02 既有输入降采样可行性

[English](./README.md) | 简体中文

2026-09-23，干净源码 `e744b5ea8586488c3e25aef9cdf9928b1e4f09da` 通过新构建的公开 CLI，在记录的 GT 1030 Vulkan 和 DX12 路径生成[输入图](./input.mix)。图仅使用既有 v2 value noise、levels、常量及颜色混合，**不**实现或验收形态处理、减法、锈层、最终法线、浏览器一致性或完整材质。

[回执](./receipt.json)记录构建／验证器／锁文件／设计摘要、实际适配器、命令、计划哈希及所有留存文件摘要。[设计](./graph-design-at-measurement.json)与[计划](./qualification-plan-at-measurement.json)保留后续契约冻结前的精确输入。十六张 PNG、四份原始 CLI 报告和生成的输入均留存，可脱离到期产物复查测量。

每条后端分别将 256² RGBA8 结果与其 1024² 结果的算术 4×4 箱式平均比较。误差以字节为单位（/255），baseColor 使用 RGB，Scalar 通道使用红分量。最大分量平均绝对误差为 **baseColor 0.158619**、**height 0.101737**，均低于选定的 4/255 输入可行性目标。仅为检查而通过 metallic 传输的曝光遮罩为 0.238492，通过 roughness 传输的细节调制为 0.332754。两条后端独立得到相同的上述指标；这不能代替跨运行时像素测试。

代理检查 256² baseColor 输入时观察到蓝色涂层和灰色露底区域，具有平滑过渡。这是输入检查，不是 PBR 评审或人工接受。整数半径映射在设计评审中另行检查。形态处理／锈迹／法线实现后，完整材质默认降采样仍须通过 4/255，不能从本简化图推断通过。

## 复现

干净检出被测提交，运行 `cargo build --locked -p mixture-cli`，再以所得 CLI 路径及新输出目录调用 [reproduce.py](./reproduce.py)：

```text
python reproduce.py <path-to-mixture-executable> <fresh-output-directory>
```

从仓库根目录运行；脚本读取该检出的 `fixtures/materials/painted-metal/` 设计／默认值，显式调用 Vulkan 与 DX12，不回退，并拒绝执行期间的源码／二进制变动。依赖 Python、Pillow 和 NumPy，记录版本见回执。开发构建足以用于像素可行性，本记录不声明耗时预算或性能验收。全部像素由 CLI／wgpu 生成；Python 只读 PNG 并计算比较。后续源码的复现生成新证据，不得覆盖本记录。
