"""Offline integrity and extraction regression tests (Python standard library)."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import warnings
import zipfile

from restore import ROOT, restore, safe_path


class RestoreTests(unittest.TestCase):
    def bundle(self, root, name="docs/evidence/old/result.json", wrong_digest=False):
        data = b'{"ok":true}\n'
        inventory = {"sourceCommit": "a" * 40, "files": {
            name: {"bytes": len(data), "sha256": "0" * 64 if wrong_digest
                   else hashlib.sha256(data).hexdigest()}}}
        archive = root / "input.zip"
        with zipfile.ZipFile(archive, "w") as z:
            z.writestr("inventory.json", json.dumps(inventory))
            z.writestr(name, data)
        record = {"sourceCommit": "a" * 40, "fileCount": 1,
                  "contentBytes": len(data), "bytes": archive.stat().st_size,
                  "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()}
        return archive, record

    def test_offline_restore_and_existing_destination(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive, record = self.bundle(root)
            target = root / "restored"
            restore(record, target, archive)
            original = (target / "docs/evidence/old/result.json").read_bytes()
            self.assertEqual(original, b'{"ok":true}\n')
            with self.assertRaises(ValueError):
                restore(record, target, archive)
            self.assertEqual((target / "docs/evidence/old/result.json").read_bytes(), original)

    def test_corrupt_archive_and_member_leave_no_destination(self):
        for wrong_member in (False, True):
            with self.subTest(wrong_member=wrong_member), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                archive, record = self.bundle(root, wrong_digest=wrong_member)
                if not wrong_member:
                    record["sha256"] = "0" * 64
                with self.assertRaises(ValueError):
                    restore(record, root / "restored", archive)
                self.assertFalse((root / "restored").exists())

    def test_traversal_is_rejected_before_extraction(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive, record = self.bundle(root, "../escaped")
            with self.assertRaises(ValueError):
                restore(record, root / "restored", archive)
            self.assertFalse((root / "escaped").exists())
            self.assertFalse((root / "restored").exists())

    def test_unsafe_cross_platform_names(self):
        for name in ("../x", "/x", "C:/x", "a\\b", "a//b", "a/./b", ".git/config",
                     "a/NUL.txt", "a/COM1", "a/file:stream", "a/x.", "a/x "):
            with self.subTest(name=name), self.assertRaises(ValueError):
                safe_path(name)

    def test_duplicate_link_and_inventory_mismatch_are_rejected(self):
        for mode in ("duplicate", "symlink", "count"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                archive, record = self.bundle(root)
                if mode == "count":
                    record["fileCount"] = 2
                elif mode == "symlink":
                    with zipfile.ZipFile(archive) as z:
                        contents = {name: z.read(name) for name in z.namelist()}
                    with zipfile.ZipFile(archive, "w") as z:
                        for name, data in contents.items():
                            info = zipfile.ZipInfo(name)
                            info.create_system = 3
                            info.external_attr = (0o120777 if name != "inventory.json" else 0o100644) << 16
                            z.writestr(info, data)
                    record["bytes"] = archive.stat().st_size
                    record["sha256"] = hashlib.sha256(archive.read_bytes()).hexdigest()
                else:
                    with warnings.catch_warnings(), zipfile.ZipFile(archive, "a") as z:
                        warnings.simplefilter("ignore", UserWarning)
                        if mode == "duplicate":
                            z.writestr("inventory.json", "{}")
                        else:
                            info = zipfile.ZipInfo("link")
                            info.create_system = 3
                            info.external_attr = 0o120777 << 16
                            z.writestr(info, "outside")
                    record["bytes"] = archive.stat().st_size
                    record["sha256"] = hashlib.sha256(archive.read_bytes()).hexdigest()
                with self.assertRaises(ValueError):
                    restore(record, root / "restored", archive)
                self.assertFalse((root / "restored").exists())

    def test_retained_local_copies_match_relocated_originals(self):
        relocations = json.loads((ROOT / "docs/evidence/archives/relocations.json").read_text())
        manifest = json.loads((ROOT / "docs/evidence/archives/manifest.json").read_text())
        copies = 0
        for name, info in relocations["files"].items():
            safe_path(name)
            self.assertIn(info["group"], manifest["groups"])
            self.assertFalse((ROOT / name).exists(), name)
            if "localCopy" in info:
                data = ROOT.joinpath(*safe_path(info["localCopy"])).read_bytes()
                self.assertEqual(len(data), info["bytes"])
                self.assertEqual(hashlib.sha256(data).hexdigest(), info["sha256"])
                copies += 1
        self.assertEqual(copies, 8)


if __name__ == "__main__":
    unittest.main()
