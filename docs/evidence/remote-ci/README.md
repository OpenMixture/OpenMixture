# Remote CI acceptance

English | [简体中文](./README.zh-CN.md)

**In progress, 2026-09-12 (Asia/Singapore).** The repository is associated with `OpenMixture/OpenMixture` using the existing `gh` account. CI fixes are on `codex/remote-ci`. Publication remains disabled and no release tag or merge is included.

## Recorded runs

| Revision | Verification | Result |
|---|---|---|
| `cda3f4b` | [Initial CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34617436927) | Windows could not replace the running `xtask.exe`; Linux passed; macOS was superseded. |
| `9f6918a` | [CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34618010038) | Linux/macOS passed. Windows reached package verification and rejected an equivalent path spelling. |
| `9f6918a` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34618010162) | Smoke, packaged consumption, three 1K materials and 2K trace passed. |
| `b9d2ca5` | [CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34619090585) | All three platforms passed, including isolated packages. |
| `b9d2ca5` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34619090576) | Concurrent node-test process terminated with SIGSEGV; material/2K steps did not run. |

[CPU run](./cpu-run.json), [artifact identities](./cpu-artifacts.json), [outer package receipts](./cpu-packages.json) and [GPU failure run](./gpu-failed-run.json) retain the exact revisions and results. GitHub artifacts contain the complete logs and reports; their retention is finite. Package staging paths refer to the original runners and successful staging trees were removed.

## Repairs and limits

- `9f6918a` builds the xtask launcher separately from the executable rebuilt by workspace integration tests, preserving all tests on Windows.
- `b9d2ca5` compares canonical manifest/target paths, accepting equivalent Windows verbatim paths while rejecting missing files, wrong package identities and targets outside their package. A filesystem regression covers canonical versus ordinary paths and existing outside targets.
- The follow-up runs independent GPU tests serially, retaining every test and each test's internal independent-context checks. The Linux signal alone does not establish the driver's root cause or certify arbitrary concurrent device destruction.
- Golden/trace reports no longer hardcode that remote CI is deferred. Their descriptive `remoteCi` field refers readers to the actual CI run/revision. Original reports, including the stale text in the first successful Linux run, remain unchanged historical evidence.
- SwiftShader took about 16 and 29 minutes to build in the first two completed Linux GPU runs. An exact driver-build cache covers platform, tool versions and the pinned workflow/setup script. Source verification/configuration/build and all acceptance tests still run after a cache hit. A successful build is saved before tests; no partial-key restore is used.

The runtime crates, shaders, lockfiles, eleven-node vocabulary and accepted goldens are unchanged. CPU checks and first-run GPU success do not conceal the later SIGSEGV; the serial harness must pass remotely before closing the remaining gate. This work does not start M5 or authorize publication.
