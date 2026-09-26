// MAT-02 fixture controls only. Core still validates and compiles every request.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const planBytes = await readFile(new URL('../fixtures/materials/painted-metal/qualification-plan.json', import.meta.url));
const plan = JSON.parse(planBytes);
const sourceBytes = await readFile(new URL('../docs/evidence/perf-mat-before/material.mix', import.meta.url));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');

function range(value, name, low, high, integer = false) {
  assert.ok(typeof value === 'number' && Number.isFinite(value) && value >= low && value <= high &&
    (!integer || Number.isInteger(value)), `${name}: expected ${integer ? 'integer' : 'finite number'} in [${low},${high}]`);
}

export function paintedMetalRequest(changes = {}, size = [1024, 1024]) {
  assert.ok(changes !== null && typeof changes === 'object' && !Array.isArray(changes), 'controls must be an object');
  assert.ok(Array.isArray(size) && size.length === 2, 'size must have two axes');
  for (let axis = 0; axis < 2; axis++) range(size[axis], `size[${axis}]`, 1, 2048, true);
  for (const name of Object.keys(changes)) assert.ok(Object.hasOwn(plan.defaults, name), `unknown control: ${name}`);
  const controls = structuredClone({ ...plan.defaults, ...changes });
  for (const name of ['exposureAmount', 'rustAmount', 'rustFill', 'detailAmount', 'paintRoughness', 'substrateRoughness', 'rustRoughness']) range(controls[name], name, 0, 1);
  range(controls.exposureScale, 'exposureScale', 1, 64, true);
  range(controls.edgeWidth, 'edgeWidth', 0, 8, true);
  for (const name of ['macroSeed', 'detailSeed']) range(controls[name], name, 0, 0xffffffff, true);
  range(controls.paintThickness, 'paintThickness', 0, 0.5);
  range(controls.rustRelief, 'rustRelief', 0, controls.paintThickness);
  range(controls.normalStrength, 'normalStrength', 0, 8);
  for (const name of ['paintColor', 'substrateColor', 'rustColor']) {
    const color = controls[name];
    assert.ok(Array.isArray(color) && color.length === 4, `${name}: expected RGBA`);
    for (let i = 0; i < 4; i++) range(color[i], `${name}[${i}]`, 0, 1);
    assert.equal(color[3], 1, `${name}: alpha must be one`);
  }
  const overrides = Object.fromEntries(['macroSeed', 'detailSeed', 'exposureScale', 'rustAmount', 'rustFill',
    'paintColor', 'substrateColor', 'rustColor', 'paintRoughness', 'substrateRoughness', 'rustRoughness',
    'normalStrength'].map(name => [name, structuredClone(controls[name])]));
  const amount = controls.exposureAmount;
  const endpoint = amount === 0 || amount === 1;
  const inputMin = endpoint ? 0 : 0.8 * (1 - amount);
  Object.assign(overrides, {
    exposureInputMin: inputMin, exposureInputMax: endpoint ? 1 : inputMin + 0.2,
    exposureOutputMin: endpoint ? amount : 0, exposureOutputMax: endpoint ? amount : 1,
    detailMin: 1 - controls.detailAmount, paintHeight: 0.2 + controls.paintThickness,
    rustHeight: 0.2 + controls.rustRelief,
  });
  for (const [index, axis] of ['X', 'Y'].entries()) {
    overrides[`radius${axis}`] = controls.edgeWidth === 0 ? 0 :
      Math.max(1, Math.floor((controls.edgeWidth * size[index] + 512) / 1024));
  }
  return { controls, request: { size: [...size], channels: [...plan.channels], overrides } };
}

export function paintedMetalMatrix() {
  return plan.cases.flatMap(preset => plan.sizes.map(size => ({
    id: `${preset.id}-${size[0]}x${size[1]}`, preset: preset.id,
    ...paintedMetalRequest(preset.controls, size),
  })));
}

export async function writePaintedMetalRequests(destination) {
  // Refuse an existing directory so partial or previous results cannot look fresh.
  await mkdir(destination);
  const source = JSON.parse(sourceBytes);
  const ids = source.exposedParameters.map(p => p.id).sort();
  const rows = paintedMetalMatrix();
  for (const row of rows) assert.deepEqual(Object.keys(row.request.overrides).sort(), ids);
  await writeFile(join(destination, 'material.mix'), sourceBytes);
  const manifest = { kind: 'mat02-material-requests', materialAccepted: false,
    sourceRevision: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
    workingTreeStatus: execFileSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8' }),
    sourceFile: 'material.mix', sourceSha256: hash(sourceBytes), qualificationPlanSha256: hash(planBytes),
    builderSha256: hash(await readFile(fileURLToPath(import.meta.url))), causality: structuredClone(plan.causality), rows };
  await writeFile(join(destination, 'requests.json'), JSON.stringify(manifest, null, 2) + '\n');
  return manifest;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3, 'usage: node scripts/painted-metal-requests.mjs <fresh-directory>');
  const manifest = await writePaintedMetalRequests(resolve(process.argv[2]));
  console.log(`Prepared ${manifest.rows.length} MAT-02 requests; no pixel qualification claimed.`);
}
