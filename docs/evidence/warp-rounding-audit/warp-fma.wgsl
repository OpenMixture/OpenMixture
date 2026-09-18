// v1: output-texel scalar field, centered at 0.5; signed displacement in UV units.
struct Parameters { strength: vec2<f32>, _pad: vec2<f32>, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;
@group(0) @binding(3) var displacement: texture_2d<f32>;

fn repeat_bilinear(pixel: vec2<i32>, delta_texels: vec2<f32>) -> f32 {
    let size = vec2<i32>(textureDimensions(input));
    // Plan textures share a resolution: pixel-center sampling is pixel + delta.
    // Keep the integer pixel out of f32 arithmetic so small weights survive.
    let base = pixel + vec2<i32>(floor(delta_texels));
    let t = fract(delta_texels);
    // Clamped field and strength in [-1,1] bound delta to [-size,size].
    // Thus base >= -size; adding size makes every neighbor modulo nonnegative.
    let a = (base + size) % size;
    let b = (base + vec2<i32>(1, 0) + size) % size;
    let c = (base + vec2<i32>(0, 1) + size) % size;
    let d = (base + vec2<i32>(1, 1) + size) % size;
    let va = textureLoad(input, a, 0).r;
    let vb = textureLoad(input, b, 0).r;
    let vc = textureLoad(input, c, 0).r;
    let vd = textureLoad(input, d, 0).r;
    // Request multiply-add interpolation to reduce intermediate rounding.
    // WGSL permits unfused fma; this does not promise backend bit identity.
    let top = fma(vb - va, t.x, va);
    let bottom = fma(vd - vc, t.x, vc);
    return fma(bottom - top, t.y, top);
}
@compute @workgroup_size(8, 8, 1)
fn warp(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    let field = clamp(textureLoad(displacement, vec2<i32>(id.xy), 0).r, 0.0, 1.0);
    if field == 0.5 || all(parameters.strength == vec2<f32>(0.0)) {
        textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(textureLoad(input, vec2<i32>(id.xy), 0).r, 0.0, 0.0, 1.0)));
        return;
    }
    let delta_texels = (2.0 * field - 1.0) * parameters.strength * vec2<f32>(size);
    textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(repeat_bilinear(vec2<i32>(id.xy), delta_texels), 0.0, 0.0, 1.0)));
}
