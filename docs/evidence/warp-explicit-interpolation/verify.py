from pathlib import Path
import json,struct
r=Path(__file__).resolve().parent
load=lambda n:json.loads((r/n).read_text(encoding='utf-8-sig'))
reference=load('native-literals.json')['cases']
assert len(reference)==23
for name in ['software-literals.json','before-literals.json']:
 cases=load(name)['cases'];assert len(cases)==23
 for a,b in zip(reference,cases):
  assert a['case']==b['case'];assert a['input']==b['input'];assert a['expectedScalar']==b['expectedScalar'];assert a['actualRgba']==b['actualRgba']
for name in ['after-chrome.json','after-edge.json']:
 result=load(name);assert result['ok'];assert len(result['cases'])==23
 for a,b in zip(reference,result['cases']):assert a['case']==b['case'] and a['actualRgba']==b['actualRgba']
red=load('before-chrome.json');assert not red['ok'];assert [c['case'] for c in red['cases'] if not c['pass']]==['interpolation-half-boundary-witness']
software=load('software-report.json');assert software['machineChecksPassed'] and not software['goldenComparisonsPassed'];assert sum(not v['comparison']['ok'] for c in software['cases'] for v in c['channels'].values())==12
native=load('native-comparison.json');assert len(native)==44 and all(v['changedPixels']==0 for v in native)
chrome=load('chrome-comparison.json');channels=[v['comparison'] for c in chrome['cases'] for v in c['channels'].values()];assert len(channels)==44 and sum(v['ok'] for v in channels)==27
print('PASS: 23 bound literal cases, original Chrome failure, unchanged native materials, and preserved failed software/browser gates')
