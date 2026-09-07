// One checker implementation shared by the fixed probe and compiled graphs.
struct Parameters {
    cells: vec2<u32>, _padding: vec2<u32>,
    color_a: vec4<f32>, color_b: vec4<f32>,
}
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@compute @workgroup_size(8, 8, 1)
fn checker(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    let cell = id.xy * parameters.cells / size;
    let value = select(parameters.color_a, parameters.color_b, (cell.x + cell.y) % 2u == 1u);
    textureStore(output, vec2<i32>(id.xy), value);
}
