struct Parameters {
    input_min: f32, input_max: f32, gamma: f32, output_min: f32,
    output_max: f32, _pad0: f32, _pad1: f32, _pad2: f32,
}
@group(0) @binding(0) var<uniform> parameters: Parameters;
@group(0) @binding(1) var output: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var input: texture_2d<f32>;
@compute @workgroup_size(8, 8, 1)
fn levels(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= textureDimensions(output)) { return; }
    let value = textureLoad(input, vec2<i32>(id.xy), 0).r;
    var result: f32;
    // Exact endpoints also avoid evaluating pow(0, x) and divisions in tiny
    // intervals containing no representable rgba16float input between bounds.
    if value <= parameters.input_min {
        result = parameters.output_min;
    } else if value >= parameters.input_max {
        result = parameters.output_max;
    } else {
        let t = clamp((value - parameters.input_min) / (parameters.input_max - parameters.input_min), 0.0, 1.0);
        result = parameters.output_min + pow(t, 1.0 / parameters.gamma) * (parameters.output_max - parameters.output_min);
    }
    textureStore(output, vec2<i32>(id.xy), vec4<f32>(result, 0.0, 0.0, 1.0));
}
