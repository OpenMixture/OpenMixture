import test from 'node:test';
import assert from 'node:assert/strict';
import { assertOrdinaryLaunch, assertFailure } from './default-browser.mjs';

const launch = {
  freshProfile: true, executable: 'C:\\Chrome\\chrome.exe', executableSha256: 'a'.repeat(64),
  arguments: ['--user-data-dir="C:\\fresh-profile"', '--remote-debugging-port=9334', 'about:blank'],
  observedCommandLine: '"C:\\Chrome\\chrome.exe" --user-data-dir="C:\\fresh-profile" --remote-debugging-port=9334 about:blank',
  endpoint: 'http://127.0.0.1:9334',
};

test('ordinary-profile evidence rejects reused profiles and GPU/headless/security overrides', () => {
  assertOrdinaryLaunch(launch);
  assertOrdinaryLaunch({ ...launch, observedCommandLine: `${launch.observedCommandLine} ` });
  assert.throws(() => assertOrdinaryLaunch({ ...launch, freshProfile: false }));
  for (const flag of ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--use-angle=swiftshader', '--disable-gpu', '--headless', '--no-sandbox']) {
    assert.throws(() => assertOrdinaryLaunch({ ...launch, arguments: [...launch.arguments, flag] }));
    assert.throws(() => assertOrdinaryLaunch({ ...launch, observedCommandLine: `${launch.observedCommandLine} ${flag}` }));
  }
  assert.throws(() => assertOrdinaryLaunch({ ...launch, endpoint: 'http://127.0.0.1:9335' }));
});

test('negative evidence requires the expected structured failure and continued CPU validation', () => {
  const failure = { code: 'MIX_GPU_ADAPTER_UNAVAILABLE', operation: 'createGpu', message: 'No adapter', cpuStillWorks: true,
    diagnostics: [{ stage: 'gpuAdapter', suggestion: 'Use a supported adapter' }] };
  assertFailure(failure, failure.code, 'gpuAdapter');
  assert.throws(() => assertFailure({ ...failure, code: 'unknown' }, failure.code, 'gpuAdapter'));
  assert.throws(() => assertFailure({ ...failure, cpuStillWorks: false }, failure.code, 'gpuAdapter'));
  assert.throws(() => assertFailure({ ...failure, diagnostics: [] }, failure.code, 'gpuAdapter'));
});
