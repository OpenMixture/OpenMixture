# M6B-03 CPU verification

English | [简体中文](./README.zh-CN.md)

[PR #43](https://github.com/OpenMixture/OpenMixture/pull/43) implements the shared CPU codec on top of M6B-02. [verification.json](./verification.json) binds the final clean local run to **162456342e1f9bb2ee82675cfcc9c6458657fb1b**, its tree, locks, tiny corpus, toolchain and package verifier. This later evidence/documentation commit is not presented as the tested runtime commit.

On Windows x86_64 with Rust/Cargo 1.98.1, `cargo xtask check` passed, including all workspace tests, Clippy, independent source/CLI consumption, isolated four-archive consumption, Rustdoc and links. The asset crate contributes four unit tests, seven integration tests and one doctest; the corpus includes 22 cases. The public Native asset test passes from both source paths and extracted archives. Codec WASM compilation and all 12 candidate/consumer Node tests pass. A standard tar reader lists the same valid archive that the writer reproduces byte-for-byte.

| Actual Rust workload | Archive capacity | Selected snapshot bytes | Borrowed charge | Owned charge |
|---|---:|---:|---:|---:|
| 1 × 1024×1024 | 4,197,888 | 4,194,304 | 4,195,096 | 8,392,984 |
| 8 × 2048×1024 | 67,119,616 | 67,108,864 | 67,113,297 | 134,232,913 |

Tests verify borrowed payload pointer ranges, unchanged pointers on ownership transfer, distinct immutable snapshots and rejection with a budget one byte below the exact charge. Counts include conservative D+M scratch. These are byte-buffer ledgers, not RSS or a global multi-asset cap. The fixture/source differs from the earlier Python format probe; compare ownership formulas, not equal archive sizes across different inputs.

Reproduce using the [implementation guide](../../m6b-03-cpu-assets.md). The receipt retains package hashes, pinned-dependency count, source/extraction invariance and negative missing-shader result. Original logs and local `.crate` files remain temporary under the recorded ignored paths; hashes do not guarantee future retrieval. Git retains the code, focused inputs/tests, this concise receipt and explanation. No package GPU/browser acceptance, release publication, visual baseline change or broader hardware support is claimed. Remote CI is separate from this local receipt.
