import json,struct
from pathlib import Path
r=Path.cwd(); out=r/'tmp/path-reduction'; src=r/'tmp/local-coordinate-engine/crates/mixture-wgpu/shaders'
f=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
def node(id,k,params,inputs=[]):return dict(id=id,entry=k.replace('-','_'),code=(src/'precision.wgsl').read_text()+'\n'+(src/'nodes'/f'{k}.wgsl').read_text(),params=params,inputs=inputs)
nodes=[node('leather-grain','fractal-noise',[271828,64,3,1,f(.35),0,0,0]),node('leather-height','levels',list(map(f,[0,.6,1.6,0,1,0,0,0])),[0]),node('wood-grain','fractal-noise',[161803,4,4,0,f(.45),0,0,0]),node('wood-stretch','transform-2d',[32,1,0,0,0,0,0,0],[2]),node('wood-distortion','fractal-noise',[314159,3,3,0,f(.45),0,0,0]),node('wood-warp','warp',[f(.018),0,0,0],[3,4]),node('wood-height','levels',list(map(f,[.12,.88,.7,0,1,0,0,0])),[5])]
(out/'pipeline.json').write_text(json.dumps(nodes,indent=2))
