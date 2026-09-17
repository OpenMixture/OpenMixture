from pathlib import Path
from PIL import Image
import json
r=Path.cwd(); result={}
for material in ['leather','wood']:
 a=Image.open(r/f'tmp/local-coordinate-native/{material}/default/height.png').convert('RGBA'); b=Image.open(r/f'tmp/local-coordinate-chrome/{material}/default/height.png').convert('RGBA')
 points=[]
 for i,(x,y) in enumerate(zip(a.getdata(),b.getdata())):
  if x!=y: points.append({'xy':[i%1024,i//1024],'native':list(x),'chrome':list(y)})
 result[material]=points
(r/'tmp/path-reduction/witnesses.json').write_text(json.dumps(result,indent=2))
print({k:v[:3] for k,v in result.items()})
