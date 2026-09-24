# ALPHA-04 exact candidate and Studio upgrade — 2026-09-20

English | [简体中文](./README.zh-CN.md)

**Storage update (2026-09-25):** Original execution and acceptance conclusions are unchanged. Retrieve complete historical attachments using the [archive instructions](../archives/README.md). Machine receipts, original hashes and capture manifests are unchanged; inspect their paths in the complete restored snapshot. Critical records and review images remain here.

**The recorded candidate and Studio upgrade gates pass.** [Release notes and support scope](../../browser-alpha-candidate.md) identify the unpublished `0.1.0-alpha.0` archive. This is a new execution against the exact candidate, not a comparison-only replay of old Studio pixels. Registry publication, hosted deployment, human trial results and ALPHA-05 branch-check enforcement remain separate.

## Bound sources and execution

The clean producer is `82b74707b2a8a998190e2f28b16f91fb9614486a`; clean Studio authoring/Player revision is `6b2d53e3de16b21725b2a4359a2263f98671a6f9`, based on product `28d5e6a`. Only its vendor archive, producer receipt and runtime lock entry changed. The [producer receipt](./producer.json), [candidate substitution](./candidate.json), [installation](./installed.json) and [actual browser probe](./build-probe.json) bind SHA-256 `a9bcfe8d849f9fb9982a750dd99d026807fdf6453b5be491deeda90e99a2c6ae` and build ID `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28`. Mixed historical WASM is rejected with `MIX_BROWSER_BUILD_MISMATCH`.

The exact archive passed the engine's [Linux CI qualification](https://github.com/OpenMixture/OpenMixture/actions/runs/35489430243), attempt 1: 28 pinned-consumer contracts, 11 material cases / 44 channels, 12 lifecycle renders and production deployment. [Qualification](./ci-qualification.json), [material receipt](./ci-material-receipt.json) and [comparison](./ci-comparison.json) retain its results. That CI consumer is `56c510a`; the separate Studio upgrade below tests `6b2d53e`. The producer revision also passed the three native CPU jobs and pinned native GPU workflow.

| New Studio verification | Result |
|---|---|
| Windows clean npm installation, public types, 20 unit tests, production build | Passed |
| Windows Chromium 153.0.8010.12 browser contracts | 52 passed, zero skipped/flaky/failed |
| Actual build identity and historical-WASM rejection | Passed |
| Normal production static deployment | Passed; both entries, WASM MIME and absent test harness |
| Ordinary Chrome 153.0.8010.48 on Windows 11 build 26100 | Passed full edit/repair/history/save/Player/four-channel PNG/disposal workflow |
| Windows Studio saved-file/native comparison | Seven 1024×1024 cases, 28/28 channels passed |
| Linux/WSL filesystem-isolated install/check/browser/deployment/Player | Passed; 20 unit tests, 52 browser tests, seven saved-file cases |
| Isolated Linux Player/native comparison | Seven 1024×1024 cases, 28/28 channels passed |

The [ordinary receipt](./ordinary/receipt.json) records the actual executable/command line, requested and observable GPU identity. Only fresh-profile/CDP/about:blank arguments were supplied, without GPU overrides. The separate injected unavailable-GPU probe preserves CPU graph editing; it is not a naturally unsupported hardware test. Studio and Player PNG bytes and sRGB/linear metadata match the checker expectations.

Native references use clean matching engine sources, the standard debug CLI, explicit DX12 and NVIDIA GeForce GT 1030. Full context and driver details remain in each native result. Controlled Windows Chromium uses the recorded unsafe-WebGPU/blocklist test flags and Playwright defaults. Linux Chromium uses additional explicit SwiftShader flags under WSL2 `6.18.33.2-microsoft-standard-WSL2`. Browser adapter fields are recorded as exposed; redacted identities are not inferred from host hardware. Node is 24.20.0, npm 11.19.0 and Playwright 1.63.0.

## Saved-file, isolation and visual evidence

[Authored downloads](./authored.json) bind real Studio save actions. The [native manifest](./native-manifest.json), [Windows Player](./windows-player.json), [Linux Player](./linux-player.json) and their [Windows](./windows-comparison.json) / [Linux](./linux-comparison.json) comparisons bind the same saved bytes, all-channel and per-channel plans, exports and candidate. Each comparison passes the frozen v2 numerical, structure, causality and relationship rules, including exact checker validation. Both have 12 byte-exact channels; the remaining 16 have maximum component error 1 and maximum local bias 0.0078125 against the frozen 0.25 ceiling. No shader, baseline, profile or saved fixture was modified for this task.

The [isolation recipe](./isolation-recipe.sh), [sandbox checks](./isolation-checks.sh) and [receipt](./isolation.json) retain the filesystem boundary. Neither source checkout nor Windows mounts/home is mounted; direct reads fail and `cargo`/`rustc` are absent. The copied consumer has its own Git data and installed archive. Network remains enabled for npm. Windows/Linux raw lock hashes differ because of line endings; the complete parsed lock graphs are verified identical, including archive SHA-512.

The agent inspected all seven Windows native/Player/difference sheets plus Linux authored leather/wood sheets: checker alternation, ceramic tiling, leather grain and directional wood agree visually. Representative sheets: [checker](./checker.png), [ceramic](./glazed-ceramic.png), [leather](./leather.png), [wood](./wood.png). This is agent inspection, not new human golden acceptance.

## Retention and reproduction

[Summary](./summary.json) hashes retained content. The [bundle](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/alpha-04/saved-file-bundles.tar.gz) and [index](./evidence-index.json) retain all 152 source/native/Windows/Linux files, including every compared PNG, plan, context, screenshot and contact sheet. The actual runtime archive is retained alongside it. The verifier reads back every retained file and can restore/hash-check every bundle member:

```sh
python scripts/evidence/restore.py alpha-04 tmp/retained-alpha04
node tmp/retained-alpha04/docs/evidence/alpha-04/verify.mjs tmp/alpha04-replay
cargo xtask studio-material-check tmp/alpha04-replay/native tmp/alpha04-replay/windows
cargo xtask studio-material-check tmp/alpha04-replay/native tmp/alpha04-replay/linux
```

For fresh execution, check out the two bound revisions, install the retained archive in Studio with its recorded lock, run `npm ci`, `npm run check`, `npm run test:browser`, `npm run test:deployment`, `npm run test:ordinary -- chrome <fresh-output>`, and `npm run capture:studio -- <fresh-downloads>`. In the engine run `MIXTURE_GPU_BACKEND=dx12 node scripts/browser-runtime/prepare-studio.mjs <fresh-native> 82b74707b2a8a998190e2f28b16f91fb9614486a <fresh-downloads>`; then Studio `npm run test:studio -- <fresh-native> <fresh-player>` and engine `cargo xtask studio-material-check <fresh-native> <fresh-player>`. Shell-specific environment assignment is required on PowerShell. The isolation recipe records exact local paths; provision its tools/paths before rerunning it.

The first local capture failed because the default Playwright cache had a Windows side-by-side launch error; an existing same-version Chromium installation succeeded. The first full repository check reached packaged rustdoc, which exited with OS `STATUS_IN_PAGE_ERROR`; the final retry is recorded in integration verification. Neither failure was counted as a pass. Complete routine logs/compiler outputs and the original 55 MB CI download remain in ignored `tmp/alpha04-*`. CI artifact `10598493452` expires `2026-10-20T04:42:43Z`; retained critical content does not depend on that expiry, but complete CI pixel/log audit does. The dedicated archive retains the full original Studio comparisons; Git retains the critical records and named comparison images, not the full repeated CI bundle.

[Final local check](./local-check.json): after moving the old package cache aside, the complete `cargo xtask check` passed, including independent packages, Rustdoc and links in 185 documents. [Original cache failure](./cached-rustdoc-failure.txt) preserves the OS error; no source change bypassed a check. Current remote PR checks are verified separately.

[The npm publication dry-run](./publish-dry-run.txt) successfully checked the exact archive with `--access public --tag alpha` (12 files, 348066 bytes) without publishing. npm reported that actual publication requires login; registry authorization, version availability and publication are unverified and belong to the separate release decision.

Integration records: [engine PR #21](https://github.com/OpenMixture/OpenMixture/pull/21) and [Studio PR #16](https://github.com/OpenMixture/Studio/pull/16). Both required Studio checks passed on documentation head `d79488882baeb47ca5630b1d7b4d460c72d86638` in [35491355073](https://github.com/OpenMixture/Studio/actions/runs/35491355073). The PRs provide authoritative merge state and current-head checks; earlier-head results do not certify later revisions.
