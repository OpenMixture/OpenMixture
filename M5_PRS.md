# M5 implementation plan — Browser runtime and independent Player

English | [简体中文](./M5_PRS.zh-CN.md)

**Current status (2026-09-20):** Native M4/M4.1, bounded M5, the recorded Studio MVP and ordinary Windows Chrome/Edge/Firefox qualification are complete within their recorded scope. Packages remain unpublished. The exact Alpha candidate and Studio upgrade pass the recorded acceptance; both browser checks are now required on main. Publication and exact registry-version consumption remain separate delivery actions. [Alpha closeout](./docs/browser-alpha.md) owns current work; M6 does not start.

The dated checkpoints below retain their status at the time; they are not the current backlog.

**M5 acceptance, 2026-09-15:** [Browser acceptance](./docs/evidence/m5-05/README.md) closes M5-05 for the recorded macOS/Linux Chromium matrix: each environment passes 11 post-freeze 1K cases/44 channel comparisons, semantics/quality gates and 12 additional lifecycle renders, alongside independent product isolation, 28 browser contracts and normal production static deployment. Complete pixels and provenance are retained. Alpha readiness is bounded to tested coverage; npm remains unpublished and Studio/M6 require separate decisions. Earlier dated entries below retain their historical status.

**Player export update, 2026-09-15:** the M5-04 open → edit → channel preview → PNG download workflow now passes local acceptance in the independent product. The clean isolated consumer passed 28 Chromium checks and nine Node tests. Twelve 128×128 channel PNGs from three materials decode to the exact public-runtime bytes with correct sRGB/linear metadata. Additional checks cover eight-channel 65×3 downloads, stale-export suppression and encoding failure. [Product evidence](https://github.com/OpenMixture/Studio/blob/77f00deb5410b73140220fabe31f4f8599b3279d/docs/evidence/m5-04-export/README.md) binds the exact source and unchanged runtime archive. M5-05 1K cross-runtime quality, stress, formal browser CI and deployment/compatibility qualification remain open; nothing is published.

**Player update, 2026-09-14:** the M5-04 parameter/preview slice is implemented and [locally verified](https://github.com/OpenMixture/Studio/blob/7370e482e2dcacb9911f5663f8ec4f9e8da6a4cc/docs/evidence/m5-04-parameters/README.md) in the independent product: Rust-metadata controls, channel selection, one active render plus one replaceable pending request, stale-preview diagnostics and lifecycle cleanup. The clean isolated consumer passed 23 Chromium checks and six Node tests, including three-material 128×128 previews. PNG export and full M5-04/M5-05 acceptance remain open; the runtime archive is unchanged and unpublished.

**Local acceptance update, 2026-09-14:** M5-02/M5-03 initial browser execution and isolated package consumption now pass the [recorded gates](./docs/evidence/m5-02-03/README.md). The unchanged archive passes 13 Chromium checks, including controlled real-device loss, mapping cleanup and repeated independent modules/devices. M5-04/M5-05 material regression, complete Player workflows and the accepted browser CI matrix remain open; the package is unpublished.

**Implementation update, 2026-09-12:** the product repository [OpenMixture/Studio](https://github.com/OpenMixture/Studio) has been created. The first M5-02/M5-03 slice implements `mixture-wasm`, the local `@openmixture/runtime@0.1.0-alpha.0` tarball and a minimal Player consuming it. The [browser start guide](./docs/browser-runtime.md) records actual build/consumer commands and current verification. Full M5 acceptance remains open, including material regression, broader browser failure/stress coverage and the complete Player export workflow. The package remains unpublished.

`M5-01` through `M5-05` are work-item IDs, not GitHub PR numbers. Record actual implementation commits, PR URLs and checks when they exist. Work follows [repository governance](./docs/governance.md), [evidence retention](./docs/evidence-policy.md), and the existing [architecture](./ARCHITECTURE.md). The [browser SDK contract](./docs/browser-sdk.md) records implemented public methods and the remaining acceptance requirements. Use the current checkpoint and run evidence to distinguish implementation from accepted gates.

## Outcome and repository boundary

Deliver one installable browser runtime, built in this engine repository, and one independent Player that uses only its public package. Keep `.mix`, node contracts, compilation and WGSL semantics in the same Rust libraries used by the native consumer.

| Owner | M5 responsibility |
|---|---|
| Existing `OpenMixture/OpenMixture` | `mixture-core`, the sole `mixture-wgpu` executor, native CLI, thin `mixture-wasm`, browser JS facade/types/WASM, and runtime package verification. |
| `OpenMixture/Studio` product repository | Player first: files, public parameter controls, 2D channel preview, request freshness and export. Later Studio may share internal runtime-client, preview, controls and file modules. |
| Local `@openmixture/runtime` npm package | The public distribution boundary for the complete browser runtime; built and versioned in the engine repository, not a separate SDK repository. |

The product repository now exists and consumes the first local runtime archive under M5-03. The package name identifies the current local distribution; npm ownership and registry release remain separate checks before publication. Do not create a second full demo alongside the product Player. Small automated engine-side browser fixtures remain appropriate.

```text
Product Player (later Studio)
    -> @openmixture/runtime (JS + TypeScript declarations + WASM)
        -> mixture-wasm
            -> mixture-core + mixture-wgpu
```

The initial npm package promises browser WebGPU only. It does not require publishing the Rust crates to crates.io, and product developers must not need Rust or engine compilation during installation. Keep Cargo publication policy separate from npm distribution.

## Work order

| Item | Owner | Depends on | Reviewable result |
|---|---|---|---|
| M5-01 | Engine documentation | Accepted M4/M4.1 | Paired plan/SDK contract, synchronized roadmap, explicit scope and gates. |
| M5-02 | Engine | M5-01 | Real browser initialization, validation, checker execution, owned readback and cleanup through thin bindings. |
| M5-03 | Engine + independent product PRs | M5-02 | Self-contained npm tarball and a production-built Player page consuming that exact archive. |
| M5-04 | Product + necessary engine contract exposure | M5-03 | Player MVP with public parameters, channel preview, diagnostics, export and bounded latest-request handling. |
| M5-05 | Both repositories | M5-04 | Browser/native regression, clean installation/deployment evidence, compatibility record and Alpha readiness assessment. |

Each item may need several PRs; keep one architectural seam per PR. Engine and product changes cannot be merged atomically across repositories: prepare the engine artifact first, identify its version and digest in the product change, and rerun acceptance after either changes. Product scaffolding can proceed independently once the contract is reviewed; runtime acceptance always uses the real packaged engine.

## M5-01 — Delivery and SDK contract

**Scope**

- Add this paired plan and the paired browser SDK contract; synchronize active roadmap, architecture references and navigation.
- Define explicit WASM loading and GPU acquisition, GPU-free catalog/validation access, raw source input, exposed overrides, requested channels, owned output, structured failures and lifetime rules.
- Retain existing `.mix v1`, eleven node contracts, nine kernels, native API behavior and accepted golden inputs/pixels. Exposed-parameter bindings already exist; expose their meaning through the SDK instead of adding a product-only mapping.
- Define the first consumption recipe as browser ESM plus Vite, with exact tool versions and browser environment recorded by implementation in the [browser start guide](./docs/browser-runtime.md) and product lockfile. Planning alone establishes no browser/platform support.

**Acceptance**

Both languages contain the same scope, identifiers and requirements. Every local link resolves. Remaining requirements are distinguishable from implemented APIs/tooling. `cargo xtask check` passes for the documentation change, and its actual PR follows the existing required checks before integration. No browser pass or release acceptance is recorded by this item.

**Out of scope:** runtime or dependency changes, creating the product repository, npm publication, node/editor implementation, and edits to historical evidence or `mixture-greenfield-docs/`.

## M5-02 — Browser execution and thin binding

**Scope**

- Introduce `mixture-wasm` for the actual WASM compilation/binding boundary. Keep browser transport in the binding and platform execution behavior in typed `mixture-wgpu` modules.
- Review target-specific wgpu features, adapter/device limits, asynchronous error scopes, callback completion, device-loss handling and timing. Existing native waiting/polling paths are not a browser readiness claim; browser readback must yield until mapping completes without blocking the event loop.
- Load WASM explicitly, expose CPU-side validation, explicitly acquire a GPU instance, render a checker `.mix` through the shared compiler/executor, return independent RGBA8 and dispose resources.
- Implement the contract's single-instance concurrency and destruction behavior; preserve meaningful failures for unsupported environments, initialization, invalid inputs and execution. Retain native behavior and existing checks.
- Select and record the first reproducible browser/OS/adapter/toolchain combination. It is a tested environment, not a promise of all browsers, mobile devices or hardware.

**Acceptance**

A real browser loads and validates the checker, renders correct requested pixels, consumes the result after another render and after disposal, and reports failures without another renderer. Verify that catalog/validation do not acquire a GPU. Check mapping/cleanup and device-loss behavior at the observable browser boundary; distinguish injected error-classification tests from actual device tests. A successful WASM build or a mocked JS test alone does not pass this item.

Run affected native checks, shader validation where applicable, `cargo xtask check`, and explicit native GPU regression for lifetime/readback changes. Add reproducible browser test/build instructions with their implementation; do not silently redefine native `gpu-smoke` as a browser test.

**Out of scope:** Player feature UI, new nodes, semantic shader changes, hidden fallback, WebGL2/CPU execution, GPU texture interop and broad performance optimization.

## M5-03 — Packaged runtime and independent loading page

**Scope**

- Produce `@openmixture/runtime` with public ESM entry, TypeScript declarations, generated binding, `.wasm`, README and licenses from one identified build. Test the actual public entry and resource-location override.
- Build a real npm tarball and consume it from the independent product repository. Select exact Vite/Node/package-manager versions and retain the product lockfile. Install/build without Rust, producer source paths, development symlinks or a compiling postinstall hook.
- Implement the smallest Player page that loads a `.mix`, explicitly initializes the runtime and displays the returned channel. Verify the Vite production build under static serving, including WASM asset paths from a non-root base path.
- Record producer revision/build/lock/archive digest and consumer revision/lock. Keep the tarball available to the acceptance run; do not commit a developer's absolute local path as the reproducible installation recipe.

The implemented producer command is `node scripts/browser-runtime/build.mjs`: it builds and runs `npm pack` in `target/browser-runtime/package/`, then writes the archive, digest and receipt under `target/browser-runtime/`. The product installs that exact archive from its `vendor/` directory with its own lockfile. The [browser start guide](./docs/browser-runtime.md) documents the build/install/test recipe. Registry publication is not a prerequisite. Development links are optional convenience, never acceptance evidence.

**Acceptance**

An isolated product checkout installs the exact archive, type-checks, builds and displays real output from the production bundle without access to the engine checkout. Missing WASM or an invalid resource location produces a tested diagnostic. Verify public JS/type/WASM consistency and asset inclusion; listing archive files or testing the producer's development server is insufficient.

**Out of scope:** registry release, extra public packages, frontend monorepo infrastructure, all-bundler adapters and SSR/Node.js GPU support.

## M5-04 — Player MVP

**Current status:** parameters, channel previews, latest-request handling and PNG export are implemented and pass the bounded local acceptance above. The workflow gates below are covered; M5-05 still owns 1K cross-runtime quality and the formal browser matrix.

**Scope**

- Open a user `.mix`; query Rust-owned contracts and existing exposed bindings to display current effective values and valid controls; apply public overrides without rewriting node semantics.
- Provide requested-channel selection, 2D preview, structured diagnostics and texture download with the documented color/scalar/normal encoding. File selection, image encoding, naming and download belong to the product.
- Keep one active render and one replaceable latest pending request. Reject stale/duplicate completion, retain an explicitly stale preview after the newest request fails, and dispose instances on product teardown. Dropping interest in a result is not GPU cancellation.
- Reuse internal runtime-client, preview, controls and files modules; defer the Studio route/editor until after M5 acceptance.

**Acceptance**

Ceramic, leather and wood complete open → change exposed parameters → choose channels → preview → export. Check invalid source, invalid overrides, unsupported GPU, newest-request failure and rapid edits. Verify exported dimensions/pixels and color metadata independently of the on-screen preview, including linear scalar/normal data. Fast UI tests may use controlled completions, but actual rendering and export gates use the installed runtime.

**Out of scope:** graph editing, undo/redo, intermediate-node previews, 3D scenes, layouts in `.mix`, accounts, project containers and embeddable Player publication.

## M5-05 — Browser regression and Alpha readiness

**Scope and acceptance gates**

1. **Cross-runtime semantics:** use identical source bytes, node versions, overrides, resolution and requested channels. Compare normalized plan semantics/hash under the same implementation; preserve source/build identity separately from the semantic hash.
2. **Material quality:** run all three accepted materials at 1024×1024 with their existing acceptance variants and requested baseColor/normal/roughness/height channels. Record pixel error, tiling, non-degeneracy and parameter-causality measurements. Measure browser differences, review and freeze explicit per-channel tolerance gates before accepting the run; retain failures used to justify them. Do not loosen native baselines or claim universal byte identity.
3. **Lifecycle and failure:** verify retained owned pixels after later render/destruction, repeated init/render/disposal, concurrent-call rejection, newest-result behavior, loss/cleanup and bounded retained state. Record workload and memory-accounting limits; do not claim measured physical GPU memory from descriptor counts.
4. **Packaged product:** repeat isolated tarball installation, public type checks, production build and static deployment/browser execution. Bind both repositories and the consumed archive; verify the deployed assets actually load. A Vite dev-server pass is insufficient.
5. **Automation and provenance:** retain reproducible browser tests and a concrete pinned CI environment. Record browser version, OS, adapter/backend when exposed, required limits/features, flags, tool versions, source/lock/fixture/archive identities and results. Unsupported, unavailable or skipped browser GPU execution is an open gate, not a pass. Preserve the existing four required native checks; coordinate any new required browser check with the live protection policy.
6. **Readiness assessment:** keep a paired browser acceptance/compatibility summary and the necessary evidence under the existing retention policy. List tested environments and remaining limitations; Alpha is not a broad stable-support guarantee.

The product provides `npm run test:browser`, `npm run test:materials` and `npm run test:deployment`; the [comparison guide](./docs/browser-materials.md) records engine commands. [M5-05 acceptance](./docs/evidence/m5-05/README.md) retains the pinned CI environment, criteria frozen before acceptance, complete matrix pixels and limitations. Gates pass for that measured scope; untested environments remain unqualified.

An actual npm Alpha publication is a separate distribution action after readiness: confirm package ownership, select a prerelease version and non-`latest` tag, inspect the exact archive and release notes, and verify the published package through an exact-version product dependency update. Runtime package, `.mix` format, node semantics and product versions remain separate. Local tarball acceptance must not be described as a registry publication.

**Out of scope:** publishing merely to close an engineering gate, new native hardware guarantees, framework plugins, zero-copy interop, Studio MVP and M6 resource packaging.

## Completion records and next scope

For each completed item, add its real PR/commit identifiers and a concise result record tied to tested sources and commands. Keep ordinary repeated output in CI artifacts or ignored local directories; retain accepted material/visual decisions and required supporting content as specified by policy. Do not overwrite historical M3/M4/M4.1 results or manufacture completion receipts in this plan.

After M5 passes, plan Studio MVP in the product repository: graph editing, connections, parameters, undo/redo, exposed bindings and standard `.mix` saving. Layout may use a separate `.mix.layout.json`; it must not be a rendering dependency. Its eventual acceptance path is Studio-saved `.mix` → independent Player → native CLI. This is a future product milestone, not engine M6: [M6](./ROADMAP.md#m6--resources-and-portable-packaging) retains its existing resource/portable-asset entry condition. 3D preview and an embeddable Player package require a separate demonstrated product need.
