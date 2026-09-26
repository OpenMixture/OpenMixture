# npm Alpha publication — 2026-09-20

English | [简体中文](./README.zh-CN.md)

`@openmixture/runtime@0.1.0-alpha.0` was published publicly at `2026-09-20T07:26:14.490Z` from the exact ALPHA-04 tarball, without rebuilding. The user authorized publication and creation of the free `openmixture` npm organization; `krapnik` is its owner. npm web security-key verification completed the publication.

The package is available from [npm](https://www.npmjs.com/package/@openmixture/runtime/v/0.1.0-alpha.0). Use the exact version:

```sh
npm install --save-exact @openmixture/runtime@0.1.0-alpha.0
```

[Registry metadata at publication](./registry-at-publication.json) and the [download receipt](./registry-pack.json) bind the 348066-byte archive. Its SHA-256 remains `a9bcfe8d849f9fb9982a750dd99d026807fdf6453b5be491deeda90e99a2c6ae`; SHA-512 equals both the registry integrity and the [consumer lock](./windows-lock.json). All 12 installed files match the accepted archive, as recorded in [Windows installation](./windows-install.json). Producer `82b74707b2a8a998190e2f28b16f91fb9614486a` and build ID `sha256:94f9cc455fcd5f805942b0196b39a276782ce0e504772c3a140ea8ad85814a28` are unchanged. [Candidate notes](../../browser-alpha-candidate.md) retain compatibility and support limits.

Publication used `npm publish <retained-tarball> --access public --tag alpha --ignore-scripts --registry=https://registry.npmjs.org/`. npm also created `latest` on the first publication. After successful security-key authentication, `npm dist-tag rm @openmixture/runtime latest` returned HTTP 400 from the registry; [final readback](./registry-readback.json) therefore retains both `alpha` and `latest` pointing to the Alpha. The first attempt expired before authentication. No alias removal or stable release is claimed. Prefer the exact version above; an unqualified install currently also resolves to this prerelease.

## Exact registry consumption

The immutable published tarball retains its build-time README wording (“unpublished”). Current repository package documentation and this release record supersede that historical status text; the archive was not repacked to change it. Future CI candidates are separate builds, not replacements for this published archive.

Studio revision `87ded9351e1c426e03aa7fb2b4c641f32399b85b` changes only the exact runtime dependency and its resolved registry URL. Integrity and executable bytes are unchanged. A fresh npm cache was used on Windows; Linux installs inside a fresh filesystem-isolated consumer with source checkouts and Rust unavailable. The retained vendor archive is an identity/test fixture, not the installation source or a fallback.

Both Windows and isolated Linux passed public types, 20 unit tests, production builds, **52/52 browser contracts**, normal static deployment, and fresh Player execution of the seven ALPHA-04 saved materials with **28/28 channel comparisons**. The [isolation recipe](./linux-isolation.sh), [boundary receipt](./linux-isolation.json) and [registry installation](./linux-install.json) identify the Linux run. Node `24.20.0`, npm `11.19.0`, controlled Chromium `153.0.8010.12`, explicit Linux SwiftShader flags and recorded Windows browser flags match the bounded candidate recipe. Windows [ordinary Chrome evidence](./ordinary/receipt.json) additionally passes edit/repair/history/save/reopen/four-channel PNG/disposal and the separately injected unavailable-GPU case.

These are new executions of registry-installed bytes. Saved material inputs and native references are reused from ALPHA-04, not reauthored or rerendered native references. They retain authoring revision `6b2d53e3de16b21725b2a4359a2263f98671a6f9`; the new Player receipts identify the registry consumer revision. No tolerances, shaders or baselines changed. No broader browser/hardware support, Rust publication, hosted trial redeployment or human trial result is claimed.

## Retention and reproduction

[Replay index](./replay-index.json) binds all 104 Windows/Linux replay files. 88 files, including repeated PNGs and comparisons, are byte-identical to the existing [ALPHA-04 bundle](https://github.com/OpenMixture/OpenMixture/blob/e249d57d9ce78e73bbe8da554a6fcce0f3375303/docs/evidence/alpha-04/saved-file-bundles.tar.gz) and reuse those retained bytes. The other 16 files are retained under `replay/`; ordinary screenshots/downloads are under `ordinary/`. [File hashes](./files.json) bind the new retained content. Full routine logs, browser profiles and npm caches remain ignored local output, not long-term evidence.

```sh
node docs/evidence/npm-alpha/verify.mjs
node docs/evidence/npm-alpha/verify.mjs tmp/npm-alpha-replay
cargo xtask studio-material-check tmp/npm-alpha-replay/native tmp/npm-alpha-replay/windows
cargo xtask studio-material-check tmp/npm-alpha-replay/native tmp/npm-alpha-replay/linux
```

The final Node command requires a fresh output directory and verifies the old bundle before overlaying new registry-consumer receipts. For fresh execution, check out the Studio revision, run `npm ci`, `npm run check`, `npm run test:browser`, `npm run test:deployment`, `npm run test:ordinary -- chrome <new-output>`, and `npm run test:studio -- <detached-native> <new-player>`; compare with the engine command above. The Linux recipe records this host's tool paths and must be provisioned accordingly. Registry metadata and dist-tags are mutable; historical snapshots do not replace a fresh remote readback.
