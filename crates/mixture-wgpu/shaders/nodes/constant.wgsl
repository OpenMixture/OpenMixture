// Shared constant kernel: core supplies scalar/color/encoded-normal packing.
struct Parameters { value: vec4<f32>, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(8, 8, 1)
fn constant(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= textureDimensions(output)) { return; }
    textureStore(output, vec2<i32>(id.xy), mixture_half4(parameters.value));
}
