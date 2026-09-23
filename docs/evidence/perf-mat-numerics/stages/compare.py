"""Compare fixed stage diagnostics; output-only evidence, never render pixels."""
import argparse,json,hashlib
from pathlib import Path
from functools import reduce
from PIL import Image,ImageChops
p=argparse.ArgumentParser();p.add_argument('native');p.add_argument('control');p.add_argument('output');a=p.parse_args()
n=Path(a.native);c=Path(a.control);out=Path(a.output)
if out.exists(): raise RuntimeError('Refuse existing comparison')
rows=[]
for stage in ['macro','exposure','erodeX','erodeY','band','rustSpread','detailNoise','detail','rustMask','coatingHeight','height']:
 ng=json.loads((n/(stage+'.mix')).read_bytes());cg=json.loads((c/(stage+'.mix')).read_bytes());assert ng==cg,stage
 nr=json.loads((n/(stage+'.json')).read_bytes());cr=json.loads((c/(stage+'.json')).read_bytes());assert nr['planHash']==cr['planHash'],stage
 row={'stage':stage,'planHash':nr['planHash'],'channels':{}}
 for channel in ['height','normal']:
  np=n/stage/(channel+'.png');cp=c/stage/(channel+'.png')
  ni=Image.open(np).convert('RGBA');ci=Image.open(cp).convert('RGBA');assert ni.size==ci.size==(1024,1024)
  d=reduce(ImageChops.lighter,ImageChops.difference(ni,ci).split());h=d.histogram()
  row['channels'][channel]={'max':max(i for i,v in enumerate(h) if v),'changedPixels':sum(h[1:]),'nativePngSha256':hashlib.sha256(np.read_bytes()).hexdigest(),'controlPngSha256':hashlib.sha256(cp.read_bytes()).hexdigest()}
 rows.append(row)
 print(stage, row['channels']['height']['max'],row['channels']['height']['changedPixels'],row['channels']['normal']['max'],row['channels']['normal']['changedPixels'],flush=True)
out.write_text(json.dumps({'kind':'stage-decoded-comparison','qualified':False,'nativeReceipt':json.loads((n/'receipt.json').read_bytes()),'controlReceipt':json.loads((c/'receipt.json').read_bytes()),'rows':rows},indent=2)+'\n',encoding='utf-8')
