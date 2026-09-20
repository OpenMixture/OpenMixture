import { defineConfig } from 'vite';

export default defineConfig(({ mode }) => ({
  base: '/consumer/',
  build: {
    outDir: mode === 'qualification' ? 'test-dist' : 'dist',
    rollupOptions: {
      input: mode === 'qualification'
        ? ['index.html', 'tests/contracts.html'] : 'index.html',
    },
  },
}));
