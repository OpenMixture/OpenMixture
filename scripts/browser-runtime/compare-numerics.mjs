// Compare raw production-shader captures; no CPU pixel execution.
import { readFile, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
const [native, browser, output] = process.argv.slice(2);
assert.ok(native && browser && output, 'usage: compare-numerics.mjs <native-capture> <browser-capture-with-native-replay> <report.json>');
const receipt = JSON.parse(await readFile(`${browser}/receipt.json`, 'utf8'));
assert.deepEqual(receipt.parameters, JSON.parse(await readFile(`${native}/parameters.json`, 'utf8')), 'parameters differ');
assert.equal(receipt.replaySha256, createHash('sha256').update(await readFile(`${native}/noise.rgba16`)).digest('hex'), 'replay must use these exact native heights');
assert.equal(receipt.shaders['fractal-noise'], createHash('sha256').update(await readFile(`${native}/noise.wgsl`)).digest('hex'), 'noise shaders differ');
function half(h) {
  const sign = h & 0x8000 ? -1 : 1, exponent = (h >> 10) & 31, fraction = h & 1023;
  assert.notEqual(exponent,31,'nonfinite half');
  return sign * (exponent ? (1+fraction/1024)*2**(exponent-15) : fraction*2**-24);
}
async function load(dir,name) {
  const b = await readFile(`${dir}/${name}.rgba16`);
  assert.equal(b.length,1024*1024*8);
  return Array.from({length:b.length/2},(_,i)=>b.readUInt16LE(i*2));
}
const a = await load(native,'noise'), b = await load(browser,'noise');
const noise = [];
for(let i=0;i<a.length;i+=4) if(a[i]!==b[i]) noise.push({x:i/4%1024,y:Math.floor(i/4096),nativeBits:a[i],browserBits:b[i],native:half(a[i]),browser:half(b[i])});
async function compare(aDir,aName,bDir,bName) {
  const a=await load(aDir,aName), b=await load(bDir,bName);
  let rawChanged=0,changed=0,max=0; const overLimit=[];
  for(let i=0;i<a.length;i++) {
    if(a[i]!==b[i])rawChanged++;
    const av=Math.round(half(a[i])*255),bv=Math.round(half(b[i])*255),d=Math.abs(av-bv);
    if(d)changed++;max=Math.max(max,d);
    if(d>1)overLimit.push({x:Math.floor(i/4)%1024,y:Math.floor(i/4096),component:i%4,a:av,b:bv,delta:d});
  }
  return {rawChanged,changedComponents:changed,maxComponentDelta:max,overLimit};
}
const result={kind:'diagnostic-not-qualification',noiseChangedPixels:noise.length,noise,normal:await compare(native,'normal',browser,'normal'),sameInputNormal:await compare(native,'normal',browser,'replay-normal')};
await writeFile(output,JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({...result,noise:undefined,normal:{...result.normal,overLimit:result.normal.overLimit.length},sameInputNormal:{...result.sameInputNormal,overLimit:result.sameInputNormal.overLimit.length}},null,2));
