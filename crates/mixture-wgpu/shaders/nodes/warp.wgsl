// v1: output-texel scalar field, centered at 0.5; signed displacement in UV units.
struct Parameters { strength: vec2<f32>, _pad: vec2<f32>, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;
@group(0) @binding(3) var displacement: texture_2d<f32>;

fn repeat_bilinear(uv: vec2<f32>) -> f32 {
    let size = vec2<i32>(textureDimensions(input));
    let p = fract(uv) * vec2<f32>(size) - vec2<f32>(0.5);
    let base = vec2<i32>(floor(p));
    let t = fract(p);
    // fract bounds base to [-1, size-1]; adding size makes each modulo nonnegative.
    let a = (base + size) % size;
    let b = (base + vec2<i32>(1, 0) + size) % size;
    let c = (base + vec2<i32>(0, 1) + size) % size;
    let d = (base + vec2<i32>(1, 1) + size) % size;
    return mix(mix(textureLoad(input, a, 0).r, textureLoad(input, b, 0).r, t.x),
               mix(textureLoad(input, c, 0).r, textureLoad(input, d, 0).r, t.x), t.y);
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
    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) / vec2<f32>(size);
    let sample_uv = uv + (2.0 * field - 1.0) * parameters.strength;
    textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(repeat_bilinear(sample_uv), 0.0, 0.0, 1.0)));
}
