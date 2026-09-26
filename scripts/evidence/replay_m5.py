"""Reconstruct the 250 original M5 logical files from a restored archive snapshot."""

import argparse
import hashlib
import json
from pathlib import Path
import tempfile

from restore import safe_path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("snapshot", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    target = args.destination.absolute()
    if target.exists() or target.is_symlink():
        raise ValueError("Destination must be fresh")
    index = json.loads((args.snapshot / "docs/evidence/m5-05/evidence-index.json").read_text())
    target.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with tempfile.TemporaryDirectory(prefix=".m5-replay-", dir=target.parent) as temp:
        staging = Path(temp) / "replay"
        staging.mkdir()
        for dataset, files in index["datasets"].items():
            safe_path(dataset)
            for item in files:
                source = args.snapshot.joinpath(*safe_path(item["storedPath"]))
                data = source.read_bytes()
                if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
                    raise ValueError(f"Original M5 digest mismatch: {item['storedPath']}")
                destination = staging.joinpath(*safe_path(dataset + "/" + item["file"]))
                destination.parent.mkdir(parents=True, exist_ok=True)
                with destination.open("xb") as output:
                    output.write(data)
                count += 1
        if target.exists() or target.is_symlink():
            raise ValueError("Destination appeared during reconstruction")
        staging.rename(target)
    print(f"Reconstructed and hash-verified {count} original M5 files: {target}")


if __name__ == "__main__":
    main()
