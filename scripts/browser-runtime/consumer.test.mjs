import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { join, dirname, basename, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertBrowserReport, candidateManifests, safePackagePath, verifyInstalled } from './consumer.mjs';
import { hash } from './candidate.mjs';

test('brick comparison keeps a fresh checkout clean without private target ignores', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'mixture-brick-routing-'));
  const git = (...args) => execFileSync('git', args, { cwd: directory, encoding: 'utf8' }).trim();
  try {
    const fixture = join(directory, 'fixtures/materials/brick-paving');
    const nativeTests = join(directory, 'examples/native-consumer/tests');
    const qualification = join(directory, 'qualification');
    await mkdir(fixture, { recursive: true });
    await mkdir(nativeTests, { recursive: true });
    await mkdir(join(qualification, 'test-results'), { recursive: true });
    await writeFile(join(directory, '.gitignore'), '/target/\n/qualification/\n/comparison/\n');
    await writeFile(join(directory, '.empty-ignore'), '');
    await writeFile(join(directory, 'rust-toolchain.toml'), await readFile(new URL('../../rust-toolchain.toml', import.meta.url)));
    const brickFixtures = {};
    for (const name of ['material.mix', 'controls.json', 'qualification-plan.json']) {
      const bytes = await readFile(new URL(`../../fixtures/materials/brick-paving/${name}`, import.meta.url));
      await writeFile(join(fixture, name), bytes);
      brickFixtures[name] = hash(bytes);
    }
    const matrix = JSON.parse(await readFile(join(fixture, 'qualification-plan.json')));
    const rows = matrix.cases.flatMap(c => matrix.sizes.map(size => ({ case: c.id, size, planHash: 'routing-fixture' })));
    const native = { ok: true, completed: true, debugAssertions: false, browserCompared: true,
      timing: [{ budgetPassed: true }, { budgetPassed: true }],
      cases: rows.map(row => ({ ...row, execution: { planHash: row.planHash }, browserComparison: matrix.channels.map(channel => ({ channel, maxComponentDelta: 0 })) })),
    };
    // A dependency-free Rust test replaces only the GPU workload. The real
    // orchestration still invokes Cargo in a separate consumer workspace.
    await writeFile(join(directory, 'examples/native-consumer/Cargo.toml'), '[package]\nname="brick-routing-fixture"\nversion="0.0.0"\nedition="2024"\n[workspace]\n');
    await writeFile(join(nativeTests, 'matrix.json'), JSON.stringify(native));
    await writeFile(join(nativeTests, 'brick_material.rs'), `#[test]\n#[ignore]\nfn brick_material_public_gpu_matrix() {\n assert!(!cfg!(debug_assertions));\n let out = std::path::PathBuf::from(std::env::var("MIXTURE_BRICK_EVIDENCE_DIR").unwrap());\n std::fs::create_dir(&out).unwrap();\n std::fs::write(out.join("native-matrix.json"), include_str!("matrix.json")).unwrap();\n}\n`);
    const env = { ...process.env };
    delete env.CARGO_TARGET_DIR;
    execFileSync('cargo', ['generate-lockfile', '--manifest-path', 'examples/native-consumer/Cargo.toml', '--offline'], { cwd: directory, env, stdio: 'pipe' });
    git('init', '--quiet');
    git('config', 'core.autocrlf', 'false');
    git('config', 'core.excludesFile', join(directory, '.empty-ignore'));
    await mkdir(join(directory, '.git/info'), { recursive: true });
    await writeFile(join(directory, '.git/info/exclude'), '');
    git('add', '.');
    git('-c', 'user.name=Routing fixture', '-c', 'user.email=routing@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '--quiet', '-m', 'fixture');
    const build = { runtimeVersion: '0.8.0-alpha.0' };
    await writeFile(join(qualification, 'qualification.json'), JSON.stringify({ ok: true, mode: 'candidate', build, consumerRevision: git('rev-parse', 'HEAD'), consumerDirty: false, brickFixtures }));
    await writeFile(join(qualification, 'test-results/brick-browser.json'), JSON.stringify({ ok: true, build, rows }));
    for (const row of rows) for (const channel of matrix.channels) {
      await writeFile(join(qualification, 'test-results', `${row.case}-${row.size[0]}x${row.size[1]}-${channel}.png`), 'routing-only fixture; pixels are not evaluated');
    }
    execFileSync(process.execPath, [fileURLToPath(new URL('./check-brick.mjs', import.meta.url)), qualification, join(directory, 'comparison')], { cwd: directory, env, stdio: 'pipe' });
    assert.equal(JSON.parse(await readFile(join(directory, 'comparison/comparison.json'))).ok, true);
    assert.equal(git('status', '--porcelain'), '', 'Cargo outputs must stay in the repository-owned ignored target directory');
  } finally {
    assert.equal(dirname(resolve(directory)), resolve(tmpdir()));
    assert.ok(basename(directory).startsWith('mixture-brick-routing-'));
    await rm(directory, { recursive: true, force: true });
  }
});

test('candidate substitution preserves the frozen dependency graph and does not accept floating versions', () => {
  const manifest = { dependencies: { '@openmixture/runtime': '0.3.0-alpha.0' } };
  const lock = { lockfileVersion: 3, packages: {
    '': structuredClone(manifest),
    'node_modules/@openmixture/runtime': { version: '0.3.0-alpha.0', resolved: 'https://registry.npmjs.org/runtime.tgz', integrity: 'old' },
    'node_modules/vite': { version: '8.3.0', integrity: 'unchanged' },
  } };
  const result = candidateManifests(manifest, lock, '0.8.0-alpha.0', Buffer.from('candidate'));
  assert.equal(result.manifest.dependencies['@openmixture/runtime'], 'file:vendor/runtime.tgz');
  assert.equal(result.lock.packages['node_modules/@openmixture/runtime'].resolved, 'file:vendor/runtime.tgz');
  assert.match(result.lock.packages['node_modules/@openmixture/runtime'].integrity, /^sha512-/);
  assert.deepEqual(result.lock.packages['node_modules/vite'], lock.packages['node_modules/vite']);
  assert.equal(manifest.dependencies['@openmixture/runtime'], '0.3.0-alpha.0');
  assert.throws(() => candidateManifests({ dependencies: { '@openmixture/runtime': '^0.1.0' } }, lock, '0.8.0-alpha.0', Buffer.from('candidate')));
});

test('archive paths cannot escape the installed package', () => {
  assert.equal(safePackagePath('wasm/mixture_wasm_bg.wasm'), 'wasm/mixture_wasm_bg.wasm');
  for (const path of ['../outside', '/absolute', 'src/../../outside', 'src\\outside', 'C:/outside', 'src//file', './file']) {
    assert.throws(() => safePackagePath(path));
  }
});

test('partial, skipped, flaky and failed browser evidence cannot pass qualification', () => {
  const report = { stats: { expected: 13, unexpected: 0, skipped: 0, flaky: 0 }, errors: [] };
  assertBrowserReport(report);
  assertBrowserReport({...report,stats:{...report.stats,expected:21}},'candidate');
  assert.throws(() => assertBrowserReport({...report,stats:{...report.stats,expected:19}}, 'candidate'));
  assert.throws(()=>assertBrowserReport(report,'candidate'));
  for (const stats of [{ expected: 7 }, { unexpected: 1 }, { skipped: 1 }, { flaky: 1 }]) {
    assert.throws(() => assertBrowserReport({ ...report, stats: { ...report.stats, ...stats } }));
  }
  assert.throws(() => assertBrowserReport({ ...report, errors: [{ message: 'worker crashed' }] }));
});

test('installed verification rejects changed WASM and mismatched build metadata', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'mixture-consumer-test-'));
  const installed = join(directory, 'node_modules/@openmixture/runtime');
  const build = { runtimeVersion: '0.1.0-alpha.0', apiSchemaVersion: 1, engineVersion: '0.1.0',
    engineRevision: 'a'.repeat(40), engineDirty: false, buildId: `sha256:${'b'.repeat(64)}` };
  try {
    await mkdir(join(installed, 'wasm'), { recursive: true });
    await writeFile(join(installed, 'build-info.json'), JSON.stringify(build));
    await writeFile(join(installed, 'wasm/runtime.wasm'), 'candidate');
    const files = { 'wasm/runtime.wasm': hash('candidate') };
    await verifyInstalled(directory, build, files);
    await writeFile(join(installed, 'wasm/runtime.wasm'), 'historical');
    await assert.rejects(verifyInstalled(directory, build, files), /installed bytes differ/);
    await writeFile(join(installed, 'build-info.json'), JSON.stringify({ ...build, engineRevision: 'c'.repeat(40) }));
    await assert.rejects(verifyInstalled(directory, build, files), /engineRevision mismatch/);
  } finally {
    assert.equal(dirname(resolve(directory)), resolve(tmpdir()));
    assert.ok(basename(directory).startsWith('mixture-consumer-test-'));
    await rm(directory, { recursive: true });
  }
});

async function assertNoiseMigration(name, fixture) {
 const legacy = await readFile(new URL(`../../examples/browser-consumer/public/${name}.mix`, import.meta.url));
 assert.deepEqual(legacy, fixture);
 const native = await readFile(new URL(`../../examples/native-consumer/tests/${name}.mix`, import.meta.url));
 const browser = await readFile(new URL(`../../examples/browser-consumer/public/${name}-v2.mix`, import.meta.url));
 assert.deepEqual(native, browser);
 const migrated = JSON.parse(fixture);
 let count = 0;
 for (const node of migrated.nodes) if (node.type === 'fractal-noise') {
  assert.equal(node.version, 1); node.version = 2; count++;
 }
 assert.ok(count > 0);
 assert.deepEqual(JSON.parse(native), migrated, 'migration changes only explicit noise versions');
}

test('ENG-04 hosts preserve the frozen fixture and share an explicit noise migration', async () => {
 const fixture = await readFile(new URL('../../fixtures/nodes/scalar-blend/two-noise.mix', import.meta.url));
 await assertNoiseMigration('scalar-blend', fixture);
});

test('published M6A resource cases are required in both modes and share the frozen source', async () => {
  const report = { stats: { expected: 13, unexpected: 0, skipped: 0, flaky: 0 }, errors: [] };
  assertBrowserReport({...report,stats:{...report.stats,expected:21}}, 'candidate');
  assert.throws(() => assertBrowserReport({...report,stats:{...report.stats,expected:20}}, 'candidate'), 'the painted-metal matrix must execute');
  assert.throws(() => assertBrowserReport({ ...report, stats: { ...report.stats, expected: 9 } }, 'candidate'));
  assertBrowserReport(report, 'registry');
  assert.throws(() => assertBrowserReport({ ...report, stats: { ...report.stats, expected: 9 } }, 'registry'));
  const fixture = await readFile(new URL('../../fixtures/nodes/image-input/height.mix', import.meta.url));
  await assertNoiseMigration('image-input', fixture);
});


test('asset closeout rejects missing, duplicate, mismatched and failed matrix evidence',async()=>{
  const {verifyAssetMatrix}=await import('./asset-matrix.mjs');
  const rows=['1024x1024','65x3'].flatMap(size=>[0,0.25,0.5,1].map(weight=>({size:size.split('x').map(Number),weight,planHash:'same',packageSha256:size,exactLoosePixels:true,repeatedLoads:2,allocations:{liveBytes:'0'},execution:{allocations:{liveBytes:0}},browserComparison:['height','normal'].map(channel=>({channel,maxComponentDelta:1,limit:1}))})));
  const browser={ownedAfterDestroy:true,rows:structuredClone(rows)},native={ok:true,ownedAfterDestroy:true,cases:rows};
  const receipt={assetFixtures:{'1024x1024.mixpack':'1024x1024','65x3.mixpack':'65x3'}};
  verifyAssetMatrix(native,browser,receipt);
  for(const mutate of [n=>n.cases.pop(),n=>n.cases[1]=n.cases[0],n=>n.cases[0].planHash='wrong',n=>n.cases[0].packageSha256='wrong',n=>n.cases[0].execution.allocations.liveBytes=1,n=>n.cases[0].browserComparison[0].maxComponentDelta=2,n=>n.ownedAfterDestroy=false]){
    const bad=structuredClone(native);mutate(bad);assert.throws(()=>verifyAssetMatrix(bad,browser,receipt));
  }
});


test('portable asset regression retains v1 independently of migrated loose resource inputs',async()=>{
  const asset = await readFile(new URL('../../examples/native-consumer/tests/asset-input.mix',import.meta.url));
  const browser = await readFile(new URL('../../examples/browser-consumer/public/image-input.mix',import.meta.url));
  assert.deepEqual(asset,browser);
  for(const node of JSON.parse(asset).nodes)if(node.type==='fractal-noise')assert.equal(node.version,1);
});
