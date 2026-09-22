# M6B-02 format experiment

English | [简体中文](./README.zh-CN.md)

[measurement.json](./measurement.json) retains exact probe/source revision, dirty state, Python/host identity, encoded byte counts and hashes, stdlib round-trip assertions, wall times, traced decoder allocations and an explicitly analytical Rust/browser buffer ledger. The working tree contains the new design/probe; no production runtime files or dependencies changed. This measures representations, not a Rust codec, GPU behavior or release. Single-run Python timings are exploratory and exclude file I/O; `tracemalloc` excludes pre-existing encoded/caller buffers and is not process RSS.

Reproduce with Python 3 using only its standard library: `python scripts/measure-package-formats.py tmp/m6b-format-run` from the repository root, choosing a fresh destination. The experiment constructs directory-equivalent bytes, compact JSON/base64, ZIP stored and canonical USTAR at 4 MiB and 64 MiB resource payloads. Standard Python ZIP/tar readers independently check all decoded entry bytes; repeated USTAR writing is exact. The [contract](../../m6b-package-format.md) chooses USTAR and defines stricter production validation.

The [small committed corpus](../../../fixtures/packages/mixpack-v1/cases.json) retains valid/malformed archive bytes and future rejection expectations. Full-size candidate buffers are temporary, never needed as a runtime dependency; large outputs and ordinary check logs stay ignored. Retained hashes bind the measured bytes but do not promise retrieval of every original large buffer. No external archive or human/GPU acceptance is claimed. Actual Rust/WASM allocation and adversarial acceptance remain M6B-03/04 gates.
