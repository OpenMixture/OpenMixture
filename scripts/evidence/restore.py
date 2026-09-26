"""Restore a pinned historical evidence snapshot; never modify the checkout."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import stat
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "docs/evidence/archives/manifest.json"
CHUNK = 1024 * 1024


def safe_path(name):
    parts = PurePosixPath(name).parts
    devices = {"CON", "PRN", "AUX", "NUL"} | {
        f"{prefix}{n}" for prefix in ("COM", "LPT") for n in range(1, 10)
    }
    if (not parts or "/".join(parts) != name or name.startswith("/")
            or "\\" in name or any(
                p in (".", "..", ".git") or ":" in p
                or p.endswith((".", " ")) or p.split(".")[0].upper() in devices
                or any(ord(c) < 32 for c in p) for p in parts)):
        raise ValueError(f"Unsafe archive path: {name!r}")
    return parts


def copy_checked(source, destination, expected):
    digest = hashlib.sha256()
    size = 0
    while block := source.read(CHUNK):
        size += len(block)
        if size > expected["bytes"]:
            raise ValueError("Content exceeds its pinned size")
        digest.update(block)
        destination.write(block)
    if size != expected["bytes"] or digest.hexdigest() != expected["sha256"]:
        raise ValueError("Content size or SHA-256 does not match the pinned record")


def unpack(archive, target, record):
    """Archive hash is checked by restore before interpreting any ZIP metadata."""
    with zipfile.ZipFile(archive) as bundle:
        infos = bundle.infolist()
        names = [i.filename for i in infos]
        if len(names) != len(set(names)) or names.count("inventory.json") != 1:
            raise ValueError("Duplicate members or missing inventory")
        inventory_info = bundle.getinfo("inventory.json")
        if inventory_info.file_size > 2 * CHUNK:
            raise ValueError("Inventory exceeds 2 MiB")
        inventory = json.loads(bundle.read(inventory_info))
        files = inventory["files"]
        if (inventory["sourceCommit"] != record["sourceCommit"]
                or len(files) != record["fileCount"]
                or sum(f["bytes"] for f in files.values()) != record["contentBytes"]
                or set(names) != {"inventory.json", *files}):
            raise ValueError("Snapshot inventory does not match the pinned record")
        # Validate the entire namespace before writing any member.
        folded = set()
        for info in infos:
            safe_path(info.filename)
            key = info.filename.casefold()
            if key in folded or stat.S_ISLNK(info.external_attr >> 16) or info.is_dir():
                raise ValueError("Ambiguous path or non-file archive member")
            folded.add(key)
            if info.filename != "inventory.json":
                expected = files[info.filename]
                if (info.file_size != expected["bytes"] or expected["bytes"] < 0
                        or not re.fullmatch(r"[0-9a-f]{64}", expected["sha256"])):
                    raise ValueError("Invalid member size or digest")
        for name, expected in files.items():
            destination = target.joinpath(*safe_path(name))
            destination.parent.mkdir(parents=True, exist_ok=True)
            with bundle.open(name) as source, destination.open("xb") as output:
                copy_checked(source, output, expected)
        (target / "inventory.json").write_bytes(bundle.read("inventory.json"))


def restore(record, target, archive=None):
    target = Path(target).absolute()
    if target.exists() or target.is_symlink():
        raise ValueError("Destination must not exist; choose a fresh directory")
    target.parent.mkdir(parents=True, exist_ok=True)
    # A failed download/verification never leaves a seemingly restored snapshot.
    with tempfile.TemporaryDirectory(prefix=".evidence-", dir=target.parent) as temp:
        temp = Path(temp)
        downloaded = temp / "bundle.zip"
        if archive:
            source = Path(archive).open("rb")
        else:
            url = record["url"]
            if not url.startswith("https://github.com/OpenMixture/OpenMixture/releases/download/"):
                raise ValueError("Archive must come from the configured repository release")
            source = urllib.request.urlopen(url, timeout=60)
        with source, downloaded.open("xb") as output:
            copy_checked(source, output, record)
        restored = temp / "snapshot"
        restored.mkdir()
        unpack(downloaded, restored, record)
        if target.exists() or target.is_symlink():
            raise ValueError("Destination appeared during verification")
        restored.rename(target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("group", choices=("alpha-04", "m5-05", "early-runs"))
    parser.add_argument("destination", type=Path)
    parser.add_argument("--archive", type=Path, help="verify an already downloaded ZIP offline")
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    restore(manifest["groups"][args.group], args.destination, args.archive)
    print(f"Restored and verified {args.group}: {args.destination.absolute()}")


if __name__ == "__main__":
    main()
