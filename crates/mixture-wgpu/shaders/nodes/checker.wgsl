// Built-in checker v1: eight cells per axis, opaque black at the top-left.
struct Parameters {
    width: u32,
    height: u32,
    cells_x: u32,
    cells_y: u32,
}

@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn checker(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= parameters.width || id.y >= parameters.height {
        return;
    }
    let cell_x = id.x * parameters.cells_x / parameters.width;
    let cell_y = id.y * parameters.cells_y / parameters.height;
    let value = f32((cell_x + cell_y) % 2u);
    textureStore(output, vec2<i32>(id.xy), vec4<f32>(value, value, value, 1.0));
}
