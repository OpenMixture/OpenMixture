"""Retain critical software texture inputs/outputs; do not install any goldens."""
from pathlib import Path
import json,shutil,zipfile,hashlib,sys,tarfile,lzma,io
here=Path(__file__).resolve().parent
source=Path(sys.argv[1]);dest=here/'software';dest.mkdir(exist_ok=True)
for name in ['adjudication.json','verdict-summary.json','final-witness-links.json','product-png-binding.json',
             'execution.json','cases.json','plans-sha256.json']:
    shutil.copy2(source/name,dest/name)
for p in source.glob('scalar-*'):
    if p.is_file():shutil.copy2(p,dest/p.name)
for p in source.glob('*.json'):
    if p.stem.endswith(('-local','-fma','-original')) and isinstance(json.loads(p.read_text()),list):
        shutil.copy2(p,dest/p.name)
hashes={};aliases={};seen={}
with zipfile.ZipFile(source/'warp-textures.zip','w',zipfile.ZIP_DEFLATED,9) as z:
    def retain(key,data):
        digest=hashlib.sha256(data).hexdigest();hashes[key]=digest
        if digest not in seen:z.writestr(key,data);seen[digest]=key
        aliases[key]=seen[digest]
    for variant in ['default','coarse-grain','horizontal-grain','straight-grain']:
        for name in ['stretch','distortion']:
            a=source/f'{variant}-local/{name}.bin';b=source/f'{variant}-fma/{name}.bin'
            assert a.read_bytes()==b.read_bytes()
            key=f'{variant}/{name}.bin';retain(key,a.read_bytes())
        for mode in ['local','fma']:
            for name in ['warp','height']:
                key=f'{variant}/{name}-{mode}.bin';data=(source/f'{variant}-{mode}/{name}.bin').read_bytes()
                retain(key,data)
with zipfile.ZipFile(dest/'literal-textures.zip','w',zipfile.ZIP_DEFLATED,9) as z:
    for case in json.loads((source/'cases.json').read_text()):
        for mode in ['original','local','fma']:
            for p in (source/f"{case['id']}-{mode}").iterdir():z.write(p,p.relative_to(source).as_posix())
shutil.copy2(source/'default-fma/context.json',dest/'material-context.json')
(dest/'texture-sha256.json').write_text(json.dumps(hashes,indent=2)+'\n')
(dest/'texture-aliases.json').write_text(json.dumps(aliases,indent=2)+'\n')
buffer=io.BytesIO()
with zipfile.ZipFile(source/'warp-textures.zip') as archive, tarfile.open(fileobj=buffer,mode='w') as tar:
    for name in archive.namelist():
        raw=archive.read(name);info=tarfile.TarInfo(name);info.size=len(raw);tar.addfile(info,io.BytesIO(raw))
data=lzma.compress(buffer.getvalue(),filters=[{'id':lzma.FILTER_LZMA2,'preset':6,'dict_size':64*1024*1024}]);parts=[]
for i,start in enumerate(range(0,len(data),65536),1):
    name=f'warp-textures.tar.xz.{i:03d}';(dest/name).write_bytes(data[start:start+65536]);parts.append(name)
(dest/'archive-parts.json').write_text(json.dumps({'format':'tar.xz','size':len(data),'sha256':hashlib.sha256(data).hexdigest(),'parts':parts},indent=2)+'\n')
for path in dest.rglob('*.json'):
    if path.stat().st_size>65536:path.write_bytes((json.dumps(json.loads(path.read_text()),separators=(',',':'))+'\n').encode())
print('Retained',len(hashes),'texture buffers;',len(data),'archive bytes in',len(parts),'parts')
