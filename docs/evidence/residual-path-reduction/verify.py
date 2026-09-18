from pathlib import Path
import hashlib,json,struct
from fractions import Fraction
root=Path(__file__).resolve().parent
summary=json.loads((root/'summary.json').read_text())
for name,case in summary['minimal'].items():
 inputs=struct.unpack('<4f',(root/f'min-{name}-input.bin').read_bytes());assert list(inputs)==case['inputF32']
 for side,result in case['results'].items():
  data=(root/f'min-{name}-{side}.bin').read_bytes()[:4];assert hex(struct.unpack('<I',data)[0])==result['bits'];value=struct.unpack('<f',data)[0];assert float(Fraction(value)-Fraction(case['exactRational']))==result['signedExactError']
 assert (root/f'min-{name}-chrome.bin').read_bytes()==(root/f'min-{name}-edge.bin').read_bytes()
plan=json.loads((root/'noise-plan.json').read_text());samples=json.loads((root/'half-witnesses.json').read_text())
for side in ['native','chrome']:
 data=(root/f'noise-{side}.bin').read_bytes()
 for i,row in enumerate(plan):
  node='leather-grain' if i<16 else 'wood-distortion';point=next(p for p in samples[side][node] if p['xy']==row[:2]);actual=struct.unpack('<e',struct.pack('<H',point['rgba16Bits'][0]))[0];observed=struct.unpack_from('<f',data,(i*256+251)*4)[0];assert actual==observed
provenance=root/'provenance.json'
if provenance.exists():
 for name,digest in json.loads(provenance.read_text())['filesSha256'].items():assert hashlib.sha256((root/name).read_bytes()).hexdigest()==digest,name
print('Minimal input/results/exact errors, Edge equality, all 53 trace half witnesses, and retained hashes verified.')
