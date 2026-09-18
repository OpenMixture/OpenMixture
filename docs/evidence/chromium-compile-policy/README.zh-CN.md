# Chrome／Edge 编译策略排查——2026-09-17

[English](./README.md) | 简体中文

**正式完整材质认证仍待完成。** 普通 Chrome 153.0.8010.48 与 Edge 153.0.4234.32，相对[Firefox 验收](../alpha-03-configured/README.zh-CN.md)使用的未修改 Release／DXC 参照，各通过 27/44 像素门槛。隔离的原生编译参数实验对两款浏览器均达到 44 通道完全一致；该实验改变了依赖图，不能作为原归档的已接受参照。

## 输入与结果

两款浏览器使用此前通过资格验证的确切归档 `08cdce838500343b34d775a62058d6adfd5c6035f109d86e03e5ac1ff11f1ec2`、引擎 `ec571816026a945a706067769fef76e24c8398b0`、固定 Studio `56c510ab57daa1b68ef660525a648a582730a37e` 和原有 11 案例／44 通道。归档及原生清单继续保留在[参照配置记录](../alpha-03-configured/README.zh-CN.md)。主机为 Windows 11 build 26100、NVIDIA GT 1030、驱动 `32.0.15.8266`。各浏览器回执记录新配置启动参数、实测硬件／构建身份；没有 GPU、黑名单、headless 或安全覆盖参数。

| 浏览器 | 未修改参照 | 严格原生像素诊断 | 生产 Player |
|---|---|---|---|
| Chrome 153.0.8010.48 | [27/44，失败](./chrome/comparison.json) | 44/44 完全一致 | [通过](./chrome/production.json) |
| Edge 153.0.4234.32 | [27/44，失败](./edge/comparison.json) | 44/44 完全一致 | [通过](./edge/production.json) |

失败比较的最大字节差为 1，最大变化像素比例为 `0.001064300537109375`。完整运行时回执记录已完成的加载、初始化、自有输出、生命周期及注入式失败探针。生产检查独立验证同一归档、65×3 精确棋盘格 PNG 下载、销毁和测试入口排除。这些成功不能替代失败的材质门槛。

## 隔离实验

未修改 wgpu、仅使用 Chrome 的 DXC DLL 时，原生输出仍与 Firefox 参照一致。隔离的[两处补丁](./diagnostic-only.patch)在 `wgpu-hal 30.0.1` 中为 FXC 加入 `D3DCOMPILE_IEEE_STRICTNESS`，为 DXC 加入 `-Gis`。该补丁的 Release／DXC 构建产生了[全部 88 组浏览器／通道零差异](./strict-comparison.json)。[诊断来源](./diagnostic.json)记录补丁源码和执行文件摘要。全量实验只执行了 DXC 分支，FXC 分支尚未独立认证。

检查的 [Dawn D3D12 实现](https://github.com/google/dawn/blob/main/src/dawn/native/d3d12/ComputePipelineD3D12.cpp)默认请求 IEEE 严格模式，除非相应开关改变该策略。Chromium 的[描述符 IDL](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/modules/webgpu/gpu_shader_module_descriptor.idl)将 `strictMath` 覆盖限制为开发者功能。源码检查与参数对照实验将编译器算术策略定位为实际差异来源，但不证明已安装浏览器的确切内部后端／开关状态。GL／ANGLE／合成器字段不能代替 WebGPU 后端身份。

诊断使用源码 `2cc36863eb5cb4a0f419722b172fb5aceaec239f` 的隔离检出、显式的本地 `wgpu-hal` Cargo 补丁和 Chrome 的 `dxcompiler.dll`。其 lock 差异把该 crate 的 registry 来源／校验和替换为本地修改源码。主检出的依赖、着色器、golden 和容差均未改变。实验后已隔离诊断执行文件，并用原锁定依赖图重新构建正常 Release 执行文件。

## 正式实现决策

当前 wgpu API 没有暴露此编译选项。打包校验的 [registry_pins](../../../xtask/src/package.rs)会拒绝未审查的非 registry 依赖；本地 Cargo patch 也不会自动传递到独立消费的 Cargo 归档。不能放宽源码漂移守卫，或把诊断清单标为原候选来制造认证通过。

建议正式路线是固定最小依赖补丁分支，暴露显式的每上下文 DX12 算术策略，保留用于 Firefox 参照的现有默认值。需要审查限定的依赖来源政策、独立打包验证中的确切版本固定、编译参数专项测试、原有 golden 检查及重新构建／验证的浏览器候选，然后使用该候选声明的参照配置重新认证普通 Chrome／Edge。另一条路线是保留失败记录，等待上游提供选项。已向用户询问依赖维护方式；尚未创建 fork、依赖政策例外、浏览器功能覆盖或发布。

## 复现与保留

先按[现有流程](../../default-browser.zh-CN.md)、保留归档及原生清单复现普通基线。仅用于诊断时：在上述源码创建隔离检出，将已发布 `wgpu-hal 30.0.1` 源码复制到独立目录并应用保留补丁；在隔离 manifest 中显式添加本地 Cargo patch，生成诊断 lock。构建 Release，仅向渲染进程 PATH 前置 Chrome 的 DXC 目录，按相同夹具案例／覆盖，以 1024 尺寸渲染 `baseColor,normal,roughness,height`，比较解码的 RGBA 像素与浏览器输出。记录已修改的依赖身份，不纳入普通资格认证。

Git 保留完整普通失败报告／回执、最小诊断补丁、全部 88 组诊断像素测量及生产回执。完整 PNG、下载源码、隔离检出、执行文件和日志仍在临时 `tmp/chromium-cert-*` 中。没有修改或接受新的视觉 golden，不声明完整原始数据永久可审计。
