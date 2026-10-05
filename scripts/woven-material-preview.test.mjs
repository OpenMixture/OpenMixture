import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {wovenHtml,replaceOnce,hash,validateReceipts} from './woven-material-preview.mjs';
import {wovenFabricMatrix,wovenFabricStress} from './woven-fabric-requests.mjs';
const root=new URL('../',import.meta.url);
const pins={
  "scripts/brick-material-preview.mjs": "1f76f4d7a62bc03688052c2a40cb05b8ea240b9d44e4f295634e2e764d2bdd4b",
  "scripts/brick-material-preview.html": "7d83de65baeaac25a7296f28349ac3a2423b91457b0b7eebe08dc7f1befe0971",
  "scripts/painted-material-preview.mjs": "934ca063cfb01748141b3c6f3fd412a13363b4f3d72044ecf3004bdd553cbb5e",
  "scripts/painted-material-preview.html": "2be310679a9a3abe6c57aaea71f5b078f4225647739623a33c42c8cec8dc9499",
  "fixtures/materials/woven-fabric/pbr-review-plan.json": "3a3e4e3980aa6d4bdc3ab872ac0dbed2e71af361687029a555fbf8655fc729b4"
};
test('frozen review plan and historical brick/painted runners/renderers remain unchanged',async()=>{
 for(const [path,expected]of Object.entries(pins))assert.equal(hash((await readFile(new URL(path,root),'utf8')).replaceAll('\r\n','\n')),expected,path);
});
test('bounded dielectric reuse preserves light/tone mapping and excludes sanity from sheets',async()=>{
 const source=await readFile(new URL('scripts/brick-material-preview.html',root),'utf8');const result=wovenHtml(source);
 const light=s=>s.slice(s.indexOf('fn light('),s.indexOf('@fragment'));
 assert.equal(light(result),light(source));assert.ok(result.includes('2.51*hdr'));assert.ok(result.includes('settings.zoom'));assert.ok(result.includes('roughness-probe'));assert.ok(result.includes('metallic'));assert.ok(result.includes("document.querySelector('main').append(figure)"));assert.ok(result.includes('input parity'));
 assert.throws(()=>replaceOnce('aa','a','b'));assert.throws(()=>wovenHtml(source.replace('const owned=[];','')));
});
function receipts(){const manifest={rows:wovenFabricMatrix(),stress:wovenFabricStress(),workingTreeStatus:'',sourceRevision:'frozen'};const rows=[...manifest.rows,...manifest.stress].map(r=>({id:r.id,size:r.request.size,overrides:r.request.overrides,planHash:'plan',repeatExact:true,packageExact:true,slicedExact:true,ownedAfterDestroy:true,comparisons:Array.from({length:5},()=>({maxComponentDelta:1}))}));const build={buildId:'build',engineDirty:false,engineRevision:'frozen'};return [{ok:true,completed:true,revision:'frozen',build},{ok:true,completed:true,browserCompared:true,requestManifest:manifest,rows,timing:Array.from({length:4},()=>({budgetPassed:true}))},{ok:true,completed:true,manifest,build,packageSha256:'package',rows:structuredClone(rows)},{ok:true,mode:'candidate',consumerDirty:false,consumerRevision:'frozen',build,wovenPackageSha256:'package',wovenRequestsSha256:hash(JSON.stringify(manifest,null,2)+'\n')},manifest];}
test('receipt chain rejects dirty, foreign plans, mismatched build/package and failed parity',()=>{
 validateReceipts(...receipts());
 for(const mutate of [r=>r[3].consumerDirty=true,r=>r[3].build={},r=>r[2].packageSha256='other',r=>r[2].rows[0].planHash='other',r=>r[1].rows[0].comparisons[0].maxComponentDelta=2,r=>r[1].timing[0].budgetPassed=false,r=>r[0].completed=false]){const r=receipts();mutate(r);assert.throws(()=>validateReceipts(...r));}
});
