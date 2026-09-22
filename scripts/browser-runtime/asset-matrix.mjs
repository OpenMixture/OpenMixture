import assert from 'node:assert/strict';
export function verifyAssetMatrix(native,browser,receipt){
  assert.equal(native.ok,true);assert.equal(native.ownedAfterDestroy,true);assert.equal(browser.ownedAfterDestroy,true);
  assert.equal(native.cases.length,8);assert.equal(browser.rows.length,8);
  const keys=new Set(['1024x1024','65x3'].flatMap(size=>[0,0.25,0.5,1].map(weight=>`${size}:${weight}`)));
  const indexed=new Map();
  for(const row of browser.rows){const key=`${row.size.join('x')}:${row.weight}`;assert.ok(keys.has(key));assert.ok(!indexed.has(key));indexed.set(key,row);}
  for(const row of native.cases){
    const key=`${row.size.join('x')}:${row.weight}`;assert.ok(keys.delete(key),'duplicate or unexpected Native case');
    const other=indexed.get(key);assert.ok(other);assert.equal(row.planHash,other.planHash);
    for(const r of [row,other]){assert.equal(r.exactLoosePixels,true);assert.equal(r.repeatedLoads,2);assert.equal(r.packageSha256,receipt.assetFixtures[r.size.join('x')+'.mixpack']);}
    assert.equal(Number(row.execution.allocations.liveBytes),0);assert.equal(Number(other.allocations.liveBytes),0);
    assert.deepEqual(row.browserComparison.map(c=>c.channel).sort(),['height','normal']);
    for(const c of row.browserComparison){assert.equal(c.limit,1);assert.ok(Number.isInteger(c.maxComponentDelta)&&c.maxComponentDelta>=0&&c.maxComponentDelta<=1);}
  }
  assert.equal(keys.size,0);
}
