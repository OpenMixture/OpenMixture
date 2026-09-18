"""Diagnostic dispatch plans; no product renderer or changed acceptance inputs."""
import json, struct, hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
OUT = ROOT / 'tmp/warp-rounding-audit'
OUT.mkdir(parents=True, exist_ok=True)
SRC = ROOT / 'crates/mixture-wgpu/shaders'
bits = lambda x: struct.unpack('<I', struct.pack('<f', x))[0]
fp = lambda b: struct.unpack('<f', struct.pack('<I', b))[0]
precision = (SRC / 'precision.wgsl').read_text()

def node(id, kernel, params, inputs=(), size=(1024, 1024), code=None):
    return dict(id=id, entry=kernel.replace('-', '_'), params=params, inputs=list(inputs),
                size=list(size), code=precision+'\n'+(code or (SRC/'nodes'/f'{kernel}.wgsl').read_text()))

def save(name, nodes):
    (OUT/f'{name}.json').write_text(json.dumps(nodes, indent=2)+'\n')

for variant, repeat, rotation, strength in [('default',32,0,.018),('coarse-grain',16,0,.018),
                                           ('horizontal-grain',32,1,.018),('straight-grain',32,0,0)]:
    for mode in ['local','fma']:
        nodes = [node('grain','fractal-noise',[161803,4,4,0,bits(.45),0,0,0]),
                 node('stretch','transform-2d',[repeat,1,rotation,0,0,0,0,0],[0]),
                 node('distortion','fractal-noise',[314159,3,3,0,bits(.45),0,0,0]),
                 node('warp','warp',[bits(strength),0,0,0],[1,2],code=(HERE/f'warp-{mode}.wgsl').read_text()),
                 node('height','levels',list(map(bits,[.12,.88,.7,0,1,0,0,0])),[3]),
                 node('baseColor','gradient-map',list(map(bits,[.05,.017,.006,1,.34,.17,.063,1])),[4]),
                 node('normal','height-to-normal',[bits(.0015),0,0,0],[4]),
                 node('roughness','levels',list(map(bits,[0,1,1,.65,.45,0,0,0])),[4])]
        save(f'{variant}-{mode}',nodes)

# Fixed before observing GPU output. Both contracts are diagnostic references,
# not new production assertions. All sample values are exact binary16 values.
cases = [dict(id='half-boundary',size=[2,1],a=873/2048,b=349/1024,strength=fp(0x3ef851e8)),
         dict(id='double-rounding',size=[2,1],a=.5,b=1,strength=fp(0x39800001)),
         dict(id='tiny-subnormal',size=[1024,1],a=0,b=1,strength=2**-26),
         dict(id='tiny-normal',size=[1024,1],a=2**-14,b=1,strength=2**-26)]
for case in cases:
    for mode in ['original','local','fma']:
        def fill(a,b):
            return ('@group(0) @binding(0) var<uniform> params:vec4<u32>;\n'
                    '@group(0) @binding(1) var output:texture_storage_2d<rgba16float,write>;\n'
                    '@compute @workgroup_size(8,8) fn fill(@builtin(global_invocation_id) id:vec3<u32>){\n'
                    'if any(id.xy>=textureDimensions(output)) || params.x==0xffffffffu {return;}\n'
                    f'let value=select(bitcast<f32>({bits(a)}u),bitcast<f32>({bits(b)}u),id.x%2u==1u);\n'
                    'textureStore(output,vec2<i32>(id.xy),vec4<f32>(value,0,0,1));}')
        size=case['size']
        save(f"{case['id']}-{mode}",[
            node('input','fill',[0]*4,size=size,code=fill(case['a'],case['b'])),
            node('field','fill',[0]*4,size=size,code=fill(1,1)),
            node('warp','warp',[bits(case['strength']),0,0,0],[0,1],size,
                 (HERE/f'warp-{mode}.wgsl').read_text())])
(OUT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
(OUT/'plans-sha256.json').write_text(json.dumps({p.name:hashlib.sha256(p.read_bytes()).hexdigest()
    for p in sorted(OUT.glob('*.json')) if p.name!='plans-sha256.json'},indent=2)+'\n')
