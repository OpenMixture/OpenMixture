"""Verify all retained decoded texels; optionally reconstruct an explicitly derived replay.

python verify.py [<fresh-replay-directory>]
Requires numpy/Pillow. A replay re-encodes PNGs and binds derived manifests;
it never claims to recover the original PNG compression stream or rerun a GPU.
"""
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib

import numpy as np
from PIL import Image, PngImagePlugin

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MATRIX = HERE / "matrix"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode())


def verify(output=None):
    for name, expected in json.loads((HERE / "control-sha256.json").read_bytes()).items():
        assert digest((HERE / name).read_bytes()) == expected, name
    hashes = json.loads((MATRIX / "sha256.json").read_bytes())
    for name, expected in hashes.items():
        assert digest((MATRIX / name).read_bytes()) == expected, name
    index = json.loads((MATRIX / "index.json").read_bytes())
    assert digest((MATRIX / "native-manifest.json").read_bytes()) == index["nativeManifestSha256"]
    assert digest((ROOT / "docs/browser-quality-v2.json").read_bytes()) == index["profileSha256"]
    if output:
        output.mkdir()
    keys = set()
    for row in index["textures"]:
        key = (row["set"], row["material"], row["case"], row["channel"])
        assert key not in keys
        keys.add(key)
        golden_manifest = json.loads((ROOT / "fixtures/materials" / row["material"] / "expected/manifest.json").read_bytes())
        expected_png = golden_manifest["files"][row["case"] + "/" + row["channel"] + ".png"]
        assert "sha256:" + digest((ROOT / row["baseline"]).read_bytes()) == expected_png
        with Image.open(ROOT / row["baseline"]) as image:
            assert image.mode == "RGBA" and image.size == (1024, 1024)
            baseline = np.asarray(image, dtype=np.uint8)
        assert digest(baseline.tobytes()) == row["baselineRgbaSha256"]
        packed = (MATRIX / "deltas" / row["delta"]).read_bytes()
        assert digest(packed) == row["deltaSha256"]
        delta = zlib.decompress(packed)
        assert len(delta) == 1024 * 1024 * 4
        actual = np.bitwise_xor(baseline, np.frombuffer(delta, np.uint8).reshape(1024, 1024, 4))
        assert digest(actual.tobytes()) == row["rgbaSha256"], key
        if output:
            target = output / row["set"] / row["material"] / row["case"] / (row["channel"] + ".png")
            target.parent.mkdir(parents=True, exist_ok=True)
            info = PngImagePlugin.PngInfo()
            if row["channel"] == "baseColor":
                info.add(b"sRGB", b"\x00")
            else:
                info.add(b"gAMA", struct.pack(">I", 100000))
            Image.fromarray(actual).save(target, pnginfo=info)
    assert len(keys) == 44 * (1 + len(index["browsers"]))
    if output:
        manifest = json.loads((MATRIX / "native-manifest.json").read_bytes())
        manifest["derivedReplayOfManifestSha256"] = index["nativeManifestSha256"]
        for case in manifest["cases"]:
            material, case_id = case["material"], case["id"]
            for channel in ("baseColor", "normal", "roughness", "height"):
                case["nativePixels"][channel] = digest((output / "native" / material / case_id / (channel + ".png")).read_bytes())
            for name in ("native", *index["browsers"]):
                filename = "native.json" if name == "native" else "result.json"
                save(output / name / material / case_id / filename, json.loads((MATRIX / name / material / case_id / filename).read_bytes()))
        save(output / "native/manifest.json", manifest)
        for name, metadata in index["browsers"].items():
            receipt = json.loads((MATRIX / name / "receipt.json").read_bytes())
            receipt["derivedReplayOfReceiptSha256"] = metadata["receiptSha256"]
            receipt["manifestSha256"] = digest((output / "native/manifest.json").read_bytes())
            save(output / name / "receipt.json", receipt)
        save(output / "DERIVED-REPLAY.json", {"gpuExecuted": False, "originalCompressedPngBytesRecovered": False,
                                            "allDecodedTextureHashesVerified": True, "scope": "offline diagnostic replay; not package/browser qualification"})
    print(f"Verified {len(keys)} complete decoded textures and all retained metadata")


if __name__ == "__main__":
    verify(Path(sys.argv[1]) if len(sys.argv) > 1 else None)
