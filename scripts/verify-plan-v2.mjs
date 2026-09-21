// Independently verify the finite v1 -> v2 material-plan migration, never pixels.
// Run after cargo build --locked -p mixture-cli. Optional output must not exist.
import { readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const hash = bytes => 'sha256:' + createHash('sha256').update(bytes).digest('hex');
const compact = text => text.replace(/("(?:\\.|[^"\\])*")|\s+/g, (match, string) => string ?? '');
const cli = resolve(process.env.MIXTURE_CLI ?? `target/debug/mixture${process.platform === 'win32' ? '.exe' : ''}`);
const migrations = [];
for (const material of ['glazed-ceramic', 'leather', 'wood']) {
  const directory = `fixtures/materials/${material}`;
  const acceptance = JSON.parse(readFileSync(`${directory}/acceptance.json`));
  const baseline = JSON.parse(readFileSync(`${directory}/expected/manifest.json`));
  for (const item of acceptance.cases) {
    const overrides = item.variant ? JSON.parse(readFileSync(`${directory}/variants/${item.variant}.json`)).overrides : {};
    const args = ['inspect', `${directory}/material.mix`, '--plan', '--json', '--size', String(acceptance.size), '--output', 'baseColor,normal,roughness,height'];
    for (const [id, value] of Object.entries(overrides)) args.push('--set', `${id}=${JSON.stringify(value)}`);
    const raw = compact(execFileSync(cli, args, { encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 }));
    const report = JSON.parse(raw);
    assert.equal(report.schemaVersion, 2);
    const plan = report.plan;
    assert.equal(plan.version, 2);
    assert.deepEqual(plan.imageResources, []);
    // The envelope contains the directly serialized typed plan. Slice the exact
    // numeric tokens rather than reserializing JS numbers and losing f32 .0.
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
    assert.equal(hash('mixture-render-plan-v2\0' + body), plan.hash);
    const resourceFields = '"resourceCount":0,"resourceUploadBytes":0,"resourceTextureBytes":0,"resourceStagingBytes":0,';
    assert.equal(body.split(resourceFields).length, 2);
    assert.ok(body.endsWith(',"imageResources":[]}'));
    const oldBody = body.replace('"version":2', '"version":1').replace(resourceFields, '').replace(/,"imageResources":\[\]}$/, '}');
    const oldHash = hash('mixture-render-plan-v1\0' + oldBody);
    assert.equal(oldHash, baseline.planHashes[item.id], `v1 semantics changed: ${material}/${item.id}`);
    migrations.push({ material, case: item.id, v1: oldHash, v2: plan.hash });
  }
}
const record = { schemaVersion: 1, kind: 'exact-resource-free-plan-v1-to-v2', migrations };
if (process.argv[2]) writeFileSync(process.argv[2], JSON.stringify(record, null, 2) + '\n', { flag: 'wx' });
else assert.deepEqual(record, JSON.parse(readFileSync('docs/plan-v2-migration.json')));
process.stdout.write(`Verified ${migrations.length} unchanged v1 material semantics and exact v2 hashes; no pixel baselines changed.\n`);
