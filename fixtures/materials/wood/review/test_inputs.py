"""Exercise pre-render guards through Blender without requiring a wood baseline."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


REVIEW = Path(__file__).resolve().parent
CASES = ("default", "coarse-grain", "straight-grain", "horizontal-grain")
CHANNELS = ("baseColor", "normal", "roughness", "height")


class InputGuards(unittest.TestCase):
    blender = None

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mixture-wood-pbr-guards-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.expected = self.root / "expected"
        files = {}
        for case in CASES:
            directory = self.expected / case
            directory.mkdir(parents=True)
            for channel in CHANNELS:
                name = f"{case}/{channel}.png"
                # Hash-only preflight fixtures, deliberately not renderable images.
                # Every test must stop before scene setup or PNG loading.
                content = f"Wood review preflight guard: {name}\n".encode()
                (self.expected / name).write_bytes(content)
                files[name] = "sha256:" + hashlib.sha256(content).hexdigest()
        self.manifest = {"schemaVersion": 1, "material": "wood", "files": files}
        self.write_manifest(self.manifest)

    def write_manifest(self, manifest):
        (self.expected / "manifest.json").write_text(json.dumps(manifest) + "\n")

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

    def test_manifest_requires_wood_version_one_and_all_sixteen_pngs(self):
        for mutation in ("material", "version", "missing", "extra"):
            with self.subTest(mutation=mutation):
                manifest = json.loads(json.dumps(self.manifest))
                if mutation == "material":
                    manifest["material"] = "leather"
                elif mutation == "version":
                    manifest["schemaVersion"] = 2
                elif mutation == "missing":
                    del manifest["files"]["horizontal-grain/height.png"]
                else:
                    manifest["files"]["unexpected/height.png"] = "sha256:unexpected"
                self.write_manifest(manifest)
                out = self.root / f"review-{mutation}"
                self.rejected(out, "Expected the wood manifest with all sixteen PNGs")
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
