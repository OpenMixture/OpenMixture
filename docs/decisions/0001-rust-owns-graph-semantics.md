# ADR 0001: Rust owns graph semantics

English | [简体中文](./0001-rust-owns-graph-semantics.zh-CN.md)

Status: Accepted for M0 foundation. Source: [architecture sections 3–4](../../ARCHITECTURE.md).

## Context

Native and future browser consumers must agree on document meaning, node contracts, validation, and compilation. Multiple semantic implementations would drift.

## Decision

`mixture-core` owns these semantics and backend-neutral diagnostics in ordinary typed Rust modules. `mixture-wgpu` consumes compiled plans; `mixture-cli` performs I/O and orchestration. A future WebAssembly crate remains a thin binding over the same libraries.

## Alternatives

A TypeScript graph runtime or CLI-owned compiler would create a second semantic owner. A single monolithic crate would couple pure graph tests to platform and command-line dependencies.

## Consequences

Core tests require no adapter, CLI, or browser environment. Product dependency edges point toward core. M0 establishes empty crate roots rather than speculative document APIs.

## Migration

This is a new repository with no compatibility promise. Legacy Mixture code is research material; no automatic import or compatibility layer is required.

## Verification

`cargo xtask deps` enforces the M0 dependency graph and `cargo xtask check` compiles all crates. Later format and plan tests belong to core and must run without a GPU.
