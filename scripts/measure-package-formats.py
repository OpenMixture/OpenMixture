"""M6B-02 representation experiment, not a production loader. Python stdlib only.
Usage: python scripts/measure-package-formats.py <fresh-output>
Measures real encoded sizes and Python decoder allocations, not Rust/WASM RSS.
"""
import base64
import gc
import hashlib
import io
import json
import platform
import struct
import subprocess
import sys
import tarfile
import time
import tracemalloc
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir()  # Fresh output; never replace previous evidence.
SHA = lambda data: hashlib.sha256(data).hexdigest()
ENC = lambda data: json.dumps(data, ensure_ascii=True, separators=(',', ':')).encode()


def header(name, size, kind=tarfile.REGTYPE):
    info = tarfile.TarInfo(name)
    info.size, info.mode, info.type = size, 0o644, kind
    return info.tobuf(format=tarfile.USTAR_FORMAT)


def archive(entries):
    return b''.join(header(n, len(b)) + b + bytes(-len(b) % 512) for n, b in entries) + bytes(1024)


def inputs(count, width, height):
    document = json.loads((ROOT/'fixtures/nodes/image-input/height.mix').read_bytes())
    resources, payloads = [], []
    for index in range(count):
        name = 'heightSource' if index == 0 else f'heightSource{index}'
        if index:
            document['nodes'].append({'id':f'image{index}', 'type':'image-input', 'version':1,
                                      'parameters':{'resourceId':name}})
        raw = bytes((index, 37, 91, 255)) * (width * height)
        path = f'images/{index:04}.rgba'
        digest = SHA(b'mixture-image-rgba8-linear-v1\0' + struct.pack('<II',width,height) + raw)
        resources.append(dict(id=name, path=path, width=width, height=height,
                              format='rgba8-linear', bytesPerRow=width*4,
                              byteLength=len(raw), contentDigest=digest))
        payloads.append((path,raw))
    source = ENC(document)
    manifest = dict(format='openmixture-asset', version=1,
                    document=dict(path='material.mix',byteLength=len(source),sha256=SHA(source)),
                    resources=resources)
    return [('manifest.json',ENC(manifest)),('material.mix',source)] + payloads


def measure_decode(label, data, expected):
    gc.collect()
    tracemalloc.start()
    start = time.perf_counter()
    if label == 'jsonBase64':
        result = json.loads(data)
        decoded = [(entry['path'],base64.b64decode(entry['data'],validate=True)) for entry in result['entries']]
    elif label == 'zipStored':
        with zipfile.ZipFile(io.BytesIO(data)) as reader:
            decoded = [(name,reader.read(name)) for name in reader.namelist()]
    else:
        with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as reader:
            decoded = [(entry.name,reader.extractfile(entry).read()) for entry in reader]
    elapsed = time.perf_counter()-start
    _, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
    assert decoded == expected
    return dict(seconds=elapsed, pythonTracedPeakBytes=peak, decodedPayloadBytes=sum(len(b) for _,b in decoded))


rows = []
for name,count,width,height in [('normal',1,1024,1024),('maximumResources',8,2048,1024)]:
    entries = inputs(count,width,height)
    tar = archive(entries)
    assert archive(entries) == tar
    with io.BytesIO() as buffer:
        with zipfile.ZipFile(buffer,'w',compression=zipfile.ZIP_STORED,allowZip64=False) as writer:
            for path,data in entries:
                info=zipfile.ZipInfo(path, date_time=(1980,1,1,0,0,0))
                writer.writestr(info,data)
        zip_bytes=buffer.getvalue()
    encoded_json=ENC({'entries':[{'path':p,'data':base64.b64encode(b).decode('ascii')} for p,b in entries]})
    total=sum(len(b) for _,b in entries)
    row=dict(case=name,resourceCount=count,size=[width,height],directoryBytes=total,
             directoryFiles=len(entries),sourceBytes=len(entries[1][1]),manifestBytes=len(entries[0][1]),
             resourceBytes=sum(len(b) for _,b in entries[2:]),representations={})
    for label,data in [('jsonBase64',encoded_json),('zipStored',zip_bytes),('ustar',tar)]:
        row['representations'][label]=dict(bytes=len(data),overheadBytes=len(data)-total,
            sha256=SHA(data),decode=measure_decode(label,data,entries))
    p,d,m,r=len(tar),len(entries[1][1]),len(entries[0][1]),row['resourceBytes']
    row['plannedBufferLedger']=dict(nativeBorrowed=d+m+r,nativeOwned=p+d+m+r,
        browserConservative=2*p+d+m+r,
        note='Analytical live byte-buffer bounds, not measured Rust/WASM allocation or process RSS; excludes caller input, typed objects and output buffers')
    rows.append(row)
    del tar,zip_bytes,encoded_json,entries
    gc.collect()

# Actual tiny archives for the future Rust reader's negative corpus.
entries=inputs(1,2,2)
valid=archive(entries)
with tarfile.open(fileobj=io.BytesIO(valid),mode='r:') as reader:
    assert [(item.name,reader.extractfile(item).read()) for item in reader] == entries
corpus=OUT/'corpus'
corpus.mkdir()
specs=[]
def save(name,data,code):
    (corpus/f'{name}.mixpack').write_bytes(data)
    specs.append(dict(file=f'{name}.mixpack',bytes=len(data),sha256=SHA(data),expectedCode=code))
save('valid',valid,None)
save('truncated',valid[:-513],'MIX_PACKAGE_INVALID')
save('trailing',valid+b'x','MIX_PACKAGE_INVALID')
save('duplicate-entry',archive(entries+[entries[-1]]),'MIX_PACKAGE_INVALID')
for label,path in [('parent','../material.mix'),('absolute','/material.mix'),('drive','C:/material.mix'),('backslash','images\\0000.rgba')]:
    save(label,archive([entries[0],(path,entries[1][1]),entries[2]]),'MIX_PACKAGE_INVALID')
bad=bytearray(valid); bad[0]^=1
save('checksum',bad,'MIX_PACKAGE_INVALID')
save('symlink',header('manifest.json',len(entries[0][1]),tarfile.SYMTYPE)+valid[512:],'MIX_PACKAGE_INVALID')
save('oversized-entry',header('manifest.json',65537)+valid[512:],'MIX_PACKAGE_LIMIT_EXCEEDED')
manifest=json.loads(entries[0][1]);manifest['version']=2
save('unsupported-version',archive([('manifest.json',ENC(manifest))]+entries[1:]),'MIX_PACKAGE_UNSUPPORTED_VERSION')
manifest['version']=1;manifest['resources'][0]['contentDigest']='0'*64
save('digest-mismatch',archive([('manifest.json',ENC(manifest))]+entries[1:]),'MIX_PACKAGE_CONTENT_MISMATCH')
duplicate=entries[0][1].replace(b'"version":1',b'"version":1,"version":1')
save('duplicate-json-key',archive([('manifest.json',duplicate)]+entries[1:]),'MIX_PACKAGE_INVALID')
save('too-many-resources',archive(inputs(9,2,2)),'MIX_PACKAGE_LIMIT_EXCEEDED')
for name,field,value,code in [
    ('integer-overflow','width',2**64,'MIX_PACKAGE_INVALID'),
    ('dimension-limit','width',2049,'MIX_PACKAGE_LIMIT_EXCEEDED'),
    ('stride','bytesPerRow',9,'MIX_PACKAGE_INVALID'),
    ('length','byteLength',15,'MIX_PACKAGE_INVALID'),
    ('unknown-format','format','rgba8-srgb','MIX_PACKAGE_INVALID')]:
    changed=json.loads(entries[0][1]);changed['resources'][0][field]=value
    save(name,archive([('manifest.json',ENC(changed))]+entries[1:]),code)
duplicate_entries=inputs(3,2,2)
duplicate_manifest=json.loads(duplicate_entries[0][1])
duplicate_manifest['resources'][1]['id']=duplicate_manifest['resources'][0]['id']
save('duplicate-id',archive([('manifest.json',ENC(duplicate_manifest))]+duplicate_entries[1:]),'MIX_PACKAGE_INVALID')
bad=bytearray(valid);bad[-1025]=1
save('nonzero-padding',bad,'MIX_PACKAGE_INVALID')
(corpus/'cases.json').write_text(json.dumps(specs,indent=2)+'\n')
report=dict(kind='M6B-02-design-experiment-not-runtime-qualification',
    sourceRevision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    dirty=bool(subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True).strip()),
    python=sys.version,host=platform.platform(),probeSha256=SHA(Path(__file__).read_bytes()),
    rows=rows,corpus=specs,
    limits=dict(packageBytes=67*1024**2,manifestBytes=65536,sourceBytes=2*1024**2,
                resources=8,resourceBytes=64*1024**2,packageBufferBytes=202*1024**2),
    scope='Python stdlib decode measurements validate representation costs and fixture byte identity; planned Rust buffer bounds require allocator/lifetime regression tests in M6B-03/04. Corpus rejection codes are contract expectations, not passed runtime tests.')
(OUT/'measurement.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(rows,indent=2))
