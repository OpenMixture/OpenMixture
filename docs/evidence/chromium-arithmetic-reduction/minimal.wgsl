@group(0) @binding(0) var<storage, read> inputs: array<f32>;
@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;
@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id: vec3<u32>) {
 if id.x >= arrayLength(&outputs) { return; }
 outputs[id.x] = inputs[0] * inputs[1] + inputs[2];
}
