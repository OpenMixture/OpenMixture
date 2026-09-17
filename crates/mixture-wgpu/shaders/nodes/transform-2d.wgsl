// v1: pixel-center UV, top-left origin, integer sample periods, visible clockwise turns.
struct Parameters {
    scale: vec2<u32>, quarter_turns: u32, _pad0: u32,
    offset: vec2<f32>, _pad1: vec2<f32>,
}
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;

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
fn transform_2d(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    if all(parameters.scale == vec2<u32>(1u)) && parameters.quarter_turns == 0u && all(parameters.offset == vec2<f32>(0.0)) {
        textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(textureLoad(input, vec2<i32>(id.xy), 0).r, 0.0, 0.0, 1.0)));
        return;
    }
    let uv = (vec2<f32>(id.xy) + vec2<f32>(0.5)) / vec2<f32>(size);
    let p = uv - vec2<f32>(0.5);
    var rotated = p;
    switch parameters.quarter_turns {
        case 1u: { rotated = vec2<f32>(p.y, -p.x); }
        case 2u: { rotated = -p; }
        case 3u: { rotated = vec2<f32>(-p.y, p.x); }
        default: {}
    }
    let sample_uv = rotated * vec2<f32>(parameters.scale) + vec2<f32>(0.5) + parameters.offset;
    textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(repeat_bilinear(sample_uv), 0.0, 0.0, 1.0)));
}
