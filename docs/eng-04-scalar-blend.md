# ENG-04 — Scalar field composition design

English | [简体中文](./eng-04-scalar-blend.zh-CN.md)

**Status (2026-09-20): design only.** This defines the bounded engine use case, proposed v1 contract and implementation acceptance plan requested after ENG-01/02. `scalar-blend` is not registered, executable or released. No shader, fixture baseline, format migration or new node is added here. The [roadmap](../ROADMAP.md) separates this design from a later implementation PR.

## Use case and decision

An engine consumer needs one height field combining independently seeded, low-frequency structure and high-frequency detail, with one exposed control. The same graph must execute through native CLI/Rust and the browser SDK without requiring Studio UI work.

```text
fractal-noise (seed 11, scale 4, octaves 1) ─ a ─┐
                                               scalar-blend ─ value ─ material-output.height
fractal-noise (seed 29, scale 32, octaves 1) ─ b ┘      │
                                                  height-to-normal ─ material-output.normal
constant-color ───────────────────────────────────────────────────── material-output.baseColor
```

Expose `detailWeight` → `combine.weight`; request height/normal for the primary comparison. `weight = 0` selects the low-frequency field, `1` selects the detail field, and interior values interpolate. This is a crossfade, not additive displacement: increasing detail also attenuates the low-frequency contribution. If the demonstrated need is additive relief instead, revisit the use case before implementation rather than silently changing this contract.

The existing [blend contract](../crates/mixture-core/src/nodes/blend.rs) requires Color `a`/`b` and returns Color; [levels](../crates/mixture-core/src/nodes/levels.rs) remaps only one Scalar input. The current catalog has no Color-to-Scalar conversion, so these nodes cannot directly express the proposed two-Scalar output. A single dedicated node is preferable to changing `blend` port kinds, implicit conversions or a general math-node family. No claim is made that this design has already passed pixel or human material acceptance.

## Proposed node contract

| Field | Proposed value |
|---|---|
| Type / version | `scalar-blend`, node version `1` |
| Input `a` | Required `Scalar`, no default |
| Input `b` | Required `Scalar`, no default |
| Output `value` | `Scalar` |
| Parameter `weight` | Finite Float in `[0, 1]`, default `0.5`, exposable through the existing binding mechanism |
| Randomness | None; upstream random nodes keep their own required explicit seeds |
| Sampling | Read both inputs at the current integer texel coordinate; no resampling or neighborhood access |
| Normalized range | Clamp each finite input sample to `[0, 1]`; output remains `[0, 1]` |

Define `a0 = clamp(a.r, 0, 1)` and `b0 = clamp(b.r, 0, 1)`. Lower the validated source weight from f64 to f32 using the existing compiler convention. At effective f32 weight `0`, return `a0`; at `1`, return `b0`. Otherwise compute `clamp(a0 + (b0 - a0) * weight, 0, 1)` in WGSL f32, then apply the existing `mixture_half4` storage convention to `(value, 0, 0, 1)` in rgba16float. Endpoint branches preserve exact endpoint selection; they do not prune source validation or introduce a compiler optimization.

Finite out-of-range texels are saturated, not extrapolated. Current public Scalar producers already provide normalized finite values; the focused shader harness must also exercise saturation with injected finite values. NaN/infinite source parameters remain rejected by existing decoding/validation. This does not add a promise to sanitize non-finite GPU data or accept arbitrary injected textures through the public SDK.

The node is pointwise: it preserves periodicity when both fields have compatible tile boundaries, but does not repair seams in a non-periodic input. No color transfer function is involved. Height readback continues to use the existing linear RGBA8 projection; it is not a lossless representation of half-precision intermediates. Interior results retain documented f32/half precision limits and do not promise cross-adapter last-bit equality.

## Source shape and compatibility

The following fragment is **illustrative and unsupported by the current runtime**; it is not a runnable fixture or a new document field:

```json
{
  "id": "combine",
  "type": "scalar-blend",
  "version": 1,
  "parameters": { "weight": 0.5 }
}
```

Connect `low.value` to `combine.a`, `detail.value` to `combine.b`, and `combine.value` to height and height-to-normal. Keep a connected baseColor producer because v1 material-output still requires it. The implementation PR must add the complete graph and its parameter variants as executable fixtures, including explicit noise seeds; this design does not place unimplemented documents in existing accepted fixture directories.

This is an additive node type, with `.mix` document version `1` unchanged. Existing nodes, source fields, defaults and document behavior remain unchanged. Old runtimes must report the existing unknown-node-type diagnostic for documents containing `scalar-blend`; a new runtime must reject unsupported node versions and wrong port kinds through existing structured errors. No silent graph repair or approximate Color-based substitute is allowed.

The proposed compiler adds a typed invocation and exhaustive KernelId mapping to one new WGSL implementation. It uses the existing one-pass allocation/scheduling model; no new crate, JavaScript semantics, cache policy or serialization framework is needed. Review the additive RenderPlan invocation schema explicitly: preserve current plan version/hash domain and old-document hash snapshots if the existing schema permits the addition without changing any old bytes. If implementation requires changing existing serialized layout, lowering or resource semantics, stop and make a separate plan-version decision before proceeding. New plans include the new kernel/ports and effective weight in their deterministic hash; adapter identities and timing stay excluded.

The explicit reviewed registry expectation changes from eleven to twelve types only in the implementation PR. Twelve is a resulting count, not a target or a new ceiling. A changed runtime package needs a new release version and qualification; the published `0.1.0-alpha.0` is immutable. No release version or publication is authorized by this design.

## Implementation and acceptance plan

| Layer | Required evidence before implementation can close |
|---|---|
| Core contract | Defaults and weights `0`, `0.5`, `1`; reject missing inputs, wrong kinds, unsupported version, invalid/unknown parameters and out-of-range/non-finite weight. Exercise exposed override validation, including invalid values on unused branches. |
| Compiler | Deterministic order/serialization/hash across source permutations and equivalent defaults; changed effective weight changes a retained plan; unused output slices omit the node; old hash snapshots unchanged. Endpoint weights still validate both inputs. |
| Focused GPU fixtures | Literal `a=0.25`, `b=0.75` at weights `0`, `0.5`, `1` produce `0.25`, `0.5`, `0.75`; equal inputs, reversed endpoints, saturation, zero/one bounds, rectangular `65×3` and one-pixel dimensions. Use independent literal expectations, not a CPU renderer. |
| Real graph | Deterministic two-noise fixture at weights `0`, `0.25`, `0.5`, `1`; endpoint height bytes match separately rendered source fields, interior output is non-degenerate and responds causally to weight. Inspect tiled contact sheets and height/normal relationship at 1K. |
| Native/browser public consumption | Execute the same fixture, overrides and channels using public Rust/CLI and a candidate browser package. Record plan hashes, build/archive identity, adapter/backend and owned output after destruction. Existing exact checker and browser v2 gates remain intact; define focused tolerances before accepting new pixels. |
| Regression | Existing three material goldens and all variants remain unchanged and pass; registry expectations, node documentation and package declarations/catalog agree. Do not reset existing goldens to accept this feature. |

Start with CPU contract/plan tests, then shader validation, focused GPU tests on the pinned software adapter, material review and Native/browser consumption. Use existing commands: `cargo xtask test-core`, `cargo xtask test-plan`, `cargo xtask shader-check`, `cargo xtask golden check`, and `cargo xtask check`. `cargo xtask test-node scalar-blend` is an **implementation acceptance target, not a working test today**: it becomes valid only when the node and fixtures exist. Add any new browser fixture execution to ENG-03's independent host without removing the retained Studio coverage.

An implementation PR may start after this bounded contract and use case are accepted; it need not wait for all historical consumer tests to migrate. Close that PR only with linked source-bound execution and visual evidence under the [evidence policy](./evidence-policy.md). Design approval, successful compilation and non-empty pixels are not material acceptance.

## Out of scope

Spatial masks, blend modes, addition/multiplication node families, signed/HDR scalar domains, graph rewrites/pass fusion, image resources, `.mixpack`, subgraphs, UI authoring, another pixel executor and changes to existing node precision are excluded. Performance work requires a measured bottleneck; the known warp precision limitation remains separate.
