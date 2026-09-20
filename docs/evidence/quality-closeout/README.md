# Quality policy closeout — 2026-09-20

English | [简体中文](./README.zh-CN.md)

PR #19 merged the v2 quality policy. The maintainer then requested removing superseded rules and research from this new project's active workflow. Browser reports now use schema 3 with three current gates; Studio reports use schema 2 and the identical frozen profile. No shader, native golden, material input, profile value or metric implementation changed.

[Verification summary](./summary.json) binds the existing complete browser and Studio datasets. Offline replay reproduced all 132 browser-channel v2 metrics, structure/causality results and case relationships exactly; all 28 Studio channels passed the shared profile and unchanged saved-file checks. These are new comparisons of recorded pixels, not new browser executions or delivery-candidate qualification. Original reports retain their original outcomes.

Focused verification includes six quality metric tests, two Studio tests and nine JavaScript candidate/ordinary-browser tests. The Studio regression proves that a one-code change admitted by the material budget is rejected by exact checker rules, for all four channels. Candidate tests reject report schemas 1/2, incomplete matrices and missing/failed current gates. The full repository check and current-package browser CI remain required for integration.

Reproduce browser replay using the [complete decoded texture verifier](../browser-quality-v2/verify.py), then run `cargo xtask browser-material-check` for each of its three browser directories. Reconstruct the 88 retained Studio files with the hash-checking snippet in the [Studio evidence record](../studio-qualification/README.md), then run `cargo xtask studio-material-check` on its native/player directories. Use fresh output directories; leave the original records untouched. Python reconstruction requires NumPy/Pillow for browser pixels. Run focused tests with:

```sh
cargo test --locked -p xtask browser_quality
cargo test --locked -p xtask golden::studio
node --test scripts/browser-runtime/candidate.test.mjs scripts/browser-runtime/default-browser.test.mjs
cargo xtask check
```

PRs #13–#18 were closed without adopting their shaders or proposed compatibility contract. Removed early ALPHA-03 experiments and the old tolerance file remain retrievable from commit `efdac411198bfa87b7eba7ff9e588330ec08baf9`; retained historical links use that immutable revision. Active code no longer depends on these files. The [current closeout plan](../../browser-alpha.md) identifies Alpha package delivery and browser branch-protection policy as subsequent tasks.
