import {readFileSync,writeFileSync} from 'node:fs';
const phase=process.argv[2]??"samples";
const root='tmp/shader-stability',plan=JSON.parse(readFileSync(`${root}/sample-plan.json`));
const f=Math.fround;const avalanche=x=>{let h=x>>>0;h=Math.imul(h^(h>>>16),0x7feb352d);h=Math.imul(h^(h>>>15),0x846ca68b);return (h^(h>>>16))>>>0;};
const lattice=(x,y,p,seed)=>(avalanche(seed^avalanche((x%p+0x9e3779b9)>>>0)^avalanche((y%p+0x85ebca6b)>>>0))>>>8)/16777216;
// Binary64 scalar samples for conditioning assessment, not a material executor.
// Start at the same rounded f32 p coordinates and f32 constants as the shader.
function reference(c,slot){let x=avalanche(slot+17)%c.width,y=avalanche(slot+991)%c.height;if(slot<4){x=slot%2*(c.width-1);y=Math.floor(slot/2)*(c.height-1);}let sum=0,weight=1,weights=0,period=c.scale;
 for(let oct=0;oct<c.octaves;oct++){
 const px=f(f((x+.5)/c.width)*period),py=f(f((y+.5)/c.height)*period),cx=Math.floor(px),cy=Math.floor(py),seed=avalanche((c.seed+Math.imul(oct,0x9e3779b9))>>>0);let nearest=100,second=100;
 for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++){let wx=(cx+dx+period)%period,wy=(cy+dy+period)%period;let ax=dx+f(.2)+f(.6)*lattice(wx,wy,period,seed)-(px-cx),ay=dy+f(.2)+f(.6)*lattice(wx,wy,period,seed^0x68bc21eb)-(py-cy);let d=ax*ax+ay*ay;if(d<nearest){second=nearest;nearest=d;}else second=Math.min(second,d);}
 sum+=Math.max(0,Math.min(1,Math.sqrt(second)-Math.sqrt(nearest)))*weight;weights+=weight;weight*=f(c.persistence);period*=2;
 }return {x,y,value:Math.max(0,Math.min(1,sum/weights))};}
const result={diagnosticOnly:true,reference:phase==="fixed"?"Binary64 scalar estimate from common stored f32 UV and constants; exact power-of-two scaling; not an exact oracle or material renderer.":"Binary64 scalar estimate from reference f32 UV division; shader UV division may differ, so this includes coordinate-generation error.",grid:[],samples:[]};
for(const v of ['original','local','local_rational']){
 const a=readFileSync(`${root}/${v}-regular.bin`),b=readFileSync(`${root}/${v}-strict.bin`);let changed=[0,0],max=0,nonFinite=0,outOfRange=0;for(let i=0;i<a.length/4;i++){if(a.readUInt32LE(i*4)!==b.readUInt32LE(i*4))changed[i%2]++;max=Math.max(max,Math.abs(a.readFloatLE(i*4)-b.readFloatLE(i*4)));for(const buf of [a,b]){let x=buf.readFloatLE(i*4);if(!Number.isFinite(x))nonFinite++;if(x<0||x>1)outOfRange++;}}
 result.grid.push({variant:v,samples:1048576,rawChanged:changed[0],halfChanged:changed[1],maxIncludingHalf:max,nonFinite,outOfRange});
 const buffers=Object.fromEntries(['regular','strict','chrome','edge'].map(m=>[m,readFileSync(`${root}/${v}-${phase}-${m}.bin`)]));
 for(let ci=0;ci<plan.cases.length;ci++){const c=plan.cases[ci],row={variant:v,case:c.name,count:256,accuracy:{},crossCompiler:{}};
 for(const [m,buffer]of Object.entries(buffers)){let max=0,total=0,worst=null;for(let slot=0;slot<256;slot++){let ref=reference(c,slot),actual=buffer.readFloatLE((ci*256+slot)*8),err=Math.abs(actual-ref.value);total+=err;if(err>max){max=err;worst={slot,...ref,actual};}}row.accuracy[m]={maxAbsolute:max,meanAbsolute:total/256,worst};}
 for(const m of ['strict','chrome','edge']){let raw=0,half=0,max=0;for(let slot=0;slot<256;slot++){const i=(ci*256+slot)*8;raw+=Number(buffers.regular.readUInt32LE(i)!==buffers[m].readUInt32LE(i));half+=Number(buffers.regular.readUInt32LE(i+4)!==buffers[m].readUInt32LE(i+4));max=Math.max(max,Math.abs(buffers.regular.readFloatLE(i)-buffers[m].readFloatLE(i)));}row.crossCompiler[m]={rawChanged:raw,halfChanged:half,maxRawAbsolute:max};}result.samples.push(row);
 }
}
writeFileSync(`${root}/measurements-${phase}.json`,JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result.grid,null,2));console.log(result.samples.map(r=>({variant:r.variant,case:r.case,maxNativeError:r.accuracy.regular.maxAbsolute,maxBrowserError:r.accuracy.chrome.maxAbsolute,chromeHalfChanged:r.crossCompiler.chrome.halfChanged})));
