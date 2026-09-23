# MAT-01 CI source-cleanliness failure

English | [简体中文](./README.zh-CN.md)

Browser run [35766417215, attempt 1](https://github.com/OpenMixture/OpenMixture/actions/runs/35766417215) failed for PR head `4bf5e9b65d2153aa2fe8ef46459839b4a7e6a8f3`; the actual CI checkout/runtime revision was synthetic merge `2d1ebf0151c63d36107ba3f81788b61082c5af48`, as retained in the [Native manifest](./native-manifest.json). The candidate SDK, Scalar/resource/package/brick comparisons, installed product contracts, original material quality and deployment checks passed. Final candidate binding rejected `native.engineDirty === true`; the subsequent explicit noise-v2 matrix was skipped. This is a failed overall browser qualification, not an accepted stage.

## Cause and repair

The newly added brick comparison invoked an independent Cargo workspace without `--target-dir`, generating untracked `examples/native-consumer/target/`. Native preparation later captured the resulting dirty status. A private `.git/info/exclude` entry masked the path on the development checkout; the repository's `.gitignore` intentionally owns `/target/`, not that nested path. The xtask brick entry point had the same omission.

Both invocations now explicitly use `--target-dir target/native-consumer`, matching the existing independent consumer commands. No dirty-source assertion, pixel tolerance, candidate identity check or required check is removed. The manual fixture command uses the same path and all required features.

The regression in [consumer.test.mjs](../../../../scripts/browser-runtime/consumer.test.mjs) creates a fresh Git repository with the tracked root target ignore, clears private/global exclusions, and invokes the real brick orchestration against a tiny dependency-free Rust consumer. Before the fix its final status assertion fails with `?? examples/native-consumer/target/`; after the fix it remains clean. The stub replaces only GPU work, so this test proves build-output routing, not material pixels. The full nineteen Node orchestration tests pass. Actual Native material and full repository checks, then current-revision CI, remain independently required.

## Retention and scope

The [first failed-step log](./failed-step.log) preserves the original binding error. The original CI artifact is `chromium-material-matrix`, ID `10714115961`, 128,635,403 bytes, service digest `sha256:6e472c1cf771d4014fd5f8083394dd5269e13e09fe499631d017bd6416350a64`; service expiry is 2026-10-22 18:51:54 UTC. It contains the passing intermediate brick comparison and failed final binding. Complete routine output is temporary; this record does not claim permanent retention of the full archive or reinterpret skipped migration checks as passes. The failed run is never rewritten as successful by a later repair.
