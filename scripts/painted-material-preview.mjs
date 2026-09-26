// Consumer visualization of qualified PNGs only; no graph or material execution.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { readFile, mkdir, writeFile, copyFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
const require = createRequire(new URL('../examples/browser-consumer/package.json', import.meta.url));
const { chromium } = require('playwright');
const browserChannel = process.env.MIXTURE_BROWSER_CHANNEL ?? 'chrome';
const browserArgs = ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist'];
const args = process.argv.slice(2);
assert.ok(args.length === 2 || args.length === 3, 'usage: painted-material-preview.mjs <painted-comparison> <fresh-output> [producer-plan.json]');
const [input, output] = args.map(p => resolve(p));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = async path => JSON.parse(await readFile(path));
const comparison = await json(join(input, 'comparison.json'));
const native = await json(join(input, 'native/native.json'));
const browserInput = await json(join(input, 'browser-qualification.json'));
for (const receipt of [comparison, native, browserInput]) assert.equal(receipt.ok, true);
assert.equal(comparison.completed, true);
assert.equal(native.completed, true);
assert.equal(native.browserCompared, true);
assert.equal(native.requestManifest.workingTreeStatus, '');
assert.equal(native.requestManifest.sourceRevision, comparison.revision);
assert.equal(browserInput.consumerRevision, comparison.revision);
assert.deepEqual(browserInput.build, comparison.build);
const contractBytes = await readFile(new URL('../fixtures/materials/painted-metal/qualification-plan.json', import.meta.url));
const producerPlan = args[2] ? await readFile(resolve(args[2])) : contractBytes;
assert.equal(hash(producerPlan), native.requestManifest.qualificationPlanSha256, 'supply the exact producer plan snapshot when its bytes differ');
assert.deepEqual(JSON.parse(producerPlan), JSON.parse(contractBytes), 'producer and preview must use the same frozen contract');
assert.equal(hash(await readFile(join(input, 'material.mix'))), native.requestManifest.sourceSha256);
const contract = JSON.parse(contractBytes);
const cases = contract.cases.map(c => c.id);
assert.equal(cases.length, 7);
const hashes = {}, maps = new Map();
for (const variant of cases) {
  assert.match(variant, /^[a-z0-9-]+$/);
  assert.equal(native.rows.filter(row => row.id === `${variant}-1024x1024`).length, 1);
  for (const channel of contract.channels) {
    const name = `${variant}-1024x1024-${channel}.png`;
    const bytes = await readFile(join(input, 'native', name));
    assert.equal(bytes.subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
    assert.equal(bytes.readUInt32BE(16), 1024);
    assert.equal(bytes.readUInt32BE(20), 1024);
    maps.set(`/maps/${name}`, bytes); hashes[name] = hash(bytes);
  }
}
await mkdir(output);
await writeFile(join(output, 'preview.json'), JSON.stringify({ ok: false, completed: false }));
const html = await readFile(new URL('./painted-material-preview.html', import.meta.url));
const server = createServer((req, res) => {
  const bytes = req.url === '/' ? html : req.url === '/config' ? Buffer.from(JSON.stringify({ cases })) : maps.get(req.url);
  res.writeHead(bytes ? 200 : 404, { 'content-type': req.url === '/' ? 'text/html' : req.url === '/config' ? 'application/json' : 'image/png' });
  res.end(bytes ?? 'missing');
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
let browser;
try {
  browser = await chromium.launch({ channel: browserChannel, headless: true, args: browserArgs });
  const page = await browser.newPage({ viewport: { width: 1320, height: 1000 }, deviceScaleFactor: 1 });
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.waitForFunction(() => window.previewResult, {}, { timeout: 120000 });
  const result = await page.evaluate(() => window.previewResult);
  assert.equal(result.ok, true, result.error);
  assert.deepEqual(result.cases, cases);
  const probes = page.locator('canvas.metallic-probe');
  assert.equal(await probes.count(), 3);
  const probeBytes = [];
  for (let i = 0; i < 3; i++) probeBytes.push(await probes.nth(i).screenshot());
  assert.deepEqual(probeBytes[0], probeBytes[2], 'same metallic setting must repeat exactly');
  assert.notDeepEqual(probeBytes[0], probeBytes[1], 'metallic map contribution must change shading');
  const screenshots = {};
  for (const variant of cases) {
    const bytes = await page.locator(`#${variant}`).screenshot({ path: join(output, `${variant}-pbr.png`) });
    screenshots[`${variant}-pbr.png`] = hash(bytes);
  }
  for (const name of ['comparison.json', 'browser-qualification.json']) await copyFile(join(input, name), join(output, name));
  await copyFile(join(input, 'native/native.json'), join(output, 'native.json'));
  await writeFile(join(output, 'producer-plan.json'), producerPlan);
  await writeFile(join(output, 'index.html'), `<!doctype html><meta charset="utf-8"><title>Painted metal review</title>
<style>body{font:16px system-ui;background:#15191f;color:#edf0f3;margin:28px}img{display:block;max-width:100%;height:auto}section{margin:36px 0}p{max-width:80ch;line-height:1.6}</style>
<h1>Painted metal — review candidate</h1><p>Fixed camera and lights; metallic GGX, plane/sphere, repeated tiling and close-ups. These views show the generated maps. Height is not displaced. Human acceptance is pending.</p>
${cases.map(id => `<section><h2>${id}</h2><img src="${id}-pbr.png" alt="${id}: plane and sphere with full, tiled and close-up views"></section>`).join('\n')}`);
  await writeFile(join(output, 'preview.json'), JSON.stringify({ ok: true, completed: true, humanAccepted: false,
    materialAccepted: false, producerRevision: comparison.revision, producerBuild: comparison.build,
    previewRevision: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
    previewDirty: execFileSync('git', ['status', '--porcelain'], { encoding: 'utf8' }).trim().length > 0,
    browser: browser.version(), browserChannel, browserArgs, ...result, inputPngSha256: hashes, screenshotSha256: screenshots,
    previewHtmlSha256: hash(html), previewRunnerSha256: hash(await readFile(new URL(import.meta.url))),
    producerPlanSha256: hash(producerPlan), previewPlanSha256: hash(contractBytes),
    metallicCheck: { dielectric: hash(probeBytes[0]), metal: hash(probeBytes[1]), repeat: hash(probeBytes[2]), changed: true, repeatExact: true },
    shading: 'metallic GGX, fixed key/fill and ambient approximation, no displacement; consumer visualization only',
  }, null, 2));
  console.log(`Controlled painted-metal PBR views: ${output}`);
} finally {
  if (browser) await browser.close();
  await new Promise(resolve => server.close(resolve));
}
