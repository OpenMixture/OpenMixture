# MAT-01 tooling integration — material acceptance pending

English | [简体中文](./README.zh-CN.md)

[PR #52](https://github.com/OpenMixture/OpenMixture/pull/52) merged as `0611d7273e368b12628bc827627779ea338dbb92` on 2026-09-23 at 03:56:42 UTC. The two nodes were integrated through PRs #50–51; this change integrates public-consumer qualification tools and retained candidate evidence. Human visual acceptance remains pending; MAT-02 implementation is not opened and no package is published.

## Tested source and gates

All six required pre-merge checks passed for PR head `1dfb879b86998f038841931a6297676221d7ac4d`: [three CPU platforms](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446565), [software GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446393), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446500), and [Chromium](https://github.com/OpenMixture/OpenMixture/actions/runs/35771446555), each attempt 1. The actual browser/runtime checkout is clean synthetic merge `9345ac22d4da7dc6d55a2d62adc295b12b3dd501`, not the PR head or later integration commit.

The [SDK receipt](./sdk-qualification.json), [original material binding](./browser-qualification.json), and [explicit noise-v2 binding](./noise-v2-qualification.json) all pass. They identify unpublished archive SHA-256 `302b5220c66b86e3e5102bae5d54f705c8c678c64c020e8924333d07a72f6181` and runtime build `sha256:a4dbb21caada8e9d11260132cfd2c46f3b3aa394bc07115c17b97f478e29366f`. The prior [dirty-source failure](../ci-failure-4bf/README.md) remains failed; the new successful binding verifies the Cargo output-path repair without weakening cleanliness checks.

[Brick comparison](./brick-comparison.json) and its [Native matrix](./brick-native.json) retain twenty cases and eighty channel comparisons: maximum component difference 0, all repeat/package comparisons exact. This is recorded Linux software/Chromium coverage, not new hardware coverage. Both receipts retain `materialAccepted: false`. The independently sourced [Windows evidence and human review images](../README.md) retain their original source identity and pending human decision.

Post-merge main checks are separate and were still running when this record was prepared: [CPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332292), [GPU](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332264), [WASM/npm](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332218), [Chromium](https://github.com/OpenMixture/OpenMixture/actions/runs/35816332263). Pre-merge results do not claim these runs passed.

## Retention

Browser artifact `chromium-material-matrix`, ID `10716612741`, is 182,294,555 bytes with SHA-256 `501782b8f78ce138513911eace06caf9f9f10903c07eecbaf5721d6bcd771dea`; service expiry is 2026-10-22 at 19:55:15 UTC. It was downloaded and its digest verified on 2026-09-23. The selected receipts above are copied byte-for-byte into Git. Complete logs, archive bytes, repeated PNGs and other reports remain temporary local/CI artifacts; linked hashes inside receipts do not promise that those full bundles remain available after expiry. This record supports tooling integration and the stated measurements, not full long-term reinspection of every CI pixel. Existing human-review pixels remain retained separately.

Reproduce with the [fixture guide](../../../../fixtures/materials/brick-paving/README.md) and the workflows at the tested revision. New runs get new source/archive identities. No format, node semantic, migration policy or publication state changes in this record.
