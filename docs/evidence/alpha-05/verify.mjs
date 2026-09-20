import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = (path) => readFileSync(new URL(path, import.meta.url), 'utf8').replace(/^\uFEFF/, '');
const json = (path) => JSON.parse(read(path));
const before = json('./before-ruleset.json');
const after = json('./after-ruleset.json');
const applied = json('./applied-ruleset.json');
const desired = json('../../../.github/main-ruleset.json');
const required = (policy) => policy.rules.find((rule) => rule.type === 'required_status_checks').parameters;
const added = ['WASM and npm package', 'Chromium WebGPU material matrix'];
for (const key of Object.keys(applied)) assert.deepEqual(after[key], applied[key]);
const original = structuredClone(applied);
required(original).required_status_checks = required(original).required_status_checks.filter((check) => !added.includes(check.context));
for (const key of Object.keys(original)) assert.deepEqual(original[key], before[key]);
assert.equal(after.id, 23016046);
assert.equal(after.enforcement, 'active');
assert.equal(after.current_user_can_bypass, 'never');
assert.deepEqual(after.bypass_actors, []);
assert.deepEqual(after.conditions.ref_name, { include: ['refs/heads/main'], exclude: [] });
assert.equal(required(after).strict_required_status_checks_policy, true);
const checks = required(after).required_status_checks;
assert.equal(checks.length, 6);
assert.equal(new Set(checks.map((check) => check.context)).size, 6);
assert.ok(checks.every((check) => check.integration_id === 15368));
for (const key of ['name', 'target', 'enforcement', 'bypass_actors', 'conditions']) assert.deepEqual(desired[key], after[key]);
for (const rule of desired.rules) {
  const actual = after.rules.find((item) => item.type === rule.type);
  assert.ok(actual);
  for (const [key, value] of Object.entries(rule.parameters ?? {})) assert.deepEqual(actual.parameters[key], value);
}
const effective = json('./effective-main-rules.json');
assert.deepEqual(effective.find((rule) => rule.type === 'required_status_checks').parameters, required(after));
assert.ok(effective.every((rule) => rule.ruleset_id === 23016046));
assert.deepEqual(json('./pr-required-checks.json').map((check) => check.name).sort(), checks.map((check) => check.context).sort());
const activation = json('./activation.json');
assert.equal(activation.baseline_checks.length, 6);
assert.deepEqual(activation.baseline_checks.map((check) => check.name).sort(), checks.map((check) => check.context).sort());
assert.ok(activation.baseline_checks.every((check) => check.head_sha === activation.baseline_main && check.status === 'completed' && check.conclusion === 'success' && check.app_id === 15368));
for (const [index, file] of ['browser-runtime.yml', 'browser-materials.yml'].entries()) {
  const workflow = read('../../../.github/workflows/' + file);
  assert.ok(workflow.includes('name: ' + added[index]));
  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /branches: \[main\]/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.doesNotMatch(workflow, /^\s+paths(?:-ignore)?:/m);
}
console.log('ALPHA-05: six required checks, unchanged protections, live snapshots and workflow coverage verified.');
