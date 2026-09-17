# Local-coordinate candidate qualification — 2026-09-17

English | [简体中文](./README.zh-CN.md)

**Do not adopt this as a standalone formal fix.** Candidate `e41410859c457c13739198c580bc1e1e2af45ed9`, evaluated in draft [PR #14](https://github.com/OpenMixture/OpenMixture/pull/14), improves conditioning but fails unchanged software material goldens and ordinary Windows Chromium gates. No baseline, tolerance, dependency policy or production shader in the evidence branch was changed. This completes the adoption evaluation following the [stability study](../shader-stability/README.md), not Chrome/Edge certification.

## Controlled candidate

The candidate branches from `2cc36863eb5cb4a0f419722b172fb5aceaec239f` and changes only the cellular delta from absolute coordinates to local coordinates, using the previously retained [one-line patch](../shader-stability/diagnostic-local.patch). Original node/material checks were recorded before that patch in the prior study; `git diff 2cc3686 909de3e -- crates fixtures Cargo.lock` is empty. No additional UV, interpolation, levels, square-root or compiler change is included. Independent target directories prevent the earlier shared-cache problem.

The [candidate receipt](./candidate.json) identifies a clean source build, archive SHA-256 `8e4ed1dc82e1a34bbd269bf04fdfe7a0719e78f2b1c49415835cddc4441047b2`, and build ID `sha256:1b2ae86ef1e2893ade8b2babb3c0102362e9336ee4235af5ec220109e8a619b5`. A fixed Studio consumer `56c510ab57daa1b68ef660525a648a582730a37e` installed this archive and passed its type/unit/production-build checks. The producer records Node 24.21.0; consumer checks used its required Node 24.20.0. Native references came from this same candidate with ordinary registry dependencies, Release/DXC, explicit DX12, and the same fixed 11 requests. These references measure cross-target agreement; the old goldens remain a separate binding gate.

## Decisive results

| Gate | Result |
| --- | --- |
| Ordinary Chrome 153.0.8010.48, Windows / GT 1030 host | 33/44 channel comparisons pass; 19 exact |
| Ordinary Edge 153.0.4234.32, same host | 33/44 pass; 19 exact; same channel metrics as Chrome |
| Ordinary Firefox 156.0, same host | 44/44 exact |
| Pinned SwiftShader node/GPU/package consumer checks | Pass |
| Pinned SwiftShader 1K material goldens | Ceramic and wood pass; all 16 leather channel comparisons fail |
| Leather software structural/relationship checks | Pass; failure is the exact-pixel baseline gate |
| Linux configured software Chromium candidate CI | Pass; a separate CI-built archive/environment, not ordinary Windows qualification |

The [machine summary](./summary.json) retains every channel count. Chrome/Edge failures comprise six leather comparisons (height and normal for default, detail-min and coarse-grain) plus five existing wood comparisons. Maximum byte difference remains 1. Default leather height changes 16 pixels against candidate native, normal 37; their fixed ratio limits permit at most 10 and 20 respectively. This is a reduction from the old candidate's 17 failed channels to 11, not a pass.

Software leather differs from the frozen golden at 64–1,739 pixels per channel, maximum byte difference 1. Default counts are baseColor 137, height 472, normal 1,110, roughness 129. The [software report](./software/leather.json), [default contact sheet](./software/contact-default.png), and retained default [height](./software/default/height.png)/[normal](./software/default/normal.png) preserve the regression. Four case contact sheets and tiling evidence are retained. Agent inspection found no obvious structural disruption at contact-sheet scale; this does not override exact comparisons or assert human acceptance.

The three ordinary browser runs completed all 11 cases, lifecycle/stress and synthetic unsupported/adapter/device diagnostics before comparison. Chrome/Edge [production](./chrome/production.json) [Player](./edge/production.json) checks separately passed checker download, encoding, disposal, and absent harness. Firefox production download is not supported by this verifier and is not claimed. Two initial Firefox connections failed before its debugging endpoint was ready; the third completed. Initial Chromium deployment attempts against Vite preview failed its 404 requirement; rerunning against the consumer's actual static server passed. Neither setup failure was discarded as a product pass.

## Decision and next boundary

Numerical conditioning, cross-target agreement, visual/structural quality and old-version compatibility answer different questions. The prior fixed-input study supports better conditioning at large periods. This candidate passes structural checks but fails old software pixels, and still fails ordinary Chromium agreement. Therefore keep PR #14 draft and do not merge or update goldens. The required GPU CI failure remains visible. No dependency fork is justified by this result.

Next reduce a remaining leather height witness through the actual candidate pipeline and independently reduce the unchanged wood path. Distinguish coordinate generation, noise, interpolation, levels and half storage with fixed input readbacks before proposing another change. A future proposal still needs an explicit compatibility decision and unchanged acceptance gates; selecting a compiler/reference to obtain green results is not an acceptance method.

## Reproduction and retention

1. Check out candidate `e41410859c457c13739198c580bc1e1e2af45ed9` in an isolated directory. Run `cargo xtask shader-check`, `cargo xtask test-node fractal-noise` with explicit adapter policy, and `cargo xtask check`. Do not share target directories.
2. On Linux use the pinned setup in `.github/scripts/setup-swiftshader.sh`, the workflow's Vulkan/software/expected-adapter environment, then `cargo xtask gpu-smoke` and `cargo xtask golden check`. The failed golden command prevents the later 2K trace; that trace was not run in this CI attempt.
3. Build with `node scripts/browser-runtime/build.mjs`; use `candidate.mjs stage` and `installed` with the pinned Studio checkout. Generate a fresh `prepare-materials.mjs` bundle with the exact candidate SHA, `MIXTURE_NATIVE_PROFILE=release`, `MIXTURE_GPU_BACKEND=dx12`, and `MIXTURE_DX12_COMPILER_DIRECTORY` as recorded in the [native manifest](./native/manifest.json).
4. Build Studio in browser-test mode and serve its static assets on loopback 4173. Launch each fresh browser using `launch-default-browser.ps1` with only profile/debug transport switches; wait for the endpoint. Run `default-browser.mjs run` with the product, candidate directory, native bundle, launch record and a fresh output directory. Keep comparison failures. For Chromium production, rebuild normally, serve with `scripts/static-server.mjs`, and run `default-browser.mjs production`.

[CI state](./ci-state.json) and [artifact metadata](./gpu-artifacts.json) bind the remote runs. GPU run [35201070526](https://github.com/OpenMixture/OpenMixture/actions/runs/35201070526), attempt 1, failed on leather; artifact `10487694047` expires 2026-10-17T08:48:48Z. CI uses the PR merge revision; Windows uses the candidate head. The configured [browser CI run](https://github.com/OpenMixture/OpenMixture/actions/runs/35201070610) is separate from this local archive. Reports, selected full-resolution failures, sheets, producer/consumer identities and hashes remain in Git. Full archive, all PNG sets, binaries, logs, initial failed setup attempts and detailed test runs remain ignored local output or expiring CI artifacts. No full-run permanent retention or acceptance of a new visual baseline is claimed.
