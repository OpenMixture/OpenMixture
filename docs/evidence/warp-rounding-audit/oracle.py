"""Exact rational scalar diagnostics, never a CPU material renderer.

Predeclared references (all roundings below use nearest, ties-to-even):
A: exact field/strength displacement, exact wrapped bilinear interpolation -> half.
B: separately rounded binary32 field multiply/subtract, strength multiply, size
multiply and fract; separately rounded differences; fused x/y interpolation -> half.
B_unfused: same coordinates, multiply/add instead of fused interpolation.
B_weighted: same coordinates, separate (1-t)*a + t*b stages.
These are hypothetical specified models, not WGSL's universal requirements.
"""
from fractions import Fraction as F
import struct, json

def pow2(e): return F(2)**e

def rn(x,p=24,emin=-126):
    x=F(x)
    if x<0: return -rn(-x,p,emin)
    if not x: return x
    e=x.numerator.bit_length()-x.denominator.bit_length()
    if x<pow2(e): e-=1
    step=pow2(max(e,emin)-p+1)
    q,r=divmod((x/step).numerator,(x/step).denominator)
    if 2*r>(x/step).denominator or (2*r==(x/step).denominator and q%2): q+=1
    return q*step

def half(x): return rn(x,11,-14)
def f32(b): return F(struct.unpack('<f',struct.pack('<I',b))[0])
def bits(x): return struct.unpack('<I',struct.pack('<f',float(x)))[0]
def hb(x): return struct.unpack('<H',struct.pack('<e',float(x)))[0]
def hv(b): return F(struct.unpack('<e',struct.pack('<H',b))[0])
def floor(x): return x.numerator//x.denominator
def frac(x): return x-floor(x)

def lerp(a,b,t,model):
    if model=='exact': return a+(b-a)*t
    if model=='fused': return rn(rn(b-a)*t+a)
    if model=='unfused': return rn(rn(rn(b-a)*t)+a)
    if model=='weighted': return rn(rn(rn(1-t)*a)+rn(t*b))
    raise ValueError(model)

def bilinear(samples,tx,ty,model):
    a,b,c,d=samples
    return lerp(lerp(a,b,tx,model),lerp(c,d,tx,model),ty,model)

def delta(field,strength,size,staged):
    if staged: return rn(rn(rn(rn(2*field)-1)*strength)*size)
    return (2*field-1)*strength*size

def result(samples,tx,ty,model):
    raw=bilinear(samples,tx,ty,model)
    return dict(raw=str(raw),rawFloat=float(raw),halfBits=hb(half(raw)),halfFloat=float(half(raw)))

def self_test():
    # Signed values, ties-even, normal/subnormal boundary, and zero.
    for x in [F(0),F(-1,3),pow2(-25),3*pow2(-25),pow2(-14)-pow2(-25),F(1,2)+pow2(-12)]:
        assert float(half(x))==struct.unpack('<e',struct.pack('<e',float(x)))[0]
    t=f32(0x3a000001); r=F(1,2)+t/2
    assert half(r)==F(1025,2048) and half(rn(r))==F(1,2)
    a,b,t=F(873,2048),F(349,1024),F(2034237,2097152)
    assert half(lerp(a,b,t,'exact'))==F(1407,4096)
    assert half(lerp(a,b,t,'fused'))==F(1407,4096)
    assert half(lerp(a,b,t,'unfused'))==F(1406,4096)
    assert half(pow2(-14)+(1-pow2(-14))*pow2(-16))==5*pow2(-16)

if __name__=='__main__':
    self_test()
    print('PASS: independent rounding-model witnesses and signed/tie/subnormal checks')
