# M6B-02 包设计语料

[English](./README.md) | 简体中文

这些是**尚未实现的 Rust codec 的设计 fixture**，由[标准库实验](../../../scripts/measure-package-formats.py)生成。[cases.json](./cases.json)绑定文件哈希、大小和未来预期错误码。`valid.mixpack` 包含现有高度 fixture 及 2×2 线性 RGBA8 资源；独立的 Python 标准 tar 读取器可精确恢复所有条目字节。

其他文件故意违反[选定 v1 契约](../../../docs/m6b-package-format.zh-CN.md)，不要解包到磁盘。预期错误码不代表生产拒绝测试已通过。M6B-03 须通过公共 Rust 装载器执行语料，并补充契约要求的边界、源码完整性、闭包、策略和生命周期用例。本目录不存储或修改 golden 像素。
