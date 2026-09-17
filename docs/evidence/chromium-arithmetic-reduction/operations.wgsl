@group(0) @binding(0) var<storage, read> inputs: array<f32>;
@group(0) @binding(1) var<storage, read_write> outputs: array<f32>;
fn avalanche(value: u32) -> u32 {
 var h=value; h=(h^(h>>16u))*0x7feb352du; h=(h^(h>>15u))*0x846ca68bu; return h^(h>>16u);
}
@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id: vec3<u32>) {
 let i=id.x; if i>=arrayLength(&inputs) {return;}
 let n=i/8u;
 let jitter=f32(avalanche(n)>>8u)*(1.0/16777216.0)+inputs[i];
 let t=f32(n%1024u)/1024.0;
 let neighbor=f32(n%64u);
 let p=neighbor+0.03125;
 var value=0.0;
 switch i%8u {
 case 0u: { value=neighbor+0.2+0.6*jitter-p; }
 case 1u: { value=0.2+0.6*jitter; }
 case 2u: { value=t*t*t*(t*(t*6.0-15.0)+10.0); }
 case 3u: { value=mix(0.025,0.09,jitter); }
 case 4u: { value=fma(0.6,jitter,0.2); }
 case 5u: { value=jitter/0.6; }
 case 6u: { value=dot(vec2<f32>(jitter,t),vec2<f32>(jitter,t)); }
 default: { value=0.76+jitter*(0.52-0.76); }
 }
 outputs[i]=value;
}
