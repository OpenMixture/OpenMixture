# Studio saved-file acceptance — 2026-09-16

English | [简体中文](./README.zh-CN.md)

**STUDIO-05 passes the recorded macOS matrix.** This completes the Studio MVP engineering gates in the independent product; PR integration remains separate. No runtime source, shader, archive, native golden or tolerance changed. The [product record](https://github.com/OpenMixture/Studio/blob/codex/studio-cross-consumer/docs/evidence/studio-qualification/README.md) retains the actual UI-saved sources and authoring actions.

## Identity and result

Frozen criteria commit: `6bee22d`; native preparation/comparison code: `4764c3bf7099c49f8d3acca903af3db444467084`; clean authoring and isolated product: `160d6d0f16d26193296b6e3f8798989d46bd7022`. Later evidence/document commits are not these executable revisions. [Summary](./summary.json) records source/build/adapter identities. The producer verified no implementation drift in crates or Cargo inputs from runtime archive revision `4b914feb9f3365d292b27ea60c5e0b6004f745e8`.

Seven saved-file cases × four 1024 × 1024 channels pass normalized all-channel and individual Player plan comparison, unchanged M5 pixel tolerances, native/browser structure and height/normal relationships, and parameter causality. [Comparison](./comparison.json): 26/28 channels are byte-exact. Only wood default/authored roughness differs: maximum 1, mean 0.00016951560974121094 / 0.00013375282287597656, within the pre-frozen roughness limits. No criteria were widened after measurements.

Native: Apple M5 / Metal on Darwin 25.5.0 arm64. Browser: Chromium 153.0.8010.12 with `--enable-unsafe-webgpu`, `--ignore-gpu-blocklist`; `BrowserWebGpu` adapter name/driver are redacted. Browser hardware identity is not inferred. Node 24.20.0, npm 11.19.0, Playwright 1.63.0. The initial acquisition context is not a dispatch certificate; actual Player PNGs and native readback establish execution.

[Isolation](./isolation.json) passes installation, 20 Node tests/type/build, 52 browser tests, normal production deployment and all seven Player cases, with both original checkouts denied and Rust absent. [Deployment](./deployment.json) verifies both entries, real rendering, edited save/reopen, WASM MIME and the absent test harness. Existing lifecycle/undo/save failures remain covered. [Rejection probes](./rejection-probes.json) confirm that manifest and plan corruption fail, and invalidate any earlier successful result.

## Visual review and retained content

The agent inspected native/Player/difference sheets for all seven cases and the saved-file Player screen. Checker alternation, ceramic tiles, leather grain and directional wood structure match; no visible structural discrepancy was found. This is an agent review, not new human acceptance or native golden authorization.

- [Checker](./checker.png)
- [Authored ceramic](./glazed-ceramic.png)
- [Authored leather](./leather.png)
- [Authored wood](./wood.png)

[Evidence index](./evidence-index.json) retains all 88 logical native/Player bundle files, including all 56 channel PNGs, seven comparison sheets, source-containing manifest, contexts/plans/reports and Player screenshots. Identical existing M5 assets are referenced without rewriting them. Every stored file was read back and its bytes/digest verified. Ordinary repeated logs/build outputs remain ignored; acceptance content is retained in Git rather than relying on expiring CI artifacts.

Reconstruct into a fresh directory and rerun comparison from this engine checkout:

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

## Limits and integration

Only the recorded macOS native Metal / Chromium saved-file matrix is accepted. Existing regression CI is separate; a CI pass does not certify unrun Studio 1K comparisons on Linux, Windows, Safari, Firefox or mobile. History remains session-only; download checkpoints mean initiation, not verified disk completion. Registry publication, public hosting, additional hardware/browser qualification and engine M6 require their own decisions. Actual PR heads and remote checks must be verified separately before integration.
