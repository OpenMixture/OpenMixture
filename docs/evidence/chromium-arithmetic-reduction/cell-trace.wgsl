
@group(0) @binding(0) var<storage, read> inputs: array<f32>;
@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;
fn avalanche(value: u32) -> u32 {
    var h = value;
    h = (h ^ (h >> 16u)) * 0x7feb352du;
    h = (h ^ (h >> 15u)) * 0x846ca68bu;
    return h ^ (h >> 16u);
}
fn lattice(cell: vec2<u32>, period: u32, seed: u32) -> f32 {
    let wrapped = cell % vec2<u32>(period);
    let h = avalanche(seed ^ avalanche(wrapped.x + 0x9e3779b9u) ^ avalanche(wrapped.y + 0x85ebca6bu));
    // The high 24 bits convert exactly to f32, then scale by an exact power of two.
    return f32(h >> 8u) * (1.0 / 16777216.0);
}

@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id: vec3<u32>) {
 let i=id.x; if i>=216u {return;}
 let octave=i/72u; let slot=(i%72u)/8u; let component=i%8u;
 let period=64u<<octave;let seed=avalanche(271828u+octave*0x9e3779b9u);
 let uv=(vec2<f32>(270.0,0.0)+vec2<f32>(0.5))/1024.0;
 let p=uv*f32(period);let cell=vec2<i32>(floor(p));
 let neighbor=cell+vec2<i32>(i32(slot%3u)-1,i32(slot/3u)-1);
 let wrapped=vec2<u32>((neighbor+vec2<i32>(i32(period)))%vec2<i32>(i32(period)));
 let jitter=vec2<f32>(lattice(wrapped,period,seed),lattice(wrapped,period,seed^0x68bc21ebu));
 let delta=vec2<f32>(neighbor)+vec2<f32>(0.2)+0.6*jitter-p+vec2<f32>(inputs[i]);
 let values=array<f32,8>(jitter.x,jitter.y,delta.x,delta.y,dot(delta,delta),f32(neighbor.x),f32(neighbor.y),p.x);
 outputs[i]=values[component];
}
