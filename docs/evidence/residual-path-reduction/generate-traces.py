from pathlib import Path
import json,struct,numpy as np
r=Path('tmp/path-reduction'); precision=Path('crates/mixture-wgpu/shaders/precision.wgsl').read_text(); source=Path('tmp/local-coordinate-engine/crates/mixture-wgpu/shaders/nodes/fractal-noise.wgsl').read_text(); funcs=source[source.index('fn avalanche'):source.index('@compute')]
header=precision+'\n@group(0) @binding(0) var<storage,read> inputs:array<f32>;\n@group(0) @binding(1) var<storage,read_write> outputs:array<f32>;\n'
w=json.loads((r/'witnesses.json').read_text()); rows=[]
for m in ['leather','wood']:
 for v in w[m]: rows.append([*v['xy'],271828 if m=='leather' else 314159,64 if m=='leather' else 3,3,.35 if m=='leather' else .45,1 if m=='leather' else 0])
# Trace exact expression trees with fixed dynamic inputs, retaining caveat of instrumentation.
funcs=funcs.replace('fn cellular_noise(uv: vec2<f32>, period: u32, seed: u32)', 'fn cellular_noise(uv: vec2<f32>, period: u32, seed: u32, start:u32)')
funcs=funcs.replace('let distance = dot(delta, delta);','let distance = dot(delta, delta);\n let o=start+u32((y+1)*3+x+1)*7u; outputs[o]=jitter.x;outputs[o+1u]=jitter.y;outputs[o+2u]=delta.x;outputs[o+3u]=delta.y;outputs[o+4u]=distance;')
funcs=funcs.replace('second = min(second, distance);\n            }','second = min(second, distance);\n            }\n outputs[o+5u]=nearest;outputs[o+6u]=second;')
funcs=funcs.replace('fn value_noise(uv: vec2<f32>, period: u32, seed: u32)', 'fn value_noise(uv: vec2<f32>, period: u32, seed: u32, start:u32)')
funcs=funcs.replace('return mix(mix(a, b, fade.x), mix(c, d, fade.x), fade.y);','outputs[start]=t.x;outputs[start+1u]=t.y;outputs[start+2u]=fade.x;outputs[start+3u]=fade.y;outputs[start+4u]=a;outputs[start+5u]=b;outputs[start+6u]=c;outputs[start+7u]=d;outputs[start+8u]=mix(a,b,fade.x);outputs[start+9u]=mix(c,d,fade.x);return mix(mix(a,b,fade.x),mix(c,d,fade.x),fade.y);')
body='''
@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>){let n=id.x;let base=n*256u;if base>=arrayLength(&inputs){return;}let uv=(vec2<f32>(inputs[base],inputs[base+1u])+vec2<f32>(0.5))/1024.0;var sum=0.0;var weights=0.0;var weight=1.0;var period=u32(inputs[base+3u]);for(var oct=0u;oct<u32(inputs[base+4u]);oct++){let seed=avalanche(u32(inputs[base+2u])+oct*0x9e3779b9u);let o=base+oct*80u;var v:f32;if inputs[base+6u]==1.0{v=cellular_noise(uv,period,seed,o);}else{v=value_noise(uv,period,seed,o);}sum+=v*weight;weights+=weight;outputs[o+64u]=v;outputs[o+65u]=weight;outputs[o+66u]=sum;outputs[o+67u]=weights;weight*=inputs[base+5u];period*=2u;}outputs[base+250u]=sum/weights;outputs[base+251u]=mixture_half(sum/weights);}
'''
(r/'noise-trace.wgsl').write_text(header+funcs+body); arr=np.zeros((len(rows),256),dtype='<f4');arr[:,:7]=rows;arr.tofile(r/'noise-input.bin');(r/'noise-plan.json').write_text(json.dumps(rows))
# Warp witness inputs use native texture values. Base/t checks will validate chosen neighborhood.
a=np.fromfile(r/'native/wood-stretch.bin',dtype='<f2').reshape(1024,1024,4)[:,:,0];field=np.fromfile(r/'native/wood-distortion.bin',dtype='<f2').reshape(1024,1024,4)[:,:,0]
coords=[v['xy'] for v in w['wood']];arr=np.zeros((len(coords),32),dtype='<f4')
for i,(x,y) in enumerate(coords):
 f=np.float32(field[y,x]);uv=np.float32((x+.5)/1024);p=np.float32(np.float32(uv+np.float32(np.float32(2*f-1)*np.float32(.018)))*1024-.5);base=int(np.floor(p));arr[i,:8]=[x,y,f,.018,a[y,base%1024],a[y,(base+1)%1024],base,p-base]
arr.tofile(r/'warp-input.bin');(r/'warp-plan.json').write_text(json.dumps(coords))
(r/'warp-trace.wgsl').write_text(header+'''@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>){let o=id.x*32u;if o>=arrayLength(&inputs){return;}let uv=(vec2<f32>(inputs[o],inputs[o+1u])+vec2<f32>(.5))/1024.0;let field=inputs[o+2u];let strength=vec2<f32>(inputs[o+3u],0.0);let sample_uv=uv+(2.0*field-1.0)*strength;let p=fract(sample_uv)*1024.0-vec2<f32>(.5);let t=fract(p);outputs[o]=uv.x;outputs[o+1u]=2.0*field-1.0;outputs[o+2u]=(2.0*field-1.0)*strength.x;outputs[o+3u]=sample_uv.x;outputs[o+4u]=p.x;outputs[o+5u]=floor(p.x);outputs[o+6u]=t.x;outputs[o+7u]=t.y;outputs[o+8u]=mix(inputs[o+4u],inputs[o+5u],t.x);outputs[o+9u]=mixture_half(mix(inputs[o+4u],inputs[o+5u],t.x));}''')
print('noise count',len(rows)*256,'warp count',len(coords)*32)
