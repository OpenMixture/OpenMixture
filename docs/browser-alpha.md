# Browser Runtime Alpha delivery closeout

English | [简体中文](./browser-alpha.zh-CN.md)

## Current status and decision — 2026-09-16

Native M4/M4.1 and [bounded M5 browser acceptance](./evidence/m5-05/README.md) are complete. The independent Studio MVP also passed its [recorded macOS saved-file qualification](./evidence/studio-qualification/README.md). These results certify their recorded sources, archives and environments, not every later build or default browser configuration. Cargo packages remain unpublished `0.1.0`; the browser archive is unpublished `@openmixture/runtime@0.1.0-alpha.0`.

There is no stable-version compatibility guarantee; the [compatibility record](./compatibility.md) defines tested contracts.

The next engine stage is **Browser Runtime Alpha delivery closeout**, not M6 or another feature milestone. This page owns the current status and claimed follow-up work; dated M5 records remain historical evidence. It adopts the engine portion of the supplied two-repository review dated 2026-09-16, whose snapshots are engine `c41fcfb` and Studio `5138afc`. Local inspection at engine `c41fcfb` confirms the workflow gap below. The review's remote CI/protection observations were not independently reverified for this planning change.

## Engine-owned work

These are work-item IDs, not GitHub PR numbers. ALPHA-01 now has [candidate qualification tooling](./browser-materials.md), requiring a passing CI receipt for each candidate. ALPHA-02 documentation was merged through [PR #10](https://github.com/OpenMixture/OpenMixture/pull/10); remaining implementation and acceptance stay pending. Each implementation PR records exact source, actual PR URL, checks and explicit exclusions under [governance](./governance.md).

| Item | Priority / state | Scope and completion criteria |
|---|---|---|
| ALPHA-01 — Current candidate browser qualification | P1 / implemented; acceptance required per candidate | Build the current engine revision's actual `.tgz`; install it in an independent pinned consumer; pass public declaration checks, production build/deployment, browser contracts and the existing 11-case/44-channel material comparison. Assert the browser's actual `getBuildInfo()` against candidate identity. Old archives and mixed JS/WASM must fail. |
| ALPHA-02 — Current status alignment | P2 / merged in PR #10 | README, Roadmap, SDK/build guides, release status and navigation point here and agree on bounded M5/Studio completion, unpublished packages and unverified default-browser coverage. Preserve historical evidence and dated checkpoints. Paired translations and repository checks are required. |
| ALPHA-03 — Default desktop browser qualification | P1 / recorded Windows / Firefox 156.0 environment passed ([record](./evidence/alpha-03-configured/README.md)) | Select and record at least one target desktop OS/browser/version with ordinary user settings, no unsafe-WebGPU, blocklist override or forced software-adapter flags. On the same candidate, demonstrate loading, explicit GPU initialization, rendering, owned output and destruction; verify structured unsupported/acquisition diagnostics. A failing target does not count as a supported environment: fix it or narrow the target and document the failure. |
| ALPHA-04 — Publishable npm Alpha candidate | P2 / claimed, pending execution; after ALPHA-01/03 | Produce a clean, versioned candidate with complete JS/declarations/WASM, build identity, archive digest, release notes, serving requirements and measured support scope. Deliver it to Studio for exact-candidate upgrade acceptance. Publication is a separate release action; after publishing, independently install the exact registry version and reverify identity and consumption before claiming delivery. |
| ALPHA-05 — Browser required-check decision | P2 / claimed, pending ALPHA-01 | Read live branch protection/rulesets after the candidate chain passes. Review inclusion of `WASM and npm package` and `Chromium WebGPU material matrix` alongside the existing four Native checks. If adopted, update the desired configuration through a PR, apply it and read it back; retain live evidence. A tracked ruleset or green optional job does not prove enforcement. |

**ALPHA-03 follow-up, 2026-09-17:** Chrome/Edge full-material certification remains pending. The [arithmetic reduction and contract review](./evidence/chromium-arithmetic-reduction/README.md) reproduces a WGSL-permitted multiply/add difference and traces half-rounding propagation. The subsequent [stability evaluation](./evidence/shader-stability/README.md) finds that local cellular coordinates reduce mismatch frequency but change existing leather pixels and do not ensure bit identity. The subsequent [full candidate evaluation](./evidence/local-coordinate-candidate/README.md) rejects the one-line change as a standalone fix: ordinary Chrome/Edge pass 33/44 channels, Firefox 44/44, and pinned SwiftShader leather fails its old baseline. Existing references and gates remain unchanged.

### ALPHA-01 implementation boundary

At the reviewed c41fcfb checkpoint, the [package workflow](../.github/workflows/browser-runtime.yml) builds a new archive, while the [material workflow](../.github/workflows/browser-materials.yml) installs Studio `56c510ab57daa1b68ef660525a648a582730a37e` and its historical vendor archive. [Native preparation](../scripts/browser-runtime/prepare-materials.mjs) already rejects changes to `crates`, `Cargo.lock` and `Cargo.toml` relative to the archived runtime revision. Preserve that protection. The uncovered inputs include the public JS facade, declarations and package tooling; two passing workflows do not prove that today's full package ran in a browser.

Connect production and consumption in a revision-bound job dependency, or build and consume the candidate in one workflow. Do not select an artifact merely by “latest successful run.” Record the tested engine SHA (including an actual PR merge SHA when used), clean source state, consumer commit/lock identity, archive SHA-256, build ID, JS/WASM identity, native reference identity and run/attempt. Independently compare observed browser build information with the producer receipt. A declaration check alone is insufficient: exercise actual return values against the public types, including bigint and owned pixel results. Keep the small handwritten declarations; code generation is not a prerequisite.

The consumer may remain pinned, but its installed runtime must be the new candidate. Keep engine sources, Rust and development absolute paths out of product runtime consumption; native references are detached comparison inputs. Add rejection coverage for a substituted old archive, wrong build identity and mixed package parts. Failures must propagate instead of leaving a reusable prior success report. Keep frozen browser tolerances and native goldens unchanged; rebuilding native references from candidate sources is not permission to reset accepted baselines. Retain accepted evidence under the [evidence policy](./evidence-policy.md).

## Cross-repository handoff

| Responsibility | Owner and handoff |
|---|---|
| Runtime candidate | OpenMixture provides exact engine revision, version, archive/digest/build receipt, public types, support limits, changelog and qualification results. Studio returns its exact commit/lock/archive identities and upgrade results. |
| Ordinary browser workflow | OpenMixture owns runtime behavior and diagnostics. Studio owns open/edit/save/Player reopen/PNG export, and useful graph/editing behavior when GPU support is unavailable. The joint receipt binds both revisions to the same candidate and ordinary browser configuration. |
| Full saved-file regression | Studio runs its seven-case/28-channel cross-consumer qualification for runtime upgrades, save-logic changes and before Alpha release. OpenMixture supplies detached native references and comparison tooling. Preserve the exact Studio-saved bytes. Routine CSS changes need not run the full matrix. |
| Product maintenance | Studio owns branch protection, Player/Studio entry separation, small editor-module extractions, public deployment and real-user trials. These are external dependencies, not implementation claims by this engine repository. |

Execution order: ALPHA-02 documentation can land first; ALPHA-01 closes candidate verification; then ALPHA-03 establishes ordinary-browser coverage while ALPHA-05 addresses governance. ALPHA-04 binds the qualified candidate and Studio upgrade results. npm prerelease publication, exact registry-version consumption, and product public trials follow as separately recorded delivery actions. npm delivery does not require publishing Rust crates, native installers or an embeddable Player package first.

## Verification and stop rules

For this documentation change, run `cargo xtask links` and `cargo xtask check`; report missing prerequisites as incomplete checks. For later workflow/tooling changes, run their focused tests and `cargo xtask check`, then require actual candidate browser CI on the tested revision. Existing build and material commands are documented in the [runtime guide](./browser-runtime.md) and [material guide](./browser-materials.md); this plan introduces no implemented command or new accepted run.

Closeout is complete only when the candidate, independent consumer, ordinary-browser support statement and required evidence agree; publication status must remain explicit. Packaging success alone cannot close browser qualification, and historical M5 acceptance is not reopened by these new delivery gates.

M6, new nodes, subgraphs, resource containers, a second pixel executor, zero-copy GPU interop, generalized optimization, N-API, a 3D preview and marketplace/collaboration remain outside this stage. Core ownership, explicit GPU state, `.mix` semantics and the single `wgpu` path remain unchanged. Further feature planning follows actual consumer feedback or a measured blocker.
