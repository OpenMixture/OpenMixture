# PR-015 local acceptance evidence

English | [简体中文](./README.zh-CN.md)

**Storage update (2026-09-25):** Original execution and acceptance conclusions are unchanged. Retrieve complete historical attachments using the [archive instructions](../archives/README.md). Machine receipts, original hashes and capture manifests are unchanged; inspect their paths in the complete restored snapshot. Critical records and review images remain here.

Accepted locally on 2026-09-11 on macOS/aarch64: real Cargo archives pass independent CPU and GPU consumption. This completes the local M4 implementation assessment; remote platform gates and release remain open. See [package verification](../../package-consumption.md), [compatibility](../../compatibility.md) and [release assessment](../../release.md).

The implementing commit is the commit introducing this directory, based on `30a190a41b8d425c40976a5cde8924e1e2ba9cd4` on `codex/pr-015-package-consumer`. Runs tested the working implementation before commit. [Environment](./environment.json), [source identities](./checks/verified-source-hashes.json), [baseline identities](./checks/baseline-hashes.json) and [capture manifest](./capture-manifest.json) identify those inputs and archived outputs without claiming an embedded commit attestation.

## Results

| Gate | Evidence |
|---|---|
| Final repository check, including package-check | [Full log](./checks/check.log) |
| Focused tooling tests and strict clippy | [Tests](./checks/tooling-tests.log), [clippy](./checks/tooling-clippy.log) |
| Independent source consumer CPU contracts | [Log](./checks/test-consumer.log) |
| Actual package CPU verification | [Log](./checks/package-check.log), [status](./packages/cpu/status.json) |
| Apple M5 / Metal GPU smoke and packaged consumption | [Smoke log](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/pr-015/gpu-smoke/metal/command.log), [package status](./packages/metal/status.json) |
| Pinned SwiftShader / Vulkan GPU smoke and packaged consumption | [Smoke log](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/pr-015/gpu-smoke/software/command.log), [package status](./packages/software/status.json) |
| Three materials at 1K, both policies | [Run index](./runs.json) links six successful reports and their artifacts. |
| Ranked largest 2K workload | [Metal trace](./trace/metal/trace.json), [software trace](./trace/software/trace.json) |

Each package attempt retains the three `.crate` files, normalized manifests, resolved metadata, isolated lock, file/binary hashes, command arguments, raw streams and consumer receipts. The package verifier confirms version-only dependencies resolve exclusively to extracted packages outside the producer. It tests packaged unit tests/Rustdoc, 32 CLI CPU cases and, on each GPU policy, real Rust owned outputs and ten CLI GPU cases. Removing the embedded constant shader must fail compilation; those nonzero logs are intentional negative evidence, followed by restoration and hash checks. Successful external staging trees were removed; recorded original paths need not still exist.

The outer package verifier's `packagedCratesValidated: true` attests those checks. The inner application retains `false` because application output alone cannot prove build origin. Source-consumer smoke also reruns device-loss, stale-result and retention cases; the complete logs preserve their result counts. Device-loss uses actual destroyed devices; OOM classification remains synthetic typed-error coverage, not physical memory exhaustion.

Ceramic, leather and wood machine checks and golden comparisons pass on both policies. Default contact sheets were inspected for appearance and differences; accepted baselines and human-review records remain unchanged. Wood/default is the ranked 2048×2048 workload with eight passes and fits the existing 512 MiB descriptor budget. The trace is a descriptor/lifetime observation, not a physical VRAM or GPU timestamp benchmark. Artifact copies preserve original bytes; raw logs may retain trailing blank lines.

## Reproduce and scope

Run the commands in the [release checklist](../../release.md#reproducible-verification); the [GPU guide](../../gpu-context.md) explains local loader preparation. CPU `cargo xtask check` now includes package verification and does not acquire an adapter. The two GPU policies are explicit; no executor fallback exists.

Package/source asset verification, exact peer versions, README/license inclusion and paired compatibility/release documentation are in scope. Publication stays disabled at `0.1.0`; archives intentionally omit locks and use the separate pinned verification lock. No lockfile update, new node, shader semantic change, format change, new product crate, runtime dependency, WebAssembly or editor is included. Original source documents and material baselines are unchanged. No remote push, CI result, merge, tag or distribution is claimed.

Restore the complete historical capture, including per-run directories and the shared 2K images:

```bash
python scripts/evidence/restore.py early-runs tmp/retained-early
```
