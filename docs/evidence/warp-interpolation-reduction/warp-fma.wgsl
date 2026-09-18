// Round f32 to binary16, ties to even, while retaining an exactly representable
// f32 result. Texture storage then cannot choose a different rounding direction.
// Integer rounding avoids backend-dependent quantizeToF16 implementations.
fn mixture_half(value: f32) -> f32 {
    let bits = bitcast<u32>(value);
    let sign = bits & 0x80000000u;
    let magnitude = bits & 0x7fffffffu;
    if magnitude >= 0x7f800000u { return value; } // Preserve invalid data for readback diagnostics.
    if magnitude >= 0x38800000u {
        let rounded = (magnitude + 0xfffu + ((magnitude >> 13u) & 1u)) & 0xffffe000u;
        return bitcast<f32>(sign | rounded);
    }
    if magnitude < 0x33000000u { return bitcast<f32>(sign); }
    // Half subnormals are integer multiples of 2^-24. Here shift is 14..24.
    let shift = 126u - (magnitude >> 23u);
    let mantissa = (magnitude & 0x7fffffu) | 0x800000u;
    let rounded = (mantissa + (1u << (shift - 1u)) - 1u + ((mantissa >> shift) & 1u)) >> shift;
    return bitcast<f32>(sign | bitcast<u32>(f32(rounded) * (1.0 / 16777216.0)));
}
fn mixture_half4(value: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(mixture_half(value.x), mixture_half(value.y), mixture_half(value.z), mixture_half(value.w));
}

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
