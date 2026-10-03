// Frozen weave-pattern@1: one geometry calculation for all three Scalar modes.
struct Parameters {
    counts_mode: vec4<u32>,
    shape: vec4<f32>,
    under_padding: vec4<f32>,
}
@group(0) @binding(0) var<uniform> params: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
struct WeaveSample {
    heights: vec2<f32>,
    occupancy: vec2<f32>,
    // H, C, Vw; the visible weft weight is C - Vw.
    fields: vec3<f32>,
}
fn weave_smooth(t: f32) -> f32 {
    let s = clamp(t, 0.0, 1.0);
    return s * s * (3.0 - 2.0 * s);
}
fn weave_parity(index: i32) -> i32 { return ((index % 2) + 2) % 2; }
fn weave_sample(uv: vec2<f32>) -> WeaveSample {
    let cell = fract(uv) * vec2<f32>(params.counts_mode.xy);
    let ij = vec2<i32>(floor(cell));
    let local = fract(cell) - vec2<f32>(0.5);
    let widths = params.shape.xy;
    let bevel = params.shape.z;
    let crown = params.shape.w;
    let r = params.under_padding.x;
    let aw = weave_smooth((widths.x * 0.5 - abs(local.x)) / bevel);
    let af = weave_smooth((widths.y * 0.5 - abs(local.y)) / bevel);
    let scaled = 2.0 * local / widths;
    let q = max(vec2<f32>(1.0) - scaled * scaled, vec2<f32>(0.0));
    let profile = q * ((1.0 - crown) + crown * q);
    let axial = cell - vec2<f32>(0.5);
    let kl = vec2<i32>(floor(axial));
    let ft = fract(axial);
    let sy = weave_smooth(ft.y);
    let sx = weave_smooth(ft.x);
    let lw = select(sy, 1.0 - sy, weave_parity(ij.x + kl.y) == 0);
    let lf = select(1.0 - sx, sx, weave_parity(ij.y + kl.x) == 0);
    let heights = profile * (r + (1.0 - r) * vec2<f32>(lw, lf));
    let e = bevel * (1.0 - r);
    let d = weave_smooth((heights.x - heights.y + e) / (2.0 * e));
    let c = aw + (1.0 - aw) * af;
    let vw = clamp(aw * (1.0 - af * (1.0 - d)), 0.0, c);
    let h = vw * heights.x + (c - vw) * heights.y;
    return WeaveSample(heights, vec2<f32>(aw, af), clamp(vec3<f32>(h,c,vw),vec3<f32>(0.0),vec3<f32>(1.0)));
}
fn weave_box(p: vec2<f32>, dimensions: vec2<f32>) -> vec3<f32> {
    let total = weave_sample((p + vec2<f32>(0.25,0.25)) / dimensions).fields
        + weave_sample((p + vec2<f32>(0.75,0.25)) / dimensions).fields
        + weave_sample((p + vec2<f32>(0.25,0.75)) / dimensions).fields
        + weave_sample((p + vec2<f32>(0.75,0.75)) / dimensions).fields;
    var share = 0.5;
    if total.y > 0.0 { share = total.z / total.y; }
    return clamp(vec3<f32>(total.x * 0.25,total.y * 0.25,share),vec3<f32>(0.0),vec3<f32>(1.0));
}
@compute @workgroup_size(8,8,1)
fn weave_pattern(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(output);
    if any(gid.xy >= size) { return; }
    let fields = weave_box(vec2<f32>(gid.xy),vec2<f32>(size));
    let value = fields[params.counts_mode.z];
    textureStore(output,vec2<i32>(gid.xy),mixture_half4(vec4<f32>(value,0.0,0.0,1.0)));
}
