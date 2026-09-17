import {readFileSync,writeFileSync} from 'node:fs';
const root='tmp/shader-stability';
const cases=[
 {name:'leather',width:1024,height:1024,seed:271828,scale:64,octaves:3,persistence:0.35},
 {name:'max-period-odd-size',width:129,height:65,seed:4294967295,scale:128,octaves:6,persistence:1},
 {name:'min-period',width:129,height:65,seed:0,scale:1,octaves:1,persistence:0},
 {name:'zero-persistence',width:129,height:65,seed:271828,scale:128,octaves:6,persistence:0},
 {name:'max-period-2k',width:2048,height:2048,seed:1,scale:128,octaves:6,persistence:1},
 {name:'low-persistence',width:1024,height:1024,seed:4294967295,scale:128,octaves:6,persistence:0.01}
];
writeFileSync(`${root}/sample-plan.json`,JSON.stringify({diagnosticOnly:true,samplesPerCase:256,cases},null,2));
const rows=cases.map(c=>`vec4<u32>(${c.width}u,${c.height}u,${c.seed}u,${c.scale}u)`).join(',');
const octaves=cases.map(c=>c.octaves+'u').join(',');const persist=cases.map(c=>c.persistence.toFixed(8)).join(',');
for(const variant of ['original','local','local_rational']){
 let s=readFileSync(`${root}/${variant}.wgsl`,'utf8');s=s.slice(0,s.indexOf('@compute'))+`
 @compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>) {
 let i=id.x;if i>=3072u{return;}let sample_index=i/2u;let case_id=sample_index/256u;let slot=sample_index%256u;
 let configs=array<vec4<u32>,6>(${rows});let octaves=array<u32,6>(${octaves});let persist=array<f32,6>(${persist});let config=configs[case_id];
 var x=avalanche(slot+17u)%config.x;var y=avalanche(slot+991u)%config.y;
 if slot<4u{x=(slot%2u)*(config.x-1u);y=(slot/2u)*(config.y-1u);}
 let uv=(vec2<f32>(f32(x),f32(y))+vec2<f32>(0.5))/vec2<f32>(config.xy);
 var sum=inputs[i];var weights=0.0;var weight=1.0;var period=config.w;
 for(var octave=0u;octave<octaves[case_id];octave++){
 let seed=avalanche(config.z+octave*0x9e3779b9u);sum+=cellular_noise(uv,period,seed)*weight;weights+=weight;weight*=persist[case_id];period*=2u;
 }
 let value=clamp(sum/weights,0.0,1.0);outputs[i]=select(value,mixture_half(value),i%2u==1u);
 }
 `;writeFileSync(`${root}/${variant}-samples.wgsl`,s);
}
