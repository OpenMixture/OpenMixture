# MAT-02 recipe revision 2 — relative height

English | [简体中文](./mat-02-relative-height.zh-CN.md)

Status: candidate recipe amendment, not material acceptance. This is a fixture/caller change under MAT-02, owned by repository qualification. Core semantics, the sole wgpu executor, node versions, `.mix`/`.mixpack`, public APIs and source package versions are unchanged. Existing documents keep their exact meaning; no automatic migration.

The revision 1 graph placed the substrate at 0.2, paint at 0.2+thickness and rust at 0.2+relief. Because each intermediate is stored as half float, this positive offset consumes precision needed by small height variations. A Windows GT 1030 Vulkan diagnostic at 1K captured actual W/R and final height/normal: removing only the offset increased unique height values from 394 to 4592; central-neighbor difference RMS error against unrounded scalar composition from the same masks decreased from about 4.74e-5 to 5.98e-6. Raw normal components differed by up to 0.0234375 before PNG encoding. These local diagnostic measurements motivate the candidate; they do not establish accepted appearance or cross-platform qualification.

## Selected correction and compatibility

[The new graph](../fixtures/materials/painted-metal/material.mix) changes exactly three defaults: coatingHeight.outputMin 0.25→0.05, coatingHeight.outputMax 0.2→0, rustHeight.value 0.225→0.025. Caller overrides use paintHeight=paintThickness and rustHeight=rustRelief. Final height is mix(paintThickness*(1-W),rustRelief,R), subject to existing intermediate half rounding; normal still derives from that same final height with unchanged strength. Color, metallic, roughness, masks, seeds, topology and pass count are unchanged.

Endpoint heights become paintThickness, 0 and rustRelief for intact/exposed/rusted. Absolute height and resulting quantized normal pixels intentionally change. No constant is added back before normal derivation. This is an explicit new fixture revision, not reinterpretation of an existing node or a silent rewrite of a saved document. A consumer needing the old height datum must retain its original graph; new fixture adoption is explicit.

Preserve the [revision 1 plan](../fixtures/materials/painted-metal/qualification-plan-v1.json), [design](../fixtures/materials/painted-metal/graph-design-v1.json), [performance graph](./evidence/perf-mat-before/material.mix), all old receipts and goldens. The performance-reuse consumer continues to render the original graph. Current painted-material requests carry recipeRevision=2 and bind the new exact source/plan hashes. A CPU regression restricts graph changes to the three constants and rejects changes to other frozen gates.

## Required verification

Re-run all seven presets/four sizes/five channels through public Native and browser consumers, exact repeat/package checks, endpoint references, scalar composition and normal replay/periodic tests, original and migrated material regressions, frozen memory/time and default downsample limits, stress measurements, PBR comparison and human review, plus all six required CI checks. Prior revision 1 passes never qualify revision 2 pixels. Preserve before/after images and their distinct producer identities.

This correction addresses measured offset sensitivity. It does not by itself repair broad rounded wear boundaries, weak rust coverage, mask quantization or undersampling. Do not claim those defects fixed, reduce normal strength to hide them, or declare MAT-02 accepted from precision metrics alone. MAT-03/04 remain in the roadmap.
