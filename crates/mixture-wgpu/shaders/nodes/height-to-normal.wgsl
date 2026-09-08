// Image u goes right, v goes down. Tangent +X goes right and +Y goes up (OpenGL).
struct Parameters { strength: f32, _pad0: f32, _pad1: f32, _pad2: f32, }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn height_to_normal(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    let left = vec2<u32>((id.x + size.x - 1u) % size.x, id.y);
    let right = vec2<u32>((id.x + 1u) % size.x, id.y);
    let up = vec2<u32>(id.x, (id.y + size.y - 1u) % size.y);
    let down = vec2<u32>(id.x, (id.y + 1u) % size.y);
    let du = (textureLoad(input, vec2<i32>(right), 0).r - textureLoad(input, vec2<i32>(left), 0).r) * (0.5 * f32(size.x));
    let dv = (textureLoad(input, vec2<i32>(down), 0).r - textureLoad(input, vec2<i32>(up), 0).r) * (0.5 * f32(size.y));
    let normal = normalize(vec3<f32>(-du * parameters.strength, dv * parameters.strength, 1.0));
    textureStore(output, vec2<i32>(id.xy), vec4<f32>(normal * 0.5 + vec3<f32>(0.5), 1.0));
}
