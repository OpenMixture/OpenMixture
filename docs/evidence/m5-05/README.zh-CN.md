# M5 浏览器验收 — 2026-09-15

[English](./README.md) | 简体中文

**M5 工程门槛已在记录的矩阵内通过；运行时具备有限范围的 Alpha 发布决策条件，但仍未发布。** 本记录在此前 [M5-02／M5-03 验收](../m5-02-03/README.zh-CN.md)及独立 Player 流程基础上关闭 M5-05，不认证未测试浏览器，也不授权 Studio／M6。[引擎 PR #8](https://github.com/OpenMixture/OpenMixture/pull/8)负责比较工具、CI 与本评估；[产品 PR #5](https://github.com/OpenMixture/Studio/pull/5)负责包消费、浏览器压力与生产部署。

## 来源与先后顺序

[校准记录](./calibration.zh-CN.md)测量两个环境、保留失败候选比较，并在 `289a23bf9026c355f557c8d637784c03defb0932` 冻结[容差](../../browser-tolerances.json)。两次正式验收均在此后开始，绑定容差 SHA-256 `f01533c2311e32319c37338126a20f8089957e770e221424c46b10263c40b627`；验收未放宽门槛或更新原生 golden。

[机器摘要](./summary.json)绑定确切源码／锁文件／归档／工具／浏览器身份、测量与 CI 产物信息。本地原生准备使用干净引擎 `289a23bf9026c355f557c8d637784c03defb0932`，隔离产品使用干净已合并 main `659a7ecde5217ef64911600ed49d1f48f98de3f2`。Linux 测试该引擎 head 的 PR 合并引用 `2d0fb9cfc93ee943d2802adfef4a700ddd62be60` 与产品代码 `56c510ab57daa1b68ef660525a648a582730a37e`；后续证据提交不冒充这些已测源码。

两端消费未变更的 `@openmixture/runtime@0.1.0-alpha.0` tarball，SHA-256 为 `9e245578de160cee1050259ef34358c3fcd3353a21bbb1672d29701bc1ac8084`，由引擎 `4b914feb9f3365d292b27ea60c5e0b6004f745e8` 生成。原生准备确认相对该版本无运行时实现漂移。运行时／shader／锁文件可从该提交找到；每份 manifest 另保留原始输入字节、契约哈希、覆盖值、原生二进制摘要和计划。vendor 归档保留在产品仓库。Node 为 24.20.0、npm 11.19.0、Playwright 1.63.0；构建配方固定 Rust 1.98.1、wasm-bindgen 0.2.128。

## 已验收矩阵

| 环境 | 原生参考／浏览器 | 材料结果 |
|---|---|---|
| macOS Darwin 25.5.0，arm64 | 显式 Metal 参考；Chromium 153.0.8010.12，使用 `--enable-unsafe-webgpu --ignore-gpu-blocklist` | 11/11 用例、44/44 通道比较通过；40 个完全一致 |
| Ubuntu 24.04 CI 镜像 20260907.300.1，内核 6.17.0-1022-azure，x64 | 原生 Vulkan SwiftShader `694585a05946e1ed49b6bd577ca6537cbb57f025`；Chromium 153.0.8010.12，除上述 flags 外加 `--use-angle=swiftshader --use-webgpu-adapter=swiftshader` | 11/11 用例、44/44 通道比较通过；30 个完全一致 |

所有现有陶瓷／皮革／木材验收变体均以 1024×1024 运行，输出 baseColor、normal、roughness、height。精确语义哈希、归一化计划、结构／接缝、非退化、参数因果及高度／法线关系全部通过。[本地测量](./local-comparison.json)的四份木材 roughness 存在差异：最大绝对值 1，最差均值 0.00029754638671875，最大变化像素比例 0.000396728515625。[Linux 测量](./linux-comparison.json)为稀疏皮革差异：最大绝对值 1，最差均值 0.000004291534423828125，最大比例 0.00001049041748046875。误差单位为解码 RGBA8；精确棋盘格／默认值／编码哨兵继续使用原有相等门槛。

浏览器报告 `BrowserWebGpu`，但隐藏适配器名称和驱动；本地 device type 为 `Other`，Linux 为 `Cpu`。Linux flags 请求 Chromium 自带 SwiftShader，不宣称其源码版本与独立构建的原生驱动相同，不能从 flags 推断硬件名称。初始化 context 的 `unverified` verdict 描述探测前获取状态；后续成功 dispatch／回读及 PNG 测量提供执行证据。

代理检查了正式验收的[陶瓷](./accepted-ceramic.png)、[皮革](./accepted-leather.png)、[木材](./accepted-wood.png)原生／浏览器／差异图，未观察到可见材料结构变化。这是代理比较评审，不替代或虚构原生 golden 的人工验收。

## 生命周期、隔离与部署

每个环境额外完成四个新模块／设备周期，每周期渲染三种默认材质，共 12 次额外 1K 渲染。每周期保留一个 4 MiB 通道，其摘要在后续渲染和销毁后不变。记录的渲染后 live descriptor bytes 全为零，流水线数最多九个。描述符估算不包含 JS 自有输出副本，也不测量物理 GPU 内存、浏览器回收或长期 soak 行为。

新的[本地隔离回执](./local-isolation.json)证明两个源码检出均不可访问、PATH 无 Rust，之后完成归档安装、九项 Node 测试／公开类型／构建检查、[28 项真实 Chromium 测试](./local-browser-results.json)、材料执行和正常生产部署。测试包含无效输入、加载失败、busy／飞行中销毁、可控真实设备丢失／映射清理、最新请求规则及 Player PNG 正确性。此前回执仍按原始范围链接，不声称覆盖自发驱动丢失。

[生产部署](./local-deployment.json)在静态 `/player/` 下提供正常 Player 构建，不含测试 harness。浏览器以 HTTP 200 和 `application/wasm` 加载 WASM，完成棋盘格渲染与独立验证的 65×3 PNG 下载，确认 harness URL 不可用并销毁实例。[截图](./local-player.png)保留部署后可见 Player。这是本地／CI 静态部署，不是公网托管；材料测试使用独立的生产测试构建及公开包 harness。

冻结后的 [Linux 材料运行 34942244839](https://github.com/OpenMixture/OpenMixture/actions/runs/34942244839)已通过。[产品 main 运行 34941633999](https://github.com/OpenMixture/Studio/actions/runs/34941633999)在合并版本 `659a7ec` 通过，包含 28 项浏览器契约及正常生产部署。新增浏览器任务提供执行覆盖，四项现有原生必需检查与无旁路的活动 ruleset 保持不变。本记录报告这些运行；PR／最终 main 状态另见 GitHub，必须针对集成 SHA 检查。

## 保留与复现

[证据索引](./evidence-index.json)保留两个已验收原生／浏览器包的全部 250 个逻辑文件，包括全部 176 份原生／浏览器通道 PNG、22 份比较图、manifest、回执及用例报告。相同字节共享按内容寻址的资源，或引用已有跟踪原生预期 PNG。2026-09-15 复制后已验证原始字节、大小与 SHA-256；校准报告使用的每个 PNG 摘要也均有保留。独立资源约 84 MB，CI 到期后仍可检查已接受内容。

已取回并检查 CI 产物 `chromium-material-matrix`，ID `10386545022`，服务摘要 `sha256:bc6f89d73ad7b200674ce72ee891ffb96676e48c61e9be233b540bf0b28f3878`，大小 53,651,974 字节，到期时间 2026-10-15T07:38:05Z。完整普通日志及附带构建输出仍位于临时目录或 30 天 CI 产物；Git 保留上述验收内容及关键校准失败。后续重跑是新证据，不能恢复缺失日志。

在引擎仓库根目录，将已保留原始文件重建到新目录并验证，然后针对未变更契约重跑比较：

```bash
python3 - <<'PYCODE'
from pathlib import Path
import hashlib, json
root = Path.cwd()
index = json.loads((root / "docs/evidence/m5-05/evidence-index.json").read_text())
target = root / "tmp/m5-05-retained"
target.mkdir(parents=True, exist_ok=False)
for dataset, files in index["datasets"].items():
    for item in files:
        data = (root / item["storedPath"]).read_bytes()
        assert len(data) == item["bytes"]
        assert hashlib.sha256(data).hexdigest() == item["sha256"]
        dest = target / dataset / item["file"]
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
PYCODE
cargo xtask browser-material-check tmp/m5-05-retained/local-native tmp/m5-05-retained/local-browser
cargo xtask browser-material-check tmp/m5-05-retained/linux-native tmp/m5-05-retained/linux-browser
```

新的 GPU 执行使用[准备／比较指南](../../browser-materials.zh-CN.md)及固定[工作流](../../../.github/workflows/browser-materials.yml)。独立产品的[验证指南](https://github.com/OpenMixture/Studio/blob/659a7ecde5217ef64911600ed49d1f48f98de3f2/docs/browser-qualification.zh-CN.md)记录隔离、部署和浏览器命令。

本记录不认证通用 Safari／Firefox／Windows 浏览器、移动端、物理硬件矩阵、SSR／Node GPU、自动恢复、零拷贝、公网托管或 Registry 发布。npm 发布仍需明确分发决策、确切归档／版本／标签及下游验证；Studio 编辑和引擎 M6 仍需单独范围决策。
