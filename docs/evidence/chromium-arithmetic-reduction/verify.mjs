// Independent exact arithmetic for the positive, normal f32 witness only.
// This is not a CPU material renderer, a portable fma implementation, or a gate change.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {resolve,join} from 'node:path';
const dir=process.argv[2]?resolve(process.argv[2]):fileURLToPath(new URL('.',import.meta.url));
const decode=bits=>{assert.equal(bits>>>31,0);const exponent=(bits>>>23)&255;assert.ok(exponent>0&&exponent<255);return {n:BigInt((bits&0x7fffff)|0x800000),e:exponent-150};};
const add=(a,b)=>{const e=Math.min(a.e,b.e);return {n:(a.n<<BigInt(a.e-e))+(b.n<<BigInt(b.e-e)),e};};
const mul=(a,b)=>({n:a.n*b.n,e:a.e+b.e});
const round=x=>{
 let shift=x.n.toString(2).length-24;assert.ok(shift>=0);
 let n=x.n>>BigInt(shift);
 if(shift){const rest=x.n-(n<<BigInt(shift)),middle=1n<<BigInt(shift-1);if(rest>middle||(rest===middle&&(n&1n)))n++;}
 if(n===0x1000000n){n>>=1n;shift++;}
 const exponent=x.e+shift+150;assert.ok(exponent>0&&exponent<255);
 return exponent*0x800000+Number(n-0x800000n);
};
const input=readFileSync(join(dir,'minimal-input.bin'));
const [a,b,c]=[0,4,8].map(i=>decode(input.readUInt32LE(i)));
const fused=round(add(mul(a,b),c));
const separate=round(add(decode(round(mul(a,b))),c));
assert.equal(fused,0x3eb07fc5);assert.equal(separate,0x3eb07fc6);
const exact=add(mul(a,b),c);
const distance=bits=>{const x=decode(bits),e=Math.min(x.e,exact.e);const difference=(x.n<<BigInt(x.e-e))-(exact.n<<BigInt(exact.e-e));return {n:difference<0n?-difference:difference,e};};
const fa=distance(fused),sa=distance(separate);const e=Math.min(fa.e,sa.e);
assert.ok((fa.n<<BigInt(fa.e-e))<(sa.n<<BigInt(sa.e-e)));
for(const [mode,expected] of Object.entries({regular:fused,strict:separate,chrome:separate,edge:separate})){
 const data=readFileSync(join(dir,`minimal-${mode}.bin`));assert.equal(data.length,16);
 for(let i=0;i<4;i++)assert.equal(data.readUInt32LE(i*4),expected,`${mode} lane ${i}`);
}
console.log(JSON.stringify({diagnosticOnly:true,fusedBits:'0x'+fused.toString(16),separateBits:'0x'+separate.toString(16),fusedIsCloserToExact:true,retainedGpuWitnessesMatch:true}));
