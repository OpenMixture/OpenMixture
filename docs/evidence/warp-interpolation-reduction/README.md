# Warp interpolation reduction — 2026-09-18

English | [简体中文](./README.zh-CN.md)

The 26 remaining same-input warp differences from the [local texel candidate](../warp-local-texel/README.md) reduce to interpolation of identical samples with identical weights. A diagnostic explicit `fma(b-a,t,a)` removes them on this host, including the full 1024×1024 texture. This is a repair proposal, not material acceptance. Production shaders, goldens, tolerances and PR15 source remain unchanged.

## Measurements

Native uses GT 1030/DX12, driver 32.0.15.8266, ordinary registry wgpu and the recorded Chrome DXC modules. Chrome 153.0.8010.48 and Edge 153.0.4234.32 use direct WebGPU diagnostic pages. Browser adapter reports identify NVIDIA/Pascal but do not expose the WebGPU backend. [Summary](./summary.json), scalar buffers, module receipts and adapter contexts bind the results.

| Test | Native versus Chrome |
|---|---:|
| Local displacement, 26 witnesses | 0 differences |
| Fractional weight, 26 witnesses | 0 differences |
| Nested mix, fixed-weight mix, or `a+(b-a)*t` | 26 half differences each |
| Explicit fma, 26 witnesses | 0 half differences |
| Full same-input warp texture, original interpolation | 26 differing pixels |
| Full same-input warp texture, explicit fma interpolation | 0 differing pixels |

Edge exactly reproduces Chrome's fixed-mix and fma scalar buffers and the fma full texture. Native before/after full texture bytes are identical. Native regenerated stretch/distortion inputs are identical to the frozen inputs. The grid changes only three interpolation expressions to explicit fma; [full external shader](./warp-fma.wgsl) and [pipeline](./pipeline.json) are retained. The warp source is based on `91015e02255f20105586145db23bbd23b3c93546`; this diagnostic pipeline inherits earlier value-noise wood stages, including unused cellular candidate spelling. It is not the archived PR15 runtime.

Each scalar execution exposes only one result: instrumenting all intermediates can alter optimization. Original mix and half probes reproduce all 26 actual texture half values on both sides. [Exact rational calculations](./oracle.json) use stored f32 samples/weights. At `(922,23)`, `a=0.42626953125`, `b=0.3408203125`, `t=0.9699997901916504`; exact interpolation is `1474822221/4294967296`. Native mix and both explicit-fma results round to half bits `13695`; Chrome/Edge mix yields `13694`. For these selected 26 witnesses, native mix and all explicit-fma results match the exact scalar nearest-even half; Chrome/Edge mix matches none. This is an oracle for fixed scalar inputs, not a new material reference or proof of exact rounding over the whole grid.

## Decision and limits

The next bounded candidate is explicit interpolation in warp alone, preserving local coordinates. It needs independent regression cases for nonzero weights on both axes, negative/full-period shifts, thin and non-power-of-two textures, then frozen software goldens and the installed three-browser material matrix. This one default wood input has zero Y displacement. No general fma accuracy improvement or universal parity is established.

[WGSL fma](https://gpuweb.github.io/gpuweb/wgsl/#fma-builtin) permits an unfused multiply followed by addition; [reassociation and fusion](https://gpuweb.github.io/gpuweb/wgsl/#reassociation-and-fusion) also permit arithmetic variation. Reviewed 2026-09-18. The observed mismatch is not evidence of a specification violation, and explicit fma is not a portable bit-identity guarantee. No compiler policy, dependency fork or upstream issue is introduced. PR15 still fails frozen software wood goldens; this experiment leaves its native pixels unchanged and does not repair that compatibility failure. No baseline or tolerance selection can turn this result into acceptance.

## Reproduction and retention

Run `python docs/evidence/warp-interpolation-reduction/verify.py` (standard library only) to verify texture-bound scalar witnesses, exact rational half results, Edge replay and grid content identities. This audits retained evidence; it does not rerun a GPU or certify spec conformance.

For a fresh GPU run, build the existing [scalar native runner](../chromium-arithmetic-reduction/native-probe.rs) and [texture native runner](../residual-path-reduction/texture_probe.rs) using their documented isolated checkout procedure and ordinary locked dependencies. Scope the recorded DXC directory to PATH. Run scalar shaders as `arithmetic_probe.exe <shader.wgsl> <output.bin> 832 <inputs.bin>`; use the existing [browser scalar runner](../chromium-arithmetic-reduction/browser-probe.mjs) with the same arguments after its CDP endpoint. Each input has 32 floats, only the first output lane is meaningful. Supply the retained input bytes unchanged.

For the grid, run `texture_probe.exe <pipeline.json> <new-native-output-directory>`. Extract [grid-evidence.zip](./grid-evidence.zip) into a temporary directory; use its original stretch/distortion bytes to overwrite those two new native input files before browser execution. Set `MIXTURE_PROBE_PRODUCT` to the pinned Studio product with Playwright installed, `MIXTURE_GRID_PIPELINE` to the absolute retained pipeline path, and `MIXTURE_GRID_INPUTS` to that native output directory. Run `node docs/evidence/warp-interpolation-reduction/browser-grid.mjs <CDP-endpoint> <new-browser-output-directory> native`. Compare full 8 MiB rgba16float textures. Browser validation and native shader compilation ran before dispatch in this experiment.

Git retains original scalar bytes, shader sources, rational results, contexts, and compressed full shared input/failing output contents. All three fma grid outputs equal the retained pre-change native bytes, so identical copies are deduplicated and separately hash-bound in the summary. Executables, routine logs and duplicated raw outputs remain in ignored `tmp/warp-residual*`. No visual acceptance or new golden is claimed.

Chrome's fresh-profile launch receipt is retained. Edge's launcher failed to observe its transient parent, but its surviving owned-profile process and CDP endpoint were observed and used; [process record](./edge-observed.json) documents that limitation. This is not a successful ordinary-browser qualification receipt. No GPU feature override was added.
