import numpy as np,json
from pathlib import Path
r=Path('tmp/path-reduction')
a=np.fromfile(r/'noise-native.bin',dtype='<f4').reshape(-1,256);b=np.fromfile(r/'noise-chrome.bin',dtype='<f4').reshape(-1,256);rows=json.loads((r/'noise-plan.json').read_text())
for n in [0,1,16,17]:
 dif=np.flatnonzero(a[n]!=b[n]);print('noise',n,rows[n],[(int(i),float(a[n,i]),float(b[n,i])) for i in dif[:8]])
for side,trace in [('native',a),('chrome',b)]:
 mism=[]
 for i,row in enumerate(rows):
  name='leather-grain' if i<16 else 'wood-distortion';v=np.fromfile(r/side/(name+'.bin'),dtype='<f2').reshape(1024,1024,4)[int(row[1]),int(row[0]),0]
  if v!=trace[i,251]:mism.append(i)
 print('trace does not match actual half',side,mism)
a=np.fromfile(r/'warp-native.bin',dtype='<f4').reshape(-1,32);b=np.fromfile(r/'warp-chrome.bin',dtype='<f4').reshape(-1,32)
print('warp first',a[0,:10],b[0,:10]); print('warp changed rows',[int(i) for i in np.flatnonzero(np.any(a!=b,axis=1))]);print('warp first difference columns',[(i,int(np.flatnonzero(a[i]!=b[i])[0])) for i in range(len(a)) if np.any(a[i]!=b[i])][:10])
