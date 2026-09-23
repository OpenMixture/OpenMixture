# MAT-02 subtraction node evidence

English | [简体中文](./README.zh-CN.md)

This record binds clean source `e7b25c4e36f6d7a2052fdba11d8cdfffad762a9d`, unpublished Rust 0.7.0 / browser 0.7.0-alpha.0. It qualifies the recorded Windows GT 1030 subtraction cases only; painted-metal acceptance, final integration checks and publication remain separate. The [receipt](./receipt.json) binds source and retained file hashes.

The exact [candidate archive](./archive-receipt.json) passed all 19 [browser consumer tests](./browser-qualification.json). The 64 [browser cases](./browser.json) cover eight input pairs, four dimensions and direct/amplified outputs. The [Vulkan comparison](./comparison-vulkan.json) and [DX12 comparison](./comparison-dx12.json) have maximum component difference zero, exact plan hashes and exact repeats. Complete [Native Vulkan pixels](./native-vulkan.json) and [Native DX12 pixels](./native-dx12.json) retain adapter identities. Amplification verifies that 0.5 minus 0.499755859375 survives as 1/4096 before RGBA8 conversion. These arithmetic probes do not assert human PBR approval.

Local `cargo xtask check` passed, including isolated package consumption and 281 document links, before this evidence-only update. Focused Vulkan probes passed during implementation; an additional DX12 raw-half run passed on clean `a49edb031f6381a1143c18b151a79493a613609a` after merging the prerequisite main commit. Those ordinary logs remain in ignored `tmp/subtract-check-initial.log` and `tmp/subtract-dx12-a49edb0.log`. Final PR checks certify their own exact checkout, not these earlier candidate bytes.

Reproduce at the bound source with `node scripts/browser-runtime/build.mjs`, then `node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime <fresh-browser-output>` with `MIXTURE_BROWSER_CHANNEL=chrome`. Run `node scripts/browser-runtime/check-subtract.mjs <fresh-browser-output> <fresh-comparison-output>` separately with `MIXTURE_GPU_BACKEND=vulkan` or `dx12` and `MIXTURE_GPU_SOFTWARE=0`. Follow the [node fixture guide](../../../fixtures/nodes/scalar-subtract/README.md) for raw-half checks. Set environment variables using `$env:` in PowerShell.

Selected original JSON bytes and complete pixels remain in Git. Original package bytes, build products and routine logs remain ignored or finite-retention CI artifacts; no durable external archive is claimed. This record does not replace the historical [composed-subtraction counterexample](../mat-02-subtraction/README.md).
