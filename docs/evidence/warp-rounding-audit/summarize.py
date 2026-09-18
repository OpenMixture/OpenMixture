"""Summarize actual half-value errors without changing reference definitions."""
from pathlib import Path
import json,sys
from oracle import F,hv
out=Path(sys.argv[1]);audit=json.loads((out/'adjudication.json').read_text())
summary={}
for variant in ['default','coarse-grain','horizontal-grain','straight-grain']:
    rows=[r for r in audit['rows'] if r['variant']==variant]
    comparisons={}
    for ref in ['A','A_frozen_weight','B','B_unfused','B_weighted']:
        counts={'beforeMatches':0,'afterMatches':0,'closer':0,'farther':0,'equalError':0}
        for row in rows:
            target=F(row['references'][ref]['raw'])
            before=abs(hv(row['beforeHalfBits'])-target);after=abs(hv(row['afterHalfBits'])-target)
            counts['beforeMatches']+=row['beforeHalfBits']==row['references'][ref]['halfBits']
            counts['afterMatches']+=row['afterHalfBits']==row['references'][ref]['halfBits']
            counts['closer' if after<before else 'farther' if after>before else 'equalError']+=1
        comparisons[ref]=counts
    summary[variant]={'changedWarpTexels':len(rows),'comparisons':comparisons,
        'coordinateProbeMismatches':sum(not r['coordinateProbeMatchesB'] for r in rows),
        'productionHalfProbeMismatches':sum(not all(r['probeMatchesTexture'].values()) for r in rows),
        'neighborSelectionChanges':sum(r['baseExact']!=r['baseStaged'] for r in rows)}
(out/'verdict-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
