# Historical evidence storage

English | [简体中文](./README.zh-CN.md)

Keep decisions inspectable without making every checkout carry every historical run. The maintainer selected a dedicated [GitHub evidence archive](https://github.com/OpenMixture/OpenMixture/releases/tag/evidence-archive-2026-09-25) on 2026-09-25. This is **not a product release**, new execution, qualification, or registry publication. No history or original acceptance receipt is rewritten.

## What stays and what moves

Current `.mix` inputs, variants, acceptance contracts, expected PNGs, human acceptance records and their bound PBR images stay in Git. So do the M5 calibration failure, named comparison sheets, critical measurements, package/source identities and original evidence indexes. Complete historical attachments are available on demand:

| Group | Complete restored snapshot | Current-tree reduction |
|---|---|---|
| `alpha-04` | Original ALPHA-04 directory, verifier, nested 152-file archive and runtime package | Only `saved-file-bundles.tar.gz` moves out |
| `m5-05` | Original M5-05 directory plus every tracked golden referenced by its evidence index | Content-addressed PNG matrix moves out; JSON/text records and named review/calibration images stay |
| `early-runs` | Complete PR-015 directory and wood's original Metal/software smoke directories | Ordinary logs and per-run CLI folders move out; status/identity/measurement records, package archives, consumer summaries and missing-shader rejection evidence stay |

Eight PR-015 2K PNGs are byte-identical to the retained wood 2K images. [Relocations](./relocations.json) records their local copies along with every removed path, original length, SHA-256 and archive group. This avoids duplicate working-tree images; Git already deduplicates identical blobs, so this is not an equivalent pack-size saving.

Original machine receipts and capture manifests still name their original paths and hashes. They describe the original snapshots, not a claim that every attachment remains in the current checkout. Restore the complete group before replaying those paths. Historical Markdown links use the immutable pre-move commit when a target leaves the current tree. Do not rewrite an old receipt to make a partial checkout look complete.

## Retrieve and verify

Python 3.10+ and its standard library are sufficient. Run from the repository root; each destination must be new. Downloads are explicit and absent from ordinary builds and CI checks.

```bash
python scripts/evidence/restore.py alpha-04 tmp/retained-alpha04
node tmp/retained-alpha04/docs/evidence/alpha-04/verify.mjs tmp/alpha04-replay

python scripts/evidence/restore.py m5-05 tmp/retained-m5
python scripts/evidence/replay_m5.py tmp/retained-m5 tmp/m5-05-retained

python scripts/evidence/restore.py early-runs tmp/retained-early
```

The restore command verifies the pinned archive size and SHA-256 before reading its inventory, then verifies the complete member set, source snapshot, lengths and each file digest. It rejects traversal, links, ambiguous Windows paths and existing destinations. Failures leave no completed destination. `--archive <downloaded.zip>` performs the same checks offline. ZIPs include their original inventory; the M5 replay additionally verifies the original 250 logical-file index, and ALPHA-04's original verifier validates the nested 152-file bundle.

`manifest.json` pins the transport identities. [Retrieval verification](./retrieval.json) records downloads from the published release; this is a storage-integrity check, not a new pixel qualification. The ZIPs preserve immutable Git blob bytes from `e249d57d9ce78e73bbe8da554a6fcce0f3375303`, avoiding checkout newline conversions. Rebuild with `python scripts/evidence/pack.py <fresh-directory>` in a clone containing that commit. Tool/zlib versions can affect ZIP compression; rebuilt archives must never silently replace pinned assets.

## Retention responsibility and limits

OpenMixture repository maintainers own the assets. Keep them without scheduled expiry; do not delete or replace this tag/assets. A move requires copying the bytes, independent retrieval and hash/member verification, and a new retrieval record before changing locations. GitHub Release availability is not immutable-storage or independent-backup assurance. The preserved original Git commit is a recovery source; ordinary expiring CI artifacts are not the archive. Any failure to retrieve must be reported as an availability failure, never a successful verification.

This reduces the current source tree. Existing complete Git history still contains the original blobs, and no claim is made that ordinary full clones or existing `.git` packs shrink. No golden pixels, shaders, package semantics, hardware support or historical failures change.

## Maintenance checks

```bash
python scripts/evidence/test_restore.py
cargo xtask evidence
cargo xtask links
cargo xtask check
```

The required CPU workflow runs the offline archive tests; it never downloads these assets. After changing retention mappings, perform a real restore and the original member checks in addition to ordinary repository checks. Lower obsolete exception counts in the same PR. Paired policy and Agent Guide changes own this retention exception; it is not blanket permission to externalize current goldens or human acceptance images.
