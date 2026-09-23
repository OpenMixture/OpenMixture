// CPU-only audit of retained default recipe and proposed slot assignment, not a renderer.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const base = new URL('./', import.meta.url);
const read = (name) => JSON.parse(readFileSync(new URL(name, base), 'utf8').replace(/^\uFEFF/, ''));
const receipt = read('receipt.json');
for (const [name, hash] of Object.entries(receipt.files)) {
  assert.equal(createHash('sha256').update(readFileSync(new URL(name, base))).digest('hex'), hash, name);
}
const recipe = read('../../../fixtures/materials/painted-metal/graph-design.json');
const frozen = read('../../../fixtures/materials/painted-metal/qualification-plan.json');
const vars = { ...frozen.defaults };
Object.assign(vars, {
  exposureInputMin: 0.8 * (1 - vars.exposureAmount),
  exposureInputMax: 0.8 * (1 - vars.exposureAmount) + 0.2,
  exposureOutputMin: 0, exposureOutputMax: 1,
  radiusX: vars.edgeWidth, radiusY: vars.edgeWidth,
  detailMin: 1 - vars.detailAmount,
  paintHeight: 0.2 + vars.paintThickness, rustHeight: 0.2 + vars.rustRelief,
});
const nodes = [], edges = [], exposedParameters = [];
for (const node of recipe.nodes) {
  const { inputs = {}, parameters, ...identity } = node;
  const result = { ...identity };
  if (parameters) {
    result.parameters = {};
    for (const [parameterId, value] of Object.entries(parameters)) {
      if (typeof value === 'string' && value.startsWith('$')) {
        const id = value.slice(1);
        assert.ok(Object.hasOwn(vars, id), id);
        result.parameters[parameterId] = vars[id];
        exposedParameters.push({ id, nodeId: node.id, parameterId });
      } else result.parameters[parameterId] = value;
    }
  }
  nodes.push(result);
  for (const [portId, source] of Object.entries(inputs)) {
    const [nodeId, sourcePort] = source.split('.');
    edges.push({ from: { nodeId, portId: sourcePort }, to: { nodeId: node.id, portId } });
  }
}
nodes.push({ id: 'out', type: 'material-output', version: 1 });
for (const [portId, source] of Object.entries(recipe.outputs)) {
  const [nodeId, sourcePort] = source.split('.');
  edges.push({ from: { nodeId, portId: sourcePort }, to: { nodeId: 'out', portId } });
}
assert.deepEqual(read('material.mix'), { version: 1, nodes, edges, exposedParameters });
const plan = read('plan-1024.json').plan;
const passes = plan.passes;
const last = passes.map((pass) => pass.id);
for (const pass of passes) {
  for (const key of ['input', 'a', 'b', 'mask', 'displacement']) {
    if (Object.hasOwn(pass.kernel, key)) last[pass.kernel[key]] = pass.id;
  }
}
for (const output of plan.outputs) last[output.resource] = passes.length;
const slots = [], assignments = [];
for (const pass of passes) {
  const descriptor = JSON.stringify(pass.outputDesc);
  let slot = slots.findIndex((entry) => entry.last < pass.id && entry.descriptor === descriptor);
  if (slot === -1) { slot = slots.length; slots.push({ descriptor, last: -1 }); }
  slots[slot].last = last[pass.output];
  assignments.push(slot);
}
// Explicit design expectation, independent of the future Rust allocator.
assert.deepEqual(assignments, [0, 1, 0, 2, 0, 3, 4, 5, 4, 6, 7, 5, 8, 9, 10, 11, 4, 12, 6, 1, 0, 13, 2]);
assert.equal(slots.length, 14);
assert.equal(slots.length * 2048 * 2048 * 8 + 464 + 2048 * 2048 * 8, 503316944);
console.log('Retained byte hashes and default recipe match; design-only simulation: 14 slots, 503316944 bytes at 2K.');
