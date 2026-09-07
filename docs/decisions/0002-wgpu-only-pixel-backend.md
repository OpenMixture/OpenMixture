# ADR 0002: wgpu is the only pixel backend

Status: Accepted for M0 foundation. Source: [architecture sections 7 and 9](../../ARCHITECTURE.md).

## Context

Maintaining CPU, WebGL, and native GPU pixel implementations would multiply node work and weaken visual parity.

## Decision

All pixels are executed by `wgpu` compute shaders owned by `mixture-wgpu`, with exactly one WGSL implementation per pixel-producing kernel. M0 introduces no GPU dependency. M1 will prove headless execution and readback before graph work begins.

## Alternatives

A CPU reference renderer duplicates pixel semantics. A separate TypeScript/WebGL executor duplicates both implementation and integration work. A pinned software GPU adapter can instead exercise the actual WGSL path.

## Consequences

No CPU pixel oracle or second renderer may be introduced. Native and browser APIs may select different adapters within the one `wgpu` execution model. Cross-adapter pixel tests use documented tolerances where needed.

## Migration

No old renderer is ported as a compatibility backend. Kernel contracts and shaders are added as focused vertical slices in the initial PR train.

## Verification

M0 checks ensure no GPU dependency or alternate product crate exists. M1 adds compute/readback smoke evidence; later node and material checks exercise the same shaders on a pinned software adapter and real hardware.
