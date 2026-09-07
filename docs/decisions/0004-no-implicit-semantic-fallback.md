# ADR 0004: No implicit semantic fallback

Status: Accepted for M0 foundation. Source: [architecture sections 9 and 12](../../ARCHITECTURE.md).

## Context

A rendering request must either use its declared execution model or explain why it failed. Hidden global state and fallback can conceal environment failures and change output semantics.

## Decision

GPU acquisition is explicit. A caller-owned context retains instance, adapter, device, and queue; renderer-owned caches have explicit lifetimes. Reports include requested policy and actual adapter/backend. Failures return typed codes and stages with preserved driver evidence, never a switch to another semantic executor.

## Alternatives

A global lazy device hides ownership and initialization failures. An automatic GPU-to-CPU or WebGL chain can report success while changing semantics. Neither is accepted.

## Consequences

Consumers must handle structured failure. `wgpu` adapter selection within explicit policy is allowed. No process-global renderer or cache is introduced. Future `doctor` health claims require the level of evidence specified by the PR train.

## Migration

There is no fallback compatibility behavior to preserve. Public error vocabulary begins in PR-002; GPU context and diagnostics follow in M1.

## Verification

M0 CLI tests ensure unimplemented operations fail instead of claiming success. M1 must test missing-adapter failure and preserve requested/selected adapter evidence before declaring compute/readback healthy.
