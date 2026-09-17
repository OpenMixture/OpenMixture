# ALPHA-03 configured-reference acceptance — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**The ordinary Firefox runtime gate passed: 11 cases, 44 channels, zero pixel differences.** Windows 11 build 26100 with host NVIDIA GeForce GT 1030 / driver `32.0.15.8266` supports the recorded WebGPU workload. This supplements the preserved [initial failures](../alpha-03/README.md) and [numerical investigation](../alpha-03-investigation/README.md). It does not broaden Chrome/Edge material coverage or publish an Alpha.

## Identity and scope

- Firefox 156.0, Mozilla-signed Windows distribution, disposable profile, headed operation, no GPU/security/blocklist overrides. The [receipt](./firefox/receipt.json) binds executable, version, launcher/child PID, profile and OS command lines. Runtime adapter identity is redacted; `isFallbackAdapter=false`. Host inventory is not a substitute for the redacted browser adapter identity.
- Pinned Studio consumer `56c510ab57daa1b68ef660525a648a582730a37e`; [candidate](./candidate.json) and browser receipt bind archive, installed lock and observed build identity.
- The [retained archive](./openmixture-runtime-0.1.0-alpha.0.tgz) has SHA-256 `08cdce838500343b34d775a62058d6adfd5c6035f109d86e03e5ac1ff11f1ec2`, version `0.1.0-alpha.0`, build ID `sha256:1d05c6c02596921f6c2a4e236fa5a3d3e2e6fa044432f99e8a5bb6a5c4c4eb10`.
- Its source is CI merge revision `ec571816026a945a706067769fef76e24c8398b0` (parents main `7b1cec4` and implementation `34a0db5`). [CI run 35176953041](https://github.com/OpenMixture/OpenMixture/actions/runs/35176953041), attempt 1, passed ALPHA-01 on these exact archive bytes; the [qualification receipt](./ci-qualification.json) retains its gate/digest summary. Artifact `10479486219` expires `2026-10-17T03:14:04Z`; its retrieved archive is retained here.

## Reference configuration and result

The native [manifest](./native-manifest.json) records Release compilation, binary/script digests, explicit DX12 policy and the requested Firefox `dxcompiler.dll` SHA-256 `787d2f6ad4b6a5b1327cbfc21b195979bb6ca1c876a0f56f1464f78223aebc60`. [Native evidence](./native-leather.json) identifies the actual NVIDIA DX12 adapter. Source drift against the candidate's runtime was rejected by the normal preparation guard; the final preparation had uncommitted verification/docs changes, honestly recorded as dirty, while runtime inputs matched exactly.

Release with the default compiler still differed in leather. With Firefox's DXC DLL scoped to the native child PATH, the [leather](./compiler-probe/leather.modules.json) and [wood](./compiler-probe/wood.modules.json) probe processes actually loaded that DLL and both default materials became exact. The [comparison](./compiler-probe/comparison.json) and [default-compiler control](./compiler-probe/release-default-comparison.json) retain the distinction. Full native references were then generated with the documented configuration and a new browser run executed. This corrects the comparison environment; it does not require users to configure Rust/DXC, change browser GPU settings, or install a driver.

The [full comparison](./firefox/comparison.json) and [success receipt](./firefox/ordinary.json) show all 11 existing 1K material cases / 44 channels pass unchanged structure, causality, relationship and pixel gates. Maximum absolute error and changed-pixel ratio are both zero. Loading, explicit initialization, owned outputs after subsequent work/destruction, zero reported live allocations, idempotent destruction, accepted in-flight work and rejected post-destruction work passed. Three separate synthetic failure pages retained the expected structured diagnostics and usable CPU validation; they are not naturally unsupported-device coverage.

The agent visually inspected the retained default [ceramic](./firefox/glazed-ceramic.png), [leather](./firefox/leather.png) and [wood](./firefox/wood.png) native/browser/difference sheets: material features agree and difference panels are black. This is a recorded comparison inspection, not a human golden-update decision. No golden or tolerance changed.

The same archive separately passed ordinary Chrome 153.0.8010.48 production Player initialization/render/disposal, exact 65×3 PNG download and production harness exclusion; [receipt](./chrome-production/receipt.json), [PNG](./chrome-production/checker.png) and [screenshot](./chrome-production/player.png) are retained. This does not certify Chrome's full material matrix or Firefox's production download flow. Studio saved-file upgrade acceptance and public deployment remain separate work.

## Reproduce and retain

Follow the [ordinary-browser procedure](../../default-browser.md), using the retained archive and candidate source above. Set native backend `dx12`, hardware policy, expected adapter, `MIXTURE_NATIVE_PROFILE=release`, and `MIXTURE_DX12_COMPILER_DIRECTORY` to the verified Firefox directory. Verify the loaded DLL, prepare a fresh native directory, stage/install the exact archive in the pinned consumer, build the browser-test entry, start its static server, launch Firefox with `-Family firefox`, then run the verifier into a fresh directory. Browser and native preparation receipts bind the exact scripts used. Do not substitute the separately built Windows investigation archive.

Critical comparisons, identity receipts, archive and representative images are retained in Git. Complete per-case PNGs, compiler products, Firefox distribution/profile, full CI bundle and logs remain temporary under `tmp/alpha03-*`; hashes alone do not preserve them. Later runtime, browser, driver or OS changes require new acceptance. This is the ALPHA-03 recorded environment gate; PR integration still requires current repository/CI checks, and ALPHA-04 release readiness is not claimed.
