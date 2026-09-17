import numpy as np
from pathlib import Path
r=Path('tmp/path-reduction');rows={'cellular':[0,.22351199388504028,.84375,0],'mix':[.09506094455718994,.5717524290084839,.8831931352615356,0],'warp':[.81201171875,.2462158203125,.018,0]};expr={'cellular':'inputs[i]+0.2+0.6*inputs[i+1u]-inputs[i+2u]','mix':'mix(inputs[i],inputs[i+1u],inputs[i+2u])','warp':'inputs[i]+(2.0*inputs[i+1u]-1.0)*inputs[i+2u]'}
for name,row in rows.items():
 np.array(row,dtype='<f4').tofile(r/f'min-{name}-input.bin');(r/f'min-{name}.wgsl').write_text('@group(0) @binding(0) var<storage,read> inputs:array<f32>;\n@group(0) @binding(1) var<storage,read_write> outputs:array<f32>;\n@compute @workgroup_size(64) fn probe(@builtin(global_invocation_id) id:vec3<u32>){let i=id.x*4u;if i>=arrayLength(&inputs){return;}outputs[i]='+expr[name]+';}')
