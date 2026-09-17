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
