from pathlib import Path
from fractions import Fraction as F
import numpy as np,json
r=Path('tmp/path-reduction');inputs=np.fromfile(r/'warp-input.bin',dtype='<f4').reshape(-1,32);exact=[]
for row in inputs:
 x,y,field,strength,a,b,base,t=map(float,row[:8]);p=F(x)+(2*F(field)-1)*F(strength)*1024;assert p.numerator//p.denominator==int(base);t=p-int(base);exact.append(F(a)*(1-t)+F(b)*t)
report={}
for label,stem in [('original','warp-only-mix'),('localTexel','warp-local')]:
 vals={s:np.fromfile(r/f'{stem}-{s}.bin',dtype='<f4').reshape(-1,32)[:,0] for s in ['native','chrome']};report[label]={'rawDifferent':int((vals['native']!=vals['chrome']).sum()),'halfDifferent':int((vals['native'].astype('f2')!=vals['chrome'].astype('f2')).sum()),'maxAbsoluteError':{s:max(float(abs(F(float(v))-e)) for v,e in zip(vs,exact)) for s,vs in vals.items()}}
(r/'proposal-samples.json').write_text(json.dumps(report,indent=2));print(report)
