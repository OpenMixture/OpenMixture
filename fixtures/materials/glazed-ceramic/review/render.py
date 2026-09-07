"""Optional Blender consumer of verified Mixture PNGs; never executes a .mix graph.

Run with Blender 4.5.13 --background --factory-startup --python-exit-code 1
--python render.py -- --expected <expected directory> --out <new directory>
--device METAL (or explicitly CPU). See the paired README files.
"""

import argparse
import hashlib
import json
from pathlib import Path
import sys
import time

import bpy
from mathutils import Vector


CASES = ("default", "fine-tiles", "matte")
CHANNELS = ("baseColor", "normal", "roughness", "height")
VERSION = (4, 5, 13)


def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def verify_inputs(expected):
    manifest_path = expected / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    required = {f"{case}/{channel}.png" for case in CASES for channel in CHANNELS}
    if manifest["material"] != "glazed-ceramic" or set(manifest["files"]) != required:
        raise ValueError("Expected the glazed-ceramic manifest with all twelve PNGs")
    for name, checksum in manifest["files"].items():
        if digest(expected / name) != checksum:
            raise ValueError(f"PNG does not match the verified manifest: {name}")
    return manifest, digest(manifest_path)


def aim(obj, target):
    obj.rotation_euler = (Vector(target) - obj.location).to_track_quat("-Z", "Y").to_euler()


def light(name, location, target, power, width, height):
    data = bpy.data.lights.new(name, "AREA")
    data.energy = power
    data.shape = "RECTANGLE"
    data.size = width
    data.size_y = height
    obj = bpy.data.objects.new(name, data)
    bpy.context.collection.objects.link(obj)
    obj.location = location
    aim(obj, target)


def plain_material(name, color, roughness):
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    shader = material.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = (*color, 1)
    shader.inputs["Roughness"].default_value = roughness
    return material


def material_from_pngs(expected, case):
    material = plain_material(case, (0.5, 0.5, 0.5), 0.5)
    nodes = material.node_tree.nodes
    links = material.node_tree.links
    shader = nodes.get("Principled BSDF")
    # A fixed consumer BRDF, shared by every case. No added coat or surface noise.
    shader.inputs["Metallic"].default_value = 0
    shader.inputs["IOR"].default_value = 1.5
    shader.inputs["Coat Weight"].default_value = 0
    shader.inputs["Transmission Weight"].default_value = 0
    shader.inputs["Subsurface Weight"].default_value = 0
    textures = {}
    for channel in CHANNELS:
        texture = nodes.new("ShaderNodeTexImage")
        texture.name = channel
        texture.image = bpy.data.images.load(str(expected / case / f"{channel}.png"))
        texture.image.colorspace_settings.name = "sRGB" if channel == "baseColor" else "Non-Color"
        texture.extension = "REPEAT"
        texture.interpolation = "Linear"
        if tuple(texture.image.size) != (1024, 1024):
            raise ValueError(f"Expected 1K input for {case}/{channel}")
        textures[channel] = texture
    links.new(textures["baseColor"].outputs["Color"], shader.inputs["Base Color"])
    links.new(textures["roughness"].outputs["Color"], shader.inputs["Roughness"])
    normal = nodes.new("ShaderNodeNormalMap")
    normal.space = "TANGENT"
    normal.inputs["Strength"].default_value = 1
    links.new(textures["normal"].outputs["Color"], normal.inputs["Color"])
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 1
    bump.inputs["Distance"].default_value = 0.02
    links.new(normal.outputs["Normal"], bump.inputs["Normal"])
    links.new(textures["height"].outputs["Color"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], shader.inputs["Normal"])
    return material


def scene_setup(device, samples, size):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = samples
    scene.cycles.seed = 8
    scene.cycles.use_animated_seed = False
    scene.cycles.use_adaptive_sampling = False
    scene.cycles.use_denoising = False
    scene.cycles.max_bounces = 8
    selected = []
    if device == "METAL":
        preferences = bpy.context.preferences.addons["cycles"].preferences
        preferences.compute_device_type = "METAL"
        preferences.refresh_devices()
        for item in preferences.devices:
            item.use = item.type == "METAL"
            if item.use:
                selected.append({"name": item.name, "type": item.type, "id": item.id})
        if not selected:
            raise RuntimeError("Explicit METAL request has no matching Cycles device")
        scene.cycles.device = "GPU"
    else:
        scene.cycles.device = "CPU"
        selected.append({"type": "CPU", "name": "explicit CPU scene review"})
    scene.render.resolution_x = size
    scene.render.resolution_y = size
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGB"
    scene.render.image_settings.color_depth = "8"
    scene.render.film_transparent = False
    scene.display_settings.display_device = "sRGB"
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = 0
    scene.view_settings.gamma = 1
    world = bpy.data.worlds.new("Neutral studio")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.18, 0.18, 0.18, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.5
    scene.world = world
    ground = plain_material("Studio floor", (0.065, 0.07, 0.08), 0.7)
    bpy.ops.mesh.primitive_plane_add(size=200)
    bpy.context.object.name = "Studio floor"
    bpy.context.object.data.materials.append(ground)
    # The rounded slab is display geometry, not texture-generated relief.
    bpy.ops.mesh.primitive_cube_add(size=1, location=(0, 0, 0.09))
    backing = bpy.context.object
    backing.name = "Neutral sample backing (geometry only)"
    backing.dimensions = (3.4, 3.4, 0.18)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    bevel = backing.modifiers.new("Display edge", "BEVEL")
    bevel.width = 0.025
    bevel.segments = 4
    backing.data.materials.append(plain_material("Backing", (0.42, 0.44, 0.42), 0.5))
    bpy.ops.mesh.primitive_plane_add(size=3.35, location=(0, 0, 0.181))
    swatch = bpy.context.object
    swatch.name = "Planar swatch (one UV repeat)"
    bpy.ops.mesh.primitive_uv_sphere_add(segments=192, ring_count=96, radius=1.04, location=(0, 0.3, 1.221))
    sphere = bpy.context.object
    sphere.name = "Review sphere (consumer geometry)"
    for polygon in sphere.data.polygons:
        polygon.use_smooth = True
    bpy.ops.object.camera_add(location=(4.6, -6.8, 4.5))
    camera = bpy.context.object
    aim(camera, (0, 0, 0.9))
    camera.data.type = "ORTHO"
    camera.data.ortho_scale = 5.65
    scene.camera = camera
    light("Key softbox", (-3, -4, 6), (0, 0, 1), 950, 3, 1.5)
    light("Narrow reflection card", (3, -1.5, 4), (0, 0, 1), 450, 0.65, 3)
    light("Rim softbox", (1, 4, 5), (0, 0, 1), 700, 3, 1.2)
    return scene, (swatch, sphere), selected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--device", choices=("METAL", "CPU"), required=True)
    parser.add_argument("--samples", type=int, default=128)
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
    if digest(Path(__file__)) != script_digest:
        raise ValueError("Scene script changed during review")
    report = {
        "schemaVersion": 1,
        "kind": "external-pbr-appearance-review",
        "material": "glazed-ceramic",
        "baselineManifestSha256": manifest_digest,
        "inputs": manifest["files"],
        "sceneScriptSha256": script_digest,
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
            "normal": "Non-Color -> tangent Normal Map -> Bump Normal",
            "height": "Non-Color -> Bump Height, distance 0.02; constant zero has no gradient",
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
