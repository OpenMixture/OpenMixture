# Studio saved-file qualification

English | [简体中文](./studio-qualification.zh-CN.md)

This engine-owned verification accepts detached Studio download bytes; the product does not import or build engine sources. Runtime implementation inputs must match the archive producer revision. No runtime schema, node or native golden changes are authorized by this verification batch.

## Frozen gates

Before measuring pixels, [studio-criteria.json](../scripts/browser-runtime/studio-criteria.json) fixes seven 1024 × 1024 cases: an authored 4 × 8 checker; default and authored ceramic (16 × 16 tiles), leather (detail 1), and wood (repeat 16). The three material pairs reuse the existing default/fine-tiles/detail-max/coarse-grain structure, seam, non-degeneracy, causality and height/normal rules verbatim. The checker has exact black/white alternation and exact default normal/roughness/height. All four channels also use the unchanged [M5 browser tolerances](./browser-tolerances.json). These criteria are frozen before acceptance; a failure must not relax them or update native goldens.

Native preparation and comparison commands are documented when implemented. A freeze alone is not an acceptance result. Product evidence must identify the actual Studio UI downloads, exact source/build identities, independent Player execution, PNGs, and recorded environment.
