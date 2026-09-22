struct Parameters { opacity: f32, _pad0: u32, _pad1: u32, _pad2: u32, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input_a: texture_2d<f32>;
@group(0) @binding(3) var input_b: texture_2d<f32>;
@group(0) @binding(4) var input_mask: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn scalar_mask_blend(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= textureDimensions(output)) { return; }
    let position = vec2<i32>(id.xy);
    let a = clamp(textureLoad(input_a, position, 0).r, 0.0, 1.0);
    let b = clamp(textureLoad(input_b, position, 0).r, 0.0, 1.0);
    let t = parameters.opacity * clamp(textureLoad(input_mask, position, 0).r, 0.0, 1.0);
    var value = a;
    if t == 1.0 { value = b; }
    else if t != 0.0 { value = clamp(a + (b - a) * t, 0.0, 1.0); }
    textureStore(output, position, mixture_half4(vec4<f32>(value, 0.0, 0.0, 1.0)));
}
