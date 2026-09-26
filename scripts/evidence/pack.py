"""Rebuild the three 2026-09-25 evidence archives from immutable Git blobs."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

SOURCE = "e249d57d9ce78e73bbe8da554a6fcce0f3375303"
TAG = "evidence-archive-2026-09-25"
ROOT = Path(__file__).resolve().parents[2]
PREFIXES = {
    "alpha-04": ("docs/evidence/alpha-04/",),
    "m5-05": ("docs/evidence/m5-05/",),
    "early-runs": ("docs/evidence/pr-015/",
                   "fixtures/materials/wood/reports/metal-gpu-smoke/",
                   "fixtures/materials/wood/reports/software-gpu-smoke/"),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    listing = subprocess.check_output(["git", "ls-tree", "-rz", SOURCE], cwd=ROOT)
    objects = {}
    for entry in listing.split(b"\0"):
        if entry:
            meta, path = entry.split(b"\t", 1)
            mode, kind, oid = meta.split()
            if kind == b"blob" and mode in (b"100644", b"100755"):
                objects[path.decode()] = oid
    with subprocess.Popen(["git", "cat-file", "--batch"], cwd=ROOT,
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE) as cat:
        def read(path):
            cat.stdin.write(objects[path] + b"\n")
            cat.stdin.flush()
            header = cat.stdout.readline().split()
            if len(header) != 3 or header[1] != b"blob":
                raise ValueError(f"Cannot read snapshot blob: {path}")
            size = int(header[2])
            data = cat.stdout.read(size)
            if len(data) != size or cat.stdout.read(1) != b"\n":
                raise ValueError("Truncated Git blob")
            return data

        groups = {}
        for group, prefixes in PREFIXES.items():
            paths = {p for p in objects if p.startswith(prefixes)}
            if group == "m5-05":
                index = json.loads(read("docs/evidence/m5-05/evidence-index.json"))
                paths.update(item["storedPath"] for dataset in index["datasets"].values()
                             for item in dataset)
            archive = args.output / f"{group}.zip"
            files = {}
            with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED,
                                 compresslevel=9) as bundle:
                def write(name, data):
                    info = zipfile.ZipInfo(name, (2026, 9, 25, 0, 0, 0))
                    info.create_system = 3
                    info.external_attr = 0o100644 << 16
                    info.compress_type = zipfile.ZIP_DEFLATED
                    bundle.writestr(info, data, compresslevel=9)

                for path in sorted(paths):
                    data = read(path)
                    files[path] = {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                    write(path, data)
                inventory = {"sourceCommit": SOURCE, "files": files}
                write("inventory.json", (json.dumps(inventory, indent=2) + "\n").encode())
            groups[group] = {
                "sourceCommit": SOURCE,
                "url": f"https://github.com/OpenMixture/OpenMixture/releases/download/{TAG}/{group}.zip",
                "bytes": archive.stat().st_size,
                "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
                "fileCount": len(files),
                "contentBytes": sum(f["bytes"] for f in files.values()),
            }
        cat.stdin.close()
        if cat.wait() != 0:
            raise ValueError("Git blob reader failed")
    manifest = {"schemaVersion": 1, "kind": "historical-evidence-not-product-release",
                "retentionOwner": "OpenMixture repository maintainers",
                "releaseTag": TAG, "groups": groups}
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
