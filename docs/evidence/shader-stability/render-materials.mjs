import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {resolve,join} from 'node:path';
import {createHash} from 'node:crypto';
const mode=process.argv[2],root=resolve('tmp/shader-stability'),exe=join(root,`mixture-${mode}.exe`),cases=JSON.parse(readFileSync('docs/evidence/shader-stability/original-full.json')).cases;
const report={diagnosticOnly:true,variant:mode,binarySha256:createHash('sha256').update(readFileSync(exe)).digest('hex'),cases:[]};
for(const c of cases){const out=join(root,`${mode}-full`,c.material,c.id);mkdirSync(out,{recursive:true});const args=['render',`fixtures/materials/${c.material}/material.mix`,'--out',out,'--json','--backend','dx12','--size','1024','--output','baseColor,normal,roughness,height'];for(const [key,value]of Object.entries(c.overrides))args.push('--set',`${key}=${JSON.stringify(value)}`);const result=JSON.parse(execFileSync(exe,args,{encoding:'utf8',maxBuffer:8*1024*1024}));writeFileSync(join(out,'render.json'),JSON.stringify(result,null,2));report.cases.push({material:c.material,id:c.id,overrides:c.overrides,planHash:result.planHash});}
writeFileSync(join(root,`${mode}-full.json`),JSON.stringify(report,null,2));console.log(`${mode}: ${cases.length} full-material renders completed`);
