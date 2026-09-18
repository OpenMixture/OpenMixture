"""Recompute the 23 fixed PR16 literal expectations under predeclared models."""
from pathlib import Path
import json
from oracle import F, bits, f32, half, floor, frac, rn, delta, bilinear, hb, self_test
self_test()
here=Path(__file__).resolve().parent
report=[]
for case in json.loads((here/'pr16-literal-inputs.json').read_text())['cases']:
    w,h=case['size']; strengths=list(map(lambda x:f32(bits(x)),case['parameters']['strength']))
    values=list(map(lambda x:half(F(x)),case['input'])); fields=list(map(lambda x:half(F(x)),case['displacement']))
    outcomes={}
    for model in ['A','B','B_unfused','B_weighted']:
        output=[]
        for i,field in enumerate(fields):
            field=max(F(0),min(F(1),field));x,y=i%w,i//w
            if field==F(1,2) or not any(strengths): value=values[i]
            else:
                dx,dy=[delta(field,s,n,model!='A') for s,n in zip(strengths,[w,h])]
                bx,by=x+floor(dx),y+floor(dy)
                samples=[values[(py%h)*w+px%w] for px,py in [(bx,by),(bx+1,by),(bx,by+1),(bx+1,by+1)]]
                tx,ty=frac(dx),frac(dy)
                if model!='A': tx,ty=rn(tx),rn(ty)
                value=half(bilinear(samples,tx,ty,{'A':'exact','B':'fused','B_unfused':'unfused','B_weighted':'weighted'}[model]))
            output.append(hb(value))
        expected=[hb(v) for v in case['expectedScalar']]
        outcomes[model]={'expectedMatches':sum(a==b for a,b in zip(output,expected)),
                         'pixels':len(output),'differentPixels':[i for i,(a,b) in enumerate(zip(output,expected)) if a!=b]}
    report.append({'case':case['case'],'models':outcomes})
(here/'literal-reference-check.json').write_bytes((json.dumps(report,indent=2)+'\n').encode())
print(json.dumps({model:{'matchingCases':sum(r['models'][model]['expectedMatches']==r['models'][model]['pixels'] for r in report),'totalCases':len(report)} for model in outcomes},indent=2))
