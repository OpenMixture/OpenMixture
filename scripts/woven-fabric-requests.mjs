// Frozen MAT-03 caller mapping; Core remains the validator and only compiler.
import assert from 'node:assert/strict';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const planBytes = await readFile(new URL('../fixtures/materials/woven-fabric/qualification-plan.json', import.meta.url));
const plan = JSON.parse(planBytes);
const sourceBytes = await readFile(new URL('../fixtures/materials/woven-fabric/material.mix', import.meta.url));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function range(value, name, low, high, integer = false) {
  assert.ok(typeof value === 'number' && Number.isFinite(value) && value >= low && value <= high &&
    (!integer || Number.isInteger(value)), `${name}: expected ${integer ? 'integer' : 'finite number'} in [${low},${high}]`);
}
export function wovenFabricRequest(changes = {}, size = [1024, 1024]) {
  assert.ok(changes !== null && typeof changes === 'object' && !Array.isArray(changes), 'controls must be an object');
  assert.ok(Array.isArray(size) && size.length === 2, 'size must have two axes');
  size.forEach((value, axis) => range(value, `size[${axis}]`, 1, 2048, true));
  for (const name of Object.keys(changes)) assert.ok(Object.hasOwn(plan.defaults, name), `unknown control: ${name}`);
  const controls = structuredClone({ ...plan.defaults, ...changes });
  for (const [name, bounds] of Object.entries(plan.controlRanges)) range(controls[name], name, ...bounds, plan.integerControls.includes(name));
  for (const name of plan.evenControls) assert.equal(controls[name] % 2, 0, `${name}: must be even; never rounded`);
  for (const name of plan.colorControls) {
    assert.ok(Array.isArray(controls[name]) && controls[name].length === 4, `${name}: expected RGBA`);
    controls[name].forEach((value, index) => range(value, `${name}[${index}]`, 0, 1));
    assert.equal(controls[name][3], 1, `${name}: alpha must be one`);
  }
  const values = { ...controls, roughnessMin: controls.yarnRoughness * (1 - 2 * controls.detailAmount) };
  for (const axis of ['warp', 'weft']) values[`${axis}Dark`] = controls[`${axis}Color`].map((v, i) => i === 3 ? 1 : v * (1 - 4 * controls.detailAmount));
  const overrides = Object.fromEntries(plan.bindings.map(b => [b.id, structuredClone(values[b.control])]));
  return { controls, request: { size: [...size], channels: [...plan.channels], overrides } };
}
export function wovenFabricMatrix() {
  return plan.cases.flatMap(preset => plan.sizes.map(size => ({ id: `${preset.id}-${size[0]}x${size[1]}`, preset: preset.id, ...wovenFabricRequest(preset.controls, size) })));
}
export function wovenFabricStress() {
  return plan.stress.flatMap(stress => stress.sizes.map(size => ({ id: `stress-${stress.id}-${size[0]}x${size[1]}`, stress: stress.id, qualityScope: stress.qualityLabel, ...wovenFabricRequest(stress.controls, size) })));
}
export async function writeWovenFabricRequests(destination) {
  await mkdir(destination);
  const ids = JSON.parse(sourceBytes).exposedParameters.map(p => p.id).sort();
  const rows = wovenFabricMatrix(), stress = wovenFabricStress();
  for (const row of [...rows, ...stress]) assert.deepEqual(Object.keys(row.request.overrides).sort(), ids);
  await writeFile(join(destination, 'material.mix'), sourceBytes);
  await writeFile(join(destination, 'qualification-plan.json'), planBytes);
  const manifest = { kind: 'mat03-material-requests', recipeRevision: plan.recipeRevision, materialAccepted: false,
    sourceRevision: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
    workingTreeStatus: execFileSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8' }),
    sourceFile: 'material.mix', sourceSha256: hash(sourceBytes), qualificationPlanSha256: hash(planBytes),
    builderSha256: hash(await readFile(fileURLToPath(import.meta.url))), rows, stress };
  await writeFile(join(destination, 'requests.json'), JSON.stringify(manifest, null, 2) + '\n');
  return manifest;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3, 'usage: node scripts/woven-fabric-requests.mjs <fresh-directory>');
  const manifest = await writeWovenFabricRequests(resolve(process.argv[2]));
  console.log(`Prepared ${manifest.rows.length} MAT-03 rows and ${manifest.stress.length} stress rows; no material acceptance claimed.`);
}
