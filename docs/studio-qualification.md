# Studio saved-file qualification

English | [简体中文](./studio-qualification.zh-CN.md)

This engine-owned verification accepts detached Studio download bytes; the product does not import or build engine sources. Runtime implementation inputs must match the archive producer revision. No runtime schema, node or native golden changes are authorized by this verification batch.

## Frozen gates

Before measuring pixels, [studio-criteria.json](../scripts/browser-runtime/studio-criteria.json) fixes seven 1024 × 1024 cases: an authored 4 × 8 checker; default and authored ceramic (16 × 16 tiles), leather (detail 1), and wood (repeat 16). The three material pairs reuse the existing default/fine-tiles/detail-max/coarse-grain structure, seam, non-degeneracy, causality and height/normal rules verbatim. The checker has exact black/white alternation and exact default normal/roughness/height. All four channels also use the unchanged [M5 browser tolerances](./browser-tolerances.json). These criteria are frozen before acceptance; a failure must not relax them or update native goldens.

## Commands

Capture in the product, prepare native references here, execute Player in the product, then compare here. Only detached data crosses the repository boundary.

```bash
# Product
npm run capture:studio -- /absolute/studio-downloads
# Engine
MIXTURE_GPU_BACKEND=metal node scripts/browser-runtime/prepare-studio.mjs /absolute/new-native 4b914feb9f3365d292b27ea60c5e0b6004f745e8 /absolute/studio-downloads
# Product
npm run test:studio -- /absolute/new-native /absolute/new-player
# Engine
cargo xtask studio-material-check /absolute/new-native /absolute/new-player
```

Preparation verifies runtime implementation identity, unique cases and saved-byte digests. Comparison fails closed, checks manifest and PNG provenance, compares all-channel and individual Player plans, and reuses existing Rust structure, relationship and causality checks. Native goldens remain unchanged. A workflow is not an acceptance result; retain source-bound results for each tested environment.
