"""Compare actual GPU PNG exports; no material execution on the CPU."""
import json, hashlib
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw
root=Path('tmp/shader-stability')
cases=json.loads((root/'original-full.json').read_text())['cases']
tol=json.loads(Path('docs/browser-tolerances.json').read_text())
rows=[]
for case in cases:
    for channel in ['baseColor','normal','roughness','height']:
        key=Path(case['material'])/case['id']/(channel+'.png')
        paths=[root/(mode+'-full')/key for mode in ['original','local']]
        a,b=[Image.open(p).convert('RGBA') for p in paths]
        assert a.size==b.size==(1024,1024)
        diff=ImageChops.difference(a,b)
        hist=diff.histogram();total=sum((i%256)*n for i,n in enumerate(hist))
        maximum=max(i%256 for i,n in enumerate(hist) if n)
        changed=sum(any(pixel) for pixel in diff.getdata())
        mean=total/(1024*1024*4);ratio=changed/(1024*1024);gate=tol[channel]
        rows.append({**case,'channel':channel,'maxAbsolute':maximum,'meanAbsolute':mean,'changedPixels':changed,'changedPixelRatio':ratio,'withinFrozenBrowserEnvelope':maximum<=gate['maxAbsolute'] and mean<=gate['meanAbsolute'] and ratio<=gate['maxChangedPixelRatio'],'originalPngSha256':hashlib.sha256(paths[0].read_bytes()).hexdigest(),'localPngSha256':hashlib.sha256(paths[1].read_bytes()).hexdigest()})
summary={'diagnosticOnly':True,'comparison':'Original native versus isolated local-coordinate native; this is not browser certification.','channels':rows}
(root/'material-comparison.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'channels':len(rows),'exact':sum(r['maxAbsolute']==0 for r in rows),'withinFrozenBrowserEnvelope':sum(r['withinFrozenBrowserEnvelope'] for r in rows),'byMaterial':{m:{'max':max(r['maxAbsolute'] for r in rows if r['material']==m),'changedChannels':sum(r['changedPixels']>0 for r in rows if r['material']==m)} for m in ['glazed-ceramic','leather','wood']}},indent=2))
sheet=Image.new('RGB',(3*384,4*420+36),'#e8e8e8');draw=ImageDraw.Draw(sheet)
for i,label in enumerate(['Original native','Local coordinates only','Absolute difference x64']):draw.text((i*384+12,10),label,fill='black')
for row,channel in enumerate(['baseColor','height','normal','roughness']):
    paths=[root/(mode+'-full')/'leather'/'default'/(channel+'.png') for mode in ['original','local']]
    a,b=[Image.open(p).convert('RGB') for p in paths];d=ImageChops.difference(a,b).point(lambda x:min(255,x*64))
    for col,img in enumerate([a,b,d]):
        sheet.paste(img.resize((384,384)),(col*384,36+row*420));draw.text((col*384+10,36+row*420+389),channel,fill='black')
sheet.save(root/'contact.png')
