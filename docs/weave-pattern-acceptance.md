# weave-pattern@1 — frozen node acceptance

English | [简体中文](./weave-pattern-acceptance.zh-CN.md)

Frozen before implementation by the 2026-10-03 maintainer decision on PR #80 (eb3ea30). The [approved node formulas, parameters, sampling and ABI](./mat-03-woven-surfaces.md) are normative. This freezes node cases only; woven-material qualification remains pending. No extra public ports, CPU renderer or schema change.

| Gate | Frozen cases and expectations |
|---|---|
| Admission | Defaults; modes height/coverage/warp-share; even counts 4,8,12,32; widths .55,.7,.9; bevel .02,.08,.12; crown 0,.5,1; underRatio .25,.5,.75. Every inclusive endpoint validates. |
| Rejection | Odd counts 5/31 on either axis, 3/33, float-token 8.0, string/null/bool, widths outside bounds, bad enum, unknown key, unsupported node version. MIX_PARAMETER_INVALID_VALUE carries node ID and parameter; unknown key MIX_PARAMETER_UNKNOWN. Apply to source, overrides and unselected branches; never round. |
| Lowering/ABI | One no-input pass, Scalar rgba16float, 8×8 dispatch; 48 bytes with mode codes 0/1/2 and four zero padding words. Defaults equal explicit defaults. Changed effective parameters change hash; node/edge order and unused changes do not. .mix v1 / plan v3 remain unchanged. |
| Analytical centers | Default 8×8 and unequal 12×8; all crossing centers: even i+j gives Hw=1,Hf=r, odd reverse, C=1, Vw=1/0, H=1. Check r=.25,.5,.75 and crown=0,.5,1; upper>lower for positive relief at analytical centers (no arbitrary tiny-half-relief guarantee). Raw helper tolerance 1/1024. |
| Support/profile | UV (0,0) gives H=C=Vw=0, S=.5. Exposed warp centerline (.5/Nw,0) has C=1 and H>=r. Positive occupancy at inward half-bevel and zero at/outside width boundary; transverse crown strictly decreases away from center. No axial coverage cap. |
| Periodicity/continuity | Reuse production sample helper at UV and UV+(1,0),(0,1),(1,1),(-1,-1), including seam neighborhoods and unequal axes. Raw half tolerance 1/1024; never compare opposite border texels. Visible weights/height approach the same boundary limit from ±1e-5 UV (difference <=.01); selectors in empty space excluded. |
| Filter/mode coherence | Four fixed offsets in documented order. On partial footprints, check output H=sumH/4, C=sumC/4 and S=sumVw/sumC, .5 when sumC=0, against production helper samples (raw-half tolerance 1/1024). Include a footprint where weighted and unweighted share differ by >.01. Check C*S=sumVw/4 within 2/1024 after separate half stores. |
| Public execution | Defaults, modes, endpoints, unequal 12×8 and odd rectangle 257×129; node render sizes 256,1024,2048 and odd rectangle. Finite normalized values, exact same-adapter repeats; Native/browser every RGBA8 component delta <=1/255. Literal fixture probes at empty gaps and full coverage centers. |
| Regression | Explicit reviewed 18-type/16-kernel catalog; shader/core/plan/node/consumer/gpu-smoke/runtime tests, clean browser candidate and hardware parity, final cargo xtask check. No existing-node semantics/golden replacement. |

GPU helper observations are test-only through the production WGSL and sole wgpu executor. Analytical assertions are sparse oracles, not a CPU rendering path. Ordinary reports and mode images stay in ignored tmp/weave-node/ or CI artifacts. Node images are sanity evidence, not accepted woven-fabric appearance.
