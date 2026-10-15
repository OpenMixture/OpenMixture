import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { writeFile } from 'node:fs/promises';
import { setTimeout } from 'node:timers/promises';

const version = process.env.RELEASE_VERSION;
let lastError;
for (let attempt = 0; attempt < 12; attempt++) {
  try {
    const response = await fetch(`https://registry.npmjs.org/@openmixture%2fruntime/${encodeURIComponent(version)}`);
    assert.ok(response.ok, `Registry HTTP ${response.status}`);
    const metadata = await response.json();
    assert.equal(metadata.name, '@openmixture/runtime');
    assert.equal(metadata.version, version);
    const url = new URL(metadata.dist.tarball);
    assert.equal(url.origin, 'https://registry.npmjs.org');
    const download = await fetch(url);
    assert.ok(download.ok);
    const sha256 = createHash('sha256').update(Buffer.from(await download.arrayBuffer())).digest('hex');
    assert.equal(sha256, process.env.RELEASE_SHA256, 'Published bytes differ from qualified archive');
    assert.ok(metadata.dist.attestations?.url, 'Missing npm provenance attestations');
    const tagsResponse = await fetch('https://registry.npmjs.org/-/package/@openmixture%2fruntime/dist-tags');
    assert.ok(tagsResponse.ok);
    const tags = await tagsResponse.json();
    assert.equal(tags[process.env.RELEASE_TAG], version);
    await writeFile('tmp/npm-release/registry.json', JSON.stringify({ version, sha256,
      dist: metadata.dist, tags, verifiedAt: new Date().toISOString() }, null, 2) + '\n');
    lastError = undefined;
    break;
  } catch (error) {
    lastError = error;
    if (attempt < 11) await setTimeout(10000);
  }
}
if (lastError) throw lastError;
