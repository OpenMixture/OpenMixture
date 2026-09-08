# M3 public native consumer probe

English | [简体中文](./README.zh-CN.md)

This detached Cargo workspace owns [input.mix](./input.mix) and uses public imports from `mixture-core` and `mixture-wgpu`. It records the native API evidence available at base revision `e9dd03bb272a29de6243aec79a3450b47b90530e`. Its local path dependencies locate the two public crates; the program does not include producer fixtures or private Rust modules.

From the repository root, with the repository Rust toolchain and dependencies already cached:

```bash
cargo run --locked --offline --manifest-path docs/reviews/m3/native-consumer/Cargo.toml --target-dir target/m3-native-consumer
```

The [source](./src/main.rs) executes source decoding, validation, an exposed-parameter override, selection of baseColor and roughness, compilation at 65×3, deterministic hashing, and a structured invalid-override failure. [run.json](./run.json) is the actual successful CPU receipt; [build.log](./build.log) is its captured build/run output. The fixture has its own [Cargo.lock](./Cargo.lock); this command does not update the main workspace lockfile.

The unused async function is compiled with the program. It checks the public types needed for explicit context acquisition, rendering, report serialization, channel metadata, RGBA8 access and ownership after renderer drop. Another unused function checks native `Send` bounds for the renderer, plan, output and render future. These are compile checks: the GPU function and its assertions never execute. The receipt therefore states `gpuExecuted: false` and `gpuPathValidation: "compile-only"`.

This probe establishes local public-API usability. It does not establish GPU output correctness, nonblocking scheduling, cancellation, stale-result handling, packaged-crate usability, registry publication or M4 completion. Rendering still requires separate runtime and independent-consumer acceptance.

## Retained M3 evidence audit

[verify_acceptance.py](./verify_acceptance.py) is a separate, standard-library Python audit of the existing committed material evidence. It does not render, alter golden files or grant human approval:

```bash
python3 docs/reviews/m3/native-consumer/verify_acceptance.py --check
```

The command compares its result with [acceptance.json](../acceptance.json). It verifies current user acceptance for ceramic, leather and wood; baseline and PBR hash bindings; all expected PNG hashes; declared gates for all eleven 1K cases on both recorded backends; and both 2K traces, input hashes and retained output hashes. Original temporary paths identify historical runs; their temporary files are not required.

The receipt separates two checks. Current retained reports, human decisions, manifests, PNGs and their audited review rules/scripts must still match the base revision byte for byte. Source hashes listed in candidate and trace `inputs` are instead checked against `git show e9dd03b:<path>`: those historical runs bind the source that actually produced them, so later checkout edits do not invalidate that binding. The receipt lists both inventories and their overlap, preserving the original 206-file coverage. A file used as both source input and current review evidence must pass both checks. New source behavior still needs its own validation; this historical audit does not establish it.

The audit checks retained machine results and their declared rules; it does not recompute image metrics. Current `human-review.json` decisions govern human acceptance. Earlier pending image captions and automated `humanAcceptanceClaimed: false` fields retain their historical meaning. All evidence is local, and remote CI remains deferred.

To print a newly verified receipt without changing the saved record:

```bash
python3 docs/reviews/m3/native-consumer/verify_acceptance.py
```
