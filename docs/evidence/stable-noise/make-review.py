import json, hashlib
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw

root = Path('tmp/stable-noise')
out = Path('docs/evidence/stable-noise')
out.mkdir(parents=True, exist_ok=True)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
old = root / 'native-materials-v1'
new = root / 'native-materials-v2'
manifest = json.loads((new/'manifest.json').read_text())
report = {'sourceRevision': manifest['engineRevision'], 'engineDirty': manifest['engineDirty'],
          'nativeManifestSha256': {'v1':sha(old/'manifest.json'), 'v2':sha(new/'manifest.json')},
          'scope': 'Native Vulkan semantic migration comparison; agent visual review, not human acceptance', 'cases':[]}
for case in manifest['cases']:
    folder = Path(case['material']) / case['id']
    sheet = Image.new('RGB', (960, 1440), '#19212b')
    draw = ImageDraw.Draw(sheet)
    row = {'material':case['material'], 'case':case['id'], 'sourceSha256':case['sourceSha256'],
           'planHash':case['plan']['hash'], 'channels':[]}
    for i, channel in enumerate(['baseColor','normal','roughness','height']):
        a, b = old/folder/f'{channel}.png', new/folder/f'{channel}.png'
        im1, im2 = Image.open(a).convert('RGB'), Image.open(b).convert('RGB')
        delta = np.abs(np.asarray(im1).astype(np.int16)-np.asarray(im2).astype(np.int16))
        metrics = {'channel':channel, 'v1PngSha256':sha(a), 'v2PngSha256':sha(b),
                   'maxRgbDelta':int(delta.max()), 'meanRgbDelta':float(delta.mean()),
                   'changedPixelRatio':float(np.any(delta!=0,axis=2).mean())}
        row['channels'].append(metrics)
        diff = Image.fromarray(np.minimum(delta*16,255).astype(np.uint8))
        for j, (im,label) in enumerate([(im1,'v1'),(im2,'v2'),(diff,'absolute difference x16')]):
            draw.text((j*320+8,i*360+5),f'{case["material"]}/{case["id"]} {channel} {label}',fill='white')
            sheet.paste(im.resize((320,320),Image.Resampling.BOX),(j*320,i*360+26))
        draw.text((8,i*360+346),f'max={metrics["maxRgbDelta"]} mean={metrics["meanRgbDelta"]:.6f} changed pixels={metrics["changedPixelRatio"]:.6%}',fill='white')
    target = out/f'{case["material"]}-{case["id"]}.png'
    sheet.save(target)
    row['reviewImage'] = target.name
    row['reviewImageSha256'] = sha(target)
    report['cases'].append(row)
(out/'migration-comparison.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps([{'case':r['material']+'/'+r['case'],'max':max(c['maxRgbDelta'] for c in r['channels'])} for r in report['cases']]))
