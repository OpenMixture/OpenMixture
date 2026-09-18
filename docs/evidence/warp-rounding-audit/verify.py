"""Offline integrity and exact-reference verification of retained evidence."""
from pathlib import Path
import json,hashlib,struct,zipfile,io,tarfile
from oracle import F, f32, bits, hb, hv, rn, frac, floor, delta, result, self_test
here=Path(__file__).resolve().parent
self_test()
manifest=json.loads((here/'retained-sha256.json').read_text())
for name,digest in manifest.items():
    assert hashlib.sha256((here/name).read_bytes()).hexdigest()==digest,name
native=json.loads((here/'native/literal-adjudication.json').read_text())
for row in native:
    i=native.index(row)
    for mode,record in row['probes'].items():
        data=(here/'native'/f'literal-{mode}.bin').read_bytes()
        assert struct.unpack_from('<I',data,i*128)[0]==record['f32Bits']
    with zipfile.ZipFile(here/'native/literal-textures.zip') as z:
        for mode,h in row['textureHalfBits'].items():
            assert struct.unpack_from('<H',z.read(f"{row['id']}-{mode}/warp.bin"),row['xy'][0]*8)[0]==h
double=next(r for r in native if r['id']=='double-rounding')
assert double['references']['A']['halfBits']==14337
assert double['references']['B']['halfBits']==14336
assert double['textureHalfBits']['fma']==14336

audit=json.loads((here/'software/adjudication.json').read_text())
parts=json.loads((here/'software/archive-parts.json').read_text())
archive_bytes=b''.join((here/'software'/name).read_bytes() for name in parts['parts'])
assert len(archive_bytes)==parts['size'] and hashlib.sha256(archive_bytes).hexdigest()==parts['sha256']
with tarfile.open(fileobj=io.BytesIO(archive_bytes),mode='r:xz') as z:
    data={name:z.extractfile(stored).read() for name,stored in json.loads((here/'software/texture-aliases.json').read_text()).items()}
    for name,digest in json.loads((here/'software/texture-sha256.json').read_text()).items():
        assert hashlib.sha256(data[name]).hexdigest()==digest,name
    def read(name,x,y):
        return struct.unpack_from('<H',data[name],((y%1024)*1024+x%1024)*8)[0]
    for variant in ['default','coarse-grain','horizontal-grain','straight-grain']:
        old=data[f'{variant}/warp-local.bin'];new=data[f'{variant}/warp-fma.bin']
        changes=[[i%1024,i//1024] for i in range(1024*1024) if old[i*8:i*8+8]!=new[i*8:i*8+8]]
        assert changes==[r['xy'] for r in audit['rows'] if r['variant']==variant]
    for row in audit['rows']:
        variant=row['variant'];x,y=row['xy'];field=hv(read(f'{variant}/distortion.bin',x,y))
        assert hb(field)==row['fieldHalfBits']
        da=delta(field,f32(row['strengthBits']),1024,False)
        db=delta(field,f32(row['strengthBits']),1024,True)
        assert str(da)==row['deltaExact'] and str(db)==row['deltaStaged']
        assert bits(rn(frac(db)))==row['weightBits']
        samples={}
        for key,d in [('Exact',da),('Staged',db)]:
            bx=x+floor(d)
            ss=[read(f'{variant}/stretch.bin',px,py) for px,py in [(bx,y),(bx+1,y),(bx,y+1),(bx+1,y+1)]]
            assert ss==row[f'samples{key}HalfBits'];samples[key]=list(map(hv,ss))
        for model,stage,weight,arithmetic in [('A','Exact',frac(da),'exact'),
            ('A_frozen_weight','Staged',rn(frac(db)),'exact'),('B','Staged',rn(frac(db)),'fused'),
            ('B_unfused','Staged',rn(frac(db)),'unfused'),('B_weighted','Staged',rn(frac(db)),'weighted')]:
            assert result(samples[stage],weight,F(0),arithmetic)==row['references'][model]
        for mode,key in [('local','beforeHalfBits'),('fma','afterHalfBits')]:
            assert read(f'{variant}/warp-{mode}.bin',x,y)==row[key]
    for i,row in enumerate(audit['rows']):
        inputs=struct.unpack_from('<32f',(here/'software/scalar-inputs.bin').read_bytes(),i*128)
        assert inputs[:3]==(float(hv(row['fieldHalfBits'])),float(f32(row['strengthBits'])),1024.0)
        assert list(inputs[3:7])==[float(hv(h)) for h in row['samplesStagedHalfBits']]
        assert bits(inputs[7])==row['weightBits'] and inputs[8]==0
        for mode,record in row['probes'].items():
            assert struct.unpack_from('<I',(here/'software'/f'scalar-{mode}.bin').read_bytes(),i*128)[0]==record['f32Bits']
        assert f32(row['probes']['mix']['f32Bits'])==(hv(row['beforeHalfBits'])+hv(row['afterHalfBits']))/2
        for side in ['native','chrome']:
            for mode in ['delta','weight','fma','fma-half']:
                replay=struct.unpack_from('<I',(here/'software'/f'{side}-replay/{mode}.bin').read_bytes(),i*128)[0]
                assert replay==row['probes'][mode]['f32Bits']

software_literals=json.loads((here/'software/literal-adjudication.json').read_text())
with zipfile.ZipFile(here/'software/literal-textures.zip') as z:
    for row in software_literals:
        for mode,h in row['textureHalfBits'].items():
            assert struct.unpack_from('<H',z.read(f"{row['id']}-{mode}/warp.bin"),row['xy'][0]*8)[0]==h
        assert row['textureHalfBits']['fma']==row['references']['B']['halfBits']

binding=json.loads((here/'software/product-png-binding.json').read_text())
assert len(binding)==32 and all(r['changedPixels']==0 and r['actualRgbaSha256']==r['recordedRgbaSha256'] for r in binding)
links=json.loads((here/'software/final-witness-links.json').read_text())
assert len(links)==14 and all(r['changedDependencies'] for r in links)
expected=[dict(path=entry['path'],**w) for entry in json.loads((here/'pr16-witnesses.json').read_text()) for w in entry['witnesses']]
assert [{k:v for k,v in link.items() if k!='changedDependencies'} for link in links]==expected
by_xy={(r['variant'],tuple(r['xy'])):r for r in audit['rows']}
for link in links:
    variant,channel=link['path'].split('/')
    x,y=link['xy']
    allowed=[(x,y)] if channel!='normal.png' else [((x-1)%1024,y),((x+1)%1024,y),(x,(y-1)%1024),(x,(y+1)%1024)]
    for dep in link['changedDependencies']:
        xy=tuple(dep['xy']);assert xy in allowed
        row=by_xy[(variant,xy)]
        assert dep['warpBefore']==row['beforeHalfBits'] and dep['warpAfter']==row['afterHalfBits']
        for mode,key in [('local','heightBefore'),('fma','heightAfter')]:
            assert read(f'{variant}/height-{mode}.bin',*xy)==dep[key]
print(f"PASS: retained hashes, dual-rounding witness, all {len(audit['rows'])} changed warp texels, 32 PNG bindings and 14 dependency links")
