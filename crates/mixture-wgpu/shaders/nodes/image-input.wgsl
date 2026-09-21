struct Params { reserved: vec4<u32>, }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input_texture: texture_2d<f32>;

@compute @workgroup_size(8, 8, 1)
fn image_input(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    if id.x >= size.x || id.y >= size.y { return; }
    let value = textureLoad(input_texture, vec2<i32>(id.xy), 0).r;
    textureStore(output_texture, vec2<i32>(id.xy), mixture_half4(vec4<f32>(value, 0.0, 0.0, 1.0)));
}
