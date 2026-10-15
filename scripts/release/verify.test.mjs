import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { validateRequest, validateRun, validateJobs, validateArchive } from './verify.mjs';

const bytes = Buffer.from('qualified archive');
const request = { ref: 'refs/heads/main', sha: 'a'.repeat(40), version: '0.11.0-alpha.0',
  digest: createHash('sha256').update(bytes).digest('hex'), tag: 'alpha' };
test('release request rejects alternate refs, injected input and prerelease latest', () => {
  validateRequest(request);
  for (const patch of [{ ref: 'refs/heads/feature' }, { version: '1.0.0; echo unsafe' },
    { digest: '../archive' }, { tag: 'latest' }, { sha: 'main' }]) {
    assert.throws(() => validateRequest({ ...request, ...patch }));
  }
  validateRequest({ ...request, version: '1.0.0', tag: 'latest' });
});
test('qualification must be successful main push in the owning workflow and repository', () => {
  const run = { repository: { full_name: 'OpenMixture/OpenMixture' },
    head_repository: { full_name: 'OpenMixture/OpenMixture' }, head_sha: request.sha,
    head_branch: 'main', event: 'push', path: '.github/workflows/ci.yml', status: 'completed', conclusion: 'success' };
  validateRun(run, request.sha, 'ci.yml');
  for (const patch of [{ event: 'pull_request' }, { head_sha: 'b'.repeat(40) },
    { conclusion: 'failure' }, { path: '.github/workflows/other.yml' },
    { head_repository: { full_name: 'someone/fork' } }, { status: 'in_progress' }]) {
    assert.throws(() => validateRun({ ...run, ...patch }, request.sha, 'ci.yml'));
  }
});
test('skipped or missing jobs do not qualify a release', () => {
  validateJobs([{ name: 'gate', status: 'completed', conclusion: 'success' }], ['gate']);
  assert.throws(() => validateJobs([], ['gate']));
  assert.throws(() => validateJobs([{ name: 'gate', status: 'completed', conclusion: 'skipped' }], ['gate']));
});
test('archive bytes and embedded identity must agree with the frozen receipt', () => {
  const receipt = { engineRevision: request.sha, engineDirty: false, runtimeVersion: request.version,
    engineVersion: '0.11.0', apiSchemaVersion: 3, buildId: 'sha256:build', sha256: request.digest };
  const manifest = { name: '@openmixture/runtime', version: request.version,
    repository: { url: 'https://github.com/OpenMixture/OpenMixture.git' } };
  validateArchive(receipt, manifest, receipt, request, bytes);
  assert.throws(() => validateArchive(receipt, manifest, receipt, request, Buffer.from('rebuilt')));
  assert.throws(() => validateArchive({ ...receipt, engineDirty: true }, manifest, receipt, request, bytes));
  assert.throws(() => validateArchive(receipt, manifest, { ...receipt, buildId: 'different' }, request, bytes));
  assert.throws(() => validateArchive(receipt, { ...manifest, name: '@other/runtime' }, receipt, request, bytes));
});
