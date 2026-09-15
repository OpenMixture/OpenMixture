// Produce an explicit native reference bundle; never install engine code in a product.
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { createHash } from 'node:crypto';
const root = resolve(import.meta.dirname, '../..');
const [destination, runtimeRevision] = process.argv.slice(2);
if (!destination || !/^[a-f0-9]{40}$/.test(runtimeRevision ?? '')) throw new Error('Usage: node scripts/browser-runtime/prepare-materials.mjs <new-directory> <runtime-engine-revision>');
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
const manifest={schemaVersion:1,engineRevision:git('rev-parse','HEAD'),engineDirty:Boolean(git('status','--porcelain')),runtimeRevision,
 startedAt:new Date().toISOString(),nativeBinarySha256:sha(readFileSync(cli)),cases:[]};
for(const material of ['glazed-ceramic','leather','wood']){
 const fixture=join(root,'fixtures/materials',material), source=readFileSync(join(fixture,'material.mix'));
 const acceptance=JSON.parse(readFileSync(join(fixture,'acceptance.json')));
 for(const item of acceptance.cases){
  const overrides=item.variant?JSON.parse(readFileSync(join(fixture,'variants',`${item.variant}.json`))).overrides:{};
  const folder=join(out,material,item.id);mkdirSync(folder,{recursive:true});
  const options=['--size','1024','--output','baseColor,normal,roughness,height'];
  for(const [id,value] of Object.entries(overrides))options.push('--set',`${id}=${JSON.stringify(value)}`);
  const run=(args)=>JSON.parse(execFileSync(cli,args,{cwd:root,encoding:'utf8',maxBuffer:8*1024*1024}));
  const inspection=run(['inspect',join(fixture,'material.mix'),'--plan','--json',...options]);
  const render=run(['render',join(fixture,'material.mix'),'--out',folder,'--json',...policy,...options]);
  if(process.env.MIXTURE_GPU_EXPECT_ADAPTER && !JSON.stringify(render.context.adapter).includes(process.env.MIXTURE_GPU_EXPECT_ADAPTER))throw new Error('Unexpected native adapter');
  writeFileSync(join(folder,'native.json'),JSON.stringify({inspection,render},null,2));
  manifest.cases.push({material,id:item.id,sourceBase64:source.toString('base64'),sourceSha256:sha(source),overrides,
   acceptanceSha256:sha(readFileSync(join(fixture,'acceptance.json'))),plan:inspection.plan,
   nativePixels:Object.fromEntries(render.outputs.map(o=>[o.channel,sha(readFileSync(o.path))]))});
  console.log(material,item.id,render.planHash);
 }
}
manifest.completedAt=new Date().toISOString();
writeFileSync(join(out,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
