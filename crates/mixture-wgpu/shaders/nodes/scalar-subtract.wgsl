struct Parameters { reserved: vec4<u32>, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input_a: texture_2d<f32>;
@group(0) @binding(3) var input_b: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn scalar_subtract(@builtin(global_invocation_id) id: vec3<u32>) {
    // The shared kernel ABI retains a reserved zero uniform even without parameters.
    if parameters.reserved.x != 0u || any(id.xy >= textureDimensions(output)) { return; }
    let position = vec2<i32>(id.xy);
    let a = clamp(textureLoad(input_a, position, 0).r, 0.0, 1.0);
    let b = clamp(textureLoad(input_b, position, 0).r, 0.0, 1.0);
    let value = max(a - b, 0.0);
    textureStore(output, position, mixture_half4(vec4<f32>(value, 0.0, 0.0, 1.0)));
}
