# Leather verification evidence

English | [简体中文](./README.zh-CN.md)

- [Software material report](./software.json) and [doctor](./software-doctor.json): exact pinned SwiftShader comparisons for sixteen 1K PNGs.
- [Metal material report](./metal.json) and [doctor](./metal-doctor.json): actual Apple M5 with the declared tolerance; [initial threshold failure](./metal-initial-tolerance-failure.json) retained.
- [Bootstrap acceptance](./bootstrap-acceptance.json), [bootstrap candidate](./bootstrap-candidate.json), and [bootstrap checks](./bootstrap-render.json): initial new material baseline only; before images are explicitly missing in initial contact sheets.
- [Overview](./overview.png), [repeat preview](./tiling.png), and [PBR comparison](./pbr/comparison.png): inspect spatial structure and controlled appearance with full input/script/output identity in [review.json](./pbr/review.json).
- [Verification index](./verification.json): focused checks, node reports, smoke evidence and repository checks.
- [Human review](./human-review.json): accepted by the user after the controlled PBR comparison; agent inspection remains separately identified.

The expected PNG hashes have not changed since first establishment. Source hashes in the bootstrap record describe that exact capture; later tooling tests and the documented hardware-tolerance adjustment are bound by subsequent check reports. The accepted ceramic baseline and review remain unchanged. Remote CI is deferred; these reports do not close M3. Reproduce from the [material guide](../README.md).

Final [software source identities](./software-candidate.json) and [Metal source identities](./metal-candidate.json) match the current source files. Both adapters also rechecked the unchanged ceramic baseline.

The comparison image retains its original "Human acceptance pending" capture caption. The later accepted decision is recorded in human-review.json; image pixels were not altered.
