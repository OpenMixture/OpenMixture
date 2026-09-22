#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { cp, mkdir, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const destination = join(root, 'target/browser-runtime');
const staging = join(destination, 'package');
const bindgen = process.env.WASM_BINDGEN ?? 'wasm-bindgen';
function command(program, args, options = {}) {
  // Windows cannot spawn npm.cmd without a shell. Invoke the npm CLI shipped
  // with the selected Node installation directly, preserving literal paths.
  if (process.platform === 'win32' && program === 'npm') {
    args = [join(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js'), ...args];
    program = process.execPath;
  }
  const result = spawnSync(program, args, { cwd: root, encoding: 'utf8', ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${program} failed (${result.status}):\n${result.stderr}\n${result.stdout}`);
  return result.stdout?.trim() ?? '';
}
const toolVersion = command(bindgen, ['--version']);
if (toolVersion !== 'wasm-bindgen 0.2.128') throw new Error(`Expected wasm-bindgen 0.2.128; got ${toolVersion}`);
command('npm', ['ci', '--ignore-scripts'], { cwd: join(root, 'packages/runtime') });
command('npm', ['run', 'build'], { cwd: join(root, 'packages/runtime') });
const typescriptVersion = JSON.parse(await readFile(join(root, 'packages/runtime/node_modules/typescript/package.json'), 'utf8')).version;
const manifest = JSON.parse(await readFile(join(root, 'packages/runtime/package.json'), 'utf8'));
const paths = ['Cargo.toml', 'Cargo.lock'];
for (const path of ['rust-toolchain.toml', '.cargo/config.toml']) {
  try { if ((await stat(join(root, path))).isFile()) paths.push(path); } catch (error) { if (error.code !== 'ENOENT') throw error; }
}
async function collect(directory) {
  for (const entry of (await readdir(join(root, directory), { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
    if (directory === 'packages/runtime' && ['node_modules', 'dist'].includes(entry.name)) continue;
    const name = `${directory}/${entry.name}`;
    if (entry.isDirectory()) await collect(name);
    else paths.push(name);
  }
}
for (const directory of ['crates/mixture-asset/src', 'crates/mixture-core/src', 'crates/mixture-wgpu/src', 'crates/mixture-wgpu/shaders',
  'crates/mixture-wasm', 'packages/runtime', 'scripts/browser-runtime']) await collect(directory);
for (const path of ['crates/mixture-asset/Cargo.toml', 'crates/mixture-core/Cargo.toml', 'crates/mixture-wgpu/Cargo.toml']) paths.push(path);
const digest = createHash('sha256');
const compilerVersion = command('rustc', ['--version', '--verbose']);
const compilationEnvironment = Object.fromEntries(Object.entries(process.env).filter(([key]) =>
  key === 'RUSTFLAGS' || key === 'CARGO_ENCODED_RUSTFLAGS' || key.startsWith('CARGO_PROFILE_') || key.startsWith('CARGO_TARGET_WASM32_')
).sort(([a], [b]) => a.localeCompare(b)));
digest.update(JSON.stringify({ compilerVersion, bindgen: toolVersion, typescriptVersion, compilationEnvironment }));
digest.update('\0');
for (const path of paths.sort()) { digest.update(path); digest.update('\0'); digest.update(await readFile(join(root, path))); digest.update('\0'); }
const revisionResult = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' });
const dirtyResult = spawnSync('git', ['status', '--porcelain', '--untracked-files=normal'], { cwd: root, encoding: 'utf8' });
const metadata = JSON.parse(command('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1']));
const engineVersion = metadata.packages.find(pkg => pkg.name === 'mixture-wasm')?.version;
if (!engineVersion) throw new Error('Could not resolve the binding engine version');
const buildInfo = { runtimeVersion: manifest.version, apiSchemaVersion: 2, engineVersion,
  engineRevision: revisionResult.status === 0 ? revisionResult.stdout.trim() : null,
  engineDirty: dirtyResult.status === 0 ? dirtyResult.stdout.trim().length > 0 : null, buildId: `sha256:${digest.digest('hex')}` };
const env = { ...process.env, MIXTURE_RUNTIME_VERSION: manifest.version, MIXTURE_BUILD_ID: buildInfo.buildId };
delete env.MIXTURE_ENGINE_REVISION;
delete env.MIXTURE_ENGINE_DIRTY;
if (buildInfo.engineRevision) env.MIXTURE_ENGINE_REVISION = buildInfo.engineRevision;
if (buildInfo.engineDirty !== null) env.MIXTURE_ENGINE_DIRTY = buildInfo.engineDirty ? '1' : '0';
console.log(`Building ${manifest.name}@${manifest.version} (${buildInfo.buildId})`);
command('cargo', ['build', '--locked', '-p', 'mixture-wasm', '--target', 'wasm32-unknown-unknown', '--target-dir', join(root, 'target'), '--release'], { env, stdio: ['inherit', 'inherit', 'pipe'] });
await rm(staging, { recursive: true, force: true });
await mkdir(join(staging, 'wasm'), { recursive: true });
await cp(join(root, 'packages/runtime/dist'), join(staging, 'src'), { recursive: true });
for (const file of ['README.md', 'README.zh-CN.md']) await cp(join(root, 'packages/runtime', file), join(staging, file));
// Compiler and development commands are producer-only, not consumer dependencies.
const { devDependencies, scripts, ...publishedManifest } = manifest;
await writeFile(join(staging, 'package.json'), JSON.stringify(publishedManifest, null, 2) + '\n');
const generated = join(destination, 'generated');
await rm(generated, { recursive: true, force: true });
command(bindgen, [join(root, 'target/wasm32-unknown-unknown/release/mixture_wasm.wasm'), '--target', 'no-modules', '--out-dir', generated, '--out-name', 'mixture_wasm']);
const glue = await readFile(join(generated, 'mixture_wasm.js'), 'utf8');
// Preserve generated glue as an isolated factory. This also makes import inert
// and permits independent loads instead of silently reusing a previous WASM.
await writeFile(join(staging, 'wasm/bindings.mjs'), `// Generated by ${toolVersion}; build ${buildInfo.buildId}\nexport default function createBindings() {\n${glue}\nreturn wasm_bindgen;\n}\n`);
await cp(join(generated, 'mixture_wasm_bg.wasm'), join(staging, 'wasm/mixture_wasm_bg.wasm'));
await writeFile(join(staging, 'src/build-info.js'), `export const buildInfo = Object.freeze(${JSON.stringify(buildInfo)});\n`);
const declarations = await readFile(join(staging, 'src/index.d.ts'), 'utf8');
await writeFile(join(staging, 'src/index.d.ts'), `// Runtime build ${buildInfo.buildId}; API schema ${buildInfo.apiSchemaVersion}\n${declarations}`);
await writeFile(join(staging, 'build-info.json'), JSON.stringify(buildInfo, null, 2) + '\n');
for (const license of ['LICENSE-MIT', 'LICENSE-APACHE']) await cp(join(root, license), join(staging, license));
const packed = JSON.parse(command('npm', ['pack', '--json', '--pack-destination', destination], { cwd: staging }));
const archive = join(destination, packed[0].filename);
const sha256 = createHash('sha256').update(await readFile(archive)).digest('hex');
await writeFile(`${archive}.sha256`, `${sha256}  ${packed[0].filename}\n`);
await writeFile(join(destination, 'receipt.json'), JSON.stringify({ ...buildInfo, tarball: relative(root, archive), sha256,
  toolchain: { rust: command('rustc', ['--version']), bindgen: toolVersion, typescript: typescriptVersion, node: process.version, npm: command('npm', ['--version']), target: 'wasm32-unknown-unknown' },
  files: packed[0].files.map(file => file.path), status: 'built; browser acceptance is a separate verification' }, null, 2) + '\n');
console.log(archive);
console.log(`SHA-256 ${sha256}`);
