"""Read production literal textures and generate single-result rounding probes."""
from pathlib import Path
import json, struct, subprocess, sys, os
from oracle import F, f32, bits, hb, half, floor, frac, rn, delta, result
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
OUT=Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'tmp/warp-rounding-audit'
rows=[]; inputs=[]
for case in json.loads((OUT/'cases.json').read_text()):
    w,h=case['size']; a,b=F(case['a']),F(case['b']); strength=F(case['strength'])
    for x in ([0,256] if w==1024 else [0]):
        d=delta(F(1),strength,w,True); t=rn(frac(d))
        samples=[a,b,a,b]
        refs={'A':result(samples,frac(delta(F(1),strength,w,False)),F(0),'exact'),
              'B':result(samples,t,F(0),'fused'),
              'B_unfused':result(samples,t,F(0),'unfused'),
              'B_weighted':result(samples,t,F(0),'weighted')}
        outputs={mode:struct.unpack_from('<H',(OUT/f"{case['id']}-{mode}/warp.bin").read_bytes(),x*8)[0]
                 for mode in ['original','local','fma']}
        rows.append(dict(id=case['id'],xy=[x,0],references=refs,textureHalfBits=outputs))
        inputs.extend([1,float(strength),w,float(a),float(b),float(t),x]+[0]*25)
(OUT/'literal-inputs.bin').write_bytes(struct.pack('<'+'f'*len(inputs),*inputs))
precision=(ROOT/'crates/mixture-wgpu/shaders/precision.wgsl').read_text()
common='''
@group(0) @binding(0) var<storage,read> inputs:array<f32>;
@group(0) @binding(1) var<storage,read_write> outputs:array<f32>;
@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>){
let o=id.x*32u;if o>=arrayLength(&inputs){return;}
let delta=(2.0*inputs[o]-1.0)*inputs[o+1u]*inputs[o+2u];
let t=fract(delta);let a=inputs[o+3u];let b=inputs[o+4u];
let uv=(inputs[o+6u]+0.5)/inputs[o+2u];
let oldt=fract(fract(uv+(2.0*inputs[o]-1.0)*inputs[o+1u])*inputs[o+2u]-0.5);
'''
runner=ROOT/'target/release/examples/rounding_buffer_probe'
if os.name=='nt': runner=runner.with_suffix('.exe')
expressions={'delta':'delta','weight':'t','old-weight':'oldt','mix':'mix(a,b,t)',
             'fma':'fma(b-a,t,a)','mix-half':'mixture_half(mix(a,b,t))',
             'fma-half':'mixture_half(fma(b-a,t,a))'}
for name,expr in expressions.items():
    shader=OUT/f'literal-{name}.wgsl';shader.write_text(precision+common+f'outputs[o]={expr};}}\n')
    if '--prepare-only' not in sys.argv:
        with (OUT/f'literal-{name}-context.json').open('w') as log:
            subprocess.run([str(runner),str(shader),str(OUT/f'literal-{name}.bin'),str(len(inputs)),str(OUT/'literal-inputs.bin')],stderr=log,check=True)
        data=(OUT/f'literal-{name}.bin').read_bytes()
        for i,row in enumerate(rows):
            value=struct.unpack_from('<f',data,i*128)[0]
            row.setdefault('probes',{})[name]={'f32Bits':bits(value),'value':value}
(OUT/'literal-adjudication.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2))
