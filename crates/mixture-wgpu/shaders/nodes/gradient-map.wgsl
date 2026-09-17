struct Parameters { color_a: vec4<f32>, color_b: vec4<f32>, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn gradient_map(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= textureDimensions(output)) { return; }
    let t = clamp(textureLoad(input, vec2<i32>(id.xy), 0).r, 0.0, 1.0);
    textureStore(output, vec2<i32>(id.xy), mixture_half4(fma(vec4<f32>(t), parameters.color_b - parameters.color_a, parameters.color_a)));
}
