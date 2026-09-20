// Attach to an explicitly launched ordinary desktop browser. Never launch a
// Playwright browser here: its default switches are not ordinary-profile evidence.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { assertBuild, assertComparison, hash, installed } from './candidate.mjs';
import { connectFirefox } from './firefox-transport.mjs';

const json = async path => JSON.parse((await readFile(path, 'utf8')).replace(/^\uFEFF/, ''));
const save = (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n');

export function assertOrdinaryLaunch(launch) {
  assert.equal(launch.freshProfile, true);
  assert.ok([undefined, 'chromium', 'firefox'].includes(launch.browserFamily));
  const firefox = launch.browserFamily === 'firefox';
  assert.equal(launch.arguments.length, firefox ? 4 : 3);
  if (firefox) {
    assert.equal(launch.arguments[0], '-profile');
    assert.match(launch.arguments[1], /^"[^"\r\n]+"$/);
  } else assert.match(launch.arguments[0], /^--user-data-dir="[^"\r\n]+"$/);
  const debugging = launch.arguments[firefox ? 2 : 1];
  assert.match(debugging, /^--remote-debugging-port=\d+$/);
  assert.equal(launch.arguments.at(-1), 'about:blank');
  assert.equal(launch.endpoint, firefox ? `ws://127.0.0.1:${debugging.split('=')[1]}/session` : `http://127.0.0.1:${debugging.split('=')[1]}`);
  if (launch.observedCommandLine === null && launch.launcherProcessObserved === false) {
    // A fast Edge relaunch can exit before the OS query. Never fabricate an
    // observed launcher command; require the actual direct child and CDP PID.
    assert.equal(launch.product, 'Microsoft Edge');
    assert.equal(launch.browserProcess?.parentProcessId, launch.processId);
    assert.equal(launch.browserProcess?.executable, launch.executable);
  } else {
    assert.equal(launch.observedCommandLine.trim(), `"${launch.executable}" ${launch.arguments.join(' ')}`);
  }
  assert.match(launch.executableSha256, /^[a-f0-9]{64}$/);
}

export function assertFirefoxProcess(launch, capabilities) {
  assert.equal(capabilities.browserName, 'firefox');
  assert.equal(capabilities.browserVersion, launch.version);
  const process = launch.browserProcess;
  assert.equal(process.parentProcessId, launch.processId);
  assert.equal(capabilities['moz:processID'], process.processId);
  assert.equal(capabilities['moz:headless'], false);
  assert.equal(capabilities['moz:profile'], launch.arguments[1].slice(1, -1));
  const tokens = process.commandLine.match(/"[^"]*"|[^\s]+/g).map(value => value.replace(/^"|"$/g, ''));
  assert.deepEqual(tokens, [launch.executable, ...launch.arguments.map(value => value.replace(/^"|"$/g, ''))]);
}

export function assertChromiumProcess(launch, systemInfo, processInfo) {
  const commandLine = systemInfo.commandLine.replace(' --flag-switches-begin --flag-switches-end', '').trim();
  const actual = processInfo.processInfo.filter(process => process.type === 'browser');
  assert.equal(actual.length, 1);
  if (launch.product === 'Microsoft Edge' && launch.browserProcess) {
    const child = launch.browserProcess;
    assert.equal(child.parentProcessId, launch.processId);
    assert.equal(child.executable, launch.executable);
    assert.equal(actual[0].id, child.processId);
    assert.equal(commandLine, child.commandLine.trim());
    const tokens = commandLine.match(/"[^"]*"|[^\s]+/g).map(value => value.replace(/^"|"$/g, ''));
    assert.deepEqual(tokens, [launch.executable, ...launch.arguments.slice(0, -1).map(value => value.replace(/^"|"$/g, '')),
      '--edge-skip-compat-layer-relaunch', 'about:blank']);
  } else {
    assert.equal(actual[0].id, launch.processId);
    assert.equal(commandLine, launch.observedCommandLine.trim());
  }
}

export function assertFailure(value, code, stage) {
  assert.equal(value.code, code);
  assert.equal(value.operation, 'createGpu');
  assert.ok(value.message.length > 0);
  assert.equal(value.cpuStillWorks, true);
  if (stage) {
    assert.equal(value.diagnostics[0].stage, stage);
    assert.ok(value.diagnostics[0].suggestion.length > 0);
  }
}

export async function run(product, candidateDirectory, nativeDirectory, launchFile, output) {
  await mkdir(output); // Every acceptance attempt starts with a fresh record.
  await save(join(output, 'ordinary.json'), { ok: false, status: 'incomplete' });
  const launch = await json(launchFile);
  assertOrdinaryLaunch(launch);
  await installed(product, candidateDirectory);
  const candidate = await json(join(candidateDirectory, 'candidate.json'));
  const manifestBytes = await readFile(join(nativeDirectory, 'manifest.json'));
  const manifest = JSON.parse(manifestBytes);
  assert.equal(manifest.runtimeRevision, candidate.receipt.engineRevision);
  assert.equal(manifest.cases.length, 11);
  const { chromium } = await import(pathToFileURL(join(product, 'node_modules/@playwright/test/index.mjs')));
  const browser = launch.browserFamily === 'firefox' ? await connectFirefox(launch.endpoint) : await chromium.connectOverCDP(launch.endpoint);
  const pages = [];
  const receipt = { schemaVersion: 1, archiveSha256: candidate.receipt.sha256, lockSha256: candidate.lockSha256,
    verifierSha256: hash(await readFile(import.meta.filename)),
    firefoxTransportSha256: launch.browserFamily === 'firefox' ? hash(await readFile(new URL('./firefox-transport.mjs', import.meta.url))) : undefined,
    manifestSha256: hash(manifestBytes), browser: browser.version(), startedAt: new Date().toISOString(),
    args: launch.arguments, launch, consumerRevision: candidate.consumerRevision, cases: [] };
  assert.equal(receipt.browser, launch.version);
  const errors = [];
  const newPage = async () => {
    const page = await browser.contexts()[0].newPage();
    pages.push(page);
    return page;
  };
  try {
    if (launch.browserFamily === 'firefox') {
      receipt.browserSystemInfo = browser.capabilities;
      assertFirefoxProcess(launch, browser.capabilities);
    } else {
      const system = await browser.newBrowserCDPSession();
      receipt.browserSystemInfo = await system.send('SystemInfo.getInfo');
      receipt.browserProcessInfo = await system.send('SystemInfo.getProcessInfo');
      assertChromiumProcess(launch, receipt.browserSystemInfo, receipt.browserProcessInfo);
    }
    const page = await newPage();
    page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(() => {
      if (!navigator.gpu) return; // BiDi may run preloads in the initial about:blank realm.
      const original = navigator.gpu.requestAdapter;
      navigator.gpu.requestAdapter = async function (options) {
        const adapter = await original.call(this, options);
        // Observe the exact returned adapter without changing options or the
        // object delivered to wgpu. Redacted/unavailable fields stay explicit.
        const info = adapter?.info;
        window.selectedAdapter = { options, info: info ? {
          vendor: info.vendor, architecture: info.architecture, device: info.device,
          description: info.description, isFallbackAdapter: info.isFallbackAdapter ?? adapter.isFallbackAdapter ?? null,
        } : null };
        return adapter;
      };
    });
    await page.goto('http://127.0.0.1:4173/player/tests/contract.html');
    const identity = await page.evaluate(async () => {
      window.runtime = await window.mixtureContract.loadRuntime();
      window.gpu = await window.runtime.createGpu();
      return JSON.parse(JSON.stringify({ secure: isSecureContext, build: window.runtime.getBuildInfo(), context: window.gpu.context, selectedAdapter: window.selectedAdapter },
        (_, value) => typeof value === 'bigint' ? value.toString() : value));
    });
    assert.equal(identity.secure, true);
    assertBuild(identity.build, candidate.receipt);
    Object.assign(receipt, identity);
    for (const item of manifest.cases) {
      assert.match(item.material, /^[a-z-]+$/);
      assert.match(item.id, /^[a-z-]+$/);
      assert.equal(hash(Buffer.from(item.sourceBase64, 'base64')), item.sourceSha256);
      const result = await page.evaluate(async item => {
        const source = Uint8Array.from(atob(item.sourceBase64), c => c.charCodeAt(0));
        const result = await window.gpu.render(source, { size: [1024, 1024], channels: ['baseColor', 'normal', 'roughness', 'height'], overrides: item.overrides });
        if (result.report.allocations.liveBytes !== 0n) throw new Error('Live allocation after readback');
        if (!window.retained) {
          window.retained = result.channels[0].pixels;
          window.retainedHash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', window.retained)));
        }
        const channels = [];
        for (const channel of result.channels) {
          if (!(channel.pixels instanceof Uint8Array) || channel.pixels.length !== 1024 * 1024 * 4) throw new Error('Invalid owned bytes');
          const bytes = new Uint8Array(await (await window.mixtureContract.encodePng(channel)).arrayBuffer());
          let text = '';
          for (let i = 0; i < bytes.length; i += 8192) text += String.fromCharCode(...bytes.subarray(i, i + 8192));
          channels.push({ channel: channel.channel, encoding: channel.encoding, base64: btoa(text) });
        }
        return JSON.parse(JSON.stringify({ plan: result.plan, report: result.report, channels }, (_, v) => typeof v === 'bigint' ? v.toString() : v));
      }, item);
      assert.equal(result.plan.hash, item.plan.hash);
      const folder = join(output, item.material, item.id);
      await mkdir(folder, { recursive: true });
      for (const channel of result.channels) {
        assert.ok(['baseColor', 'normal', 'roughness', 'height'].includes(channel.channel));
        await writeFile(join(folder, `${channel.channel}.png`), Buffer.from(channel.base64, 'base64'));
        delete channel.base64;
      }
      await save(join(folder, 'result.json'), result);
      receipt.cases.push({ material: item.material, id: item.id, planHash: result.plan.hash });
      console.log(item.material, item.id, 'rendered with ordinary browser settings');
    }
    receipt.lifecycle = await page.evaluate(async source64 => {
      const first = window.gpu.destroy(), second = window.gpu.destroy();
      const samePromise = first === second;
      await first;
      const retainedHash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', window.retained)));
      let rejected;
      try { await window.gpu.render(atob(source64)); } catch (error) { rejected = error.code; }
      // Explicitly acquire another context and destroy while accepted work is pending.
      const next = await window.runtime.createGpu();
      const accepted = next.render(atob(source64), { size: [65, 3] });
      const closing = next.destroy();
      const result = await accepted;
      await closing;
      return { samePromise, retainedBefore: window.retainedHash, retainedAfter: retainedHash,
        rejected, acceptedWorkCompleted: result.channels[0].pixels.length === 65 * 3 * 4,
        retainedBytes: window.retained.length };
    }, manifest.cases[0].sourceBase64);
    assert.deepEqual(receipt.lifecycle.retainedAfter, receipt.lifecycle.retainedBefore);
    assert.equal(receipt.lifecycle.samePromise, true);
    assert.equal(receipt.lifecycle.acceptedWorkCompleted, true);
    assert.equal(receipt.lifecycle.rejected, 'MIX_BROWSER_RUNTIME_DESTROYED');
    receipt.failures = [];
    for (const mode of ['unavailable', 'adapter', 'device']) {
      const failurePage = await newPage();
      await failurePage.addInitScript(mode => {
        if (mode === 'unavailable') Object.defineProperty(navigator, 'gpu', { value: undefined });
        if (mode === 'adapter') navigator.gpu.requestAdapter = async () => null;
        if (mode === 'device') GPUAdapter.prototype.requestDevice = async () => { throw new DOMException('ALPHA-03 injected device denial', 'NotSupportedError'); };
      }, mode);
      await failurePage.goto('http://127.0.0.1:4173/player/tests/contract.html');
      const failure = await failurePage.evaluate(async source64 => {
        const runtime = await window.mixtureContract.loadRuntime();
        try { await runtime.createGpu(); return { unexpectedSuccess: true }; }
        catch (error) {
          return JSON.parse(JSON.stringify({ code: error.code, operation: error.operation, message: error.message,
            diagnostics: error.diagnostics, browserFailure: error.browserFailure, evidence: error.evidence,
            cpuStillWorks: runtime.validate(atob(source64)).ok }, (_, v) => typeof v === 'bigint' ? v.toString() : v));
        }
      }, manifest.cases[0].sourceBase64);
      const expected = { unavailable: ['MIX_BROWSER_WEBGPU_UNAVAILABLE'], adapter: ['MIX_GPU_ADAPTER_UNAVAILABLE', 'gpuAdapter'], device: ['MIX_GPU_DEVICE_REQUEST_FAILED', 'gpuDevice'] }[mode];
      assertFailure(failure, ...expected);
      receipt.failures.push({ mode, injection: 'page-local synthetic failure; not a naturally unsupported host', ...failure });
      await failurePage.close();
    }
    assert.deepEqual(errors, []);
    await save(join(output, 'receipt.json'), receipt);
    // Reuse the existing frozen native/browser quality gate, without baseline changes.
    execFileSync(process.env.CARGO ?? 'cargo', ['xtask', 'browser-material-check', nativeDirectory, output], {
      cwd: resolve(import.meta.dirname, '../..'), stdio: 'inherit',
    });
    const comparison = await json(join(output, 'comparison.json'));
    assertComparison(comparison);
    await save(join(output, 'ordinary.json'), { schemaVersion: 1, ok: true,
      receiptSha256: hash(await readFile(join(output, 'receipt.json'))), comparisonSha256: hash(await readFile(join(output, 'comparison.json'))),
      verifierSha256: hash(await readFile(import.meta.filename)), finishedAt: new Date().toISOString(),
      scope: 'Recorded desktop browser/Windows fresh profile only; failure paths are separately injected; production Player verification is separate.' });
  } catch (error) {
    await save(join(output, 'failure.json'), { message: error.message, stack: error.stack, receipt });
    throw error;
  } finally {
    for (const page of pages) if (!page.isClosed()) await page.close();
    await browser.close(); // Disconnect; the explicitly owned browser remains available for the production check.
  }
}

export async function production(product, candidateDirectory, launchFile, output) {
  await mkdir(output);
  await save(join(output, 'receipt.json'), { ok: false, status: 'incomplete' });
  const launch = await json(launchFile);
  assertOrdinaryLaunch(launch);
  assert.notEqual(launch.browserFamily, 'firefox', 'The production download verifier currently requires Chromium; use run for Firefox runtime qualification');
  await installed(product, candidateDirectory);
  const candidate = await json(join(candidateDirectory, 'candidate.json'));
  const { chromium, expect } = await import(pathToFileURL(join(product, 'node_modules/@playwright/test/index.mjs')));
  const { decodePng } = await import(pathToFileURL(join(product, 'tests/helpers/png.ts')));
  const browser = await chromium.connectOverCDP(launch.endpoint);
  assert.equal(browser.version(), launch.version);
  const system = await browser.newBrowserCDPSession();
  const info = await system.send('SystemInfo.getInfo');
    assertChromiumProcess(launch, info, await system.send('SystemInfo.getProcessInfo'));
  const page = await browser.contexts()[0].newPage();
  try {
    await page.goto('http://127.0.0.1:4173/player/');
    await expect(page.locator('#status')).toHaveText('Checker loaded. Initialize WebGPU to render.');
    await page.locator('#width').fill('65');
    await page.locator('#height').fill('3');
    await page.locator('#initialize').click();
    await expect(page.locator('#status')).toHaveText('WebGPU ready. Render when ready.');
    await page.locator('#render').click();
    await expect(page.locator('#status')).toHaveText('Render complete.');
    const downloadPromise = page.waitForEvent('download');
    await page.locator('#download').click();
    const download = await downloadPromise;
    const bytes = await readFile(await download.path());
    const png = decodePng(bytes);
    assert.deepEqual([png.width, png.height, png.gamma, png.srgb], [65, 3, 45455, 0]);
    const row = '00000000011111111000000001111111100000000111111110000000011111111';
    const expected = [...row + row + [...row].map(v => v === '0' ? '1' : '0').join('')]
      .flatMap(v => v === '0' ? [0, 0, 0, 255] : [255, 255, 255, 255]);
    assert.deepEqual([...png.pixels], expected);
    await writeFile(join(output, 'checker.png'), bytes);
    await page.screenshot({ path: join(output, 'player.png'), fullPage: true });
    await page.locator('#dispose').click();
    await expect(page.locator('#status')).toBeVisible();
    await expect(page.locator('#status')).toContainText('GPU disposed.');
    const harness = await page.request.get('http://127.0.0.1:4173/player/tests/contract.html');
    assert.equal(harness.status(), 404, 'production assets must not include the test harness');
    await save(join(output, 'receipt.json'), { ok: true, browser: browser.version(), launch,
      archiveSha256: candidate.receipt.sha256, htmlSha256: hash(await readFile(join(product, 'dist/index.html'))),
      pngSha256: hash(bytes), pixelsExact: true, testHarnessAbsent: true, disposed: true,
      completedAt: new Date().toISOString() });
  } finally {
    await page.close();
    await browser.close();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [mode, ...paths] = process.argv.slice(2);
  const operation = { run, production }[mode];
  assert.ok(operation && paths.length === operation.length, 'Usage: default-browser.mjs run <product> <candidate-dir> <native-dir> <launch.json> <fresh-output> | production <product> <candidate-dir> <launch.json> <fresh-output>');
  await operation(...paths.map(value => resolve(value)));
}
