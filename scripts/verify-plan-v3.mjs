// Prove the finite v2 -> v3 allocation migration without changing pixels or old records.
import { readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const hash = bytes => 'sha256:' + createHash('sha256').update(bytes).digest('hex');
const compact = text => text.replace(/("(?:\\.|[^"\\])*")|\s+/g, (match, string) => string ?? '');
const cli = resolve(process.env.MIXTURE_CLI ?? `target/debug/mixture${process.platform === 'win32' ? '.exe' : ''}`);
const before = JSON.parse(readFileSync('docs/plan-v2-migration.json'));
const migrations = [];
for (const material of ['glazed-ceramic', 'leather', 'wood']) {
  const directory = `fixtures/materials/${material}`;
  const acceptance = JSON.parse(readFileSync(`${directory}/acceptance.json`));
  for (const item of acceptance.cases) {
    const overrides = item.variant ? JSON.parse(readFileSync(`${directory}/variants/${item.variant}.json`)).overrides : {};
    const args = ['inspect', `${directory}/material.mix`, '--plan', '--json', '--size', String(acceptance.size), '--output', 'baseColor,normal,roughness,height'];
    for (const [id, value] of Object.entries(overrides)) args.push('--set', `${id}=${JSON.stringify(value)}`);
    const raw = compact(execFileSync(cli, args, { encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 }));
    const report = JSON.parse(raw), plan = report.plan;
    assert.equal(report.schemaVersion, 3);
    assert.equal(plan.version, 3);
    assert.deepEqual(plan.imageResources, []);
    // Preserve Rust's exact float tokens when recovering the previous body.
    const start = raw.indexOf('"plan":') + 7;
    let end = start, depth = 0, quoted = false, escaped = false;
    for (; end < raw.length; end++) {
      const c = raw[end];
      if (quoted) { if (escaped) escaped = false; else if (c === '\\') escaped = true; else if (c === '"') quoted = false; }
      else if (c === '"') quoted = true;
      else if (c === '{') depth++;
      else if (c === '}' && --depth === 0) { end++; break; }
    }
    let body = raw.slice(start, end);
    const suffix = `,"hash":"${plan.hash}"}`;
    assert.ok(body.endsWith(suffix));
    body = body.slice(0, -suffix.length) + '}';
    assert.equal(hash('mixture-render-plan-v3\0' + body), plan.hash);
    const estimates = { ...plan.estimates };
    assert.ok(Object.values(estimates).every(Number.isSafeInteger));
    const delta = estimates.logicalTextureBytes - estimates.textureBytes;
    assert.ok(delta >= 0);
    estimates.textureBytes = estimates.logicalTextureBytes;
    estimates.peakBytes += delta;
    estimates.cumulativeBytes += delta;
    delete estimates.textureCount;
    delete estimates.logicalTextureBytes;
    const allocation = `,"allocation":${JSON.stringify(plan.allocation)}`;
    assert.equal(body.split(allocation).length, 2);
    const serializedEstimates = JSON.stringify(plan.estimates);
    assert.equal(body.split(serializedEstimates).length, 2);
    const oldBody = body.replace('"version":3', '"version":2').replace(allocation, '')
      .replace(serializedEstimates, JSON.stringify(estimates));
    const oldHash = hash('mixture-render-plan-v2\0' + oldBody);
    const previous = before.migrations.find(entry => entry.material === material && entry.case === item.id);
    assert.ok(previous);
    assert.equal(oldHash, previous.v2, `v2 semantics changed: ${material}/${item.id}`);
    migrations.push({ material, case: item.id, v2: oldHash, v3: plan.hash });
  }
}
const record = { schemaVersion: 1, kind: 'exact-resource-free-plan-v2-to-v3-allocation', migrations };
if (process.argv[2]) writeFileSync(process.argv[2], JSON.stringify(record, null, 2) + '\n', { flag: 'wx' });
else assert.deepEqual(record, JSON.parse(readFileSync('docs/plan-v3-migration.json')));
console.log(`Verified ${migrations.length} unchanged v2 material semantics and exact v3 hashes; no pixel baselines changed.`);
