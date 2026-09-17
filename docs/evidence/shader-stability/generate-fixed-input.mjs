import {readFileSync,writeFileSync} from 'node:fs';
const root='tmp/shader-stability',{cases}=JSON.parse(readFileSync(`${root}/sample-plan.json`));
const avalanche=x=>{let h=x>>>0;h=Math.imul(h^(h>>>16),0x7feb352d);h=Math.imul(h^(h>>>15),0x846ca68b);return (h^(h>>>16))>>>0;};
const b=Buffer.alloc(3072*4);cases.forEach((c,ci)=>{for(let slot=0;slot<256;slot++){let x=avalanche(slot+17)%c.width,y=avalanche(slot+991)%c.height;if(slot<4){x=slot%2*(c.width-1);y=Math.floor(slot/2)*(c.height-1);}b.writeFloatLE((x+.5)/c.width,(ci*256+slot)*8);b.writeFloatLE((y+.5)/c.height,(ci*256+slot)*8+4);}});writeFileSync(`${root}/uv-input.bin`,b);
for(const v of ['original','local','local_rational']){let s=readFileSync(`${root}/${v}-samples.wgsl`,'utf8').replace('let uv=(vec2<f32>(f32(x),f32(y))+vec2<f32>(0.5))/vec2<f32>(config.xy);','let uv=vec2<f32>(inputs[sample_index*2u],inputs[sample_index*2u+1u]);').replace('var sum=inputs[i];','var sum=0.0;');writeFileSync(`${root}/${v}-fixed.wgsl`,s);}
