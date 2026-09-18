from pathlib import Path
import numpy as np,json
from PIL import Image
r=Path('tmp/path-reduction'); witnesses=json.loads((r/'witnesses.json').read_text()); summary={}
for p in (r/'native').glob('*.bin'):
 a=np.fromfile(p,dtype='<f2').reshape(1024,1024,4)[:,:,0]; b=np.fromfile(r/'chrome'/p.name,dtype='<f2').reshape(1024,1024,4)[:,:,0]; diff=a!=b
 summary[p.stem]={'different':int(diff.sum()),'witnesses':[{'xy':v['xy'],'native':float(a[v['xy'][1],v['xy'][0]]),'chrome':float(b[v['xy'][1],v['xy'][0]])} for v in witnesses[p.stem.split('-')[0]]]}
 if p.stem.endswith('height'):
  mat=p.stem.split('-')[0]
  for side,vals in [('native',a),('chrome',b)]:
   orig=np.array(Image.open(r.parent/f'local-coordinate-{side}/{mat}/default/height.png'))[:,:,0]
   rgba=np.floor(vals.astype('f4')*255+0.5).astype('uint8'); print('pipeline matches PNG',mat,side,int((orig!=rgba).sum()))
 print(p.stem,summary[p.stem]['different'],summary[p.stem]['witnesses'][:2])
(r/'stages.json').write_text(json.dumps(summary,indent=2))
