"""Losslessly retain decoded pixels; no shader or material execution.

Usage: python retain.py <native-directory> <name=browser-directory> ...
Run once after the fresh browser jobs finish. Requires numpy and Pillow.
"""
import hashlib
import json
from pathlib import Path
import sys
import zlib

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode())


def rgba(path):
    with Image.open(path) as image:
        if image.mode != "RGBA" or image.size != (1024, 1024):
            raise ValueError(f"Wrong image contract: {path}")
        return np.asarray(image, dtype=np.uint8).tobytes()


def capture(native, browsers):
    destination = HERE / "matrix"
    destination.mkdir()  # Never overwrite a prior capture.
    manifest_bytes = (native / "manifest.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    (destination / "native-manifest.json").write_bytes(manifest_bytes)
    (destination / "profile.json").write_bytes((ROOT / "docs/browser-quality-v2.json").read_bytes())
    index = {"schemaVersion": 1, "representation": "zlib-compressed XOR of decoded RGBA8 against committed software golden; exact decoded pixels, not original PNG compression bytes",
             "profileSha256": digest((ROOT / "docs/browser-quality-v2.json").read_bytes()),
             "nativeManifestSha256": digest(manifest_bytes), "textures": [], "browsers": {}}
    (destination / "deltas").mkdir()
    for name, folder in browsers.items():
        receipt_bytes = (folder / "receipt.json").read_bytes()
        report = json.loads((folder / "comparison.json").read_bytes())
        receipt = json.loads(receipt_bytes)
        assert receipt["manifestSha256"] == digest(manifest_bytes)
        assert report["browserReceiptSha256"] == "sha256:" + digest(receipt_bytes)
        assert report["profileSha256"] == "sha256:" + index["profileSha256"]
        assert json.loads((folder / "ordinary.json").read_bytes())["ok"]
        index["browsers"][name] = {"receiptSha256": digest(receipt_bytes), "version": receipt["browser"],
                                    "runtimeRevision": receipt["build"]["engineRevision"]}
        for filename in ("receipt.json", "comparison.json", "ordinary.json"):
            target = destination / name / filename
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((folder / filename).read_bytes())
    for case in manifest["cases"]:
        material, case_id = case["material"], case["id"]
        native_folder = native / material / case_id
        save(destination / "native" / material / case_id / "native.json", json.loads((native_folder / "native.json").read_bytes()))
        for name, folder in browsers.items():
            save(destination / name / material / case_id / "result.json", json.loads((folder / material / case_id / "result.json").read_bytes()))
        for channel in ("baseColor", "normal", "roughness", "height"):
            relative = Path("fixtures/materials") / material / "expected" / case_id / f"{channel}.png"
            baseline = rgba(ROOT / relative)
            for name, folder in {"native": native, **browsers}.items():
                path = folder / material / case_id / f"{channel}.png"
                original_png = path.read_bytes()
                if name == "native":
                    assert digest(original_png) == case["nativePixels"][channel]
                else:
                    report = json.loads((folder / "comparison.json").read_bytes())
                    row = next(r for r in report["cases"] if r["material"] == material and r["case"] == case_id)
                    assert row["channels"][channel]["browserPngSha256"] == "sha256:" + digest(original_png)
                actual = rgba(path)
                delta = zlib.compress(np.bitwise_xor(np.frombuffer(baseline, np.uint8), np.frombuffer(actual, np.uint8)).tobytes(), 9)
                delta_name = digest(delta) + ".zlib"
                (destination / "deltas" / delta_name).write_bytes(delta)
                index["textures"].append({"set": name, "material": material, "case": case_id, "channel": channel,
                                           "baseline": relative.as_posix(), "baselineRgbaSha256": digest(baseline),
                                           "originalPngSha256": digest(original_png), "rgbaSha256": digest(actual),
                                           "delta": delta_name, "deltaSha256": digest(delta)})
    save(destination / "index.json", index)
    save(destination / "sha256.json", {p.relative_to(destination).as_posix(): digest(p.read_bytes())
                                       for p in sorted(destination.rglob("*")) if p.is_file()})
    print(len(index["textures"]), "complete decoded textures retained")


if __name__ == "__main__":
    capture(Path(sys.argv[1]), {name: Path(path) for name, path in (arg.split("=", 1) for arg in sys.argv[2:])})
