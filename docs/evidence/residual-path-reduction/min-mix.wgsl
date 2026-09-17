@group(0) @binding(0) var<storage,read> inputs:array<f32>;
@group(0) @binding(1) var<storage,read_write> outputs:array<f32>;
@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>){let i=id.x*4u;if i>=arrayLength(&inputs){return;}outputs[i]=mix(inputs[i],inputs[i+1u],inputs[i+2u]);}