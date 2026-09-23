# Node fixtures

English | [简体中文](./README.zh-CN.md)

PR-007 provides defaults, boundaries, nontrivial pixels, and invalid inputs for every M2 node:

- [constant-scalar](./constant-scalar/README.md)
- [constant-color](./constant-color/README.md)
- [checker](./checker/README.md)
- [levels](./levels/README.md)
- [blend](./blend/README.md)
- [material-output](./material-output/README.md)

Each directory has readable `.mix` input and a `cases.json` manifest. Fixed RGBA8 sample coordinates/values include their tolerance; invalid cases specify a stable diagnostic code. Public-contract validation runs without a GPU; `cargo xtask test-node <id>` explicitly runs the selected node's GPU cases. Smoke runs all of them and records actual adapter/plan evidence. PR-009 adds explicit seeded noise and its color/normal consumers:

The existing [checker PNG/raw golden](./checker/README.md) and SwiftShader provenance are unchanged. No test command overwrites pixel baselines. See [graph execution and evidence](../../docs/graph-rendering.md), [node workflow](../../AGENTS.md), and [implementation train](../../INITIAL_PRS.md).

- [fractal-noise](./fractal-noise/README.md)
- [gradient-map](./gradient-map/README.md)
- [height-to-normal](./height-to-normal/README.md)

PR-010 adds periodic scalar resampling and literal interpolation/rotation probes:

- [transform-2d](./transform-2d/README.md)
- [warp](./warp/README.md)

- [scalar-morphology](./scalar-morphology/README.md)

- [scalar-subtract](./scalar-subtract/README.md)
