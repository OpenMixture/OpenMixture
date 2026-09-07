"""Exercise review input/overwrite guards through the real Blender entry point."""

import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


REVIEW = Path(__file__).resolve().parent


class InputGuards(unittest.TestCase):
    blender = None

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mixture-pbr-guards-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.expected = self.root / "expected"
        shutil.copytree(REVIEW.parent / "expected", self.expected)

    def rejected(self, out, reason):
        result = subprocess.run([
            self.blender, "--background", "--factory-startup", "--python-exit-code", "1",
            "--python", str(REVIEW / "render.py"), "--",
            "--expected", str(self.expected), "--out", str(out), "--device", "CPU",
        ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=60)
        self.assertEqual(result.returncode, 1, result.stdout)
        self.assertIn(reason, result.stdout)

    def test_changed_png_is_rejected_before_output_creation(self):
        path = self.expected / "default" / "roughness.png"
        path.write_bytes(path.read_bytes() + b"changed")
        out = self.root / "review"
        self.rejected(out, "PNG does not match the verified manifest")
        self.assertFalse(out.exists())

    def test_existing_review_is_preserved(self):
        out = self.root / "review"
        out.mkdir()
        sentinel = out / "existing.txt"
        sentinel.write_text("preserve me")
        self.rejected(out, "FileExistsError")
        self.assertEqual(sentinel.read_text(), "preserve me")
        self.assertEqual(list(out.iterdir()), [sentinel])

    def test_output_inside_expected_is_rejected_without_mutation(self):
        before = sorted(path.relative_to(self.expected) for path in self.expected.rglob("*"))
        self.rejected(self.expected / "review", "Review output cannot be inside expected/")
        after = sorted(path.relative_to(self.expected) for path in self.expected.rglob("*"))
        self.assertEqual(before, after)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--blender", required=True)
    args = parser.parse_args()
    InputGuards.blender = args.blender
    unittest.main(argv=[__file__], verbosity=2)
