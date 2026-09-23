# Painted metal — MAT-02a design review

English | [简体中文](./README.zh-CN.md)

These are design inputs for the [MAT-02 contract](../../../docs/mat-02-layered-weathering.md), not an implemented material or accepted result. [Graph design](./graph-design.json) is deliberately **not `.mix`**: the two new node identities are not implemented and its `$` values are caller substitutions, not a new runtime expression language. [Qualification plan](./qualification-plan.json) specifies seven presets × four sizes × five channels (140 comparisons per Native/browser pairing), but remains `mat-02a-review-before-freeze` until its listed prerequisites are resolved. Do not run it as a material fixture or report its numerical targets as measurements.

## Exact caller controls

Unknown controls, non-finite values and values outside these ranges must be rejected by the fixture request builder. This is a future fixture harness contract, not a new Core API. Merge preset controls over defaults before mapping. Explicit macro/detail seeds are u32; preserve both, including when detail is disabled. All colors are linear RGBA in [0,1] with alpha fixed at 1.

| Public control | Range | Mapping |
|---|---|---|
| `exposureAmount` | Float [0,1] | For 0<a<1, exposure levels inputMin=0.8*(1-a), inputMax=inputMin+0.2, outputMin=0, outputMax=1. At a=0 or 1 use inputMin=0, inputMax=1 and both outputs equal a. Gamma is always 1. Exact endpoints do not depend on noise extrema. |
| `exposureScale` | Integer [1,64] | Macro noise scale; three octaves and persistence 0.5 are fixed. Default quality applies to the default scale, not all high-frequency inputs. |
| `macroSeed`, `detailSeed` | u32 | Respective noise seed. Detail noise scale 32, two octaves, persistence 0.5, both bases explicitly `value` with version 2. |
| `edgeWidth` | Integer [0,8] | Reference texels at 1024; use the plan's independent per-axis radius formula. Reject axes >2048 in this qualification harness. |
| `rustAmount`, `rustFill`, `detailAmount` | Float [0,1] | Rust mask opacity, band-to-exposure interpolation weight, and detail levels outputMin=1-detailAmount respectively; outputMax=1. |
| `paintColor`, `substrateColor`, `rustColor` | Linear RGBA | Corresponding constant-color value; alpha must remain 1. |
| `paintRoughness`, `substrateRoughness`, `rustRoughness` | Float [0,1] | First two are coatingRoughness output endpoints; third is rustRoughness constant. Reversed levels output endpoints are intentional and supported. |
| `paintThickness` | Float [0,0.5] | coatingHeight outputMin=0.2+paintThickness, outputMax=0.2. |
| `rustRelief` | Float [0,paintThickness] | rustHeight constant=0.2+rustRelief; reject relief above thickness instead of silently clamping. |
| `normalStrength` | Float [0,8] | Existing height-to-normal strength. |

Output names map to ordinary material-output ports when the real `.mix` is constructed. Inspection aliases expose intermediate masks only to qualification requests; they do not add material channel types or public expression syntax. Every render starts from the same graph topology, including endpoint presets, so controls cannot hide unwanted dependencies by replacing the graph. Dependency slicing for a requested output remains Core-owned.

The conceptual formulas describe structure; actual references must honor each existing node's half-float intermediate rounding. For mask inequalities, inspect raw half values rather than infer exact values from RGBA8. Endpoint pixels are compared with independently constructed constant reference graphs through the sole wgpu executor, including its existing normal encoding/conversion. A half-float difference must not be hidden by an RGBA8-only subtraction probe.

`detailAmount` modulates rust coverage, hence downstream relief/roughness/color; it does not move W. `rustFill=0` restricts rust to the eroded edge band, while `rustFill=1` permits coverage throughout exposure. Zero edge width forces B=0; it does not force R=0 when rustFill>0. Paint remains at or above the permitted rust/substrate height endpoints. This does not claim every transition height equals one endpoint.

## Resource decision and remaining freeze work

The explicit graph has 23 pixel nodes and one eventual structural material-output node. The 24-pass ceiling is retained. Using levels for the two coating endpoint interpolations avoids four redundant constant/interpolation passes while preserving their public controls; it changes no node implementation.

The current executor retains every rgba16float intermediate. At 2048², 23 textures alone require **771,751,936 bytes**, above the unchanged **536,870,912-byte** descriptor ceiling, before uniforms and the sequential readback buffer. This is a static design estimate, not a compiled plan, measured GPU allocation or accepted performance result. Core also defaults to a 512 MiB transient-byte limit, so the current 2K graph is expected to be rejected before execution. Once its nodes exist, retain the actual structured 2K budget failure plus a valid 1K compiled plan and render baseline before opening a separate PERF-MAT lifetime/reuse slice. Both Core estimates and executor lifetimes must agree; do not raise the global limit to collect a passing 2K result. Do not raise the ceiling or add pass fusion/global caches to conceal the failure. The graph and quality goals stay fixed through that optimization.

Before freezing MAT-02a, establish default downsample feasibility from existing-node inputs and the declared radius mapping, review exact probe fields and analytical oracles, and record catalog/version selection. Existing-node feasibility does not qualify unimplemented morphology or the finished material. Admission, implementation, qualification and publication remain separate.
