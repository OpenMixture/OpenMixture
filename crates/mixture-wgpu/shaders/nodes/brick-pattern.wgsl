// Integer pixel-center remainders preserve cell boundaries and row parity.
struct Parameters {
    cells: vec2<u32>, half_offset: u32, _pad: u32,
    gap: f32, bevel: f32, _pad2: vec2<f32>,
}
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(8, 8, 1)
fn brick_pattern(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    let denominator = 2u * size;
    var numerator = (2u * id.xy + vec2<u32>(1u)) * parameters.cells;
    let row = numerator.y / denominator.y;
    if parameters.half_offset != 0u && (row % 2u) == 1u {
        numerator.x += size.x;
    }
    let remainder = numerator % denominator;
    let distance = vec2<f32>(min(remainder, denominator - remainder)) / vec2<f32>(denominator);
    let inset = min(distance.x, distance.y) - parameters.gap * 0.5;
    var value = 0.0;
    if inset > 0.0 {
        value = 1.0;
        if parameters.bevel > 0.0 { value = min(inset / parameters.bevel, 1.0); }
    }
    textureStore(output, vec2<i32>(id.xy), mixture_half4(vec4<f32>(value, 0.0, 0.0, 1.0)));
}
