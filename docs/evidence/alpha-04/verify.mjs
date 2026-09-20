// Verify retained bytes and restore the exact saved-file comparison inputs.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, mkdir } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('.', import.meta.url));
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const summary = await json(join(root, 'summary.json'));
for (const [name, expected] of Object.entries(summary.files)) {
  assert.ok(!name.startsWith('/') && !name.split('/').includes('..'));
  const bytes = await readFile(join(root, name));
  assert.equal(bytes.length, expected.bytes, name);
  assert.equal(hash(bytes), expected.sha256, name);
}
const producer = await json(join(root, 'producer.json'));
assert.equal(producer.engineDirty, false);
assert.equal(hash(await readFile(join(root, 'openmixture-runtime-0.1.0-alpha.0.tgz'))), producer.sha256);
const authored = await json(join(root, 'authored.json'));
for (const environment of ['windows', 'linux']) {
  const player = await json(join(root, `${environment}-player.json`));
  assert.equal(player.archiveSha256, producer.sha256);
  assert.equal(player.build.buildId, producer.buildId);
  assert.equal(player.build.engineRevision, producer.engineRevision);
  assert.equal(player.productRevision, summary.studioRevision);
  assert.equal(player.authoredProductRevision, authored.productRevision);
  assert.equal(player.clean, true);
  assert.equal(player.lockSha256, hash(await readFile(join(root, `${environment}-lock.json`))));
  const comparison = await json(join(root, `${environment}-comparison.json`));
  assert.equal(comparison.ok, true);
  assert.equal(comparison.schemaVersion, 2);
  assert.equal(comparison.cases.length, 7);
}
assert.deepEqual(await json(join(root, 'windows-lock.json')), await json(join(root, 'linux-lock.json')));
if (process.argv[2]) {
  const target = resolve(process.argv[2]);
  await mkdir(target); // Require a fresh location; never overwrite prior evidence.
  execFileSync('tar', ['-xzf', join(root, 'saved-file-bundles.tar.gz'), '-C', target]);
  const index = await json(join(root, 'evidence-index.json'));
  for (const [dataset, entries] of Object.entries(index.datasets)) {
    for (const entry of entries) {
      const bytes = await readFile(join(target, dataset, entry.file));
      assert.equal(bytes.length, entry.bytes, entry.file);
      assert.equal(hash(bytes), entry.sha256, entry.file);
    }
  }
  console.log(`Restored and verified ${target}`);
}
console.log('ALPHA-04 retained archive, identities and evidence hashes verified.');
