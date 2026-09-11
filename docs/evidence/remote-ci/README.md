# Remote CI acceptance

English | [简体中文](./README.zh-CN.md)

**Accepted, 2026-09-12 (Asia/Singapore), revision `8b43c84`.** The three-platform CPU matrix and Linux pinned SwiftShader smoke/package/material/2K gates all pass on the same revision. The repository is associated with `OpenMixture/OpenMixture` using the existing `gh` account. CI fixes are on `codex/remote-ci`. Publication remains disabled and no release tag or merge is included.

## Recorded runs

| Revision | Verification | Result |
|---|---|---|
| `cda3f4b` | [Initial CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34617436927) | Windows could not replace the running `xtask.exe`; Linux passed; macOS was superseded. |
| `9f6918a` | [CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34618010038) | Linux/macOS passed. Windows reached package verification and rejected an equivalent path spelling. |
| `9f6918a` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34618010162) | Smoke, packaged consumption, three 1K materials and 2K trace passed. |
| `b9d2ca5` | [CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34619090585) | All three platforms passed, including isolated packages. |
| `b9d2ca5` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34619090576) | Concurrent node-test process terminated with SIGSEGV; material/2K steps did not run. |
| `8b43c84` | [CPU matrix](https://github.com/OpenMixture/OpenMixture/actions/runs/34622547271) | All three platforms and isolated package consumers passed. |
| `8b43c84` | [Linux SwiftShader](https://github.com/OpenMixture/OpenMixture/actions/runs/34622547778) | Serial GPU smoke, packaged consumers, all three 1K materials and 2K trace passed. |

[CPU run](./cpu-run.json), [artifact identities](./cpu-artifacts.json), [outer package receipts](./cpu-packages.json) and [GPU failure run](./gpu-failed-run.json) retain the exact revisions and results. GitHub artifacts contain the complete logs and reports; their retention is finite. Package staging paths refer to the original runners and successful staging trees were removed.

## Repairs and limits

- `9f6918a` builds the xtask launcher separately from the executable rebuilt by workspace integration tests, preserving all tests on Windows.
- `b9d2ca5` compares canonical manifest/target paths, accepting equivalent Windows verbatim paths while rejecting missing files, wrong package identities and targets outside their package. A filesystem regression covers canonical versus ordinary paths and existing outside targets.
- `8b43c84` runs independent GPU tests serially, retaining every test and each test's internal independent-context checks. The Linux signal alone does not establish the driver's root cause or certify arbitrary concurrent device destruction.
- Golden/trace reports no longer hardcode that remote CI is deferred. Their descriptive `remoteCi` field refers readers to the actual CI run/revision. Original reports, including the stale text in the first successful Linux run, remain unchanged historical evidence.
- SwiftShader took about 16 and 29 minutes to build in the first two completed Linux GPU runs. An exact driver-build cache covers platform, tool versions and the pinned workflow/setup script. Source verification/configuration/build and all acceptance tests still run after a cache hit. A successful build is saved before tests; no partial-key restore is used.

The runtime crates, shaders, lockfiles, eleven-node vocabulary and accepted goldens are unchanged. The failed parallel run is retained alongside the successful serial run; the prescribed remote gates are now closed. This acceptance does not establish the root cause of the earlier SIGSEGV or certify arbitrary concurrent device destruction. This work does not start M5 or authorize publication.

## Accepted evidence and reproduction

[Accepted CPU run](./accepted-cpu-run.json), [CPU artifacts](./accepted-cpu-artifacts.json), [package receipts](./accepted-cpu-packages.json), [accepted GPU run](./accepted-gpu-run.json) and [GPU artifacts](./accepted-gpu-artifacts.json) identify the successful revision and retained evidence. The failed process's [stdout](./failures/gpu-tests.stdout.log), [stderr](./failures/gpu-tests.stderr.log), [doctor](./failures/doctor.json) and [environment](./failures/software-environment.log) are preserved without rewriting the failure.

The [successful command log](./accepted-gpu.log) retains smoke, material and trace results. The accepted Linux workload uses explicit Vulkan/SwiftShader, the existing material gates and unchanged baselines. The 2K descriptor budget remains 512 MiB; it is not a physical VRAM measurement. M0/M1 remote clean-checkout gates and the M4 remote acceptance gap are closed for this documented matrix. Other hardware remains outside this certification; release and M5 decisions remain separate.

```bash
gh run view 34622547271 --repo OpenMixture/OpenMixture --log
gh run view 34622547778 --repo OpenMixture/OpenMixture --log
gh run download 34622547778 --repo OpenMixture/OpenMixture --name swiftshader-material-evidence --dir tmp/remote-ci/accepted-gpu
cargo xtask check
```

The repository GPU workflow contains the complete clean-runner setup and commands. No test or golden gate is skipped on cache hits.
