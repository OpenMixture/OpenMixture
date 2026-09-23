struct Parameters { operation: u32, axis: u32, radius: u32, _pad: u32 }
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn scalar_morphology(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(output);
    if any(id.xy >= size) { return; }
    let p = vec2<i32>(id.xy);
    let extent = i32(select(size.x, size.y, parameters.axis == 1u));
    var result = clamp(textureLoad(input, p, 0).r, 0.0, 1.0);
    let radius = i32(parameters.radius);
    for (var k = -radius; k <= radius; k += 1) {
        var q = p;
        // Double modulo handles negative offsets even when the radius exceeds the axis.
        if parameters.axis == 0u { q.x = ((p.x + k) % extent + extent) % extent; }
        else { q.y = ((p.y + k) % extent + extent) % extent; }
        let value = clamp(textureLoad(input, q, 0).r, 0.0, 1.0);
        if parameters.operation == 0u { result = min(result, value); }
        else { result = max(result, value); }
    }
    textureStore(output, p, mixture_half4(vec4<f32>(result, 0.0, 0.0, 1.0)));
}
