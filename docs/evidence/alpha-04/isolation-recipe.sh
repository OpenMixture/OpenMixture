#!/usr/bin/env bash
set -euo pipefail
source_root=/mnt/d/Coding/OpenMixture/tmp
run_root=$(mktemp -d /tmp/studio-alpha04-isolation.XXXXXX)
printf '%s\n' "$run_root" > "$source_root/alpha04-isolation-path.txt"
mkdir "$run_root/toolchain" "$run_root/consumer" "$run_root/reference"
tar -xJf "/mnt/d/Coding/OpenMixtureStudio/work/p1/linux-tools/node-v24.20.0-linux-x64.tar.xz" --strip-components=1 -C "$run_root/toolchain"
cp -a "$source_root/alpha04-isolated-source/." "$run_root/consumer/"
mkdir -p "$run_root/consumer/work"
cp -a /tmp/studio-p1-isolation.UGLzk9/consumer/work/browsers "$run_root/consumer/work/browsers"
cp -a "$source_root/alpha04-native/." "$run_root/reference/"
cp /etc/resolv.conf "$run_root/resolv.conf"
git -C "$run_root/consumer" config core.autocrlf true
cat > "$run_root/checks.sh" <<'CHECKS'
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
CHECKS
set +e
bwrap --ro-bind /usr /usr --symlink usr/bin /bin --symlink usr/lib /lib --symlink usr/lib64 /lib64 \
 --tmpfs /etc --ro-bind /etc/ssl /etc/ssl --ro-bind /etc/fonts /etc/fonts \
 --ro-bind /etc/ld.so.cache /etc/ld.so.cache --ro-bind /etc/nsswitch.conf /etc/nsswitch.conf \
 --ro-bind /etc/passwd /etc/passwd --ro-bind /etc/group /etc/group --ro-bind /etc/hosts /etc/hosts \
 --ro-bind "$run_root/resolv.conf" /etc/resolv.conf --ro-bind /etc/os-release /etc/os-release \
 --proc /proc --dev /dev --tmpfs /tmp --unshare-user --unshare-pid --unshare-ipc --unshare-uts --die-with-parent \
 --bind "$run_root/toolchain" /toolchain --bind "$run_root/consumer" /consumer \
 --ro-bind "$run_root/reference" /reference --ro-bind "$run_root/checks.sh" /checks.sh \
 --clearenv --setenv PATH /toolchain/bin:/usr/bin:/bin --setenv HOME /tmp/home --setenv LANG C.UTF-8 \
 --setenv PLAYWRIGHT_BROWSERS_PATH /consumer/work/browsers --setenv MIXTURE_TEST_PORT 4190 \
 --setenv MIXTURE_BROWSER_ARGS '["--use-angle=swiftshader","--use-webgpu-adapter=swiftshader"]' \
 --chdir /consumer /bin/bash /checks.sh > "$run_root/run.log" 2>&1
result=$?
set -e
mkdir "$source_root/alpha04-isolation"
cp "$run_root/run.log" "$source_root/alpha04-isolation/run.log"
cp "$run_root/checks.sh" "$source_root/alpha04-isolation/checks.sh"
cp "$run_root/consumer/work/isolation.json" "$source_root/alpha04-isolation/isolation.json"
if test -d "$run_root/consumer/test-results"; then cp -a "$run_root/consumer/test-results" "$source_root/alpha04-isolation/test-results"; fi
if test -d "$run_root/consumer/work/player"; then cp -a "$run_root/consumer/work/player" "$source_root/alpha04-isolation/player"; fi
printf '%s\n' "$result" > "$source_root/alpha04-isolation/exit-code.txt"
tail -n 20 "$run_root/run.log"
exit "$result"
