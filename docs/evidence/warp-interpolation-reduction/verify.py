from pathlib import Path
import json,struct,hashlib,zipfile
from fractions import Fraction
r=Path(__file__).resolve().parent
s=json.loads((r/'summary.json').read_text());points=json.loads((r/'points.json').read_text());oracle=json.loads((r/'oracle.json').read_text())
def value(name,i):return struct.unpack_from('<f',(r/name).read_bytes(),i*128)[0]
def hb(x):return struct.unpack('<H',struct.pack('<e',x))[0]
def half(h):return Fraction(struct.unpack('<e',struct.pack('<H',h))[0])
with zipfile.ZipFile(r/'grid-evidence.zip') as z:
 data={n:z.read(n) for n in z.namelist()}
 for n,b in data.items():assert hashlib.sha256(b).hexdigest()==s['textureSha256'][n]
 a=data['warp-before-native.bin'];b=data['warp-before-chrome.bin'];xy=[]
 for k in range(1024*1024):
  if a[k*8:k*8+8]!=b[k*8:k*8+8]:xy.append([k%1024,k//1024])
 assert xy==[p['xy'] for p in points]
 for i,p in enumerate(points):
  x,y=p['xy'];row=struct.unpack_from('<32f',(r/'inputs.bin').read_bytes(),i*128);bx=int(row[11]);f=struct.unpack_from('<e',data['wood-distortion.bin'],(y*1024+x)*8)[0];assert row[0]==f
  for j,(tx,ty) in enumerate([(bx%1024,y),((bx+1)%1024,y),(bx%1024,(y+1)%1024),((bx+1)%1024,(y+1)%1024)]):assert row[3+j]==struct.unpack_from('<e',data['wood-stretch.bin'],(ty*1024+tx)*8)[0]
  assert value('delta-native.bin',i)==value('delta-chrome.bin',i)
  assert value('weight-native.bin',i)==value('weight-chrome.bin',i)==row[9]
  for mode in ['mix','fixed-mix','difference']:
   assert hb(value(mode+'-half-native.bin',i))==p['nativeBits'][0]
   assert hb(value(mode+'-half-chrome.bin',i))==p['chromeBits'][0]
  aa,bb,t=map(Fraction,[row[3],row[4],row[9]]);exact=aa+(bb-aa)*t;approx=hb(float(exact));nearest=min(range(approx-2,approx+3),key=lambda h:(abs(half(h)-exact),h%2))
  assert str(exact)==oracle['rows'][i]['exact'];assert nearest==oracle['rows'][i]['correctHalfBits']
  for side in ['native','chrome','edge']:assert hb(value('fma-half-'+side+'.bin',i))==nearest
  assert hb(value('fixed-mix-half-native.bin',i))==nearest
  assert hb(value('fixed-mix-half-chrome.bin',i))!=nearest
for n in ['fixed-mix','fixed-mix-half','fma','fma-half']:assert (r/(n+'-chrome.bin')).read_bytes()==(r/(n+'-edge.bin')).read_bytes()
assert len({s['textureSha256'][n] for n in ['warp-before-native.bin','warp-after-native.bin','warp-after-chrome.bin','warp-after-edge.bin']})==1
print('PASS: 26 texture-bound scalar witnesses, exact rational oracle, Edge replay, and full-grid content hashes')
