# Studio 保存文件验收 — 2026-09-16

[English](./README.md) | 简体中文

**STUDIO-05 在记录的 macOS 矩阵内通过。** 独立产品的 Studio MVP 工程门槛至此完成，PR 集成仍单独处理。运行时源码、shader、归档、原生 golden 和容差均未改变。[产品记录](https://github.com/OpenMixture/Studio/blob/codex/studio-cross-consumer/docs/evidence/studio-qualification/README.zh-CN.md)保留真实界面保存的源码及创作操作。

## 身份与结果

冻结标准提交：`6bee22d`；原生准备／比较代码：`4764c3bf7099c49f8d3acca903af3db444467084`；干净创作与隔离产品：`160d6d0f16d26193296b6e3f8798989d46bd7022`。后续证据／文档提交不是这些可执行修订。[摘要](./summary.json)记录源码／构建／适配器身份。生产者确认 crates 和 Cargo 输入相对于运行时归档修订 `4b914feb9f3365d292b27ea60c5e0b6004f745e8` 无实现漂移。

七个保存文件用例 × 四个 1024 × 1024 通道，通过规范化全通道及单通道 Player 计划、不变 M5 像素容差、原生／浏览器结构与高度／法线关系及参数因果比较。[比较结果](./comparison.json)：26/28 个通道逐字节一致。仅木材默认／创作版粗糙度不同：最大差异 1，平均值分别为 0.00016951560974121094 / 0.00013375282287597656，处于预先冻结的粗糙度门槛内。测量后未放宽标准。

原生：Darwin 25.5.0 arm64 上的 Apple M5 / Metal。浏览器：Chromium 153.0.8010.12，使用 `--enable-unsafe-webgpu`、`--ignore-gpu-blocklist`；`BrowserWebGpu` 适配器名称／驱动被隐藏，不推断浏览器硬件身份。Node 24.20.0、npm 11.19.0、Playwright 1.63.0。初始获取上下文不是 dispatch 证明；实际 Player PNG 和原生读回构成执行证据。

[隔离回执](./isolation.json)通过安装、20 项 Node 测试／类型／构建、52 项浏览器测试、正常生产部署及七项 Player 用例；拒绝访问两个原工作区且 Rust 不可用。[部署](./deployment.json)验证两个入口、真实渲染、编辑后保存／重开、WASM MIME 和测试页缺失。保留已有生命周期／撤销／保存失败覆盖。[拒绝探针](./rejection-probes.json)确认清单和计划损坏会失败，并使先前成功结果失效。

## 视觉检查与保留内容

代理检查了七个用例的原生／Player／差异图及保存文件 Player 界面。棋盘格交替、陶瓷砖块、皮革颗粒和方向性木纹一致，未发现可见结构差异。这是代理检查，不构成新的人工验收或原生 golden 授权。

- [棋盘格](./checker.png)
- [创作后的陶瓷](./glazed-ceramic.png)
- [创作后的皮革](./leather.png)
- [创作后的木材](./wood.png)

[证据索引](./evidence-index.json)保留原生／Player 数据包全部 88 个逻辑文件，包括 56 张通道 PNG、七张比较图、含源字节的清单、上下文／计划／报告及 Player 截图。相同既有 M5 资源直接引用，不改写历史文件。每份保存文件都经回读和字节／摘要校验。普通重复日志／构建输出继续忽略；验收内容保留在 Git，不依赖会过期的 CI 产物。

在引擎工作区恢复到全新目录并重跑比较：

```bash
python3 - <<'PYCODE'
from pathlib import Path
import hashlib, json
root = Path.cwd()
index = json.loads((root / 'docs/evidence/studio-qualification/evidence-index.json').read_text())
target = root / 'tmp/studio-retained'
target.mkdir(parents=True, exist_ok=False)
for dataset, files in index['datasets'].items():
    for item in files:
        data = (root / item['storedPath']).read_bytes()
        assert len(data) == item['bytes']
        assert hashlib.sha256(data).hexdigest() == item['sha256']
        destination = target / dataset / item['file']
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
PYCODE
cargo xtask studio-material-check tmp/studio-retained/native tmp/studio-retained/player
```

## 限制与集成

仅验收记录的 macOS 原生 Metal / Chromium 保存文件矩阵。现有回归 CI 单独记录；CI 通过不证明未执行的 Linux、Windows、Safari、Firefox 或移动端 Studio 1K 比较。历史仍仅在会话内保留；下载检查点表示启动，而不是已验证的磁盘写入。Registry 发布、公网部署、额外硬件／浏览器验收和引擎 M6 需单独决策。集成前仍须核对实际 PR head 和远端检查。
