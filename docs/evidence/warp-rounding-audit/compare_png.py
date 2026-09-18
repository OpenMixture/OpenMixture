"""Validate diagnostic textures against recorded product PNGs, not new goldens.

Only the existing readback encoding is mirrored here; material pixel computation
is performed by production WGSL. All final pixels must match, not just witnesses.
Requires NumPy and Pillow. Arguments: capture directory, PR15 PNG root, PR16 root.
"""
from pathlib import Path
import json,sys,hashlib
import numpy as np
from PIL import Image

out,old,new=map(Path,sys.argv[1:4]); rows=[]
for variant in ['default','coarse-grain','horizontal-grain','straight-grain']:
    for mode,reference in [('local',old),('fma',new)]:
        for channel in ['baseColor','height','normal','roughness']:
            path=out/f'{variant}-{mode}/{channel}.bin'
            v=np.frombuffer(path.read_bytes(),dtype='<f2').astype(np.float32).reshape(1024,1024,4)
            assert np.isfinite(v).all() and ((v>=0)&(v<=1)).all()
            if channel=='baseColor':
                rgb=v[:,:,:3]
                v[:,:,:3]=np.where(rgb<=np.float32(.0031308),rgb*np.float32(12.92),
                    np.float32(1.055)*np.power(rgb,np.float32(1)/np.float32(2.4))-np.float32(.055))
            elif channel in ['height','roughness']:
                v[:,:,1]=v[:,:,0]; v[:,:,2]=v[:,:,0]; v[:,:,3]=1
            actual=np.floor(v*np.float32(255)+np.float32(.5)).astype(np.uint8)
            png=reference/variant/f'{channel}.png'
            expected=np.array(Image.open(png).convert('RGBA'))
            changed=np.any(actual!=expected,axis=2)
            rows.append(dict(variant=variant,mode=mode,channel=channel,changedPixels=int(changed.sum()),
                textureSha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                recordedPngSha256=hashlib.sha256(png.read_bytes()).hexdigest(),
                actualRgbaSha256=hashlib.sha256(actual.tobytes()).hexdigest(),
                recordedRgbaSha256=hashlib.sha256(expected.tobytes()).hexdigest()))
(out/'product-png-binding.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps({'channels':len(rows),'failures':[r for r in rows if r['changedPixels']]},indent=2))
assert all(r['changedPixels']==0 for r in rows), 'Diagnostic output did not reproduce recorded product pixels'
