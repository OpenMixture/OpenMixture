import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
const root='tmp/shader-stability';mkdirSync(root,{recursive:true});
const src=readFileSync('docs/evidence/chromium-arithmetic-reduction/noise-grid.wgsl','utf8');
const old='let delta = vec2<f32>(neighbor) + vec2<f32>(0.2) + 0.6 * jitter - p;';
const local='let delta = vec2<f32>(vec2<i32>(x, y)) + vec2<f32>(0.2) + 0.6 * jitter - fract(p);';
const rational='return clamp((second - nearest) / (sqrt(second) + sqrt(nearest)), 0.0, 1.0);';
const variants={original:src,local:src.replace(old,local),local_rational:src.replace(old,local).replace('return clamp(sqrt(second) - sqrt(nearest), 0.0, 1.0);',rational)};
for(const [name,source] of Object.entries(variants))writeFileSync(`${root}/${name}.wgsl`,source);
writeFileSync(`${root}/plan.json`,JSON.stringify({diagnosticOnly:true,variants:['original','local','local_rational'],grid:{size:[1024,1024],seed:271828,scale:64,octaves:3,persistence:0.35},selection:'Fixed before execution. Compare original, local coordinates only, and local coordinates plus rationalized sqrt difference. No quantization or threshold changes.'},null,2));
