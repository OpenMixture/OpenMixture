import test from 'node:test';
import assert from 'node:assert/strict';
import { wovenFabricRequest, wovenFabricMatrix, wovenFabricStress, writeWovenFabricRequests } from './woven-fabric-requests.mjs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
test('frozen cases, sizes, defaults and three identical geometry instances', () => {
  const rows = wovenFabricMatrix(); assert.equal(rows.length, 48); assert.equal(wovenFabricStress().length, 3);
  assert.equal(new Set(rows.map(r => r.id)).size, 48);
  const { controls, request } = wovenFabricRequest();
  assert.equal(controls.relief, .025); assert.equal(controls.crown, .5); assert.equal(controls.underRatio, .5);
  for (const row of [...rows, ...wovenFabricStress()]) for (const key of ['warpCount','weftCount','warpWidth','weftWidth','bevel','crown','underRatio']) {
    for (const mode of ['height','coverage','warpshare']) assert.equal(row.request.overrides[`${mode}_${key}`], row.controls[key]);
  }
  assert.equal(request.overrides.roughnessMin, .8 * .84);
  assert.deepEqual(wovenFabricRequest({detailAmount:0}).request.overrides.warpDark, controls.warpColor);
  for (const preset of ['combined-low','combined-high','flat','neutral-normal','constant-low','constant-high']) assert.equal(rows.filter(r=>r.preset===preset).length,4);
});
test('invalid caller controls fail before Core; counts never rounded', () => {
  for (const controls of [null,[],{oops:1},{warpCount:5},{weftCount:33},{crown:1.1},{relief:-1},{warpSeed:1.5},{weftSeed:4294967296},{warpWidth:'0.7'},{detailAmount:NaN},{warpColor:[0,0,0,0]},{backingColor:[1,1,1]}]) assert.throws(()=>wovenFabricRequest(controls));
  for (const size of [[0,1],[2049,1],[1],[1.5,2]]) assert.throws(()=>wovenFabricRequest({},size));
});
test('requests do not alias caller arrays; manifests bind exact bytes and refuse reuse', async () => {
  const input={warpColor:[.1,.2,.3,1]}; const r=wovenFabricRequest(input); input.warpColor[0]=.9; assert.equal(r.controls.warpColor[0],.1);
  const dir=await mkdtemp(join(tmpdir(),'woven-requests-'));
  try { const out=join(dir,'fresh'); const m=await writeWovenFabricRequests(out); for(const k of ['sourceSha256','qualificationPlanSha256','builderSha256'])assert.match(m[k],/^[a-f0-9]{64}$/);assert.equal(JSON.parse(await readFile(join(out,'requests.json'))).rows.length,48); await assert.rejects(writeWovenFabricRequests(out)); } finally { await rm(dir,{recursive:true,force:true}); }
});
