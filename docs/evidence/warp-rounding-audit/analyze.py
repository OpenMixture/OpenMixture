"""Bind rational references to actual half textures, retaining all warp changes."""
from pathlib import Path
import json, struct, hashlib, subprocess, sys, os
from oracle import F, f32, bits, hb, hv, floor, frac, rn, delta, result, self_test

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
OUT=ROOT/'tmp/warp-rounding-audit'
self_test()
rows=[]
summary=[]

def sample(data,x,y,w=1024,h=1024):
    return hv(struct.unpack_from('<H',data,((y%h)*w+(x%w))*8)[0])

for variant in ['default','coarse-grain','horizontal-grain','straight-grain']:
    old=OUT/f'{variant}-local'; new=OUT/f'{variant}-fma'
    for name in ['grain','stretch','distortion']:
        assert (old/f'{name}.bin').read_bytes()==(new/f'{name}.bin').read_bytes()
    a=(old/'warp.bin').read_bytes(); b=(new/'warp.bin').read_bytes()
    changed=[i for i in range(1024*1024) if a[i*8:i*8+8]!=b[i*8:i*8+8]]
    src=(old/'stretch.bin').read_bytes(); field=(old/'distortion.bin').read_bytes()
    strength=f32(json.loads((OUT/f'{variant}-local.json').read_text())[3]['params'][0])
    summary.append(dict(variant=variant,warpChangedTexels=len(changed)))
    for i in changed:
        x,y=i%1024,i//1024; f=sample(field,x,y)
        da=delta(f,strength,1024,False); db=delta(f,strength,1024,True)
        def neighbors(d):
            bx=x+floor(d)
            return [sample(src,bx,y),sample(src,bx+1,y),sample(src,bx,y+1),sample(src,bx+1,y+1)]
        sa,sb=neighbors(da),neighbors(db); tx=rn(frac(db))
        refs={'A':result(sa,frac(da),F(0),'exact'),
              'A_frozen_weight':result(sb,tx,F(0),'exact'),
              'B':result(sb,tx,F(0),'fused'),
              'B_unfused':result(sb,tx,F(0),'unfused'),
              'B_weighted':result(sb,tx,F(0),'weighted')}
        ob,nb=struct.unpack_from('<H',a,i*8)[0],struct.unpack_from('<H',b,i*8)[0]
        rows.append(dict(variant=variant,xy=[x,y],fieldHalfBits=hb(f),strengthBits=bits(strength),
            deltaExact=str(da),deltaStaged=str(db),weightBits=bits(tx),
            baseExact=[(x+floor(da))%1024,y],baseStaged=[(x+floor(db))%1024,y],
            samplesExactHalfBits=list(map(hb,sa)),samplesStagedHalfBits=list(map(hb,sb)),
            beforeHalfBits=ob,afterHalfBits=nb,references=refs,
            beforeMatches=[k for k,v in refs.items() if ob==v['halfBits']],
            afterMatches=[k for k,v in refs.items() if nb==v['halfBits']]))

# Single-result scalar probes. Production texture observations remain authoritative
# for what actually ran; scalar mismatches are reported rather than hidden.
inputs=[]
for row in rows:
    samples=[float(hv(v)) for v in row['samplesStagedHalfBits']]
    inputs.extend([float(hv(row['fieldHalfBits'])),float(f32(row['strengthBits'])),1024,*samples,
                   float(f32(row['weightBits'])),0]+[0]*23)
(OUT/'scalar-inputs.bin').write_bytes(struct.pack('<'+'f'*len(inputs),*inputs))
precision=(ROOT/'crates/mixture-wgpu/shaders/precision.wgsl').read_text()
common='''
@group(0) @binding(0) var<storage,read> inputs:array<f32>;
@group(0) @binding(1) var<storage,read_write> outputs:array<f32>;
@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>){
let o=id.x*32u;if o>=arrayLength(&inputs){return;}
let delta=(2.0*inputs[o]-1.0)*inputs[o+1u]*inputs[o+2u];
let t=fract(delta); let a=inputs[o+3u];let b=inputs[o+4u];
let c=inputs[o+5u];let d=inputs[o+6u];let ty=inputs[o+8u];
'''
mix='mix(mix(a,b,t),mix(c,d,t),ty)'
fma='fma(fma(d-c,t,c)-fma(b-a,t,a),ty,fma(b-a,t,a))'
runner=ROOT/'target/release/examples/rounding_buffer_probe'
if os.name=='nt': runner=runner.with_suffix('.exe')
for name,expr in [('delta','delta'),('weight','t'),('mix',mix),('fma',fma),
                  ('mix-half',f'mixture_half({mix})'),('fma-half',f'mixture_half({fma})')]:
    shader=OUT/f'scalar-{name}.wgsl';shader.write_text(precision+common+f'outputs[o]={expr};}}\n')
    if rows:
        with (OUT/f'scalar-{name}-context.json').open('w') as log:
            subprocess.run([str(runner),str(shader),str(OUT/f'scalar-{name}.bin'),str(len(inputs)),str(OUT/'scalar-inputs.bin')],stderr=log,check=True)
        data=(OUT/f'scalar-{name}.bin').read_bytes()
        for i,row in enumerate(rows):
            value=struct.unpack_from('<f',data,i*128)[0]
            row.setdefault('probes',{})[name]={'f32Bits':bits(value),'value':value}

for row in rows:
    row['probeMatchesTexture']={mode:hb(row['probes'][mode+'-half']['value'])==row[key]
        for mode,key in [('mix','beforeHalfBits'),('fma','afterHalfBits')]}
    row['coordinateProbeMatchesB']=row['probes']['weight']['f32Bits']==row['weightBits']
for entry in summary:
    subset=[r for r in rows if r['variant']==entry['variant']]
    entry['matches']={which:{key:sum(key in r[which+'Matches'] for r in subset)
        for key in ['A','A_frozen_weight','B','B_unfused','B_weighted']} for which in ['before','after']}
    entry['probeTextureMismatch']=sum(not all(r['probeMatchesTexture'].values()) for r in subset)
(OUT/'adjudication.json').write_text(json.dumps({'models':__import__('oracle').__doc__,'summary':summary,'rows':rows},indent=2)+'\n')
print(json.dumps(summary,indent=2))
