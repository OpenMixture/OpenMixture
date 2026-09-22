# M6B-02 格式实验

[English](./README.md) | 简体中文

[measurement.json](./measurement.json)留存精确探针/源码版本、dirty 状态、Python/宿主身份、编码字节数与摘要、标准库往返断言、耗时、跟踪解码分配及明确标为分析计算的 Rust/browser 缓冲账本。工作区仅含新设计/探针，未改生产运行时文件或依赖。测量的是表示，不是 Rust codec、GPU 行为或发行。单次 Python 耗时仅作探索，不含文件 I/O；`tracemalloc` 不计已有编码/调用方缓冲区，也不是进程 RSS。

从仓库根目录以 Python 3 标准库运行 `python scripts/measure-package-formats.py tmp/m6b-format-run`，选择全新目录。实验在 4 MiB 和 64 MiB 资源载荷下构造等价目录字节、紧凑 JSON/base64、ZIP stored 和规范 USTAR。Python 标准 ZIP/tar 读取器独立核对全部解码条目字节，重复 USTAR 写入完全一致。[契约](../../m6b-package-format.zh-CN.md)选择 USTAR 并定义更严格的生产验证。

[已提交的小型语料](../../../fixtures/packages/mixpack-v1/cases.json)留存合法/畸形归档字节及未来拒绝预期。全尺寸候选缓冲区是临时数据，不成为运行时依赖；大输出及普通检查日志留在忽略目录。摘要绑定测量字节，不承诺可取回每份原始大缓冲区。不声称外部归档、人类或 GPU 验收。真实 Rust/WASM 分配及恶意输入验收仍是 M6B-03/04 门禁。
