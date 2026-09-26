import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: process.env.MIXTURE_RESOURCE_TESTS !== '0' ? '*.spec.mjs' : 'sdk.spec.mjs',
  testIgnore: [
    ...(process.env.MIXTURE_PAINTED_TESTS === '1' ? [] : ['painted.spec.mjs']),
    ...(process.env.MIXTURE_REUSE_TESTS === '1' ? [] : ['reuse.spec.mjs']),
    ...(process.env.MIXTURE_ASSET_TESTS === '1' ? [] : ['assets.spec.mjs']),
    ...(process.env.MIXTURE_SUBTRACT_TESTS === '1' ? [] : ['subtract.spec.mjs']),
    ...(process.env.MIXTURE_MORPHOLOGY_TESTS === '1' ? [] : ['morphology.spec.mjs']),
    ...(process.env.MIXTURE_BRICK_TESTS === '1' ? [] : ['brick.spec.mjs']),
  ],
  workers: 1,
  retries: 0,
  timeout: 60_000,
  reporter: [['list'], ['json', { outputFile: 'test-results/report.json' }]],
  use: {
    baseURL: 'http://127.0.0.1:4175/consumer/',
    headless: true,
    channel: process.env.MIXTURE_BROWSER_CHANNEL ?? 'chromium',
    launchOptions: {
      args: ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist',
        ...JSON.parse(process.env.MIXTURE_BROWSER_ARGS ?? '[]')],
    },
  },
  webServer: {
    command: 'npm run preview -- --outDir test-dist --port 4175 --strictPort',
    url: 'http://127.0.0.1:4175/consumer/',
    reuseExistingServer: false,
  },
});
