# Brick pattern local visual review

English | [简体中文](./README.zh-CN.md)

These are local Windows hardware previews from the new brick graph, not accepted cross-runtime or archive qualification. [Receipt](./local.json) records the graph hash, selected adapter, plan identities and twelve original PNG hashes. The source contract and exact generation commands are in the [node guide](../../brick-pattern.md). The three variants use defaults, layout=aligned and gap=0.2 at 1024x1024, requesting baseColor,height,normal,roughness.

![Variants: color, height, normal, roughness](./contact.png)

Agent review: brick courses alternate only in the offset variants; mortar is dark in height and has neutral normals away from bevels. Increasing gap visibly reduces the brick plateau. Normal edge colors follow the shared height boundaries. The image is a procedural geometric foundation, not a photorealistic brick material. Contact panels are resized to 256x256; inspect the retained original PNGs for numerical work. Maintainer visual approval and software/browser release qualification are not claimed.

![Two by two height repetition](./tiled-height.png)

The tiled image repeats a 512x512 display reduction of the default height. Separate GPU tests compare the complete doubled-resolution/doubled-cell output to four original tiles byte for byte; the preview alone is not that proof. Original and migrated historical material goldens are unchanged.

## Windows public-consumer result

[Exact candidate receipt](./windows.json) and [Native/browser comparison](./windows-comparison.json) bind source 847fe53c6a42dfac573ee498ccfdc15b1e978b79 and the clean unpublished 0.6.0-alpha.0 archive. All 17 candidate browser tests passed. Three 1K variants and one 65x3 control have equal plan hashes and maximum component delta **0** for all eight height/normal comparisons against the unchanged <=1 gate. This covers the recorded Windows adapter and installed Chrome only, not arbitrary hardware or a published release. cargo xtask check, shader validation, focused node GPU tests and the independent Native consumer passed locally. Six remote checks are separate; PR #47 carries their current status. An initial browser evidence write failed on BigInt serialization; the source above includes the repair and a complete fresh qualification.

Reproduce from that source using a clean candidate build, then:

```sh
node scripts/browser-runtime/build.mjs
node scripts/browser-runtime/consumer.mjs candidate target/browser-runtime tmp/brick-browser-candidate
node scripts/browser-runtime/check-bricks.mjs tmp/brick-browser-candidate tmp/brick-native-browser
```

On this Windows host MIXTURE_BROWSER_CHANNEL=chrome selected the installed browser; native used the explicitly recorded auto adapter policy. Use fresh output directories. Candidate build/registry identities remain separate. The software CI matrix retains its own reports under sdk-brick-comparison in the Chromium artifact.
