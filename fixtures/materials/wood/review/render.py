"""Optional Blender consumer of verified wood PNGs; never executes a .mix graph."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import time

import bpy

sys.dont_write_bytecode = True
HELPER_PATH = Path(__file__).resolve().parents[2] / "glazed-ceramic/review/render.py"
spec = importlib.util.spec_from_file_location("ceramic_scene", HELPER_PATH)
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
CASES = ("default", "coarse-grain", "straight-grain", "horizontal-grain")
CHANNELS = helper.CHANNELS
VERSION = helper.VERSION
digest = helper.digest
scene_setup = helper.scene_setup


def verify_inputs(expected):
    manifest_path = expected / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    required = {f"{case}/{channel}.png" for case in CASES for channel in CHANNELS}
    if manifest.get("schemaVersion") != 1 or manifest["material"] != "wood" or set(manifest["files"]) != required:
        raise ValueError("Expected the wood manifest with all sixteen PNGs")
    for name, checksum in manifest["files"].items():
        if digest(expected / name) != checksum:
            raise ValueError(f"PNG does not match the verified manifest: {name}")
    return manifest, digest(manifest_path)


def material_from_pngs(expected, case):
    material = helper.material_from_pngs(expected, case)
    nodes, links = material.node_tree.nodes, material.node_tree.links
    # Exported normal already derives from exported height. Apply it exactly once.
    normal = next(node for node in nodes if node.type == "NORMAL_MAP")
    shader = nodes.get("Principled BSDF")
    links.new(normal.outputs["Normal"], shader.inputs["Normal"])
    nodes.remove(next(node for node in nodes if node.type == "BUMP"))
    return material


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--device", choices=("METAL", "CPU"), required=True)
    parser.add_argument("--samples", type=int, default=512)
    parser.add_argument("--size", type=int, default=1000)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    if tuple(bpy.app.version) != VERSION:
        raise ValueError(f"This review scene pins Blender {VERSION}; got {bpy.app.version}")
    if not 1 <= args.samples <= 1024 or not 256 <= args.size <= 2048:
        raise ValueError("Samples must be 1..1024 and size 256..2048")
    expected = args.expected.resolve(strict=True)
    manifest, manifest_digest = verify_inputs(expected)
    # Always write to a new directory. No baseline or prior review can be overwritten.
    out = args.out.resolve()
    if expected == out or expected in out.parents:
        raise ValueError("Review output cannot be inside expected/")
    out.mkdir(parents=True, exist_ok=False)
    script_digest = digest(Path(__file__))
    helper_digest = digest(HELPER_PATH)
    scene, objects, selected = scene_setup(args.device, args.samples, args.size)
    results = {}
    for case in CASES:
        material = material_from_pngs(expected, case)
        for obj in objects:
            obj.data.materials.clear()
            obj.data.materials.append(material)
        scene.render.filepath = str(out / f"{case}.png")
        started = time.perf_counter()
        bpy.ops.render.render(write_still=True)
        results[case] = {
            "file": f"{case}.png",
            "sha256": digest(out / f"{case}.png"),
            "renderWallSeconds": time.perf_counter() - started,
        }
    # Recheck input identity before recording a completed review.
    if verify_inputs(expected) != (manifest, manifest_digest):
        raise ValueError("Input manifest changed during review")
    if digest(Path(__file__)) != script_digest or digest(HELPER_PATH) != helper_digest:
        raise ValueError("Scene script changed during review")
    report = {
        "schemaVersion": 1,
        "kind": "external-pbr-appearance-review",
        "material": "wood",
        "baselineManifestSha256": manifest_digest,
        "inputs": manifest["files"],
        "sceneScriptSha256": script_digest,
        "sceneHelperSha256": helper_digest,
        "sceneHelper": "../../glazed-ceramic/review/render.py",
        "blenderVersion": bpy.app.version_string,
        "blenderBuildHash": bpy.app.build_hash.decode(),
        "engine": "CYCLES",
        "requestedDevice": args.device,
        "selectedDevices": selected,
        "samples": args.samples,
        "seed": 8,
        "denoising": False,
        "adaptiveSampling": False,
        "size": [args.size, args.size],
        "view": {"transform": "AgX", "look": scene.view_settings.look, "exposure": 0},
        "brdf": {"model": "Principled BSDF", "ior": 1.5, "metallic": 0, "coat": 0},
        "channelUse": {
            "baseColor": "sRGB -> Base Color",
            "roughness": "Non-Color -> Roughness, no remap",
            "normal": "Non-Color -> tangent Normal Map -> Principled Normal, strength 1",
            "height": "Loaded as Non-Color for inspection; no extra bump or displacement because normal already derives from height",
        },
        "geometry": "UV sphere and one-repeat planar swatch on neutral rounded backing; all consumer geometry, no generated relief",
        "controlledComparison": "Same camera, lights, geometry, BRDF, exposure and sampling for every case; only verified PNG inputs change",
        "scope": "Supplementary appearance evidence, not a Mixture texture executor, material golden, runtime dependency, or M4 acceptance",
        "humanAcceptanceClaimed": False,
        "results": results,
    }
    (out / "review.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Review images and input evidence written to {out}")


if __name__ == "__main__":
    main()
