# M3 公开原生消费端探针

[English](./README.md) | 简体中文

这个独立 Cargo workspace 拥有自己的 [input.mix](./input.mix)，仅使用 `mixture-core` 和 `mixture-wgpu` 的公开 imports。它记录基准修订 `e9dd03bb272a29de6243aec79a3450b47b90530e` 下的原生 API 证据。本地 path 依赖用于定位两个公开 crate；程序不包含生产端夹具或私有 Rust 模块。

在仓库根目录执行；需要已安装仓库指定的 Rust 工具链，并缓存所需依赖：

```bash
cargo run --locked --offline --manifest-path docs/reviews/m3/native-consumer/Cargo.toml --target-dir target/m3-native-consumer
```

[源代码](./src/main.rs) 实际执行源文件解码、验证、公开参数覆盖、选择 baseColor 与 roughness、以 65×3 编译、确定性哈希检查，以及结构化非法覆盖错误检查。[run.json](./run.json) 是实际成功的 CPU 回执；[build.log](./build.log) 保留该次构建与运行输出。夹具有自己的 [Cargo.lock](./Cargo.lock)；这个命令不会更新主 workspace 的锁文件。

未调用的 async 函数随程序一起编译，检查显式获取 context、渲染、报告序列化、通道元数据、RGBA8 访问，以及 renderer 销毁后结果所有权所需的公开类型。另一个未调用的函数检查 renderer、plan、output 和 render future 在原生环境中的 `Send` 约束。这些都是编译检查：GPU 函数及其中的断言从未执行。因此回执明确记录 `gpuExecuted: false` 和 `gpuPathValidation: "compile-only"`。

这个探针证明本地公开 API 可被消费，但不证明 GPU 输出正确性、非阻塞调度、取消、过期结果处理、已打包 crate 的可用性、注册表发布或 M4 完成。渲染仍需要独立的运行时和外部消费端验收。

## 留存 M3 证据审计

[verify_acceptance.py](./verify_acceptance.py) 是独立的 Python 标准库审计脚本，用于核验已提交的材质证据。它不渲染、不修改 golden，也不授予人工批准：

```bash
python3 docs/reviews/m3/native-consumer/verify_acceptance.py --check
```

命令将验证结果与 [acceptance.json](../acceptance.json) 比较。它验证陶瓷、皮革与木材当前的用户验收状态、基准与 PBR 哈希绑定、全部 expected PNG 哈希、两个已记录后端上全部十一种 1K 用例的声明门禁，以及两份 2K 跟踪、输入哈希和留存输出哈希。原始临时路径仅用于识别历史运行，不要求临时文件仍然存在。

回执区分两种检查。当前留存的报告、人工决定、manifest、PNG 及受审计的评审规则／脚本仍须与基准修订逐字节一致。候选与 trace 的 `inputs` 中列出的源码哈希则通过 `git show e9dd03b:<path>` 核验：历史运行绑定的是实际生成它们的源码，因此后续 checkout 修改不会破坏这个历史绑定。回执分别列出两类文件及其交集，保留原有 206 个文件的覆盖。同时作为输入源码和当前评审证据的文件必须通过两种检查。新源码行为仍需单独验证，本历史审计不证明该行为。

审计检查留存的机器结果及其声明规则，不重新计算图像度量。当前 `human-review.json` 的决定是人工验收依据。更早图像中的 pending 标题及自动报告中的 `humanAcceptanceClaimed: false` 保留其历史含义。所有证据均来自本地，远端 CI 仍暂缓。

打印重新验证的回执而不改变保存的记录：

```bash
python3 docs/reviews/m3/native-consumer/verify_acceptance.py
```
