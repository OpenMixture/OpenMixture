"""Trace the pre-existing 14 final PNG changes to captured warp texels."""
from pathlib import Path
import json,struct,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1])
audit=json.loads((out/'adjudication.json').read_text())
by_xy={(r['variant'],tuple(r['xy'])):r for r in audit['rows']}
links=[]
for channel in json.loads((here/'pr16-witnesses.json').read_text()):
    variant,file=channel['path'].split('/')
    for witness in channel['witnesses']:
        x,y=witness['xy']
        positions=[(x,y)] if file!='normal.png' else [((x-1)%1024,y),((x+1)%1024,y),(x,(y-1)%1024),(x,(y+1)%1024)]
        causes=[]
        for xy in positions:
            row=by_xy.get((variant,xy))
            if not row: continue
            before=(out/f'{variant}-local/height.bin').read_bytes()
            after=(out/f'{variant}-fma/height.bin').read_bytes()
            offset=(xy[1]*1024+xy[0])*8
            h0=struct.unpack_from('<H',before,offset)[0];h1=struct.unpack_from('<H',after,offset)[0]
            if h0==h1: continue
            causes.append(dict(xy=list(xy),warpBefore=row['beforeHalfBits'],warpAfter=row['afterHalfBits'],
                heightBefore=h0,heightAfter=h1,beforeMatches=row['beforeMatches'],afterMatches=row['afterMatches']))
        links.append(dict(path=channel['path'],**witness,changedDependencies=causes))
(out/'final-witness-links.json').write_text(json.dumps(links,indent=2)+'\n')
print(json.dumps({'finalWitnesses':len(links),'linked':sum(bool(r['changedDependencies']) for r in links)},indent=2))
assert len(links)==14 and all(r['changedDependencies'] for r in links)
