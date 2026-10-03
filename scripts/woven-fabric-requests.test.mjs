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
  assert.equal(controls.relief, .025); assert.equal(controls.crown, 0); assert.equal(controls.underRatio, .25);
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

import { createHash } from 'node:crypto';
test('recipe revision 2 changes only the two named defaults; all frozen gates and explicit case overrides remain identical', async () => {
  const bytes = name => readFile(new URL('../fixtures/materials/woven-fabric/'+name, import.meta.url));
  const json = async name => JSON.parse(await bytes(name));
  assert.equal(createHash('sha256').update(await bytes('qualification-plan-v1.json')).digest('hex'), 'fc6bda606e6c9c41f9a5e34a7afff1ba27e548886f2e236d8d06df79f8b1217d', 'revision 1 plan bytes and historical receipts are immutable');
  const original = await json('material-v1.mix'), candidate = await json('material.mix');
  const weave = candidate.nodes.filter(n=>n.type==='weave-pattern');
  assert.equal(weave.length,3);
  for (const n of weave) { assert.equal(n.parameters.underRatio,.25); assert.equal(n.parameters.crown,0); n.parameters.underRatio=.5; n.parameters.crown=.5; }
  assert.deepEqual(candidate,original,'only six literal defaults on the same three instances may change; topology, colors, relief, normal strength, seeds and node versions stay identical');
  const current = await json('qualification-plan.json'), previous = await json('qualification-plan-v1.json');
  assert.equal(current.recipeRevision,2); assert.equal(current.previousPlan,'qualification-plan-v1.json');
  assert.deepEqual([current.defaults.underRatio,current.defaults.crown],[.25,0]);
  current.recipeRevision=1; delete current.previousPlan; current.defaults.underRatio=.5; current.defaults.crown=.5;
  assert.deepEqual(current,previous,'no threshold, range, size, case, stress setting, budget, timing target, acceptance flag or historical receipt may change');
  const spec = await json('graph-proposal.json'), oldSpec = await json('graph-proposal-v1.json');
  assert.deepEqual([spec.proposedDefaults.underRatio,spec.proposedDefaults.crown],[.25,0]);
  spec.proposedDefaults.underRatio=.5; spec.proposedDefaults.crown=.5; assert.deepEqual(spec,oldSpec);
  for (const row of wovenFabricMatrix()) {
    const explicit=previous.cases.find(c=>c.id===row.preset).controls;
    assert.equal(row.controls.underRatio,explicit.underRatio ?? .25); assert.equal(row.controls.crown,explicit.crown ?? 0);
  }
});
