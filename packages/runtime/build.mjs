import { rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// Resolve only this package's generated output; stale emitted files cannot ship.
await rm(new URL('./dist/', import.meta.url), { recursive: true, force: true });
const result = spawnSync(process.execPath, [
  fileURLToPath(new URL('./node_modules/typescript/bin/tsc', import.meta.url)),
  '-p', fileURLToPath(new URL('./tsconfig.json', import.meta.url)),
], { stdio: 'inherit' });
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
