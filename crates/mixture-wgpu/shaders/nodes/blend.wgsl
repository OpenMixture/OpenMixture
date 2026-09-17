struct Parameters { mode: u32, opacity: f32, _pad0: u32, _pad1: u32, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input_a: texture_2d<f32>;
@group(0) @binding(3) var input_b: texture_2d<f32>;
@group(0) @binding(4) var input_mask: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn blend(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= textureDimensions(output)) { return; }
    let position = vec2<i32>(id.xy);
    let a = textureLoad(input_a, position, 0);
    let b = textureLoad(input_b, position, 0);
    let t = parameters.opacity * clamp(textureLoad(input_mask, position, 0).r, 0.0, 1.0);
    var mode_rgb = b.rgb;
    if parameters.mode == 1u { mode_rgb = a.rgb * b.rgb; }
    if parameters.mode == 2u { mode_rgb = vec3<f32>(1.0) - (vec3<f32>(1.0) - a.rgb) * (vec3<f32>(1.0) - b.rgb); }
    textureStore(output, position, mixture_half4(vec4<f32>(mix(a.rgb, mode_rgb, t), mix(a.a, b.a, t))));
}
