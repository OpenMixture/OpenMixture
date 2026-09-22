import fs from 'node:fs';
import path from 'node:path';
import cp from 'node:child_process';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import { fileURLToPath } from 'node:url';

const [binary, source, destination] = process.argv.slice(2);
if (!binary || !source || !destination) throw Error('Usage: node reproduce.mjs <CLI> <input.mix> <fresh-output>');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const git = (...args) => cp.execFileSync('git', args, { encoding: 'utf8' }).trim();
if (git('status', '--porcelain')) throw Error('Run from a clean source checkout after building its CLI.');
const sourceRevision = git('rev-parse', 'HEAD');
fs.mkdirSync(destination);
fs.copyFileSync(source, path.join(destination, 'input.mix'));
const input = path.join(destination, 'input.mix');
const receipt = {
  schemaVersion: 1, sourceRevision, sourceDirty: false, createdAt: new Date().toISOString(),
  buildCommand: 'cargo build --locked -p mixture-cli', buildProfile: 'dev',
  cliSha256: sha(fs.readFileSync(binary)), lockSha256: sha(fs.readFileSync('Cargo.lock')),
  verifierSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))),
  toolchain: cp.execFileSync('rustc', ['--version'], { encoding: 'utf8' }).trim(),
  node: process.version, platform: process.platform, arch: process.arch,
  arithmetic: { a: 0.5, b: 0.499755859375, expectedDifference: 0.000244140625, amplification: 4096 },
  scope: 'Existing three-node subtraction composition only; no new kernel or material acceptance.',
  ok: false, runs: [], files: {},
};
const save = () => fs.writeFileSync(path.join(destination, 'receipt.json'), JSON.stringify(receipt, null, 2) + '\n');
save();
function pixel(filename) {
  const bytes = fs.readFileSync(filename);
  if (!bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) throw Error('Invalid PNG');
  const chunks = [];
  let header = false;
  for (let i = 8; i < bytes.length;) {
    const length = bytes.readUInt32BE(i), type = bytes.toString('ascii', i + 4, i + 8);
    if (type === 'IHDR') {
      if (length !== 13 || bytes.readUInt32BE(i + 8) !== 1 || bytes.readUInt32BE(i + 12) !== 1 || bytes[i + 16] !== 8 || bytes[i + 17] !== 6 || bytes[i + 20] !== 0) throw Error('Expected noninterlaced 1x1 RGBA8');
      header = true;
    }
    if (type === 'IDAT') chunks.push(bytes.subarray(i + 8, i + 8 + length));
    i += length + 12;
  }
  const row = zlib.inflateSync(Buffer.concat(chunks), { maxOutputLength: 5 });
  if (!header || row.length !== 5 || row[0] > 4) throw Error('Unexpected scanline');
  // At the first and only pixel, all PNG filter predictors are zero.
  return [...row.subarray(1)];
}
for (const backend of ['vulkan', 'dx12']) {
  const args = ['render', input, '--size', '1', '--output', 'height,roughness', '--backend', backend, '--out', path.join(destination, backend), '--json'];
  const result = cp.spawnSync(path.resolve(binary), args, { encoding: 'utf8' });
  fs.writeFileSync(path.join(destination, backend + '.json'), result.stdout || '');
  if (result.status !== 0) throw Error(result.stderr || result.stdout || 'CLI failed');
  const report = JSON.parse(result.stdout);
  if (!report.ok || report.context.adapter.backend.toLowerCase() !== backend) throw Error('Unexpected execution');
  const actual = pixel(path.join(destination, backend, 'height.png'));
  const reference = pixel(path.join(destination, backend, 'roughness.png'));
  if (String(actual) !== '0,0,0,255' || String(reference) !== '255,255,255,255') throw Error('Counterexample not reproduced');
  receipt.runs.push({ backend, args, adapter: report.context.adapter.name, driver: report.context.adapter.driverInfo, planHash: report.planHash, actual, reference });
  save();
}
if (git('status', '--porcelain') || git('rev-parse', 'HEAD') !== sourceRevision || sha(fs.readFileSync(binary)) !== receipt.cliSha256) throw Error('Source or binary changed during execution');
for (const name of ['input.mix', 'vulkan.json', 'dx12.json', 'vulkan/height.png', 'vulkan/roughness.png', 'dx12/height.png', 'dx12/roughness.png']) receipt.files[name] = sha(fs.readFileSync(path.join(destination, name)));
receipt.ok = true;
save();
console.log(JSON.stringify({ ok: true, sourceRevision, runs: receipt.runs.map(({backend, actual, reference}) => ({backend, actual, reference})) }));
