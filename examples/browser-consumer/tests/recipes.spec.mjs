import { test, expect } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

// MAT-04c browser adapter: explicit recipe sources, closure reporting and
// byte-identical pixels against the accepted v1 graphs through public WebGPU.
test('recipe parents decode only with explicit recipes and render like the accepted graphs', async ({ page, browser }, testInfo) => {
  test.setTimeout(300_000);
  const read = path => readFile(new URL(`../public/${path}`, import.meta.url), 'utf8');
  const expectedBuild = JSON.parse(await readFile(new URL('../node_modules/@openmixture/runtime/build-info.json', import.meta.url)));
  expect(expectedBuild.runtimeVersion).toBe('0.11.0-alpha.0');
  const materials = {
    'brick-paving': {
      recipes: ['brick-structure', 'masked-layer-surface'],
      channels: ['baseColor', 'normal', 'roughness', 'height'], exposed: 10,
      v2: { columns: 6, rows: 12, seed: 7 },
      v1: { heightColumns: 6, colorColumns: 6, maskColumns: 6, heightRows: 12, colorRows: 12, maskRows: 12,
        heightSeed: 7, colorSeed: 7, maskSeed: 7 },
    },
    'woven-fabric': {
      recipes: ['weave-structure', 'directional-grain', 'masked-layer-surface'],
      channels: ['baseColor', 'metallic', 'roughness', 'height', 'normal'], exposed: 19,
      v2: { warpCount: 12, warpSeed: 7 },
      v1: { warpshare_warpCount: 12, coverage_warpCount: 12, height_warpCount: 12, warpSeed: 7 },
    },
  };
  const inputs = {};
  for (const [material, spec] of Object.entries(materials)) {
    inputs[material] = {
      parent: await read(`recipes/${material}.mix`),
      accepted: await read(`${material}/material.mix`),
      recipes: await Promise.all(spec.recipes.map(id => read(`recipes/${id}.mixrecipe`))),
    };
  }
  await page.goto('tests/contracts.html');
  await page.waitForFunction(() => Boolean(window.sdk));
  const evidence = await page.evaluate(async ({ materials, inputs }) => {
    const runtime = await window.sdk.loadRuntime();
    const rows = [], checks = {};
    for (const [material, spec] of Object.entries(materials)) {
      const { parent, accepted, recipes } = inputs[material];
      const bare = runtime.validate(parent);
      if (bare.ok || bare.diagnostics[0].code !== 'MIX_FORMAT_UNSUPPORTED_VERSION') throw Error(`${material}: v2 without recipes must fail by version`);
      const missing = runtime.validate(parent, { recipes: recipes.slice(1) });
      if (missing.ok || missing.diagnostics[0].code !== 'MIX_RECIPE_MISSING') throw Error(`${material}: missing recipe must be reported`);
      const unused = runtime.validate(accepted, { recipes });
      if (unused.ok || unused.diagnostics.some(d => d.code !== 'MIX_RECIPE_UNUSED')) throw Error(`${material}: v1 with recipes must report unused recipes`);
      const inspection = runtime.inspect(parent, { recipes, channels: spec.channels });
      if (inspection.documentVersion !== 2) throw Error(`${material}: documentVersion`);
      const ids = inspection.exposedParameters.map(p => p.id);
      if (ids.length !== spec.exposed || new Set(ids).size !== ids.length) throw Error(`${material}: exposed IDs ${ids}`);
      checks[material] = { closure: inspection.recipes.map(r => ({ id: r.id, recipeVersion: r.recipeVersion, sha256: r.sha256 })), exposed: ids };
      const v1Inspection = runtime.inspect(accepted, { channels: spec.channels });
      if ('recipes' in v1Inspection) throw Error(`${material}: v1 inspection must not report recipes`);
      const gpu = await runtime.createGpu({ powerPreference: 'high-performance' });
      try {
        for (const size of [[64, 64], [257, 129]]) for (const [label, v2, v1] of [['default', {}, {}], ['varied', spec.v2, spec.v1]]) {
          const parentResult = await gpu.render(parent, { size, channels: spec.channels, overrides: v2, recipes });
          const acceptedResult = await gpu.render(accepted, { size, channels: spec.channels, overrides: v1 });
          for (const channel of parentResult.channels) {
            const other = acceptedResult.channels.find(c => c.channel === channel.channel);
            if (!other || other.pixels.length !== channel.pixels.length || channel.pixels.some((v, i) => v !== other.pixels[i])) {
              throw Error(`${material} ${size} ${label}: ${channel.channel} differs from the accepted graph`);
            }
          }
          rows.push({ material, size, case: label, channels: parentResult.channels.length,
            parentPlanHash: parentResult.plan.hash, acceptedPlanHash: acceptedResult.plan.hash, identical: true });
        }
      } finally { await gpu.destroy(); }
    }
    return { build: runtime.getBuildInfo(), checks, rows };
  }, { materials, inputs });
  expect(evidence.build).toEqual(expectedBuild);
  expect(evidence.rows).toHaveLength(8);
  expect(evidence.checks['brick-paving'].closure.map(r => r.id)).toEqual(['brick-structure', 'masked-layer-surface']);
  expect(evidence.checks['woven-fabric'].closure.map(r => r.id)).toEqual(['directional-grain', 'masked-layer-surface', 'weave-structure']);
  const content = JSON.stringify({ ...evidence, browser: browser.version() }, (_key, value) => typeof value === 'bigint' ? value.toString() : value, 2);
  await writeFile(testInfo.outputPath('recipes-evidence.json'), content);
  await testInfo.attach('recipes-evidence', { body: content, contentType: 'application/json' });
});
