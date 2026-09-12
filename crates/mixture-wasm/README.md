# mixture-wasm

English | [简体中文](./README.zh-CN.md)

The browser compilation and transport boundary for OpenMixture M5. This unpublished crate calls `mixture-core` for strict raw-source decoding, validation, catalog and immutable compilation, and calls `mixture-wgpu` for explicit browser WebGPU acquisition and asynchronous rendering. It introduces no pixel code or product state.

Public consumers install the independently built `@openmixture/runtime` npm archive. Its facade owns safe JS argument capture and single-render/destroy scheduling. Rust returns fresh catalog/binding metadata and JS-owned RGBA8 copies, and preserves core/GPU diagnostic evidence. Typed u64 values project as bigint; validated numeric parameter values remain numbers. Generated wasm-bindgen glue is an implementation detail.

```sh
cargo check --locked -p mixture-wasm --target wasm32-unknown-unknown
node --test packages/runtime/test/runtime.test.mjs
node scripts/browser-runtime/build.mjs
```

Run these commands from the engine repository. The build script requires the pinned Rust WASM target and wasm-bindgen CLI 0.2.128, and injects one source build identity into JS, types and WASM. Direct Cargo builds identify themselves as unpackaged and cannot be substituted for checked package artifacts. Building does not establish browser rendering acceptance: use the separate product's production consumption tests and retain actual browser/adapter evidence. Full M5 release gates remain separate.
