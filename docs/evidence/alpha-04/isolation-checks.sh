set -euo pipefail
mkdir -p /tmp/home /consumer/work
node --version
npm --version
test "$(node --version)" = v24.20.0
test "$(npm --version)" = 11.19.0
test -z "$(git status --porcelain)"
node --input-type=module <<'JS'
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
const blocked=['/mnt/d/Coding/OpenMixture/README.md','/mnt/d/Coding/OpenMixtureStudio/README.md','/mnt/d/Coding/OpenMixture/tmp/alpha04-studio/README.md'];
for(const path of blocked) assert.throws(()=>readFileSync(path),{code:'ENOENT'});
for(const name of ['cargo','rustc']) assert.equal(spawnSync(name,['--version']).error?.code,'ENOENT');
const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
writeFileSync('work/isolation.json',JSON.stringify({schemaVersion:1,sourceCheckoutsUnavailable:blocked,rustUnavailable:true,productRevision:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),archiveSha256:hash('vendor/openmixture-runtime-0.1.0-alpha.0.tgz'),lockSha256:hash('package-lock.json'),node:process.version,mountInfo:readFileSync('/proc/self/mountinfo','utf8')},null,2));
JS
npm ci
npm run check
npx playwright install chromium
npm run test:browser -- --max-failures=1
npm run test:deployment
npm run test:studio -- /reference /consumer/work/player
printf '%s\n' 'All isolated consumer execution gates passed; native comparison remains separate.'
