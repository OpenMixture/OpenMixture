struct Parameters {
    cells_seed: vec4<u32>,
    shape: vec4<f32>,
    variation_padding: vec4<f32>,
}
@group(0) @binding(0) var<uniform> params: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;

fn brick_hash(input: u32) -> u32 {
    var z = input;
    z ^= z >> 16u;
    z *= 0x7feb352du;
    z ^= z >> 15u;
    z *= 0x846ca68bu;
    z ^= z >> 16u;
    return z;
}

fn brick_sample(uv: vec2<f32>) -> f32 {
    let y = uv.y * f32(params.cells_seed.y);
    let row = u32(floor(y));
    let x = uv.x * f32(params.cells_seed.x) - f32(row % 2u) * params.shape.x;
    let local = fract(vec2<f32>(x, y));
    let inward = (vec2<f32>(1.0) - params.shape.yz) * 0.5 - abs(local - vec2<f32>(0.5));
    let d = min(inward.x, inward.y);
    if d <= 0.0 { return 0.0; }
    var profile = 1.0;
    if params.shape.w > 0.0 {
        let t = clamp(d / params.shape.w, 0.0, 1.0);
        profile = t * t * (3.0 - 2.0 * t);
    }
    let columns = i32(params.cells_seed.x);
    let column = u32((i32(floor(x)) + columns) % columns);
    let h = brick_hash(params.cells_seed.z ^ brick_hash(column) ^ brick_hash(row + 0x9e3779b9u));
    let r = f32(h >> 20u) / 4096.0;
    return profile * (1.0 - params.variation_padding.x * r);
}

@compute @workgroup_size(8, 8, 1)
fn brick_pattern(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(output);
    if any(gid.xy >= size) { return; }
    let p = vec2<f32>(gid.xy);
    let dimensions = vec2<f32>(size);
    let value = (
        brick_sample((p + vec2<f32>(0.25, 0.25)) / dimensions) +
        brick_sample((p + vec2<f32>(0.75, 0.25)) / dimensions) +
        brick_sample((p + vec2<f32>(0.25, 0.75)) / dimensions) +
        brick_sample((p + vec2<f32>(0.75, 0.75)) / dimensions)
    ) * 0.25;
    textureStore(output, vec2<i32>(gid.xy), mixture_half4(vec4<f32>(value, 0.0, 0.0, 1.0)));
}
