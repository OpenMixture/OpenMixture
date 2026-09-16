// Produce an explicit native reference bundle; never install engine code in a product.
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
const root = resolve(import.meta.dirname, '../..');
const [destination, runtimeRevision, authoredDirectory] = process.argv.slice(2);
if (!authoredDirectory || !destination || !/^[a-f0-9]{40}$/.test(runtimeRevision ?? '')) throw new Error('Usage: node scripts/browser-runtime/prepare-materials.mjs <new-directory> <runtime-engine-revision> <detached-studio-downloads>');
const out = resolve(destination);
if (existsSync(out)) throw new Error('Reference directory already exists; select a fresh run directory');
const git = (...args) => execFileSync('git', args, {cwd:root,encoding:'utf8'}).trim();
// A plan hash alone cannot certify an implementation. Require matching runtime inputs.
git('diff','--exit-code',runtimeRevision,'--','crates','Cargo.lock','Cargo.toml');
execFileSync('cargo',['build','--locked','-p','mixture-cli'],{cwd:root,stdio:'inherit'});
mkdirSync(out,{recursive:true});
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const cli=join(root,'target/debug',process.platform==='win32'?'mixture.exe':'mixture');
const backend=process.env.MIXTURE_GPU_BACKEND;
if(!['metal','vulkan','dx12'].includes(backend))throw new Error('Set MIXTURE_GPU_BACKEND explicitly');
const policy=['--backend',backend,...(process.env.MIXTURE_GPU_SOFTWARE==='1'?['--software']:[])];
const manifest={schemaVersion:1,kind:"studio",engineRevision:git('rev-parse','HEAD'),engineDirty:Boolean(git('status','--porcelain')),runtimeRevision,
 startedAt:new Date().toISOString(),nativeBinarySha256:sha(readFileSync(cli)),cases:[]};
const criteriaPath=join(root,'scripts/browser-runtime/studio-criteria.json');
const criteria=JSON.parse(readFileSync(criteriaPath));
const authored=JSON.parse(readFileSync(join(authoredDirectory,'authored.json')));
if(authored.schemaVersion!==1 || authored.cases.length!==criteria.cases.length)throw Error('Wrong authored matrix');
manifest.criteriaSha256=sha(readFileSync(criteriaPath));
manifest.authoredManifestSha256=sha(readFileSync(join(authoredDirectory,'authored.json')));
manifest.productRevision=authored.productRevision;
manifest.archiveSha256=authored.archiveSha256;
for(const item of criteria.cases){
 const {material,id}=item;
 const matching=authored.cases.filter(c=>c.material===material && c.id===id);
 if(matching.length!==1)throw Error('Missing/duplicate authored case');
 const sourcePath=resolve(authoredDirectory,material,`${id}.mix`),source=readFileSync(sourcePath);
 if(sha(source)!==matching[0].sourceSha256)throw Error('Authored source digest mismatch');
 const folder=join(out,material,id);mkdirSync(folder,{recursive:true});
 const options=['--size','1024','--output','baseColor,normal,roughness,height'];
 const run=args=>JSON.parse(execFileSync(cli,args,{cwd:root,encoding:'utf8',maxBuffer:8*1024*1024}));
 const validation=run(['validate',sourcePath,'--json']);
 const inspection=run(['inspect',sourcePath,'--plan','--json',...options]);
 const render=run(['render',sourcePath,'--out',folder,'--json',...policy,...options]);
 const channelPlans={};
 for(const channel of criteria.channels)channelPlans[channel]=run(['inspect',sourcePath,'--plan','--json','--size','1024','--output',channel]).plan;
 writeFileSync(join(folder,'native.json'),JSON.stringify({validation,inspection,render},null,2));
 manifest.cases.push({material,id,sourceBase64:source.toString('base64'),sourceSha256:sha(source),overrides:{},plan:inspection.plan,channelPlans,
  nativePixels:Object.fromEntries(render.outputs.map(o=>[o.channel,sha(readFileSync(o.path))]))});
 console.log(material,id,render.planHash);
}
manifest.completedAt=new Date().toISOString();
writeFileSync(join(out,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
